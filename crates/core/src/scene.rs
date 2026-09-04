use crate::geometry::{Point, Rect, Size};
use crate::widget::{BoxedWidget, Painter};
use std::rc::Rc;
use taffy::prelude::{AvailableSpace, TaffyTree};

struct Instance {
    widget: BoxedWidget,
    children: Vec<Instance>,
    node_id: taffy::NodeId,
}

fn remove_instance(tree: &mut TaffyTree<()>, instance: Instance) {
    for child in instance.children {
        remove_instance(tree, child);
    }
    let _ = tree.remove(instance.node_id);
}

/// Reconciles `widget` against a previous frame's `Instance` at the same
/// tree position, if any.
///
/// Widgets carry no persistent identity of their own (all long-lived state
/// lives in `Signal`s, not in widget structs — see `creamui_reactive`), so
/// this reconciles purely structurally: a widget is considered "the same
/// node" as whatever widget previously occupied the same position among its
/// parent's children. That's enough to avoid recreating `taffy` nodes (and
/// their subtrees) on every reactive re-render for the common case where a
/// re-render only changes leaf styles/content, not the tree shape.
///
/// This does not (yet) support keyed reconciliation, so reordering a list
/// of children will be treated as every item after the reorder point
/// changing, rather than being matched up by identity — tracked on the
/// roadmap alongside a real virtual-list/keyed-diff widget.
fn reconcile(tree: &mut TaffyTree<()>, existing: Option<Instance>, mut widget: BoxedWidget) -> Instance {
    let new_child_widgets = widget.children();
    let new_style = widget.style();

    let Some(mut old) = existing else {
        // No previous node at this position: build a fresh subtree.
        let mut child_ids = Vec::with_capacity(new_child_widgets.len());
        let mut children = Vec::with_capacity(new_child_widgets.len());
        for child_widget in new_child_widgets {
            let child = reconcile(tree, None, child_widget);
            child_ids.push(child.node_id);
            children.push(child);
        }
        let node_id = tree
            .new_with_children(new_style, &child_ids)
            .expect("taffy node creation is infallible for well-formed styles");
        return Instance { widget, children, node_id };
    };

    tree.set_style(old.node_id, new_style)
        .expect("updating the style of an existing node should not fail");

    let mut new_children = Vec::with_capacity(new_child_widgets.len());
    let mut old_children = old.children.drain(..);
    for child_widget in new_child_widgets {
        new_children.push(reconcile(tree, old_children.next(), child_widget));
    }
    for leftover in old_children {
        remove_instance(tree, leftover);
    }

    let child_ids: Vec<_> = new_children.iter().map(|c| c.node_id).collect();
    tree.set_children(old.node_id, &child_ids)
        .expect("setting children of an existing node should not fail");

    Instance {
        widget,
        children: new_children,
        node_id: old.node_id,
    }
}

fn paint_instance(
    tree: &TaffyTree<()>,
    instance: &Instance,
    painter: &mut dyn Painter,
    parent_origin: Point,
    hits: &mut Vec<(Rect, Rc<dyn Fn()>)>,
) {
    let layout = tree
        .layout(instance.node_id)
        .expect("layout was computed for every instantiated node");
    let rect = Rect {
        x: parent_origin.x + layout.location.x,
        y: parent_origin.y + layout.location.y,
        width: layout.size.width,
        height: layout.size.height,
    };

    instance.widget.paint(painter, rect);
    if let Some(handler) = instance.widget.on_click() {
        hits.push((rect, handler));
    }

    let origin = Point { x: rect.x, y: rect.y };
    for child in &instance.children {
        paint_instance(tree, child, painter, origin, hits);
    }
}

/// The result of rendering one frame: nothing but the click hit-regions,
/// since painting has already happened by the time this is returned.
///
/// Hit-regions are ordered parent-before-child, so hit-testing should walk
/// them in reverse to prefer the most specific (topmost) match.
pub struct Scene {
    hits: Vec<(Rect, Rc<dyn Fn()>)>,
}

impl Scene {
    /// Returns the click handler for the topmost widget containing `point`, if any.
    pub fn hit_test(&self, point: Point) -> Option<&Rc<dyn Fn()>> {
        self.hits
            .iter()
            .rev()
            .find(|(rect, _)| rect.contains(point))
            .map(|(_, handler)| handler)
    }
}

