use super::*;
use crate::anchor::{Anchor, AnchorPoint};
use crate::layout::recompute_layouts;
use crate::widget_def::{AnchorDef, Attr, WidgetDef};

fn named_widget(tag: &'static str, name: &str) -> WidgetDef {
    let mut widget = WidgetDef::new(tag);
    widget.name = Some(name.to_string());
    widget
}

fn top_left_anchor(relative_to: &str, relative_point: &str, x: &str) -> AnchorDef {
    AnchorDef {
        point: "TOPLEFT".to_string(),
        relative_to: relative_to.to_string(),
        relative_point: relative_point.to_string(),
        x: x.to_string(),
        y: "0".to_string(),
    }
}

fn sizing_screen(tag: &'static str) -> Screen {
    Screen::new(move |_| {
        let mut widget = named_widget(tag, "Sized");
        widget.attrs = vec![
            Attr::new_static("text", "Hi".to_string()),
            Attr::new_static("font_size", "20".to_string()),
        ];
        widget
            .anchors
            .push(top_left_anchor("$parent", "TOPLEFT", "12"));
        vec![WidgetChild::Widget(widget)]
    })
}

fn add_follower(registry: &mut FrameRegistry, target: u64) -> u64 {
    let follower = registry.create_frame("Follower", None);
    let frame = registry.get_mut(follower).unwrap();
    frame.width = Dimension::Fixed(10.0);
    frame.height = Dimension::Fixed(10.0);
    registry
        .set_point(
            follower,
            Anchor {
                point: AnchorPoint::TopLeft,
                relative_to: Some(target),
                relative_point: AnchorPoint::BottomRight,
                x_offset: 3.0,
                y_offset: 0.0,
            },
        )
        .unwrap();
    follower
}

fn assert_follower_at_bottom_right(registry: &FrameRegistry, sized: u64, follower: u64) {
    let target = registry.get(sized).unwrap().layout_rect.as_ref().unwrap();
    let follower = registry
        .get(follower)
        .unwrap()
        .layout_rect
        .as_ref()
        .unwrap();
    assert_eq!(
        (follower.x, follower.y),
        (target.x + target.width + 3.0, target.y + target.height)
    );
}

#[test]
fn screen_layout_text_autosize_updates_settled_layout_and_follower() {
    let mut screen = sizing_screen("FontString");
    let ctx = SharedContext::new();
    let mut registry = FrameRegistry::new(800.0, 600.0);
    screen.sync(&ctx, &mut registry);
    let label = registry.get_by_name("Sized").unwrap();
    let follower = add_follower(&mut registry, label);
    recompute_layouts(&mut registry);
    let original_width = registry
        .get(label)
        .unwrap()
        .layout_rect
        .as_ref()
        .unwrap()
        .width;
    let frame = registry.get_mut(label).unwrap();
    frame.width = Dimension::Fixed(0.0);
    let Some(WidgetData::FontString(text)) = &mut frame.widget_data else {
        panic!("expected FontString");
    };
    text.text = "A much longer label".to_string();
    registry.create_frame("UnrelatedDirtyFrame", None);
    screen.sync(&ctx, &mut registry);
    let desired_width = registry.get(label).unwrap().width.value();
    assert!(desired_width > original_width);
    recompute_layouts(&mut registry);
    assert_eq!(
        registry
            .get(label)
            .unwrap()
            .layout_rect
            .as_ref()
            .unwrap()
            .width,
        desired_width
    );
    assert_follower_at_bottom_right(&registry, label, follower);
    registry.render_dirty.clear();
    screen.sync(&ctx, &mut registry);
    recompute_layouts(&mut registry);
    assert!(registry.render_dirty.is_empty());
}

#[test]
fn screen_layout_editbox_autosize_updates_settled_layout_and_follower() {
    let mut screen = sizing_screen("EditBox");
    let ctx = SharedContext::new();
    let mut registry = FrameRegistry::new(800.0, 600.0);
    screen.sync(&ctx, &mut registry);
    let input = registry.get_by_name("Sized").unwrap();
    let follower = add_follower(&mut registry, input);
    recompute_layouts(&mut registry);
    let frame = registry.get_mut(input).unwrap();
    frame.height = Dimension::Fixed(0.0);
    let Some(WidgetData::EditBox(editbox)) = &mut frame.widget_data else {
        panic!("expected EditBox");
    };
    editbox.font_size = 30.0;
    editbox.text_insets = [1.0, 2.0, 3.0, 3.0];
    registry.create_frame("UnrelatedDirtyFrame", None);
    screen.sync(&ctx, &mut registry);
    recompute_layouts(&mut registry);
    assert_eq!(
        registry
            .get(input)
            .unwrap()
            .layout_rect
            .as_ref()
            .unwrap()
            .height,
        50.0
    );
    assert_follower_at_bottom_right(&registry, input, follower);
    registry.render_dirty.clear();
    screen.sync(&ctx, &mut registry);
    recompute_layouts(&mut registry);
    assert!(registry.render_dirty.is_empty());
}

#[test]
fn screen_layout_pending_named_anchor_follows_target_after_first_layout() {
    let mut screen = Screen::new(|_| {
        let mut follower = named_widget("Frame", "Follower");
        follower
            .anchors
            .push(top_left_anchor("Target", "TOPLEFT", "3"));
        let mut target = named_widget("Frame", "Target");
        target
            .anchors
            .push(top_left_anchor("$parent", "TOPLEFT", "10"));
        vec![WidgetChild::Widget(follower), WidgetChild::Widget(target)]
    });
    let mut registry = FrameRegistry::new(800.0, 600.0);
    screen.sync(&SharedContext::new(), &mut registry);
    let target = registry.get_by_name("Target").unwrap();
    let follower = registry.get_by_name("Follower").unwrap();
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
    registry
        .set_point(
            target,
            Anchor {
                point: AnchorPoint::TopLeft,
                relative_to: None,
                relative_point: AnchorPoint::TopLeft,
                x_offset: 40.0,
                y_offset: 0.0,
            },
        )
        .unwrap();
    recompute_layouts(&mut registry);
    assert_eq!(
        registry
            .get(follower)
            .unwrap()
            .layout_rect
            .as_ref()
            .unwrap()
            .x,
        43.0
    );
}

#[test]
fn screen_layout_autosize_preserves_explicit_unanchored_position() {
    let mut screen = Screen::new(|_| vec![WidgetChild::Widget(named_widget("EditBox", "Input"))]);
    let ctx = SharedContext::new();
    let mut registry = FrameRegistry::new(800.0, 600.0);
    screen.sync(&ctx, &mut registry);
    let input = registry.get_by_name("Input").unwrap();
    let frame = registry.get_mut(input).unwrap();
    frame.layout_rect = Some(crate::layout::LayoutRect {
        x: 73.0,
        y: 41.0,
        width: 120.0,
        height: 18.0,
    });
    frame.height = Dimension::Fixed(0.0);
    screen.sync(&ctx, &mut registry);
    recompute_layouts(&mut registry);
    let rect = registry.get(input).unwrap().layout_rect.as_ref().unwrap();
    assert_eq!((rect.x, rect.y), (73.0, 41.0));
    assert!(registry.get(input).unwrap().height.value() > 0.0);
}
