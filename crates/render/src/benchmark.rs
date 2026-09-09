//! Optional FPS/CPU/RAM/frame-time overlay, gated by `CUI_ENABLE_BENCHMARK`
//! (or `WindowOptions::benchmark`) so it costs nothing when unused.
//!
//! CPU% and RAM are sampled from `/proc/self/{stat,status}` on Linux and
//! from `getrusage` on macOS (RAM there is peak, not current — `getrusage`
//! has no current-RSS field without a `mach`-based crate). Neither is
//! implemented on Windows or wasm32; the overlay shows "n/a" for both there.

use creamui_core::{Painter, Rect, Size, TextAlign};
use creamui_theme::Color;
use std::collections::VecDeque;

#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

/// How (or whether) a window shows live performance stats. Resolved against
/// `CUI_ENABLE_BENCHMARK` — see [`BenchmarkMode::resolve`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BenchmarkMode {
    #[default]
    Off,
    /// A small panel in one corner of the window; F3 cycles its corner (or
    /// hides it) while the window has focus.
    Overlay,
    /// The same stats, formatted into the OS window title instead of a
    /// panel — no F3 handling, since there's no corner to cycle.
    Title,
}

impl BenchmarkMode {
    fn from_env_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "on" => Some(BenchmarkMode::Overlay),
            "title" => Some(BenchmarkMode::Title),
            "0" | "false" | "off" | "" => Some(BenchmarkMode::Off),
            _ => None,
        }
    }

    /// Resolves `requested` against `CUI_ENABLE_BENCHMARK` (`1`/`true`/`on`,
    /// `title`, or `off`, case-insensitive), which wins over `requested` if
    /// set to a recognized value — same override precedence as
    /// `CUI_OVERRIDE_RENDER_BACKEND` (see `RenderBackend::resolve`).
    pub(crate) fn resolve(requested: Self) -> Self {
        let Ok(raw) = std::env::var("CUI_ENABLE_BENCHMARK") else {
            return requested;
        };
        match Self::from_env_str(&raw) {
            Some(mode) => {
                if mode != requested {
                    log::info!(
                        "creamui-render: CUI_ENABLE_BENCHMARK={raw} overrides requested benchmark mode {requested:?} -> {mode:?}"
                    );
                }
                mode
            }
            None => {
                log::warn!(
                    "creamui-render: ignoring CUI_ENABLE_BENCHMARK={raw:?}, expected \"1\"/\"true\"/\"on\"/\"title\"/\"off\""
                );
                requested
            }
        }
    }
}

/// Which corner the overlay panel sits in. `Hidden` keeps sampling stats
/// without painting anything. F3 cycles through all five, in this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DebugPosition {
    #[default]
    BottomRight,
    BottomLeft,
    TopLeft,
    TopRight,
    Hidden,
}

impl DebugPosition {
    pub fn cycle(self) -> Self {
        match self {
            DebugPosition::BottomRight => DebugPosition::BottomLeft,
            DebugPosition::BottomLeft => DebugPosition::TopLeft,
            DebugPosition::TopLeft => DebugPosition::TopRight,
            DebugPosition::TopRight => DebugPosition::Hidden,
            DebugPosition::Hidden => DebugPosition::BottomRight,
        }
    }
}

/// Family the overlay/title text resolves against — falls back to the
/// bundled default (`creamui_fonts::DEFAULT_FAMILY`) unless the app
/// registers a font (e.g. a pixel-art face) under this name first.
pub const DEBUG_FONT_FAMILY: &str = "CreamUI Debug, monospace";

const FRAME_HISTORY_LEN: usize = 120;

/// Rolling frame-paint-duration and FPS tracking.
pub struct FrameStats {
    history: VecDeque<f32>,
    repaint_count: u64,
    fps: f32,
    fps_window_start: Instant,
    fps_window_frames: u32,
}

impl FrameStats {
    pub fn new() -> Self {
        FrameStats {
            history: VecDeque::with_capacity(FRAME_HISTORY_LEN),
            repaint_count: 0,
            fps: 0.0,
            fps_window_start: Instant::now(),
            fps_window_frames: 0,
        }
    }