/// Owns a persistent `taffy` layout tree across frames, reconciling each new
/// widget tree against the previous one instead of rebuilding from scratch.
///
/// This is CreamUI's render loop entry point: construct one `Renderer` per
/// window/surface and call [`Renderer::render`] once per reactive re-render.
pub struct Renderer {
    tree: TaffyTree<()>,
    root: Option<Instance>,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            tree: TaffyTree::new(),
            root: None,
        }
    }

    /// Reconciles `root` against the previously rendered tree (if any),
    /// computes layout for `viewport`, paints via `painter`, and returns the
    /// resulting click hit-regions.
    pub fn render(&mut self, root: BoxedWidget, viewport: Size, painter: &mut dyn Painter) -> Scene {
        let previous = self.root.take();
        let instance = reconcile(&mut self.tree, previous, root);

        self.tree
            .compute_layout(
                instance.node_id,
                taffy::geometry::Size {
                    width: AvailableSpace::Definite(viewport.width),
                    height: AvailableSpace::Definite(viewport.height),
                },
            )
            .expect("layout computation should not fail for a well-formed tree");

        let mut hits = Vec::new();
        paint_instance(&self.tree, &instance, painter, Point::default(), &mut hits);
        self.root = Some(instance);
        Scene { hits }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Renderer::new()
    }
}

/// Renders a widget tree from scratch with no retained state across calls.
///
/// Prefer [`Renderer`] for anything rendered more than once (it reconciles
/// against the previous frame instead of rebuilding every `taffy` node);
/// this is a convenience for one-shot rendering (e.g. tests, or a single
/// static frame).
pub fn render_frame(root: BoxedWidget, viewport: Size, painter: &mut dyn Painter) -> Scene {
    Renderer::new().render(root, viewport, painter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::TextAlign;
    use creamui_theme::Color;

    struct NoopPainter;
    impl Painter for NoopPainter {
        fn fill_rect(&mut self, _rect: Rect, _color: Color, _corner_radius: f32) {}
        fn stroke_rect(&mut self, _rect: Rect, _color: Color, _width: f32, _corner_radius: f32) {}
        fn fill_text(&mut self, _rect: Rect, _text: &str, _color: Color, _font_size: f32, _align: TextAlign) {}
    }

    struct Branch {
        child_count: usize,
    }
    impl crate::widget::Widget for Branch {
        fn style(&self) -> taffy::style::Style {
            taffy::style::Style::default()
        }
        fn paint(&self, _painter: &mut dyn Painter, _rect: Rect) {}
        fn children(&mut self) -> Vec<BoxedWidget> {
            (0..self.child_count)
                .map(|_| Box::new(Branch { child_count: 0 }) as BoxedWidget)
                .collect()
        }
    }

    const VIEWPORT: Size = Size { width: 100.0, height: 100.0 };

    #[test]
    fn reconcile_reuses_taffy_nodes_when_tree_shape_is_unchanged() {
        let mut renderer = Renderer::new();
        let mut painter = NoopPainter;

        renderer.render(Box::new(Branch { child_count: 2 }), VIEWPORT, &mut painter);
        let root1 = &renderer.root.as_ref().unwrap();
        let root_id_1 = root1.node_id;
        let child_ids_1: Vec<_> = root1.children.iter().map(|c| c.node_id).collect();

        renderer.render(Box::new(Branch { child_count: 2 }), VIEWPORT, &mut painter);
        let root2 = &renderer.root.as_ref().unwrap();
        let root_id_2 = root2.node_id;
        let child_ids_2: Vec<_> = root2.children.iter().map(|c| c.node_id).collect();

        assert_eq!(root_id_1, root_id_2, "root node identity should be preserved across renders");
        assert_eq!(child_ids_1, child_ids_2, "child node identities should be preserved across renders");
    }

    #[test]
    fn reconcile_adjusts_taffy_children_when_child_count_shrinks() {
        let mut renderer = Renderer::new();
        let mut painter = NoopPainter;

        renderer.render(Box::new(Branch { child_count: 3 }), VIEWPORT, &mut painter);
        assert_eq!(renderer.root.as_ref().unwrap().children.len(), 3);

        renderer.render(Box::new(Branch { child_count: 1 }), VIEWPORT, &mut painter);
        let root = renderer.root.as_ref().unwrap();
        assert_eq!(root.children.len(), 1);
        assert_eq!(
            renderer.tree.children(root.node_id).unwrap().len(),
            1,
            "the underlying taffy tree's child list should also shrink, not just our Instance tree"
        );
    }

    #[test]
    fn reconcile_grows_taffy_children_when_child_count_increases() {
        let mut renderer = Renderer::new();
        let mut painter = NoopPainter;

        renderer.render(Box::new(Branch { child_count: 1 }), VIEWPORT, &mut painter);
        renderer.render(Box::new(Branch { child_count: 4 }), VIEWPORT, &mut painter);

        let root = renderer.root.as_ref().unwrap();
        assert_eq!(root.children.len(), 4);
        assert_eq!(renderer.tree.children(root.node_id).unwrap().len(), 4);
    }
}
