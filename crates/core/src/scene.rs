use crate::geometry::{Point, Rect, Size};
use crate::widget::{BoxedWidget, Painter};
use std::rc::Rc;
use taffy::prelude::{AvailableSpace, TaffyTree};

struct Instance {
    widget: BoxedWidget,
    children: Vec<Instance>,
    node_id: taffy::NodeId,
}

fn instantiate(tree: &mut TaffyTree<()>, mut widget: BoxedWidget) -> Instance {
    let child_widgets = widget.children();
    let mut children = Vec::with_capacity(child_widgets.len());
    let mut child_ids = Vec::with_capacity(child_widgets.len());
    for child_widget in child_widgets {
        let instance = instantiate(tree, child_widget);
        child_ids.push(instance.node_id);
        children.push(instance);
    }
    let node_id = tree
        .new_with_children(widget.style(), &child_ids)
        .expect("taffy node creation is infallible for well-formed styles");
    Instance {
        widget,
        children,
        node_id,
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

/// Builds a widget tree rooted at `root`, computes its layout for `viewport`,
/// paints it via `painter`, and returns the resulting click hit-regions.
///
/// Since [`Widget`](crate::Widget) trees are rebuilt on every reactive
/// re-render, calling this once per frame is the entire CreamUI render loop.
pub fn render_frame(root: BoxedWidget, viewport: Size, painter: &mut dyn Painter) -> Scene {
    let mut tree = TaffyTree::new();
    let instance = instantiate(&mut tree, root);
    tree.compute_layout(
        instance.node_id,
        taffy::geometry::Size {
            width: AvailableSpace::Definite(viewport.width),
            height: AvailableSpace::Definite(viewport.height),
        },
    )
    .expect("layout computation should not fail for a well-formed tree");

    let mut hits = Vec::new();
    paint_instance(&tree, &instance, painter, Point::default(), &mut hits);
    Scene { hits }
}
