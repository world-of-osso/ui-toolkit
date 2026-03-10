//! Tests for UI panel rendering: login screen + char select panels.
//!
//! Login screen panels already use nine-slice textures and atlas buttons.
//! Char select panels currently use plain background colors — tests here
//! document the expected nine-slice behavior that needs to be ported from
//! wow-ui-sim.

use crate::frame::{NineSlice, WidgetData, WidgetType};
use crate::registry::FrameRegistry;
use crate::widgets::button::ButtonData;
use crate::widgets::edit_box::EditBoxData;
use crate::widgets::font_string::GameFont;
use crate::widgets::texture::TextureSource;

fn test_registry() -> FrameRegistry {
    FrameRegistry::new(1920.0, 1080.0)
}

fn create_frame(
    reg: &mut FrameRegistry,
    name: &str,
    parent: Option<u64>,
    wt: WidgetType,
    w: f32,
    h: f32,
) -> u64 {
    let id = reg.next_id();
    let mut frame = crate::frame::Frame::new(id, Some(name.to_string()), wt);
    frame.parent_id = parent;
    frame.width = w;
    frame.height = h;
    frame.mouse_enabled = true;
    reg.insert_frame(frame);
    id
}

fn create_button(
    reg: &mut FrameRegistry,
    name: &str,
    parent: Option<u64>,
    w: f32,
    h: f32,
    text: &str,
) -> u64 {
    let id = create_frame(reg, name, parent, WidgetType::Button, w, h);
    if let Some(frame) = reg.get_mut(id) {
        frame.widget_data = Some(WidgetData::Button(ButtonData {
            text: text.to_string(),
            ..Default::default()
        }));
    }
    id
}

fn create_editbox(reg: &mut FrameRegistry, name: &str, parent: Option<u64>, w: f32, h: f32) -> u64 {
    let id = create_frame(reg, name, parent, WidgetType::EditBox, w, h);
    if let Some(frame) = reg.get_mut(id) {
        frame.widget_data = Some(WidgetData::EditBox(EditBoxData::default()));
    }
    id
}

fn set_button_atlases(
    reg: &mut FrameRegistry,
    id: u64,
    normal: &str,
    pushed: &str,
    highlight: &str,
) {
    if let Some(WidgetData::Button(bd)) = reg.get_mut(id).and_then(|f| f.widget_data.as_mut()) {
        bd.normal_texture = Some(TextureSource::Atlas(normal.to_string()));
        bd.pushed_texture = Some(TextureSource::Atlas(pushed.to_string()));
        bd.highlight_texture = Some(TextureSource::Atlas(highlight.to_string()));
    }
}

fn set_nine_slice_editbox(reg: &mut FrameRegistry, id: u64) {
    if let Some(frame) = reg.get_mut(id) {
        frame.nine_slice = Some(NineSlice {
            edge_size: 8.0,
            part_textures: Some(std::array::from_fn(|i| {
                TextureSource::File(format!("Common-Input-Border-{i}.blp"))
            })),
            bg_color: [1.0; 4],
            border_color: [1.0; 4],
            ..Default::default()
        });
    }
}

// =============================================================================
// Login screen panel tests (these should PASS — already implemented)
// =============================================================================

#[test]
fn login_editbox_has_nine_slice() {
    let mut reg = test_registry();
    let root = create_frame(
        &mut reg,
        "LoginRoot",
        None,
        WidgetType::Frame,
        1920.0,
        1080.0,
    );
    let eb = create_editbox(&mut reg, "UsernameInput", Some(root), 320.0, 42.0);
    set_nine_slice_editbox(&mut reg, eb);

    let frame = reg.get(eb).unwrap();
    assert!(
        frame.nine_slice.is_some(),
        "login editbox should have nine_slice"
    );
}

#[test]
fn login_editbox_nine_slice_has_9_part_textures() {
    let mut reg = test_registry();
    let root = create_frame(
        &mut reg,
        "LoginRoot",
        None,
        WidgetType::Frame,
        1920.0,
        1080.0,
    );
    let eb = create_editbox(&mut reg, "UsernameInput", Some(root), 320.0, 42.0);
    set_nine_slice_editbox(&mut reg, eb);

    let ns = reg.get(eb).unwrap().nine_slice.as_ref().unwrap();
    let parts = ns
        .part_textures
        .as_ref()
        .expect("should have per-part textures");
    for (i, part) in parts.iter().enumerate() {
        assert!(
            !matches!(part, TextureSource::None),
            "nine-slice part {i} should have a texture"
        );
    }
}

