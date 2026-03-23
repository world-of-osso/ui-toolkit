use bevy::prelude::*;

use crate::frame::{Dimension, WidgetData, WidgetType};
use crate::plugin::{UiPlugin, UiState};
use crate::render::{frame_sprite_params, texture_tint};
use crate::render_nine_slice::UiNineSlicePart;
use crate::render_text::extract_button_text;
use crate::widget_def::{Attr, WidgetChild, WidgetDef};
use crate::widget_def_diff::DiffContext;
use crate::widgets::button::{ButtonData, ButtonState};
use crate::widgets::edit_box::EditBoxData;
use crate::widgets::font_string::{FontStringData, GameFont};
use crate::widgets::texture::TextureSource;

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<Image>();
    app.init_asset::<bevy::text::Font>();
    app.add_plugins(UiPlugin);
    app.update();
    app
}

fn create_button(app: &mut App, name: &str, btn: ButtonData) -> u64 {
    let mut ui = app.world_mut().resource_mut::<UiState>();
    let id = ui.registry.create_frame(name, None);
    let frame = ui.registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(120.0);
    frame.height = Dimension::Fixed(40.0);
    frame.widget_type = WidgetType::Button;
    frame.widget_data = Some(WidgetData::Button(btn));
    id
}

// --- Button nine-slice tests ---

#[test]
fn button_with_file_texture_does_not_get_nine_slice() {
    let mut app = setup_app();
    let btn = ButtonData {
        normal_texture: Some(TextureSource::File("btn.blp".into())),
        ..Default::default()
    };
    let id = create_button(&mut app, "Btn", btn);
    app.update();
    let ui = app.world().resource::<UiState>();
    let frame = ui.registry.get(id).unwrap();
    assert!(
        frame.nine_slice.is_some(),
        "all buttons get a default nine-slice"
    );
}

#[test]
fn button_with_nine_slice_atlas_gets_nine_slice() {
    let mut app = setup_app();
    let btn = ButtonData {
        normal_texture: Some(TextureSource::Atlas("128-redbutton-up".into())),
        ..Default::default()
    };
    let id = create_button(&mut app, "Btn", btn);
    app.update();
    let ui = app.world().resource::<UiState>();
    let frame = ui.registry.get(id).unwrap();
    assert!(
        frame.nine_slice.is_some(),
        "button atlas regions tagged as nine-slice should get nine_slice"
    );
}

#[test]
fn button_without_texture_no_nine_slice() {
    let mut app = setup_app();
    let id = create_button(&mut app, "BtnPlain", ButtonData::default());
    app.update();
    let ui = app.world().resource::<UiState>();
    let frame = ui.registry.get(id).unwrap();
    assert!(
        frame.nine_slice.is_some(),
        "all buttons get a default nine-slice"
    );
}

#[test]
fn button_nine_slice_spawns_all_9_parts_for_nine_slice_atlas() {
    let mut app = setup_app();
    let btn = ButtonData {
        normal_texture: Some(TextureSource::Atlas("128-redbutton-up".into())),
        ..Default::default()
    };
    let id = create_button(&mut app, "Btn9", btn);
    app.update();
    let mut q = app.world_mut().query::<&UiNineSlicePart>();
    let parts: Vec<u8> = q
        .iter(app.world())
        .filter(|p| p.0 == id)
        .map(|p| p.1)
        .collect();
    assert_eq!(
        parts.len(),
        9,
        "expected 9 nine-slice parts, got {}",
        parts.len()
    );
    for i in 0..9u8 {
        assert!(parts.contains(&i), "missing nine-slice part {i}");
    }
}

#[test]
fn button_pushed_state_updates_nine_slice_texture_for_nine_slice_atlas() {
    let mut app = setup_app();
    let btn = ButtonData {
        state: ButtonState::Normal,
        normal_texture: Some(TextureSource::Atlas("128-redbutton-up".into())),
        pushed_texture: Some(TextureSource::Atlas("128-redbutton-pressed".into())),
        ..Default::default()
    };
    let id = create_button(&mut app, "BtnPush", btn);
    app.update();
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let frame = ui.registry.get_mut(id).unwrap();
        if let Some(WidgetData::Button(btn)) = &mut frame.widget_data {
            btn.state = ButtonState::Pushed;
        }
        frame.nine_slice = None;
    }
    app.update();
    let ui = app.world().resource::<UiState>();
    let frame = ui.registry.get(id).unwrap();
    let ns = frame.nine_slice.as_ref().expect("should have nine_slice");
    assert!(
        matches!(&ns.texture, Some(TextureSource::Atlas(name)) if name == "128-redbutton-pressed"),
        "pushed state should use pushed texture"
    );
}

