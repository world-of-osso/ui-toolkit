use super::*;
use crate::anchor::{Anchor, AnchorPoint};
use crate::layout::recompute_layouts;

fn apply(registry: &mut FrameRegistry, id: u64, name: &str, value: &str) {
    apply_attribute(
        registry,
        id,
        name,
        value,
        &mut HashSet::new(),
        &mut HashSet::new(),
    );
}

fn anchored_frame(registry: &mut FrameRegistry, name: &str, parent: Option<u64>) -> u64 {
    let id = registry.create_frame(name, parent);
    apply(registry, id, "width", "50");
    apply(registry, id, "height", "20");
    registry
        .set_point(
            id,
            Anchor {
                point: AnchorPoint::TopLeft,
                relative_to: parent,
                relative_point: AnchorPoint::TopLeft,
                x_offset: 0.0,
                y_offset: 0.0,
            },
        )
        .unwrap();
    id
}

#[test]
fn layout_attributes_resize_with_other_dirty_frames() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = anchored_frame(&mut registry, "Sized", None);
    recompute_layouts(&mut registry);
    apply(&mut registry, id, "width", "120");
    apply(&mut registry, id, "height", "35");
    anchored_frame(&mut registry, "Unrelated", None);
    recompute_layouts(&mut registry);
    let rect = registry.get(id).unwrap().layout_rect.as_ref().unwrap();
    assert_eq!((rect.width, rect.height), (120.0, 35.0));
}

#[test]
fn layout_attributes_reflow_changed_flex_gap() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let row = anchored_frame(&mut registry, "Row", None);
    apply(&mut registry, row, "layout", "flex-row");
    apply(&mut registry, row, "width", "300");
    apply(&mut registry, row, "align", "start");
    let _first = anchored_frame(&mut registry, "First", Some(row));
    let second = anchored_frame(&mut registry, "Second", Some(row));
    recompute_layouts(&mut registry);
    let before = registry
        .get(second)
        .unwrap()
        .layout_rect
        .as_ref()
        .unwrap()
        .x;
    apply(&mut registry, row, "gap", "17");
    anchored_frame(&mut registry, "Unrelated", None);
    recompute_layouts(&mut registry);
    assert_eq!(
        registry
            .get(second)
            .unwrap()
            .layout_rect
            .as_ref()
            .unwrap()
            .x,
        before + 17.0
    );
}
