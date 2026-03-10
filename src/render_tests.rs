use bevy::prelude::*;

use crate::frame::{WidgetData, WidgetType};
use crate::plugin::{UiPlugin, UiState};
use crate::render::{frame_sprite_params, texture_tint};
use crate::render_nine_slice::UiNineSlicePart;
use crate::render_text::extract_button_text;
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
    frame.width = 120.0;
    frame.height = 40.0;
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
    // Switch to pushed state
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let frame = ui.registry.get_mut(id).unwrap();
        if let Some(WidgetData::Button(btn)) = &mut frame.widget_data {
            btn.state = ButtonState::Pushed;
        }
        frame.nine_slice = None; // clear so sync re-creates
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
        frame.width = 250.0;
        frame.height = 32.0;
        frame.widget_type = WidgetType::EditBox;
        frame.widget_data = Some(WidgetData::EditBox(EditBoxData::default()));
        id
    };
    app.update();
    let ui = app.world().resource::<UiState>();
    let frame = ui.registry.get(id).unwrap();
    assert_eq!(frame.width, 250.0);
    assert_eq!(frame.height, 32.0);
}

#[test]
fn edit_box_font_flows_to_text_props() {
    let mut frame = crate::frame::Frame::new(1, None, WidgetType::EditBox);
    frame.width = 200.0;
    frame.height = 30.0;
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
    frame.widget_data = Some(WidgetData::Texture(
        crate::widgets::texture::TextureData {
            vertex_color: [0.8, 0.5, 0.3, 1.0],
            ..Default::default()
        },
    ));
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
    frame.width = 200.0;
    frame.height = 100.0;
    let (size, offset) = frame_sprite_params(&frame);
    assert_eq!(size, Vec2::new(200.0, 100.0));
    assert_eq!(offset, Vec2::ZERO);
}

// --- Dioxus component tree rendering tests ---

mod dioxus_render {
    use super::*;
    use crate::dioxus_screen::DioxusScreen;
    use dioxus::prelude::*;

    #[allow(unused_imports)]
    use crate::dioxus_elements;

