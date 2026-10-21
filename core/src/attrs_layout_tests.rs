use super::*;
use crate::widgets::button::ButtonData;

fn apply(registry: &mut FrameRegistry, id: u64, name: &str, value: &str) {
    apply_attribute(registry, id, name, value);
}

// R SharedUIPanelTemplates.xml and SecureUIPanelTemplates.xml author these state presets.
#[test]
fn ui_template_art_named_buttons_supply_authored_state_sources() {
    for (template, normal, pressed, disabled) in [
        (
            "SharedButtonTemplate",
            TextureSource::Atlas("128-redbutton-left".into()),
            TextureSource::Atlas("128-redbutton-left-pressed".into()),
            TextureSource::Atlas("128-redbutton-left-disabled".into()),
        ),
        (
            "UIPanelButtonTemplate",
            TextureSource::FileDataId(130828),
            TextureSource::FileDataId(130825),
            TextureSource::FileDataId(130824),
        ),
        (
            "UIPanelCloseButton",
            TextureSource::Atlas("RedButton-Exit".into()),
            TextureSource::Atlas("RedButton-exit-pressed".into()),
            TextureSource::Atlas("RedButton-Exit-Disabled".into()),
        ),
    ] {
        let mut registry = FrameRegistry::new(800.0, 600.0);
        let id = registry.create_frame(template, None);
        registry.get_mut(id).unwrap().widget_data = Some(WidgetData::Button(Default::default()));
        apply(&mut registry, id, "button_template", template);
        let Some(WidgetData::Button(button)) = &registry.get(id).unwrap().widget_data else {
            panic!("button")
        };
        assert_eq!(
            button.normal_texture.as_ref().map(|art| &art.source),
            Some(&normal),
            "{template}"
        );
        assert_eq!(
            button.pushed_texture.as_ref().map(|art| &art.source),
            Some(&pressed),
            "{template}"
        );
        assert_eq!(
            button.disabled_texture.as_ref().map(|art| &art.source),
            Some(&disabled),
            "{template}"
        );
        assert!(!button.use_default_skin, "{template}");
    }
}

#[test]
fn ui_template_art_button_templates_resolve_before_local_regions_and_reset_on_rebuild() {
    use crate::widget_def::{Attr, WidgetChild, WidgetDef};
    use crate::widget_def_diff::DiffContext;
    for template_first in [false, true] {
        let mut registry = FrameRegistry::new(800.0, 600.0);
        let mut diff = DiffContext::new();
        let mut definition = WidgetDef::new("button");
        definition.name = Some("TemplateOverrides".into());
        definition.attrs = vec![
            Attr::new_static("button_highlight_alpha", "0.4".into()),
            Attr::new_static("button_normal_vertex_color", "0.2,0.3,0.4,0.5".into()),
        ];
        let template = Attr::new_static("button_template", "UIPanelButtonTemplate".into());
        if template_first {
            definition.attrs.insert(0, template);
        } else {
            definition.attrs.push(template);
        }
        diff.diff_roots(&[WidgetChild::Widget(definition)], None, &mut registry);
        let id = registry.get_by_name("TemplateOverrides").unwrap();
        let Some(WidgetData::Button(button)) = &registry.get(id).unwrap().widget_data else {
            panic!("button")
        };
        assert_eq!(button.highlight_texture.as_ref().unwrap().alpha, 0.4);
        assert_eq!(
            button.normal_texture.as_ref().unwrap().vertex_color,
            [0.2, 0.3, 0.4, 0.5]
        );
        let mut definition = WidgetDef::new("button");
        definition.name = Some("TemplateOverrides".into());
        definition.attrs = vec![Attr::new_static(
            "button_template",
            "UIPanelCloseButton".into(),
        )];
        diff.diff_roots(&[WidgetChild::Widget(definition)], None, &mut registry);
        let Some(WidgetData::Button(button)) = &registry.get(id).unwrap().widget_data else {
            panic!("button")
        };
        assert_eq!(button.highlight_texture.as_ref().unwrap().alpha, 1.0);
        assert_eq!(
            button.normal_texture.as_ref().unwrap().vertex_color,
            [1.0; 4]
        );
        assert_eq!(
            button.normal_texture.as_ref().unwrap().source,
            TextureSource::Atlas("RedButton-Exit".into())
        );
    }
}

