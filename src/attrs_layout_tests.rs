use super::*;

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

#[test]
fn texture_rotation_round_trips_negative_and_positive_degrees() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("Spinner", None);
    registry.get_mut(id).unwrap().widget_data = Some(WidgetData::Texture(Default::default()));
    for degrees in [-65.0, 180.0, -360.0, 0.0] {
        apply(&mut registry, id, "rotation", &degrees.to_string());
        let Some(WidgetData::Texture(texture)) = registry.get(id).unwrap().widget_data.as_ref()
        else {
            panic!("missing spinner texture");
        };
        assert_eq!(texture.rotation, degrees);
        assert_eq!(
            read_attribute(&registry, id, "rotation"),
            Some(degrees.to_string())
        );
    }
}

#[test]
fn hit_rect_insets_reject_malformed_values() {
    for value in ["1,2,3", "1,2,nope,4", "NaN,0,0,0", "0,0,inf,0"] {
        let mut registry = FrameRegistry::new(800.0, 600.0);
        let id = registry.create_frame("Target", None);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            apply(&mut registry, id, "hit_rect_insets", value);
        }));
        assert!(result.is_err(), "malformed hit insets accepted: {value}");
        assert_eq!(registry.get(id).unwrap().hit_rect_insets, [0.0; 4]);
    }
}

#[test]
fn button_hover_alpha_rejects_invalid_values_without_changing_the_button() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("Choice", None);
    registry.get_mut(id).unwrap().widget_data = Some(WidgetData::Button(ButtonData::default()));
    assert_eq!(
        read_attribute(&registry, id, "button_highlight_alpha").as_deref(),
        Some("0.5")
    );
    for value in ["NaN", "inf", "-0.01", "1.01", "nope"] {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            apply(&mut registry, id, "button_highlight_alpha", value);
        }));
        assert!(result.is_err(), "invalid highlight alpha accepted: {value}");
        assert_eq!(
            read_attribute(&registry, id, "button_highlight_alpha").as_deref(),
            Some("0.5")
        );
    }
}

#[test]
fn button_hover_size_round_trips_and_rejects_invalid_dimensions_without_mutation() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("RingedButton", None);
    registry.get_mut(id).unwrap().widget_data = Some(WidgetData::Button(ButtonData::default()));
    assert_eq!(read_attribute(&registry, id, "button_highlight_size"), None);
    apply(&mut registry, id, "button_highlight_size", "99,100");
    assert_eq!(
        read_attribute(&registry, id, "button_highlight_size").as_deref(),
        Some("99,100")
    );
    for value in [
        "",
        "99",
        "99,100,101",
        "NaN,100",
        "inf,100",
        "0,100",
        "99,-1",
        "oops,100",
    ] {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            apply(&mut registry, id, "button_highlight_size", value);
        }));
        assert!(result.is_err(), "invalid highlight size accepted: {value}");
        assert_eq!(
            read_attribute(&registry, id, "button_highlight_size").as_deref(),
            Some("99,100"),
            "invalid input changed previous value: {value}"
        );
    }
}

#[test]
fn button_default_skin_is_authored_and_can_be_switched_off() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("InteractionButton", None);
    registry.get_mut(id).unwrap().widget_data = Some(WidgetData::Button(ButtonData::default()));
    assert_eq!(
        read_attribute(&registry, id, "button_default_skin").as_deref(),
        Some("true")
    );
    apply(&mut registry, id, "button_default_skin", "false");
    assert_eq!(
        read_attribute(&registry, id, "button_default_skin").as_deref(),
        Some("false")
    );
}

