use super::*;
use crate::frame::Dimension;
use crate::layout::recompute_layouts;

fn frame(registry: &mut FrameRegistry, name: &str) -> u64 {
    let id = registry.create_frame(name, None);
    let frame = registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(20.0);
    frame.height = Dimension::Fixed(10.0);
    id
}

#[test]
fn screen_layout_resolved_anchor_replacement_tracks_new_target() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let first = frame(&mut registry, "First");
    let second = frame(&mut registry, "Second");
    let follower = frame(&mut registry, "Follower");
    apply_anchor_resolved(&mut registry, first, "TOPLEFT,$parent,TOPLEFT,10,0");
    apply_anchor_resolved(&mut registry, second, "TOPLEFT,$parent,TOPLEFT,40,0");
    apply_anchor_resolved(&mut registry, follower, "TOPLEFT,First,TOPLEFT,3,0");
    recompute_layouts(&mut registry);
    assert_eq!(
        registry
            .get(follower)
            .unwrap()
            .layout_rect
            .as_ref()
            .unwrap()
            .x,
        13.0
    );
    apply_anchor_resolved(&mut registry, follower, "TOPLEFT,Second,TOPLEFT,5,0");
    recompute_layouts(&mut registry);
    assert_eq!(
        registry
            .get(follower)
            .unwrap()
            .layout_rect
            .as_ref()
            .unwrap()
            .x,
        45.0
    );
    assert_eq!(registry.get(follower).unwrap().anchors.len(), 1);
    apply_anchor_resolved(&mut registry, second, "TOPLEFT,$parent,TOPLEFT,80,0");
    recompute_layouts(&mut registry);
    assert_eq!(
        registry
            .get(follower)
            .unwrap()
            .layout_rect
            .as_ref()
            .unwrap()
            .x,
        85.0
    );
}

#[test]
#[should_panic(expected = "frame cannot anchor to itself")]
fn screen_layout_resolved_self_anchor_fails_explicitly() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = frame(&mut registry, "Self");
    apply_anchor_resolved(&mut registry, id, "TOPLEFT,Self,TOPLEFT,0,0");
}