#[test]
fn login_editbox_has_text_insets() {
    let mut reg = test_registry();
    let eb = create_editbox(&mut reg, "TestEditBox", None, 320.0, 42.0);
    if let Some(frame) = reg.get_mut(eb)
        && let Some(WidgetData::EditBox(eb_data)) = &mut frame.widget_data
    {
        eb_data.text_insets = [12.0, 5.0, 0.0, 5.0];
    }

    let frame = reg.get(eb).unwrap();
    let WidgetData::EditBox(eb_data) = frame.widget_data.as_ref().unwrap() else {
        panic!("expected editbox");
    };
    assert_eq!(eb_data.text_insets[0], 12.0, "left inset");
    assert_eq!(eb_data.text_insets[1], 5.0, "right inset");
}

#[test]
fn login_editbox_has_font() {
    let mut reg = test_registry();
    let eb = create_editbox(&mut reg, "TestEditBox", None, 320.0, 42.0);
    if let Some(frame) = reg.get_mut(eb)
        && let Some(WidgetData::EditBox(eb_data)) = &mut frame.widget_data
    {
        eb_data.font = GameFont::ArialNarrow;
        eb_data.font_size = 16.0;
    }

    let frame = reg.get(eb).unwrap();
    let WidgetData::EditBox(eb_data) = frame.widget_data.as_ref().unwrap() else {
        panic!("expected editbox");
    };
    assert_eq!(eb_data.font, GameFont::ArialNarrow);
    assert_eq!(eb_data.font_size, 16.0);
}

#[test]
fn login_button_has_atlas_textures() {
    let mut reg = test_registry();
    let btn = create_button(&mut reg, "ConnectButton", None, 250.0, 66.0, "Login");
    set_button_atlases(
        &mut reg,
        btn,
        "128-redbutton-up",
        "128-redbutton-pressed",
        "128-redbutton-highlight",
    );

    let frame = reg.get(btn).unwrap();
    let WidgetData::Button(bd) = frame.widget_data.as_ref().unwrap() else {
        panic!("expected button");
    };
    assert!(matches!(&bd.normal_texture, Some(TextureSource::Atlas(n)) if n == "128-redbutton-up"));
    assert!(
        matches!(&bd.pushed_texture, Some(TextureSource::Atlas(n)) if n == "128-redbutton-pressed")
    );
    assert!(
        matches!(&bd.highlight_texture, Some(TextureSource::Atlas(n)) if n == "128-redbutton-highlight")
    );
}

#[test]
fn login_button_font_size_customizable() {
    let mut reg = test_registry();
    let btn = create_button(&mut reg, "ConnectButton", None, 250.0, 66.0, "Login");
    if let Some(WidgetData::Button(bd)) = reg.get_mut(btn).and_then(|f| f.widget_data.as_mut()) {
        bd.font_size = 22.0;
    }

    let frame = reg.get(btn).unwrap();
    let WidgetData::Button(bd) = frame.widget_data.as_ref().unwrap() else {
        panic!("expected button");
    };
    assert_eq!(bd.font_size, 22.0);
}

#[test]
fn editbox_focus_changes_nine_slice_colors() {
    let mut reg = test_registry();
    let eb = create_editbox(&mut reg, "TestEditBox", None, 320.0, 42.0);
    set_nine_slice_editbox(&mut reg, eb);

    // Simulate focus: change border color
    let frame = reg.get_mut(eb).unwrap();
    let ns = frame.nine_slice.as_mut().unwrap();
    ns.border_color = [1.0, 0.92, 0.72, 1.0]; // focused gold

    let frame = reg.get(eb).unwrap();
    let ns = frame.nine_slice.as_ref().unwrap();
    assert!(
        (ns.border_color[1] - 0.92).abs() < 0.01,
        "focused border should be golden"
    );
}

// =============================================================================
// Char select panel tests — these document EXPECTED behavior.
// Tests marked #[ignore] will fail until nine-slice is ported to char_select.
// =============================================================================

