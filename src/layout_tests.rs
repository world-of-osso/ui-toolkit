use crate::anchor::{Anchor, AnchorPoint};
use crate::frame::{Dimension, FlexAlign, FlexDirection, FlexJustify, FlexLayout};
use crate::layout::{LayoutRect, recompute_layouts, resolve_anchors, resolve_frame_layout};
use crate::registry::FrameRegistry;

fn parent() -> LayoutRect {
    LayoutRect {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    }
}

fn anchor(point: AnchorPoint, relative_point: AnchorPoint, x_offset: f32, y_offset: f32) -> Anchor {
    Anchor {
        point,
        relative_to: None,
        relative_point,
        x_offset,
        y_offset,
    }
}

fn approx_eq(a: &LayoutRect, b: &LayoutRect) -> bool {
    (a.x - b.x).abs() < 0.01
        && (a.y - b.y).abs() < 0.01
        && (a.width - b.width).abs() < 0.01
        && (a.height - b.height).abs() < 0.01
}

#[test]
fn no_anchors_at_parent_topleft() {
    let result = resolve_anchors(&[], 100.0, 50.0, &parent());
    assert_eq!(
        result,
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0
        }
    );
}

#[test]
fn single_center_to_center() {
    let anchors = [anchor(AnchorPoint::Center, AnchorPoint::Center, 0.0, 0.0)];
    let result = resolve_anchors(&anchors, 200.0, 100.0, &parent());
    assert_eq!(
        result,
        LayoutRect {
            x: 300.0,
            y: 250.0,
            width: 200.0,
            height: 100.0
        }
    );
}

#[test]
fn single_topleft_with_offset() {
    let anchors = [anchor(
        AnchorPoint::TopLeft,
        AnchorPoint::TopLeft,
        10.0,
        -5.0,
    )];
    let result = resolve_anchors(&anchors, 100.0, 50.0, &parent());
    assert_eq!(
        result,
        LayoutRect {
            x: 10.0,
            y: 5.0,
            width: 100.0,
            height: 50.0
        }
    );
}

#[test]
fn two_anchors_horizontal_stretch() {
    let anchors = [
        anchor(AnchorPoint::Left, AnchorPoint::Left, 20.0, 0.0),
        anchor(AnchorPoint::Right, AnchorPoint::Right, -20.0, 0.0),
    ];
    let result = resolve_anchors(&anchors, 100.0, 50.0, &parent());
    let expected = LayoutRect {
        x: 20.0,
        y: 275.0,
        width: 760.0,
        height: 50.0,
    };
    assert!(
        approx_eq(&result, &expected),
        "got {result:?}, expected {expected:?}"
    );
}

#[test]
fn two_anchors_vertical_stretch() {
    let anchors = [
        anchor(AnchorPoint::Top, AnchorPoint::Top, 0.0, -10.0),
        anchor(AnchorPoint::Bottom, AnchorPoint::Bottom, 0.0, 10.0),
    ];
    let result = resolve_anchors(&anchors, 200.0, 100.0, &parent());
    let expected = LayoutRect {
        x: 300.0,
        y: 10.0,
        width: 200.0,
        height: 580.0,
    };
    assert!(
        approx_eq(&result, &expected),
        "got {result:?}, expected {expected:?}"
    );
}

#[test]
fn single_bottomright_anchor() {
    let anchors = [anchor(
        AnchorPoint::BottomRight,
        AnchorPoint::BottomRight,
        0.0,
        0.0,
    )];
    let result = resolve_anchors(&anchors, 100.0, 50.0, &parent());
    assert_eq!(
        result,
        LayoutRect {
            x: 700.0,
            y: 550.0,
            width: 100.0,
            height: 50.0
        }
    );
}

#[test]
fn resolve_frame_layout_uses_relative_frame_rect() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let target = registry.create_frame("Target", None);
    let child = registry.create_frame("Child", None);

    registry.get_mut(target).unwrap().layout_rect = Some(LayoutRect {
        x: 100.0,
        y: 80.0,
        width: 200.0,
        height: 120.0,
    });

    let frame = registry.get_mut(child).unwrap();
    frame.width = Dimension::Fixed(50.0);
    frame.height = Dimension::Fixed(30.0);
    frame.anchors.push(Anchor {
        point: AnchorPoint::TopLeft,
        relative_to: Some(target),
        relative_point: AnchorPoint::BottomRight,
        x_offset: 10.0,
        y_offset: -5.0,
    });

    let rect = resolve_frame_layout(&registry, child).unwrap();
    assert_eq!(
        rect,
        LayoutRect {
            x: 310.0,
            y: 205.0,
            width: 50.0,
            height: 30.0
        }
    );
}