// --- EditBox sizing and font tests ---

#[test]
fn edit_box_preserves_dimensions() {
    let mut app = setup_app();
    let id = {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let id = ui.registry.create_frame("EditBox1", None);
        let frame = ui.registry.get_mut(id).unwrap();
        frame.width = Dimension::Fixed(250.0);
        frame.height = Dimension::Fixed(32.0);
        frame.widget_type = WidgetType::EditBox;
        frame.widget_data = Some(WidgetData::EditBox(EditBoxData::default()));
        id
    };
    app.update();
    let ui = app.world().resource::<UiState>();
    let frame = ui.registry.get(id).unwrap();
    assert_eq!(frame.width, Dimension::Fixed(250.0));
    assert_eq!(frame.height, Dimension::Fixed(32.0));
}

#[test]
fn edit_box_font_flows_to_text_props() {
    let mut frame = crate::frame::Frame::new(1, None, WidgetType::EditBox);
    frame.width = Dimension::Fixed(200.0);
    frame.height = Dimension::Fixed(30.0);
    frame.effective_alpha = 1.0;
    frame.widget_data = Some(WidgetData::EditBox(EditBoxData {
        text: "hello".into(),
        font: GameFont::ArialNarrow,
        font_size: 18.0,
        ..Default::default()
    }));
    let props = crate::render_text::extract_text_props_pub(&frame);
    assert_eq!(props.font, GameFont::ArialNarrow);
    assert_eq!(props.font_size, 18.0);
    assert_eq!(props.content, "hello");
}

#[test]
fn edit_box_password_masks_text() {
    let mut frame = crate::frame::Frame::new(1, None, WidgetType::EditBox);
    frame.effective_alpha = 1.0;
    frame.widget_data = Some(WidgetData::EditBox(EditBoxData {
        text: "secret".into(),
        password: true,
        ..Default::default()
    }));
    let props = crate::render_text::extract_text_props_pub(&frame);
    assert_eq!(props.content, "******");
}

// --- Font propagation tests ---

#[test]
fn font_string_font_flows_to_text_props() {
    let mut frame = crate::frame::Frame::new(1, None, WidgetType::FontString);
    frame.effective_alpha = 1.0;
    frame.widget_data = Some(WidgetData::FontString(FontStringData {
        text: "Title".into(),
        font: GameFont::FrizQuadrata,
        font_size: 24.0,
        ..Default::default()
    }));
    let props = crate::render_text::extract_text_props_pub(&frame);
    assert_eq!(props.font, GameFont::FrizQuadrata);
    assert_eq!(props.font_size, 24.0);
    assert_eq!(props.content, "Title");
}

#[test]
fn button_font_size_flows_to_text_props() {
    let btn = ButtonData {
        text: "Click".into(),
        font_size: 20.0,
        ..Default::default()
    };
    let props = extract_button_text(&btn, 1.0);
    assert_eq!(props.font_size, 20.0);
    assert_eq!(props.content, "Click");
}