#[test]
#[ignore = "char_select panels use set_bg, need nine-slice port"]
fn char_select_create_panel_has_nine_slice() {
    let mut reg = test_registry();
    let root = create_frame(
        &mut reg,
        "CharSelectRoot",
        None,
        WidgetType::Frame,
        1920.0,
        1080.0,
    );
    let panel = create_frame(
        &mut reg,
        "CreatePanel",
        Some(root),
        WidgetType::Frame,
        300.0,
        120.0,
    );

    // Currently: set_bg(reg, panel, [0.08, 0.08, 0.18, 0.95])
    // Expected: nine_slice with backdrop texture
    let frame = reg.get(panel).unwrap();
    assert!(
        frame.nine_slice.is_some(),
        "CreatePanel should use nine-slice backdrop, not plain background"
    );
}

#[test]
#[ignore = "char_select buttons use set_bg, need atlas texture port"]
fn char_select_action_buttons_have_textures() {
    let mut reg = test_registry();
    let root = create_frame(&mut reg, "Root", None, WidgetType::Frame, 1920.0, 1080.0);
    let btn = create_button(
        &mut reg,
        "EnterWorld",
        Some(root),
        250.0,
        66.0,
        "Enter World",
    );

    // Currently: set_bg(reg, btn, [0.15, 0.35, 0.6, 1.0])
    // Expected: atlas textures like login screen buttons
    let frame = reg.get(btn).unwrap();
    let WidgetData::Button(bd) = frame.widget_data.as_ref().unwrap() else {
        panic!("expected button");
    };
    assert!(
        bd.normal_texture.is_some(),
        "EnterWorld button should have atlas normal texture, not plain bg"
    );
}

#[test]
#[ignore = "char_select editbox uses set_bg, need nine-slice port"]
fn char_select_editbox_has_nine_slice() {
    let mut reg = test_registry();
    let root = create_frame(&mut reg, "Root", None, WidgetType::Frame, 1920.0, 1080.0);
    let panel = create_frame(
        &mut reg,
        "CreatePanel",
        Some(root),
        WidgetType::Frame,
        300.0,
        120.0,
    );
    let eb = create_editbox(&mut reg, "CreateNameInput", Some(panel), 280.0, 30.0);

    // Currently: set_bg(reg, name_input, [0.12, 0.12, 0.2, 1.0])
    // Expected: nine_slice with Common-Input-Border textures
    let frame = reg.get(eb).unwrap();
    assert!(
        frame.nine_slice.is_some(),
        "CreateNameInput should use nine-slice border, not plain bg"
    );
}

#[test]
#[ignore = "char_select confirm button uses set_bg, need atlas texture port"]
fn char_select_confirm_button_has_texture() {
    let mut reg = test_registry();
    let root = create_frame(&mut reg, "Root", None, WidgetType::Frame, 1920.0, 1080.0);
    let btn = create_button(&mut reg, "CreateConfirm", Some(root), 120.0, 30.0, "Create");

    let frame = reg.get(btn).unwrap();
    let WidgetData::Button(bd) = frame.widget_data.as_ref().unwrap() else {
        panic!("expected button");
    };
    assert!(
        bd.normal_texture.is_some(),
        "CreateConfirm should have atlas texture, not plain bg"
    );
}

#[test]
#[ignore = "char list buttons use set_bg, need atlas texture port"]
fn char_select_list_buttons_have_textures() {
    let mut reg = test_registry();
    let root = create_frame(&mut reg, "Root", None, WidgetType::Frame, 1920.0, 1080.0);
    let btn = create_button(
        &mut reg,
        "Char_1",
        Some(root),
        380.0,
        32.0,
        "TestChar - Lv1 R1 C1",
    );

    // Currently: set_bg(reg, btn, [0.12, 0.12, 0.22, 1.0])
    // Expected: textured button or nine-slice panel
    let frame = reg.get(btn).unwrap();
    let WidgetData::Button(bd) = frame.widget_data.as_ref().unwrap() else {
        panic!("expected button");
    };
    assert!(
        bd.normal_texture.is_some(),
        "Character list buttons should have textures"
    );
}

// =============================================================================
// Alpha propagation through panel hierarchy
// =============================================================================

