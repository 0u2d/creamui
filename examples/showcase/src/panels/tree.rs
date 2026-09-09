use crate::prelude::*;

/// The "Tree" panel: the "Data View" category's first entry — browsing a
/// hierarchy is a different job from picking one value (see `ListBox` on
/// the "Selection" page instead). A `TreeController` tracks which folders
/// are expanded and which row is selected.
#[component]
pub fn TreePanel(theme: Theme, tree_scroll: ScrollController, tree: TreeController) -> BoxedWidget {
    let nodes = vec![
        TreeNode::new(1, "src").with_children(vec![
            TreeNode::new(2, "main.rs"),
            TreeNode::new(3, "widgets").with_children(vec![
                TreeNode::new(4, "button.rs"),
                TreeNode::new(5, "scroll.rs"),
            ]),
        ]),
        TreeNode::new(6, "Cargo.toml"),
        TreeNode::new(7, "README.md"),
    ];
    let tree_style = Style {
        size: creamui_core::layout::Size {
            width: Dimension::Length(260.0),
            height: Dimension::Length(280.0),
        },
        flex_shrink: 0.0,
        ..Default::default()
    };
    let view = TreeView::new(&theme, tree_style, tree_scroll, tree, &nodes);
    Box::new(jsx! {
        <RawView style={column(section_gap(&theme))}>
            <SectionHeader theme={theme} title={"Tree".to_owned()} subtitle={"Click a chevron to expand or collapse a folder, click a row to select it, or use the arrow keys once focused.".to_owned()} />
            {field_card(&theme, "File browser", Box::new(view))}
        </RawView>
    })
}