    /// Records one painted frame's duration, updating the rolling history,
    /// the repaint counter, and (once a second) the FPS estimate.
    pub fn record_frame(&mut self, duration: Duration) {
        self.repaint_count += 1;
        if self.history.len() == FRAME_HISTORY_LEN {
            self.history.pop_front();
        }
        self.history.push_back(duration.as_secs_f32() * 1000.0);

        self.fps_window_frames += 1;
        let elapsed = self.fps_window_start.elapsed();
        if elapsed >= Duration::from_secs(1) {
            self.fps = self.fps_window_frames as f32 / elapsed.as_secs_f32();
            self.fps_window_frames = 0;
            self.fps_window_start = Instant::now();
        }
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn repaint_count(&self) -> u64 {
        self.repaint_count
    }

    pub fn current_ms(&self) -> f32 {
        self.history.back().copied().unwrap_or(0.0)
    }

    pub fn min_ms(&self) -> f32 {
        if self.history.is_empty() {
            0.0
        } else {
            self.history.iter().copied().fold(f32::MAX, f32::min)
        }
    }

    pub fn max_ms(&self) -> f32 {
        self.history.iter().copied().fold(0.0, f32::max)
    }

    pub fn avg_ms(&self) -> f32 {
        if self.history.is_empty() {
            0.0
        } else {
            self.history.iter().sum::<f32>() / self.history.len() as f32
        }
    }
}

/// Process CPU%/RAM sampling, resampled at most every 200ms — see the
/// module doc comment for per-platform support.
pub struct ProcessStats {
    last_sample: Instant,
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    last_cpu_time: Duration,
    cpu_percent: Option<f32>,
    ram_mb: Option<f32>,
}

/// Below this, a fresh CPU-time delta is too noisy (or, right after
/// construction, meaningless — there's no prior sample yet) to divide by
/// without producing a wildly inflated percentage.
const MIN_SAMPLE_INTERVAL: Duration = Duration::from_millis(200);

impl ProcessStats {
    pub fn new() -> Self {
        let (cpu_time, ram_mb) = read_raw();
        ProcessStats {
            last_sample: Instant::now(),
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            last_cpu_time: cpu_time.unwrap_or(Duration::ZERO),
            cpu_percent: None,
            ram_mb,
        }
    }

    /// Cheap enough to call every frame: a no-op unless
    /// [`MIN_SAMPLE_INTERVAL`] has passed since the last sample, since a
    /// CPU% needs a real time delta to mean anything.
    pub fn maybe_sample(&mut self) {
        let elapsed = self.last_sample.elapsed();
        if elapsed < MIN_SAMPLE_INTERVAL {
            return;
        }
        let (cpu_time, ram_mb) = read_raw();
        self.last_sample = Instant::now();
        self.ram_mb = ram_mb;
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if let Some(cpu_time) = cpu_time {
            let delta = cpu_time.saturating_sub(self.last_cpu_time);
            self.cpu_percent = Some(delta.as_secs_f32() / elapsed.as_secs_f32() * 100.0);
            self.last_cpu_time = cpu_time;
        }
    }

    pub fn cpu_percent(&self) -> Option<f32> {
        self.cpu_percent
    }

    pub fn ram_mb(&self) -> Option<f32> {
        self.ram_mb
    }
}

/// One-shot read of (cumulative process CPU time, current/peak RAM in MB).
#[cfg(target_os = "linux")]
fn read_raw() -> (Option<Duration>, Option<f32>) {
    (read_proc_cpu_time(), read_proc_vm_rss_mb())
}

#[cfg(target_os = "macos")]
fn read_raw() -> (Option<Duration>, Option<f32>) {
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    if unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) } != 0 {
        return (None, None);
    }
    let cpu_time = Duration::from_secs(usage.ru_utime.tv_sec.max(0) as u64)
        + Duration::from_micros(usage.ru_utime.tv_usec.max(0) as u64)
        + Duration::from_secs(usage.ru_stime.tv_sec.max(0) as u64)
        + Duration::from_micros(usage.ru_stime.tv_usec.max(0) as u64);
    // ru_maxrss is peak RSS since process start on macOS, not current — the
    // closest single-syscall approximation without a `mach`-based crate.
    let ram_mb = usage.ru_maxrss as f32 / (1024.0 * 1024.0);
    (Some(cpu_time), Some(ram_mb))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn read_raw() -> (Option<Duration>, Option<f32>) {
    (None, None)
}