#[test]
fn resolve_frame_layout_falls_back_to_screen_for_root_frame() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let child = registry.create_frame("Child", None);
    let frame = registry.get_mut(child).unwrap();
    frame.width = Dimension::Fixed(100.0);
    frame.height = Dimension::Fixed(40.0);
    frame.anchors.push(Anchor {
        point: AnchorPoint::Center,
        relative_to: None,
        relative_point: AnchorPoint::Center,
        x_offset: 0.0,
        y_offset: 0.0,
    });

    let rect = resolve_frame_layout(&registry, child).unwrap();
    assert_eq!(
        rect,
        LayoutRect {
            x: 350.0,
            y: 280.0,
            width: 100.0,
            height: 40.0
        }
    );
}

#[test]
fn recompute_layouts_updates_anchored_children() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let p = registry.create_frame("Parent", None);
    let child = registry.create_frame("Child", Some(p));

    {
        let frame = registry.get_mut(p).unwrap();
        frame.width = Dimension::Fixed(300.0);
        frame.height = Dimension::Fixed(200.0);
        frame.layout_rect = Some(LayoutRect {
            x: 40.0,
            y: 50.0,
            width: 300.0,
            height: 200.0,
        });
    }
    {
        let frame = registry.get_mut(child).unwrap();
        frame.width = Dimension::Fixed(100.0);
        frame.height = Dimension::Fixed(40.0);
        frame.anchors.push(Anchor {
            point: AnchorPoint::TopLeft,
            relative_to: Some(p),
            relative_point: AnchorPoint::BottomRight,
            x_offset: 5.0,
            y_offset: -10.0,
        });
    }

    recompute_layouts(&mut registry);
    assert_eq!(
        registry.get(child).unwrap().layout_rect,
        Some(LayoutRect {
            x: 345.0,
            y: 260.0,
            width: 100.0,
            height: 40.0
        })
    );
}

// --- Flex layout tests ---

fn setup_flex_container(registry: &mut FrameRegistry, w: f32, h: f32, flex: FlexLayout) -> u64 {
    let id = registry.create_frame("Container", None);
    let frame = registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(w);
    frame.height = Dimension::Fixed(h);
    frame.layout_rect = Some(LayoutRect {
        x: 0.0,
        y: 0.0,
        width: w,
        height: h,
    });
    frame.flex_layout = Some(flex);
    id
}

fn add_flex_child(registry: &mut FrameRegistry, parent: u64, w: f32, h: f32) -> u64 {
    let id = registry.create_frame("", Some(parent));
    let frame = registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(w);
    frame.height = Dimension::Fixed(h);
    id
}

#[test]
fn flex_column_stacks_vertically() {
    let mut reg = FrameRegistry::new(800.0, 600.0);
    let c = setup_flex_container(
        &mut reg,
        400.0,
        300.0,
        FlexLayout {
            direction: FlexDirection::Column,
            gap: 10.0,
            ..Default::default()
        },
    );
    let a = add_flex_child(&mut reg, c, 200.0, 50.0);
    let b = add_flex_child(&mut reg, c, 200.0, 50.0);

    recompute_layouts(&mut reg);

    let ra = reg.get(a).unwrap().layout_rect.as_ref().unwrap();
    let rb = reg.get(b).unwrap().layout_rect.as_ref().unwrap();
    assert!((ra.x - 100.0).abs() < 0.01); // centered: (400-200)/2
    assert!((ra.y - 0.0).abs() < 0.01);
    assert!((rb.y - 60.0).abs() < 0.01); // 50 + 10 gap
}

#[test]
fn flex_row_stacks_horizontally() {
    let mut reg = FrameRegistry::new(800.0, 600.0);
    let c = setup_flex_container(
        &mut reg,
        400.0,
        100.0,
        FlexLayout {
            direction: FlexDirection::Row,
            gap: 20.0,
            ..Default::default()
        },
    );
    let a = add_flex_child(&mut reg, c, 80.0, 40.0);
    let b = add_flex_child(&mut reg, c, 80.0, 40.0);

    recompute_layouts(&mut reg);

    let ra = reg.get(a).unwrap().layout_rect.as_ref().unwrap();
    let rb = reg.get(b).unwrap().layout_rect.as_ref().unwrap();
    assert!((ra.x - 0.0).abs() < 0.01);
    assert!((ra.y - 30.0).abs() < 0.01); // centered: (100-40)/2
    assert!((rb.x - 100.0).abs() < 0.01); // 80 + 20
}