/// R UIButtonTemplate.lua: SetButtonArtKit authors the same four suffix rules for each kit.
#[test]
fn ui_template_art_button_art_kit_supplies_single_region_states_without_name_guesses() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("CartToggle", None);
    registry.get_mut(id).unwrap().widget_data = Some(WidgetData::Button(Default::default()));
    apply(
        &mut registry,
        id,
        "button_art_kit",
        "128-redbutton-cart-add",
    );
    let Some(WidgetData::Button(button)) = &registry.get(id).unwrap().widget_data else {
        panic!("button")
    };
    assert_eq!(
        button.normal_texture.as_ref().map(|art| &art.source),
        Some(&TextureSource::Atlas("128-redbutton-cart-add".into()))
    );
    assert_eq!(
        button.pushed_texture.as_ref().map(|art| &art.source),
        Some(&TextureSource::Atlas(
            "128-redbutton-cart-add-Pressed".into()
        ))
    );
    assert_eq!(
        button.highlight_texture.as_ref().map(|art| &art.source),
        Some(&TextureSource::Atlas(
            "128-redbutton-cart-add-Highlight".into()
        ))
    );
    assert!(!button.use_default_skin);
}

// R FontStyles.xml56–60 → Fonts.xml39–43: RomanFriz10, gold, black(+1,-1) shadow.
#[test]
fn authored_font_object_small_resolves_its_inherited_face_size_color_and_shadow() {
    use crate::widgets::font_string::{GameFont, JustifyH};
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("AchievementFrameTitle", None);
    registry.get_mut(id).unwrap().widget_data = Some(WidgetData::FontString(Default::default()));
    apply(&mut registry, id, "font_object", "GameFontNormalSmall");
    let Some(WidgetData::FontString(font)) = &registry.get(id).unwrap().widget_data else {
        panic!("font")
    };
    assert_eq!(font.font, GameFont::FrizQuadrata);
    assert_eq!(font.font_size, 10.0);
    assert_eq!(font.color, [1.0, 0.82, 0.0, 1.0]);
    assert_eq!(font.shadow_color, Some([0.0, 0.0, 0.0, 1.0]));
    assert_eq!(font.shadow_offset, [1.0, -1.0]);
    assert_eq!(font.justify_h, JustifyH::Center);
    assert_eq!(
        read_attribute(&registry, id, "font_object").as_deref(),
        Some("GameFontNormalSmall")
    );
}