#[test]
fn panel_alpha_propagates_to_children() {
    let mut reg = test_registry();
    let root = create_frame(&mut reg, "Root", None, WidgetType::Frame, 1920.0, 1080.0);
    let panel = create_frame(
        &mut reg,
        "Panel",
        Some(root),
        WidgetType::Frame,
        300.0,
        120.0,
    );
    let btn = create_button(&mut reg, "Btn", Some(panel), 120.0, 30.0, "Test");

    reg.set_alpha(root, 0.5);

    let btn_frame = reg.get(btn).unwrap();
    assert!(
        (btn_frame.effective_alpha - 0.5).abs() < 0.01,
        "button alpha should inherit from root, got {}",
        btn_frame.effective_alpha
    );
}

#[test]
fn hiding_panel_hides_children() {
    let mut reg = test_registry();
    let root = create_frame(&mut reg, "Root", None, WidgetType::Frame, 1920.0, 1080.0);
    let panel = create_frame(
        &mut reg,
        "Panel",
        Some(root),
        WidgetType::Frame,
        300.0,
        120.0,
    );
    let btn = create_button(&mut reg, "Btn", Some(panel), 120.0, 30.0, "Test");

    reg.set_shown(panel, false);

    let btn_frame = reg.get(btn).unwrap();
    assert!(
        !btn_frame.visible,
        "button should be hidden when panel is hidden"
    );
    assert!(
        btn_frame.effective_alpha < 0.01,
        "hidden button alpha should be ~0, got {}",
        btn_frame.effective_alpha
    );
}

// =============================================================================
// Nine-slice geometry for panels
// =============================================================================

#[test]
fn nine_slice_edge_size_determines_corner_dimensions() {
    use crate::render_nine_slice::part_geometry;

    let mut frame = crate::frame::Frame::new(1, None, WidgetType::Frame);
    frame.width = 300.0;
    frame.height = 120.0;
    frame.layout_rect = Some(crate::layout::LayoutRect {
        x: 100.0,
        y: 200.0,
        width: 300.0,
        height: 120.0,
    });
    let ns = NineSlice {
        edge_size: 12.0,
        ..Default::default()
    };

    // Part 0 = top-left corner → should be 12x12
    let (_, size, _) = part_geometry(&frame, &ns, 0, 1920.0, 1080.0, 0.0);
    assert_eq!(size.x, 12.0, "TL corner width should equal edge_size");
    assert_eq!(size.y, 12.0, "TL corner height should equal edge_size");

    // Part 4 = center → should be (300-24) x (120-24) = 276 x 96
    let (_, size, _) = part_geometry(&frame, &ns, 4, 1920.0, 1080.0, 0.0);
    assert_eq!(size.x, 276.0, "center width should be frame_w - 2*edge");
    assert_eq!(size.y, 96.0, "center height should be frame_h - 2*edge");
}

#[test]
fn nine_slice_border_color_applied_to_edge_parts() {
    use crate::render_nine_slice::part_geometry;

    let mut frame = crate::frame::Frame::new(1, None, WidgetType::Frame);
    frame.width = 200.0;
    frame.height = 100.0;
    frame.effective_alpha = 0.8;
    frame.layout_rect = Some(crate::layout::LayoutRect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
    });
    let ns = NineSlice {
        edge_size: 8.0,
        border_color: [1.0, 0.92, 0.72, 1.0],
        bg_color: [0.0, 0.0, 0.0, 0.8],
        texture: Some(TextureSource::File("border.blp".into())),
        ..Default::default()
    };

    // Part 1 = top edge → should use border_color with alpha
    let (_, _, color) = part_geometry(&frame, &ns, 1, 1920.0, 1080.0, 0.0);
    let bevy::color::Color::Srgba(srgba) = color else {
        panic!("expected srgba")
    };
    assert!(
        (srgba.green - 0.92).abs() < 0.01,
        "edge should use border_color green"
    );
    assert!(
        (srgba.alpha - 0.8).abs() < 0.01,
        "edge alpha should include effective_alpha"
    );

    // Part 4 = center → should use bg_color with alpha
    let (_, _, color) = part_geometry(&frame, &ns, 4, 1920.0, 1080.0, 0.0);
    let bevy::color::Color::Srgba(srgba) = color else {
        panic!("expected srgba")
    };
    assert!(srgba.red < 0.01, "center should use bg_color red=0");
    assert!(
        (srgba.alpha - 0.64).abs() < 0.01,
        "center alpha = bg_alpha * effective_alpha"
    );
}