#[test]
fn flex_justify_center() {
    let mut reg = FrameRegistry::new(800.0, 600.0);
    let c = setup_flex_container(
        &mut reg,
        400.0,
        300.0,
        FlexLayout {
            direction: FlexDirection::Column,
            justify: FlexJustify::Center,
            ..Default::default()
        },
    );
    let a = add_flex_child(&mut reg, c, 100.0, 50.0);

    recompute_layouts(&mut reg);

    let ra = reg.get(a).unwrap().layout_rect.as_ref().unwrap();
    assert!((ra.y - 125.0).abs() < 0.01); // (300-50)/2
}

#[test]
fn flex_justify_end() {
    let mut reg = FrameRegistry::new(800.0, 600.0);
    let c = setup_flex_container(
        &mut reg,
        400.0,
        300.0,
        FlexLayout {
            direction: FlexDirection::Column,
            justify: FlexJustify::End,
            ..Default::default()
        },
    );
    let a = add_flex_child(&mut reg, c, 100.0, 50.0);

    recompute_layouts(&mut reg);

    let ra = reg.get(a).unwrap().layout_rect.as_ref().unwrap();
    assert!((ra.y - 250.0).abs() < 0.01); // 300-50
}

#[test]
fn flex_align_stretch() {
    let mut reg = FrameRegistry::new(800.0, 600.0);
    let c = setup_flex_container(
        &mut reg,
        400.0,
        300.0,
        FlexLayout {
            direction: FlexDirection::Column,
            align: FlexAlign::Stretch,
            ..Default::default()
        },
    );
    let a = add_flex_child(&mut reg, c, 100.0, 50.0);

    recompute_layouts(&mut reg);

    let ra = reg.get(a).unwrap().layout_rect.as_ref().unwrap();
    assert!((ra.width - 400.0).abs() < 0.01);
    assert!((ra.x - 0.0).abs() < 0.01);
}

#[test]
fn flex_with_padding() {
    let mut reg = FrameRegistry::new(800.0, 600.0);
    let c = setup_flex_container(
        &mut reg,
        400.0,
        300.0,
        FlexLayout {
            direction: FlexDirection::Column,
            padding: 20.0,
            ..Default::default()
        },
    );
    let a = add_flex_child(&mut reg, c, 100.0, 50.0);

    recompute_layouts(&mut reg);

    let ra = reg.get(a).unwrap().layout_rect.as_ref().unwrap();
    assert!((ra.y - 20.0).abs() < 0.01);
    // cross centered within padded area: 20 + (360-100)/2 = 150
    assert!((ra.x - 150.0).abs() < 0.01, "x={}", ra.x);
}

#[test]
fn flex_space_between() {
    let mut reg = FrameRegistry::new(800.0, 600.0);
    let c = setup_flex_container(
        &mut reg,
        400.0,
        300.0,
        FlexLayout {
            direction: FlexDirection::Column,
            justify: FlexJustify::SpaceBetween,
            ..Default::default()
        },
    );
    let a = add_flex_child(&mut reg, c, 100.0, 50.0);
    let b = add_flex_child(&mut reg, c, 100.0, 50.0);

    recompute_layouts(&mut reg);

    let ra = reg.get(a).unwrap().layout_rect.as_ref().unwrap();
    let rb = reg.get(b).unwrap().layout_rect.as_ref().unwrap();
    assert!((ra.y - 0.0).abs() < 0.01);
    assert!((rb.y - 250.0).abs() < 0.01, "y={}", rb.y); // 300-50
}