#[test]
fn authored_font_object_declarations_preserve_local_overrides_in_either_order_and_rebuild() {
    use crate::widget_def::{Attr, WidgetChild, WidgetDef};
    use crate::widget_def_diff::DiffContext;
    for inherited_first in [false, true] {
        let mut registry = FrameRegistry::new(800.0, 600.0);
        let mut diff = DiffContext::new();
        let mut definition = WidgetDef::new("fontstring");
        definition.name = Some("LocalTitle".into());
        definition.attrs = [
            ("font", "ArialNarrow"),
            ("font_color", "0.2,0.4,0.8,1"),
            ("shadow_color", "0.1,0.2,0.3,0.5"),
            ("shadow_offset", "2,-2"),
            ("font_size", "13.5"),
        ]
        .into_iter()
        .map(|(name, value)| Attr::new_static(name, value.into()))
        .collect();
        let inherited = Attr::new_static("font_object", "GameFontNormalSmall".into());
        if inherited_first {
            definition.attrs.insert(0, inherited);
        } else {
            definition.attrs.push(inherited);
        }
        diff.diff_roots(&[WidgetChild::Widget(definition)], None, &mut registry);
        let id = registry.get_by_name("LocalTitle").unwrap();
        let Some(WidgetData::FontString(font)) = &registry.get(id).unwrap().widget_data else {
            panic!("font")
        };
        assert_eq!(
            font.font,
            crate::widgets::font_string::GameFont::ArialNarrow
        );
        assert_eq!(font.font_size, 13.5);
        assert_eq!(font.color, [0.2, 0.4, 0.8, 1.0]);
        assert_eq!(font.shadow_color, Some([0.1, 0.2, 0.3, 0.5]));
        assert_eq!(font.shadow_offset, [2.0, -2.0]);
        let mut changed = WidgetDef::new("fontstring");
        changed.name = Some("LocalTitle".into());
        changed
            .attrs
            .push(Attr::new_static("font_object", "GameFontHighlight".into()));
        diff.diff_roots(&[WidgetChild::Widget(changed)], None, &mut registry);
        assert_eq!(registry.get_by_name("LocalTitle"), Some(id));
        let Some(WidgetData::FontString(font)) = &registry.get(id).unwrap().widget_data else {
            panic!("font")
        };
        assert_eq!(
            font.font,
            crate::widgets::font_string::GameFont::FrizQuadrata
        );
        assert_eq!(font.font_size, 12.0);
        assert_eq!(font.color, [1.0; 4]);
        assert_eq!(font.shadow_color, Some([0.0, 0.0, 0.0, 1.0]));
        assert_eq!(font.shadow_offset, [1.0, -1.0]);
    }
}

#[test]
fn authored_font_object_rejects_unknown_names_without_replacing_existing_style() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("StrictFont", None);
    registry.get_mut(id).unwrap().widget_data = Some(WidgetData::FontString(Default::default()));
    apply(&mut registry, id, "font_object", "GameFontNormalSmall");
    let before = registry.get(id).unwrap().widget_data.clone();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply(&mut registry, id, "font_object", "NotARetailFontObject");
    }));
    assert!(result.is_err());
    assert_eq!(registry.get(id).unwrap().widget_data, before);
}

// R PerksProgramElements.xml897: source-identical highlight authors alpha0.6;
// GuildBankUI.xml156 authors CheckedTexture with independent ADD metadata.
#[test]
fn authored_state_region_metadata_survives_source_updates_in_either_order() {
    for (state, source_attr) in [
        ("normal", "button_fdid_up"),
        ("pushed", "button_fdid_pressed"),
        ("disabled", "button_fdid_disabled"),
        ("highlight", "button_fdid_highlight"),
        ("checked", "button_fdid_checked"),
        ("disabled_checked", "button_fdid_disabled_checked"),
    ] {
        for source_first in [false, true] {
            let mut registry = FrameRegistry::new(800.0, 600.0);
            let id = registry.create_frame("AuthoredStateRegion", None);
            registry.get_mut(id).unwrap().widget_data =
                Some(WidgetData::Button(Default::default()));
            if source_first {
                apply(&mut registry, id, source_attr, "130724");
            }
            for (field, value) in [
                ("tex_coords", "0.125,0.875,0.25,0.75"),
                ("vertex_color", "0.2,0.4,0.8,0.5"),
                ("alpha", "0.6"),
                ("blend_mode", "ADD"),
            ] {
                apply(&mut registry, id, &format!("button_{state}_{field}"), value);
            }
            apply(&mut registry, id, source_attr, "130718");
            for (field, expected) in [
                ("tex_coords", "0.125,0.875,0.25,0.75"),
                ("vertex_color", "0.2,0.4,0.8,0.5"),
                ("alpha", "0.6"),
                ("blend_mode", "ADD"),
            ] {
                let attr = format!("button_{state}_{field}");
                assert_eq!(
                    read_attribute(&registry, id, &attr).as_deref(),
                    Some(expected),
                    "{attr}, source_first={source_first}"
                );
            }
        }
    }
}