#[test]
fn native_layout_attributes_round_trip() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("Positioned", None);
    let attributes = [
        ("pos_x", "12"),
        ("pos_y", "-8"),
        ("pos_type", "absolute"),
        ("anchor", "screen"),
        ("right", "25%"),
        ("bottom", "auto"),
        ("translate_x", "-50%"),
        ("translate_y", "4"),
        ("margin_left", "auto"),
        ("margin_right", "8"),
        ("margin_top", "2%"),
        ("margin_bottom", "-3"),
        ("width", "auto"),
        ("height", "auto"),
        ("hit_rect_insets", "15,15,15,15"),
    ];
    for (name, value) in attributes {
        apply(&mut registry, id, name, value);
        assert_eq!(read_attribute(&registry, id, name).as_deref(), Some(value));
    }
    let frame = registry.get(id).unwrap();
    assert_eq!(frame.position.left, Val::Px(12.0));
    assert_eq!(frame.position.top, Val::Px(-8.0));
    assert_eq!(frame.position.right, Val::Percent(25.0));
    assert_eq!(frame.position.bottom, Val::Auto);
    assert_eq!(frame.position_type, PositionType::Absolute);
    assert_eq!(frame.anchor, AnchorTarget::Screen);
    assert_eq!(frame.translation.x, Val::Percent(-50.0));
    assert_eq!(frame.translation.y, Val::Px(4.0));
    assert_eq!(frame.margin.left, Val::Auto);
    assert_eq!(frame.margin.right, Val::Px(8.0));
    assert_eq!(frame.margin.top, Val::Percent(2.0));
    assert_eq!(frame.margin.bottom, Val::Px(-3.0));
    assert!(matches!(frame.width, Dimension::Auto));
    assert!(matches!(frame.height, Dimension::Auto));
}

#[test]
fn native_edges_accept_pixels_percent_and_auto() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("Edges", None);
    for name in [
        "left",
        "right",
        "top",
        "bottom",
        "translate_x",
        "translate_y",
        "margin_left",
        "margin_right",
        "margin_top",
        "margin_bottom",
    ] {
        for (input, expected) in [
            ("7", Val::Px(7.0)),
            ("-25%", Val::Percent(-25.0)),
            ("auto", Val::Auto),
        ] {
            apply(&mut registry, id, name, input);
            assert_eq!(
                *layout_val_mut(registry.get_mut(id).unwrap(), name).unwrap(),
                expected
            );
        }
    }
    apply(&mut registry, id, "pos_type", "relative");
    apply(&mut registry, id, "anchor", "parent");
    assert_eq!(
        registry.get(id).unwrap().position_type,
        PositionType::Relative
    );
    assert_eq!(registry.get(id).unwrap().anchor, AnchorTarget::Parent);
}

#[test]
fn defaults_target_parent_without_positioning() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let parent = registry.create_frame("Parent", None);
    let id = registry.create_frame("Child", Some(parent));
    let frame = registry.get(id).unwrap();
    assert_eq!(frame.anchor, AnchorTarget::Parent);
    assert_eq!(frame.position_type, PositionType::Relative);
    assert_eq!(frame.position, bevy::prelude::UiRect::AUTO);
    assert_eq!(frame.translation, bevy::prelude::Val2::ZERO);
    assert_eq!(frame.margin, bevy::prelude::UiRect::ZERO);
}

#[test]
fn stretch_uses_native_full_insets() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("Stretch", None);
    apply(&mut registry, id, "width", "50");
    apply(&mut registry, id, "height", "20");
    apply(&mut registry, id, "stretch", "true");
    let frame = registry.get(id).unwrap();
    assert_eq!(frame.position, bevy::prelude::UiRect::ZERO);
    assert_eq!(frame.position_type, PositionType::Absolute);
    assert!(matches!(frame.width, Dimension::Auto));
    assert!(matches!(frame.height, Dimension::Auto));
}

#[test]
#[should_panic(expected = "invalid anchor")]
fn rejects_arbitrary_frame_name_anchor() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    registry.create_frame("Other", None);
    let id = registry.create_frame("Frame", None);
    apply(&mut registry, id, "anchor", "Other");
}

#[test]
#[should_panic(expected = "invalid pos_x")]
fn pixel_position_rejects_percent() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("Frame", None);
    apply(&mut registry, id, "pos_x", "50%");
}

#[test]
fn dimensions_and_flex_attributes_remain_independent() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("Row", None);
    for (name, value) in [
        ("width", "120"),
        ("height", "35"),
        ("layout", "flex-row"),
        ("gap", "17"),
    ] {
        apply(&mut registry, id, name, value);
    }
    let frame = registry.get(id).unwrap();
    assert!(matches!(frame.width, Dimension::Fixed(120.0)));
    assert!(matches!(frame.height, Dimension::Fixed(35.0)));
    let flex = frame.flex_layout.as_ref().unwrap();
    assert!(matches!(flex.direction, FlexDirection::Row));
    assert_eq!(flex.gap, 17.0);
}