#[test]
fn flex_row_wrap_wraps_to_next_row() {
    let mut reg = FrameRegistry::new(800.0, 600.0);
    // 200px wide container, 5px gap, children are 60px wide
    // Row 1: 60 + 5 + 60 + 5 + 60 = 190 (fits), next would be 190+5+60=255 > 200
    let c = setup_flex_container(
        &mut reg,
        200.0,
        300.0,
        FlexLayout {
            direction: FlexDirection::RowWrap,
            gap: 5.0,
            ..Default::default()
        },
    );
    let a = add_flex_child(&mut reg, c, 60.0, 30.0);
    let b = add_flex_child(&mut reg, c, 60.0, 30.0);
    let d = add_flex_child(&mut reg, c, 60.0, 30.0);
    let e = add_flex_child(&mut reg, c, 60.0, 30.0); // wraps to row 2

    recompute_layouts(&mut reg);

    let ra = reg.get(a).unwrap().layout_rect.as_ref().unwrap();
    let rb = reg.get(b).unwrap().layout_rect.as_ref().unwrap();
    let rd = reg.get(d).unwrap().layout_rect.as_ref().unwrap();
    let re = reg.get(e).unwrap().layout_rect.as_ref().unwrap();

    // Row 1: a at x=0, b at x=65, d at x=130
    assert!((ra.x - 0.0).abs() < 0.01, "a.x={}", ra.x);
    assert!((ra.y - 0.0).abs() < 0.01, "a.y={}", ra.y);
    assert!((rb.x - 65.0).abs() < 0.01, "b.x={}", rb.x);
    assert!((rb.y - 0.0).abs() < 0.01, "b.y={}", rb.y);
    assert!((rd.x - 130.0).abs() < 0.01, "d.x={}", rd.x);
    assert!((rd.y - 0.0).abs() < 0.01, "d.y={}", rd.y);

    // Row 2: e wraps, x=0, y=30+5=35
    assert!((re.x - 0.0).abs() < 0.01, "e.x={}", re.x);
    assert!((re.y - 35.0).abs() < 0.01, "e.y={}", re.y);
}

#[test]
fn flex_row_wrap_with_padding() {
    let mut reg = FrameRegistry::new(800.0, 600.0);
    // 200px container, 10px padding → 180px available
    // Two 100px children: first fits, second wraps
    let c = setup_flex_container(
        &mut reg,
        200.0,
        300.0,
        FlexLayout {
            direction: FlexDirection::RowWrap,
            gap: 5.0,
            padding: 10.0,
            ..Default::default()
        },
    );
    let a = add_flex_child(&mut reg, c, 100.0, 40.0);
    let b = add_flex_child(&mut reg, c, 100.0, 40.0);

    recompute_layouts(&mut reg);

    let ra = reg.get(a).unwrap().layout_rect.as_ref().unwrap();
    let rb = reg.get(b).unwrap().layout_rect.as_ref().unwrap();

    // a at padding offset
    assert!((ra.x - 10.0).abs() < 0.01, "a.x={}", ra.x);
    assert!((ra.y - 10.0).abs() < 0.01, "a.y={}", ra.y);

    // b wraps: x=padding, y=padding+40+5=55
    assert!((rb.x - 10.0).abs() < 0.01, "b.x={}", rb.x);
    assert!((rb.y - 55.0).abs() < 0.01, "b.y={}", rb.y);
}

#[test]
fn flex_row_wrap_auto_height_expands_to_fit_children() {
    let mut reg = FrameRegistry::new(800.0, 600.0);
    let c = setup_flex_container(
        &mut reg,
        200.0,
        0.0,
        FlexLayout {
            direction: FlexDirection::RowWrap,
            gap: 5.0,
            padding: 10.0,
            ..Default::default()
        },
    );
    add_flex_child(&mut reg, c, 100.0, 40.0);
    add_flex_child(&mut reg, c, 100.0, 40.0);

    recompute_layouts(&mut reg);

    let container = reg.get(c).unwrap().layout_rect.as_ref().unwrap();
    assert!(
        (container.height - 105.0).abs() < 0.01,
        "height={}",
        container.height
    );
}

#[test]
fn flex_row_wrap_mixed_heights() {
    let mut reg = FrameRegistry::new(800.0, 600.0);
    // 150px wide, children: 70x20, 70x40, 70x25
    // Row 1: 70+5+70=145 fits, row height=max(20,40)=40
    // Row 2: 70x25 wraps at y=40+5=45
    let c = setup_flex_container(
        &mut reg,
        150.0,
        300.0,
        FlexLayout {
            direction: FlexDirection::RowWrap,
            gap: 5.0,
            ..Default::default()
        },
    );
    let a = add_flex_child(&mut reg, c, 70.0, 20.0);
    let b = add_flex_child(&mut reg, c, 70.0, 40.0);
    let d = add_flex_child(&mut reg, c, 70.0, 25.0);

    recompute_layouts(&mut reg);

    let ra = reg.get(a).unwrap().layout_rect.as_ref().unwrap();
    let rb = reg.get(b).unwrap().layout_rect.as_ref().unwrap();
    let rd = reg.get(d).unwrap().layout_rect.as_ref().unwrap();

    assert!((ra.y - 0.0).abs() < 0.01);
    assert!((rb.x - 75.0).abs() < 0.01);
    assert!((rb.y - 0.0).abs() < 0.01);
    // Row 2 starts at max_height(40) + gap(5) = 45
    assert!((rd.x - 0.0).abs() < 0.01, "d.x={}", rd.x);
    assert!((rd.y - 45.0).abs() < 0.01, "d.y={}", rd.y);
}