#[test]
fn authored_state_region_checked_state_survives_disabled_and_source_changes() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("GuildBankCheckedTab", None);
    registry.get_mut(id).unwrap().widget_data = Some(WidgetData::Button(Default::default()));
    assert_eq!(
        read_attribute(&registry, id, "button_checked").as_deref(),
        Some("false")
    );
    apply(&mut registry, id, "button_checked", "true");
    apply(&mut registry, id, "button_fdid_checked", "130724");
    apply(&mut registry, id, "button_checked_blend_mode", "ADD");
    apply(&mut registry, id, "disabled", "true");
    apply(&mut registry, id, "button_fdid_checked", "130718");
    assert_eq!(
        read_attribute(&registry, id, "button_checked").as_deref(),
        Some("true")
    );
    assert_eq!(
        read_attribute(&registry, id, "button_checked_blend_mode").as_deref(),
        Some("ADD")
    );
    apply(&mut registry, id, "button_checked", "false");
    assert_eq!(
        read_attribute(&registry, id, "button_checked").as_deref(),
        Some("false")
    );
}

#[test]
fn authored_state_region_invalid_metadata_rejects_without_partial_mutation() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("StrictStateRegion", None);
    registry.get_mut(id).unwrap().widget_data = Some(WidgetData::Button(Default::default()));
    apply(&mut registry, id, "button_fdid_up", "130824");
    let before = registry.get(id).unwrap().widget_data.clone();
    for (name, value) in [
        ("button_normal_alpha", "NaN"),
        ("button_normal_alpha", "1.5"),
        ("button_normal_tex_coords", "0,NaN,0,1"),
        ("button_checked_vertex_color", "0,0,0,infinity"),
        ("button_checked_blend_mode", "MOD"),
    ] {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            apply(&mut registry, id, name, value);
        }));
        assert!(result.is_err(), "accepted invalid {name}={value}");
        assert_eq!(registry.get(id).unwrap().widget_data, before);
    }
}

// R/Blizzard_Transmog/Blizzard_TransmogTemplates.xml858/872: identical
// atlas regions authored as ordinary Border and additive BorderHighlight.
#[test]
fn authored_blend_mode_preserves_distinct_modes_for_the_same_atlas() {
    use crate::widgets::texture::{BlendMode, TextureData, TextureSource};
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let source = TextureSource::Atlas("transmog-itemcard-default".into());
    for (name, attribute, expected) in [
        ("Border", None, BlendMode::Blend),
        ("BorderExplicit", Some("BLEND"), BlendMode::Blend),
        ("BorderHighlight", Some("ADD"), BlendMode::Additive),
    ] {
        let id = registry.create_frame(name, None);
        registry.get_mut(id).unwrap().widget_data = Some(WidgetData::Texture(TextureData {
            source: source.clone(),
            ..Default::default()
        }));
        if let Some(value) = attribute {
            apply(&mut registry, id, "blend_mode", value);
        }
        let Some(WidgetData::Texture(texture)) = registry.get(id).unwrap().widget_data.as_ref()
        else {
            panic!("authored texture")
        };
        assert_eq!(texture.source, source);
        assert_eq!(texture.blend_mode, expected, "{name}");
        assert_eq!(texture.vertex_color, [1.0; 4]);
        assert_eq!(texture.tex_coords, [0.0, 1.0, 0.0, 1.0]);
    }
}

#[test]
fn authored_blend_mode_rejects_unsupported_modes_without_mutation() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("UnsupportedRegion", None);
    registry.get_mut(id).unwrap().widget_data = Some(WidgetData::Texture(Default::default()));
    let before = registry.get(id).unwrap().widget_data.clone();
    for value in ["MOD", "DISABLE", "ALPHAKEY", "not-a-mode"] {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            apply(&mut registry, id, "blend_mode", value);
        }));
        assert!(result.is_err(), "accepted unsupported blend_mode {value}");
        assert_eq!(registry.get(id).unwrap().widget_data, before);
    }
}