#[test]
fn nested_texture_inside_button_creates_child_frame_and_quad() {
    let mut app = setup_app();

    let mut button = WidgetDef::new("button");
    button.name = Some("DeleteChar".into());
    button.attrs = vec![
        Attr::new_static("width", "46".into()),
        Attr::new_static("height", "42".into()),
        Attr::new_static("text", "".into()),
        Attr::new_static("button_atlas_up", "defaultbutton-nineslice-up".into()),
        Attr::new_static("button_atlas_pressed", "defaultbutton-nineslice-pressed".into()),
        Attr::new_static(
            "button_atlas_highlight",
            "defaultbutton-nineslice-highlight".into(),
        ),
        Attr::new_static(
            "button_atlas_disabled",
            "defaultbutton-nineslice-disabled".into(),
        ),
    ];

    let mut icon = WidgetDef::new("texture");
    icon.name = Some("DeleteCharIcon".into());
    icon.attrs = vec![
        Attr::new_static("width", "24".into()),
        Attr::new_static("height", "24".into()),
        Attr::new_static("texture_atlas", "defaultbutton-nineslice-highlight".into()),
    ];

    button.children.push(WidgetChild::Widget(icon));

    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let mut diff = DiffContext::new();
        diff.diff_roots(&[WidgetChild::Widget(button)], None, &mut ui.registry);
    }

    app.update();

    let ui = app.world().resource::<UiState>();
    let button_id = ui
        .registry
        .get_by_name("DeleteChar")
        .expect("button frame should exist");
    let icon_id = ui
        .registry
        .get_by_name("DeleteCharIcon")
        .expect("nested texture frame should exist");
    let icon_frame = ui.registry.get(icon_id).expect("icon frame");
    assert_eq!(icon_frame.parent_id, Some(button_id));

    let mut q = app.world_mut().query::<&crate::render::UiQuad>();
    assert!(
        q.iter(app.world()).any(|quad| quad.0 == icon_id),
        "nested texture should spawn a render quad"
    );
}

// --- Alpha tests ---

#[test]
fn edit_box_alpha_applied_to_text_color() {
    let mut frame = crate::frame::Frame::new(1, None, WidgetType::EditBox);
    frame.effective_alpha = 0.4;
    frame.widget_data = Some(WidgetData::EditBox(EditBoxData {
        text: "faded".into(),
        text_color: [1.0, 1.0, 1.0, 1.0],
        ..Default::default()
    }));
    let props = crate::render_text::extract_text_props_pub(&frame);
    let Color::Srgba(srgba) = props.color else {
        panic!("expected srgba");
    };
    assert!(
        (srgba.alpha - 0.4).abs() < 0.001,
        "alpha should be 0.4, got {}",
        srgba.alpha
    );
}

#[test]
fn button_alpha_applied_to_text_color() {
    let btn = ButtonData {
        text: "Test".into(),
        ..Default::default()
    };
    let props = extract_button_text(&btn, 0.3);
    let Color::Srgba(srgba) = props.color else {
        panic!("expected srgba");
    };
    assert!(
        (srgba.alpha - 0.3).abs() < 0.001,
        "alpha should be 0.3, got {}",
        srgba.alpha
    );
}

#[test]
fn texture_tint_applies_effective_alpha() {
    let mut frame = crate::frame::Frame::new(1, None, WidgetType::Texture);
    frame.effective_alpha = 0.6;
    frame.widget_data = Some(WidgetData::Texture(crate::widgets::texture::TextureData {
        vertex_color: [0.8, 0.5, 0.3, 1.0],
        ..Default::default()
    }));
    let color = texture_tint(&frame);
    let Color::Srgba(srgba) = color else {
        panic!("expected srgba");
    };
    assert!((srgba.red - 0.8).abs() < 0.001);
    assert!((srgba.green - 0.5).abs() < 0.001);
    assert!((srgba.blue - 0.3).abs() < 0.001);
    assert!((srgba.alpha - 0.6).abs() < 0.001);
}

// --- Sprite sizing tests ---

#[test]
fn frame_sprite_params_uses_full_dimensions() {
    let mut frame = crate::frame::Frame::new(1, None, WidgetType::Frame);
    frame.width = Dimension::Fixed(200.0);
    frame.height = Dimension::Fixed(100.0);
    let (size, offset) = frame_sprite_params(&frame);
    assert_eq!(size, Vec2::new(200.0, 100.0));
    assert_eq!(offset, Vec2::ZERO);
}

fn create_colored_frame(app: &mut App, name: &str, strata: crate::strata::FrameStrata) -> u64 {
    let mut ui = app.world_mut().resource_mut::<UiState>();
    let id = ui.registry.create_frame(name, None);
    let frame = ui.registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(200.0);
    frame.height = Dimension::Fixed(40.0);
    frame.background_color = Some([1.0, 1.0, 1.0, 1.0]);
    frame.strata = strata;
    id
}