#[test]
fn column_flex_auto_height() {
    let mut reg = FrameRegistry::new(800.0, 600.0);
    let root_id = reg.create_frame("Root", None);
    let root = reg.get_mut(root_id).unwrap();
    root.width = Dimension::Fixed(200.0);
    root.height = Dimension::Fixed(0.0); // auto-height
    root.layout_rect = Some(LayoutRect {
        x: 100.0,
        y: 50.0,
        width: 200.0,
        height: 0.0,
    });
    root.flex_layout = Some(FlexLayout {
        direction: FlexDirection::Column,
        gap: 10.0,
        padding: 20.0,
        ..Default::default()
    });

    let a = reg.create_frame("A", Some(root_id));
    reg.get_mut(a).unwrap().width = Dimension::Fixed(180.0);
    reg.get_mut(a).unwrap().height = Dimension::Fixed(40.0);
    let b = reg.create_frame("B", Some(root_id));
    reg.get_mut(b).unwrap().width = Dimension::Fixed(180.0);
    reg.get_mut(b).unwrap().height = Dimension::Fixed(60.0);
    let c = reg.create_frame("C", Some(root_id));
    reg.get_mut(c).unwrap().width = Dimension::Fixed(180.0);
    reg.get_mut(c).unwrap().height = Dimension::Fixed(30.0);

    recompute_layouts(&mut reg);

    let root_rect = reg.get(root_id).unwrap().layout_rect.as_ref().unwrap();
    // Expected: padding(20) + 40 + gap(10) + 60 + gap(10) + 30 + padding(20) = 190
    assert!(
        (root_rect.height - 190.0).abs() < 1.0,
        "auto-height should be ~190, got {}",
        root_rect.height
    );
}

#[test]
fn auto_height_center_anchor_settles_in_one_pass() {
    let mut reg = FrameRegistry::new(800.0, 600.0);

    // Screen-filling parent
    let screen_id = reg.create_frame("Screen", None);
    let screen = reg.get_mut(screen_id).unwrap();
    screen.width = Dimension::Fixed(800.0);
    screen.height = Dimension::Fixed(600.0);
    screen.layout_rect = Some(LayoutRect {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    });

    // Auto-height panel centered in screen
    let panel_id = reg.create_frame("Panel", Some(screen_id));
    let panel = reg.get_mut(panel_id).unwrap();
    panel.width = Dimension::Fixed(200.0);
    panel.height = Dimension::Fixed(0.0);
    panel.flex_layout = Some(FlexLayout {
        direction: FlexDirection::Column,
        gap: 0.0,
        padding: 10.0,
        ..Default::default()
    });
    panel.anchors.push(Anchor {
        point: AnchorPoint::Center,
        relative_to: None,
        relative_point: AnchorPoint::Center,
        x_offset: 0.0,
        y_offset: 0.0,
    });

    // Two children: 200x50 each
    for name in ["A", "B"] {
        let id = reg.create_frame(name, Some(panel_id));
        let f = reg.get_mut(id).unwrap();
        f.width = Dimension::Fixed(180.0);
        f.height = Dimension::Fixed(50.0);
    }

    recompute_layouts(&mut reg);

    let rect = reg.get(panel_id).unwrap().layout_rect.as_ref().unwrap();
    // Auto-height: padding(10) + 50 + 50 + padding(10) = 120
    assert!((rect.height - 120.0).abs() < 1.0, "height={}", rect.height);
    // Centered: (600 - 120) / 2 = 240
    let expected_y = (600.0 - rect.height) / 2.0;
    assert!(
        (rect.y - expected_y).abs() < 1.0,
        "panel y={} expected={} (not vertically centered)",
        rect.y,
        expected_y
    );
}