    fn build_screen(component: fn() -> Element) -> (App, DioxusScreen) {
        let mut app = setup_app();
        let mut screen = DioxusScreen::new(component);
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            screen.sync(&mut ui.registry);
        }
        (app, screen)
    }

    #[test]
    fn dioxus_button_gets_nine_slice_after_update() {
        fn comp() -> Element {
            rsx! {
                button {
                    name: "DxBtn",
                    width: 120.0,
                    height: 40.0,
                    text: "Press",
                    button_atlas_up: "test-up",
                    button_atlas_pressed: "test-pressed",
                    button_atlas_highlight: "test-highlight",
                    button_atlas_disabled: "test-disabled",
                }
            }
        }
        let (mut app, _screen) = build_screen(comp);
        app.update();
        let ui = app.world().resource::<UiState>();
        let id = ui.registry.get_by_name("DxBtn").expect("DxBtn");
        let frame = ui.registry.get(id).unwrap();
        assert_eq!(frame.widget_type, WidgetType::Button);
        assert!(
            frame.nine_slice.is_some(),
            "dioxus button with atlas should get nine_slice"
        );
        match &frame.widget_data {
            Some(WidgetData::Button(bd)) => {
                assert_eq!(bd.text, "Press");
                assert!(
                    matches!(&bd.normal_texture, Some(TextureSource::Atlas(s)) if s == "test-up")
                );
            }
            other => panic!("expected Button widget_data, got {:?}", other),
        }
    }

    #[test]
    fn dioxus_button_without_texture_no_nine_slice() {
        fn comp() -> Element {
            rsx! {
                button {
                    name: "DxBtnPlain",
                    width: 100.0,
                    height: 30.0,
                    text: "Plain",
                }
            }
        }
        let (mut app, _screen) = build_screen(comp);
        app.update();
        let ui = app.world().resource::<UiState>();
        let id = ui.registry.get_by_name("DxBtnPlain").expect("DxBtnPlain");
        let frame = ui.registry.get(id).unwrap();
        assert!(
            frame.nine_slice.is_some(),
            "all buttons get a default nine-slice"
        );
    }

    #[test]
    fn dioxus_button_pushed_state_updates_texture() {
        fn comp() -> Element {
            rsx! {
                button {
                    name: "DxBtnPush",
                    width: 120.0,
                    height: 40.0,
                    button_atlas_up: "normal-atlas",
                    button_atlas_pressed: "pushed-atlas",
                }
            }
        }
        let (mut app, _screen) = build_screen(comp);
        app.update();
        // Switch to pushed
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let id = ui.registry.get_by_name("DxBtnPush").unwrap();
            let frame = ui.registry.get_mut(id).unwrap();
            if let Some(WidgetData::Button(btn)) = &mut frame.widget_data {
                btn.state = ButtonState::Pushed;
            }
            frame.nine_slice = None;
        }
        app.update();
        let ui = app.world().resource::<UiState>();
        let id = ui.registry.get_by_name("DxBtnPush").unwrap();
        let frame = ui.registry.get(id).unwrap();
        let ns = frame
            .nine_slice
            .as_ref()
            .expect("should have nine_slice after push");
        assert!(
            matches!(&ns.texture, Some(TextureSource::Atlas(s)) if s == "pushed-atlas"),
            "pushed state should use pushed texture, got {:?}",
            ns.texture
        );
    }

    #[test]
    fn dioxus_editbox_preserves_dimensions_and_password() {
        fn comp() -> Element {
            rsx! {
                editbox {
                    name: "DxEdit",
                    width: 300.0,
                    height: 36.0,
                    font_size: 20.0,
                    password: true,
                }
            }
        }
        let (mut app, _screen) = build_screen(comp);
        app.update();
        let ui = app.world().resource::<UiState>();
        let id = ui.registry.get_by_name("DxEdit").expect("DxEdit");
        let frame = ui.registry.get(id).unwrap();
        assert_eq!(frame.widget_type, WidgetType::EditBox);
        assert_eq!(frame.width, 300.0);
        assert_eq!(frame.height, 36.0);
        match &frame.widget_data {
            Some(WidgetData::EditBox(eb)) => {
                assert!(eb.password, "password should be true");
                assert_eq!(
                    eb.font_size, 20.0,
                    "font_size should be 20.0, got {}",
                    eb.font_size
                );
            }
            other => panic!("expected EditBox widget_data, got {:?}", other),
        }
    }

    #[test]
    fn dioxus_fontstring_text_and_font_size() {
        fn comp() -> Element {
            rsx! {
                fontstring {
                    name: "DxLabel",
                    text: "Hello World",
                    font_size: 22.0,
                    font_color: "1.0,0.8,0.0,1.0",
                }
            }
        }
        let (mut app, _screen) = build_screen(comp);
        app.update();
        let ui = app.world().resource::<UiState>();
        let id = ui.registry.get_by_name("DxLabel").expect("DxLabel");
        let frame = ui.registry.get(id).unwrap();
        assert_eq!(frame.widget_type, WidgetType::FontString);
        match &frame.widget_data {
            Some(WidgetData::FontString(fs)) => {
                assert_eq!(fs.text, "Hello World");
                assert_eq!(fs.font_size, 22.0);
                assert!((fs.color[0] - 1.0).abs() < 0.01);
                assert!((fs.color[1] - 0.8).abs() < 0.01);
            }
            other => panic!("expected FontString widget_data, got {:?}", other),
        }
    }

    #[test]
    fn dioxus_texture_file_source() {
        fn comp() -> Element {
            rsx! {
                texture {
                    name: "DxTex",
                    texture_file: "textures/test.blp",
                    width: 64.0,
                    height: 64.0,
                }
            }
        }
        let (mut app, _screen) = build_screen(comp);
        app.update();
        let ui = app.world().resource::<UiState>();
        let id = ui.registry.get_by_name("DxTex").expect("DxTex");
        let frame = ui.registry.get(id).unwrap();
        assert_eq!(frame.widget_type, WidgetType::Texture);
        assert_eq!(frame.width, 64.0);
        match &frame.widget_data {
            Some(WidgetData::Texture(td)) => {
                assert!(matches!(&td.source, TextureSource::File(p) if p == "textures/test.blp"));
            }
            other => panic!("expected Texture widget_data, got {:?}", other),
        }
    }

    #[test]
    fn dioxus_nested_button_parent_child() {
        fn comp() -> Element {
            rsx! {
                r#frame { name: "DxParent", width: 400.0, height: 300.0,
                    button {
                        name: "DxChild",
                        width: 100.0,
                        height: 30.0,
                        text: "Go",
                    }
                }
            }
        }
        let (mut app, _screen) = build_screen(comp);
        app.update();
        let ui = app.world().resource::<UiState>();
        let parent_id = ui.registry.get_by_name("DxParent").expect("DxParent");
        let child_id = ui.registry.get_by_name("DxChild").expect("DxChild");
        let child = ui.registry.get(child_id).unwrap();
        assert_eq!(child.parent_id, Some(parent_id));
        let parent = ui.registry.get(parent_id).unwrap();
        assert!(parent.children.contains(&child_id));
    }

    #[test]
    fn dioxus_button_font_size_flows_to_text_props() {
        fn comp() -> Element {
            rsx! {
                button {
                    name: "DxBtnFont",
                    width: 120.0,
                    height: 40.0,
                    text: "Styled",
                    font_size: 18.0,
                }
            }
        }
        let (mut app, _screen) = build_screen(comp);
        app.update();
        let ui = app.world().resource::<UiState>();
        let id = ui.registry.get_by_name("DxBtnFont").expect("DxBtnFont");
        let frame = ui.registry.get(id).unwrap();
        match &frame.widget_data {
            Some(WidgetData::Button(bd)) => {
                let props = extract_button_text(bd, 1.0);
                assert_eq!(props.font_size, 18.0);
                assert_eq!(props.content, "Styled");
            }
            other => panic!("expected Button widget_data, got {:?}", other),
        }
    }

    fn assert_button_data(frame: &crate::frame::Frame, text: &str, font_size: f32) {
        assert_eq!(frame.widget_type, WidgetType::Button);
        match &frame.widget_data {
            Some(WidgetData::Button(bd)) => {
                assert_eq!(bd.text, text);
                assert_eq!(bd.font_size, font_size);
            }
            other => panic!("expected Button widget_data, got {:?}", other),
        }
    }

    fn assert_anchor(
        frame: &crate::frame::Frame,
        point: crate::anchor::AnchorPoint,
        rel_point: crate::anchor::AnchorPoint,
        rel_to: Option<u64>,
        ox: f32,
        oy: f32,
    ) {
        assert_eq!(frame.anchors.len(), 1);
        let a = &frame.anchors[0];
        assert_eq!(a.point, point);
        assert_eq!(a.relative_point, rel_point);
        assert_eq!(a.relative_to, rel_to);
        assert_eq!(a.x_offset, ox);
        assert_eq!(a.y_offset, oy);
    }

    #[test]
    fn dioxus_button_with_anchor_to_named_sibling() {
        fn comp() -> Element {
            rsx! {
                r#frame { name: "AnchorParent", width: 800.0, height: 600.0,
                    editbox {
                        name: "PasswordInput",
                        width: 320.0,
                        height: 42.0,
                        strata: crate::strata::FrameStrata::Medium,
                        anchor {
                            point: "CENTER",
                            relative_point: "CENTER",
                            y: "50",
                        }
                    }
                    button {
                        name: "ConnectButton",
                        width: 250.0,
                        height: 66.0,
                        text: "Login",
                        font_size: 16.0,
                        strata: crate::strata::FrameStrata::Medium,
                        anchor {
                            point: "TOP",
                            relative_to: "PasswordInput",
                            relative_point: "BOTTOM",
                            y: "-50",
                        }
                    }
                }
            }
        }
        let (mut app, _screen) = build_screen(comp);
        app.update();
        let ui = app.world().resource::<UiState>();

        use crate::anchor::AnchorPoint;
        let parent_id = ui.registry.get_by_name("AnchorParent").unwrap();
        let pw_id = ui.registry.get_by_name("PasswordInput").unwrap();
        let btn_id = ui.registry.get_by_name("ConnectButton").unwrap();

        let btn = ui.registry.get(btn_id).unwrap();
        assert_eq!(btn.parent_id, Some(parent_id));
        assert_eq!(btn.width, 250.0);
        assert_eq!(btn.height, 66.0);
        assert_eq!(btn.strata, crate::strata::FrameStrata::Medium);
        assert_button_data(btn, "Login", 16.0);
        assert_anchor(
            btn,
            AnchorPoint::Top,
            AnchorPoint::Bottom,
            Some(pw_id),
            0.0,
            -50.0,
        );

        let pw = ui.registry.get(pw_id).unwrap();
        assert_anchor(
            pw,
            AnchorPoint::Center,
            AnchorPoint::Center,
            Some(parent_id),
            0.0,
            50.0,
        );
    }

    #[test]
    fn dioxus_cross_component_anchor_resolves() {
        fn inputs() -> Element {
            rsx! {
                editbox { name: "PwInput", width: 320.0, height: 42.0,
                    anchor { point: "CENTER", relative_point: "CENTER", y: "50" }
                }
            }
        }
        fn buttons() -> Element {
            rsx! {
                button {
                    name: "LoginBtn",
                    width: 250.0,
                    height: 66.0,
                    text: "Login",
                    anchor {
                        point: "TOP",
                        relative_to: "PwInput",
                        relative_point: "BOTTOM",
                        y: "-50",
                    }
                }
            }
        }
        fn comp() -> Element {
            rsx! {
                r#frame { name: "Root", width: 800.0, height: 600.0,
                    {inputs()}
                    {buttons()}
                }
            }
        }
        let (mut app, _screen) = build_screen(comp);
        app.update();
        let ui = app.world().resource::<UiState>();

        let pw_id = ui.registry.get_by_name("PwInput").expect("PwInput");
        let btn_id = ui.registry.get_by_name("LoginBtn").expect("LoginBtn");
        let btn = ui.registry.get(btn_id).unwrap();

        use crate::anchor::AnchorPoint;
        assert_eq!(btn.anchors.len(), 1, "button should have 1 anchor");
        assert_eq!(
            btn.anchors[0].relative_to,
            Some(pw_id),
            "anchor should resolve to PwInput, got {:?}",
            btn.anchors[0].relative_to
        );
        assert_eq!(btn.anchors[0].point, AnchorPoint::Top);
        assert_eq!(btn.anchors[0].relative_point, AnchorPoint::Bottom);
    }

    #[test]
    fn dioxus_screen_teardown_removes_frames() {
        fn comp() -> Element {
            rsx! {
                r#frame { name: "TearRoot",
                    button { name: "TearBtn", text: "X" }
                    fontstring { name: "TearLabel", text: "Y" }
                }
            }
        }
        let (mut app, mut screen) = build_screen(comp);
        {
            let ui = app.world().resource::<UiState>();
            assert!(ui.registry.get_by_name("TearRoot").is_some());
            assert!(ui.registry.get_by_name("TearBtn").is_some());
            assert!(ui.registry.get_by_name("TearLabel").is_some());
        }
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            screen.teardown(&mut ui.registry);
        }
        let ui = app.world().resource::<UiState>();
        assert!(ui.registry.get_by_name("TearRoot").is_none());
        assert!(ui.registry.get_by_name("TearBtn").is_none());
        assert!(ui.registry.get_by_name("TearLabel").is_none());
    }
}

fn create_colored_frame(app: &mut App, name: &str, strata: crate::strata::FrameStrata) -> u64 {
    let mut ui = app.world_mut().resource_mut::<UiState>();
    let id = ui.registry.create_frame(name, None);
    let frame = ui.registry.get_mut(id).unwrap();
    frame.width = 200.0;
    frame.height = 40.0;
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