// R SharedUIPanelTemplates.xml3 authors the button highlight with ADD.
#[test]
fn authored_button_blend_mode_survives_source_attribute_order() {
    for attributes in [
        [
            ("button_fdid_highlight", "130826"),
            ("button_highlight_blend_mode", "ADD"),
        ],
        [
            ("button_highlight_blend_mode", "ADD"),
            ("button_fdid_highlight", "130826"),
        ],
    ] {
        let mut registry = FrameRegistry::new(800.0, 600.0);
        let id = registry.create_frame("PanelButton", None);
        registry.get_mut(id).unwrap().widget_data = Some(WidgetData::Button(Default::default()));
        for (name, value) in attributes {
            apply(&mut registry, id, name, value);
        }
        assert_eq!(
            read_attribute(&registry, id, "button_highlight_blend_mode").as_deref(),
            Some("ADD")
        );
        let Some(WidgetData::Button(button)) = &registry.get(id).unwrap().widget_data else {
            panic!("expected authored panel button");
        };
        let highlight = button.highlight_texture.as_ref().unwrap();
        assert_eq!(highlight.source, TextureSource::FileDataId(130826));
        assert_eq!(highlight.blend_mode, BlendMode::Additive);
    }
}

#[test]
fn authored_button_blend_mode_defaults_to_blend_including_highlights() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("UnadornedHighlight", None);
    registry.get_mut(id).unwrap().widget_data = Some(WidgetData::Button(Default::default()));
    apply(&mut registry, id, "button_fdid_highlight", "130826");
    assert_eq!(
        read_attribute(&registry, id, "button_highlight_blend_mode").as_deref(),
        Some("BLEND")
    );
}

#[test]
fn justify_v_attribute_updates_font_string_vertical_alignment() {
    use crate::widgets::font_string::{FontStringData, JustifyV};
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("Description", None);
    registry.get_mut(id).unwrap().widget_data =
        Some(WidgetData::FontString(FontStringData::default()));
    for (value, expected) in [
        ("TOP", JustifyV::Top),
        ("BOTTOM", JustifyV::Bottom),
        ("MIDDLE", JustifyV::Middle),
    ] {
        apply(&mut registry, id, "justify_v", value);
        let Some(WidgetData::FontString(font)) = registry.get(id).unwrap().widget_data.as_ref()
        else {
            panic!("description FontString");
        };
        assert_eq!(font.justify_v, expected, "justify_v={value}");
    }
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
fn texture_rotation_rejects_malformed_values_without_mutating_the_texture() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("Spinner", None);
    registry.get_mut(id).unwrap().widget_data = Some(WidgetData::Texture(Default::default()));
    apply(&mut registry, id, "rotation", "22.5");
    for value in ["nope", "NaN", "inf", "-inf"] {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            apply(&mut registry, id, "rotation", value);
        }));
        assert!(result.is_err(), "accepted invalid rotation {value}");
        assert_eq!(
            read_attribute(&registry, id, "rotation").as_deref(),
            Some("22.5")
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
        Some("1")
    );
    for value in ["NaN", "inf", "-0.01", "1.01", "nope"] {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            apply(&mut registry, id, "button_highlight_alpha", value);
        }));
        assert!(result.is_err(), "invalid highlight alpha accepted: {value}");
        assert_eq!(
            read_attribute(&registry, id, "button_highlight_alpha").as_deref(),
            Some("1")
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
    assert_eq!(frame.position, crate::layout_values::UiRect::AUTO);
    assert_eq!(frame.translation, crate::layout_values::Val2::ZERO);
    assert_eq!(frame.margin, crate::layout_values::UiRect::ZERO);
}

#[test]
fn stretch_uses_native_full_insets() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("Stretch", None);
    apply(&mut registry, id, "width", "50");
    apply(&mut registry, id, "height", "20");
    apply(&mut registry, id, "stretch", "true");
    let frame = registry.get(id).unwrap();
    assert_eq!(frame.position, crate::layout_values::UiRect::ZERO);
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