#[cfg(target_os = "linux")]
fn read_proc_vm_rss_mb() -> Option<f32> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            let kb: f32 = rest.trim().trim_end_matches("kB").trim().parse().ok()?;
            return Some(kb / 1024.0);
        }
    }
    None
}

/// utime+stime from `/proc/self/stat`, converted from clock ticks to
/// seconds. `comm` (field 2) can itself contain spaces/parens, so this
/// splits after the last `)` rather than trusting field position from the
/// start of the line.
#[cfg(target_os = "linux")]
fn read_proc_cpu_time() -> Option<Duration> {
    let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
    let after_comm = stat.rsplit_once(')')?.1;
    let fields: Vec<&str> = after_comm.split_whitespace().collect();
    let utime: u64 = fields.get(11)?.parse().ok()?;
    let stime: u64 = fields.get(12)?.parse().ok()?;
    let ticks_per_sec = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    if ticks_per_sec <= 0 {
        return None;
    }
    Some(Duration::from_secs_f64(
        (utime + stime) as f64 / ticks_per_sec as f64,
    ))
}

fn format_ram(ram_mb: Option<f32>) -> String {
    ram_mb.map_or_else(|| "n/a".to_string(), |v| format!("{v:.1} MB"))
}

fn format_cpu(cpu_percent: Option<f32>) -> String {
    cpu_percent.map_or_else(|| "n/a".to_string(), |v| format!("{v:.1}%"))
}

fn overlay_text(frame_stats: &FrameStats, process_stats: &ProcessStats) -> String {
    format!(
        "FPS {:.0}\nFrame {:.1}/{:.1}/{:.1}/{:.1} ms\n(cur/avg/min/max)\nRepaints {}\nRAM {}\nCPU {}",
        frame_stats.fps(),
        frame_stats.current_ms(),
        frame_stats.avg_ms(),
        frame_stats.min_ms(),
        frame_stats.max_ms(),
        frame_stats.repaint_count(),
        format_ram(process_stats.ram_mb()),
        format_cpu(process_stats.cpu_percent()),
    )
}

/// Paints the overlay panel in `position`'s corner of `viewport`. A no-op
/// when `position` is [`DebugPosition::Hidden`].
pub fn draw_overlay(
    painter: &mut dyn Painter,
    viewport: Size,
    position: DebugPosition,
    frame_stats: &FrameStats,
    process_stats: &ProcessStats,
) {
    if position == DebugPosition::Hidden {
        return;
    }
    let text = overlay_text(frame_stats, process_stats);
    let font_size = 12.0;
    let line_height = font_size * 1.5;
    let line_count = text.lines().count().max(1) as f32;
    let padding = 10.0;
    let width = 190.0;
    let height = line_height * line_count + padding * 2.0;
    let margin = 12.0;

    let (x, y) = match position {
        DebugPosition::BottomRight => {
            (viewport.width - width - margin, viewport.height - height - margin)
        }
        DebugPosition::BottomLeft => (margin, viewport.height - height - margin),
        DebugPosition::TopLeft => (margin, margin),
        DebugPosition::TopRight => (viewport.width - width - margin, margin),
        DebugPosition::Hidden => unreachable!("handled above"),
    };

    painter.fill_rect(
        Rect { x, y, width, height },
        Color::rgba(0, 0, 0, 180),
        6.0,
    );
    let text_rect = Rect {
        x: x + padding,
        y: y + padding,
        width: width - padding * 2.0,
        height: height - padding * 2.0,
    };
    painter.fill_text_font(
        text_rect,
        &text,
        Color::rgb(120, 255, 140),
        font_size,
        TextAlign::Start,
        Some(DEBUG_FONT_FAMILY),
        false,
        false,
    );
}

