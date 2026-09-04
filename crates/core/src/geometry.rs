/// A 2D point in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// A width/height pair in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

/// An axis-aligned rectangle in logical pixels, positioned by its top-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains_inside_point() {
        let r = Rect { x: 10.0, y: 10.0, width: 20.0, height: 20.0 };
        assert!(r.contains(Point { x: 15.0, y: 15.0 }));
    }

    #[test]
    fn rect_excludes_outside_point() {
        let r = Rect { x: 10.0, y: 10.0, width: 20.0, height: 20.0 };
        assert!(!r.contains(Point { x: 100.0, y: 100.0 }));
    }

    #[test]
    fn rect_boundary_is_inclusive() {
        let r = Rect { x: 0.0, y: 0.0, width: 10.0, height: 10.0 };
        assert!(r.contains(Point { x: 10.0, y: 10.0 }));
        assert!(r.contains(Point { x: 0.0, y: 0.0 }));
    }
}