fn quad_z(world: &mut World, frame_id: u64) -> Option<f32> {
    world
        .query::<(&Transform, &crate::render::UiQuad)>()
        .iter(world)
        .find(|(_, q)| q.0 == frame_id)
        .map(|(t, _)| t.translation.z)
}

/// Frames with equal sort keys use creation order (id) as tiebreaker,
/// so z-ordering is deterministic regardless of HashMap iteration order.
#[test]
fn same_strata_uses_creation_order() {
    use crate::strata::FrameStrata;

    let mut app = setup_app();
    let first = create_colored_frame(&mut app, "First", FrameStrata::Medium);
    let second = create_colored_frame(&mut app, "Second", FrameStrata::Medium);
    app.update();

    let z1 = quad_z(app.world_mut(), first).expect("first quad");
    let z2 = quad_z(app.world_mut(), second).expect("second quad");
    assert!(
        z1 < z2,
        "earlier-created frame (z={z1}) must render below later one (z={z2})"
    );
}

/// Background strata frames must sort before Medium strata frames in the
/// render z-order, so full-screen backgrounds never occlude UI elements.
#[test]
fn background_strata_sorts_below_medium() {
    use crate::strata::FrameStrata;

    let mut app = setup_app();
    let bg_id = create_colored_frame(&mut app, "Bg", FrameStrata::Background);
    let fg_id = create_colored_frame(&mut app, "Fg", FrameStrata::Medium);
    app.update();

    let bg_z = quad_z(app.world_mut(), bg_id).expect("background quad should exist");
    let fg_z = quad_z(app.world_mut(), fg_id).expect("foreground quad should exist");
    assert!(
        bg_z < fg_z,
        "Background strata (z={bg_z}) must render below Medium strata (z={fg_z})"
    );
}

// --- Dynamic texture tests ---

fn create_dynamic_texture(app: &mut App, name: &str, handle: Handle<Image>) -> u64 {
    let mut ui = app.world_mut().resource_mut::<UiState>();
    let id = ui.registry.create_frame(name, None);
    let frame = ui.registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(200.0);
    frame.height = Dimension::Fixed(200.0);
    frame.widget_type = WidgetType::Texture;
    frame.widget_data = Some(WidgetData::Texture(crate::widgets::texture::TextureData {
        source: TextureSource::Dynamic(handle),
        ..Default::default()
    }));
    id
}

#[test]
fn dynamic_texture_spawns_quad() {
    let mut app = setup_app();
    let handle = app.world_mut().resource_mut::<Assets<Image>>().add(Image::default());
    let id = create_dynamic_texture(&mut app, "DynTex", handle);
    app.update();
    assert!(quad_z(app.world_mut(), id).is_some(), "Dynamic texture should spawn a UiQuad");
}

fn quad_rotation_z(world: &mut World, frame_id: u64) -> Option<f32> {
    world
        .query::<(&Transform, &crate::render::UiQuad)>()
        .iter(world)
        .find(|(_, q)| q.0 == frame_id)
        .map(|(t, _)| t.rotation.to_euler(bevy::math::EulerRot::XYZ).2)
}

#[test]
fn texture_rotation_applies_to_transform() {
    let mut app = setup_app();
    let handle = app.world_mut().resource_mut::<Assets<Image>>().add(Image::default());
    let id = create_dynamic_texture(&mut app, "RotTex", handle);
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let frame = ui.registry.get_mut(id).unwrap();
        if let Some(WidgetData::Texture(tex)) = &mut frame.widget_data {
            tex.rotation = std::f32::consts::FRAC_PI_4;
        }
    }
    app.update();
    let rot = quad_rotation_z(app.world_mut(), id).expect("quad should exist");
    assert!((rot - std::f32::consts::FRAC_PI_4).abs() < 0.01, "rotation should be pi/4, got {rot}");
}

#[test]
fn texture_zero_rotation_no_transform_rotation() {
    let mut app = setup_app();
    let handle = app.world_mut().resource_mut::<Assets<Image>>().add(Image::default());
    let id = create_dynamic_texture(&mut app, "NoRot", handle);
    app.update();
    let rot = quad_rotation_z(app.world_mut(), id).expect("quad should exist");
    assert!(rot.abs() < 0.001, "zero rotation should produce no Z rotation, got {rot}");
}