/// Appends the same stats [`draw_overlay`] shows to `base_title`, for
/// [`BenchmarkMode::Title`].
pub fn format_title(base_title: &str, frame_stats: &FrameStats, process_stats: &ProcessStats) -> String {
    format!(
        "{base_title} — {:.0} FPS | {:.1}ms (avg {:.1}) | {} repaints | {} RAM | {} CPU",
        frame_stats.fps(),
        frame_stats.current_ms(),
        frame_stats.avg_ms(),
        frame_stats.repaint_count(),
        format_ram(process_stats.ram_mb()),
        format_cpu(process_stats.cpu_percent()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test for a real bug: sampling `cpu_percent` immediately
    /// on construction (before any real elapsed time) divided a nonzero CPU
    /// delta by a near-zero duration, producing readings like 400000000%.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn process_stats_cpu_percent_is_sane_after_a_real_sample_interval() {
        let mut stats = ProcessStats::new();
        assert_eq!(stats.cpu_percent(), None, "no percent until a real interval has passed");

        // Busy-loop so there's guaranteed nonzero CPU time to measure.
        let start = Instant::now();
        let mut acc: u64 = 0;
        while start.elapsed() < MIN_SAMPLE_INTERVAL + Duration::from_millis(50) {
            acc = acc.wrapping_add(1);
        }
        std::hint::black_box(acc);

        stats.maybe_sample();
        let cpu = stats.cpu_percent().expect("a real interval has now passed");
        assert!(
            (0.0..=800.0).contains(&cpu),
            "cpu% should be a plausible reading (0-800 to allow a few cores), got {cpu}"
        );
    }

    #[test]
    fn benchmark_mode_from_env_recognizes_all_documented_values() {
        assert_eq!(BenchmarkMode::from_env_str("1"), Some(BenchmarkMode::Overlay));
        assert_eq!(BenchmarkMode::from_env_str("true"), Some(BenchmarkMode::Overlay));
        assert_eq!(BenchmarkMode::from_env_str("ON"), Some(BenchmarkMode::Overlay));
        assert_eq!(BenchmarkMode::from_env_str("Title"), Some(BenchmarkMode::Title));
        assert_eq!(BenchmarkMode::from_env_str("off"), Some(BenchmarkMode::Off));
        assert_eq!(BenchmarkMode::from_env_str("nonsense"), None);
    }

    #[test]
    fn debug_position_cycles_through_all_five_and_back() {
        let mut pos = DebugPosition::BottomRight;
        let mut seen = vec![pos];
        for _ in 0..4 {
            pos = pos.cycle();
            seen.push(pos);
        }
        assert_eq!(
            seen,
            vec![
                DebugPosition::BottomRight,
                DebugPosition::BottomLeft,
                DebugPosition::TopLeft,
                DebugPosition::TopRight,
                DebugPosition::Hidden,
            ]
        );
        assert_eq!(pos.cycle(), DebugPosition::BottomRight);
    }

    #[test]
    fn frame_stats_tracks_current_min_max_avg() {
        let mut stats = FrameStats::new();
        stats.record_frame(Duration::from_millis(10));
        stats.record_frame(Duration::from_millis(20));
        stats.record_frame(Duration::from_millis(30));
        assert_eq!(stats.current_ms(), 30.0);
        assert_eq!(stats.min_ms(), 10.0);
        assert_eq!(stats.max_ms(), 30.0);
        assert!((stats.avg_ms() - 20.0).abs() < 0.01);
        assert_eq!(stats.repaint_count(), 3);
    }

    #[test]
    fn frame_stats_history_is_bounded() {
        let mut stats = FrameStats::new();
        for _ in 0..(FRAME_HISTORY_LEN + 10) {
            stats.record_frame(Duration::from_millis(5));
        }
        assert_eq!(stats.repaint_count(), (FRAME_HISTORY_LEN + 10) as u64);
        assert_eq!(stats.history.len(), FRAME_HISTORY_LEN);
    }

    #[test]
    fn format_title_keeps_the_base_title_and_appends_stats() {
        let mut frame_stats = FrameStats::new();
        frame_stats.record_frame(Duration::from_millis(16));
        let process_stats = ProcessStats {
            last_sample: Instant::now(),
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            last_cpu_time: Duration::ZERO,
            cpu_percent: Some(12.5),
            ram_mb: Some(64.0),
        };
        let title = format_title("My App", &frame_stats, &process_stats);
        assert!(title.starts_with("My App —"));
        assert!(title.contains("16.0ms"));
        assert!(title.contains("1 repaints"));
        assert!(title.contains("64.0 MB"));
        assert!(title.contains("12.5% CPU"));
    }

    #[test]
    fn format_ram_and_cpu_show_n_a_when_unsupported() {
        assert_eq!(format_ram(None), "n/a");
        assert_eq!(format_cpu(None), "n/a");
        assert_eq!(format_ram(Some(64.0)), "64.0 MB");
        assert_eq!(format_cpu(Some(12.5)), "12.5%");
    }
}
