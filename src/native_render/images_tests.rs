use super::*;
use crate::frame::{Backdrop, Border, Dimension, NineSlice, ThreeSlice, WidgetData};
use crate::layout::LayoutRect;
use crate::widgets::{button::ButtonData, slider::StatusBarData, texture::TextureData};

fn frame() -> Frame {
    let mut frame = Frame::default();
    frame.visible = true;
    frame.width = Dimension::Fixed(200.0);
    frame.height = Dimension::Fixed(100.0);
    frame.layout_rect = Some(LayoutRect {
        x: 40.0,
        y: 60.0,
        width: 200.0,
        height: 100.0,
    });
    frame.effective_alpha = 0.5;
    frame
}

fn load(_: &TextureSource) -> Option<(LoadedTexture, Option<Vec2>)> {
    Some((
        LoadedTexture {
            handle: Handle::default(),
            rect: Some(Rect::new(10.0, 20.0, 110.0, 70.0)),
        },
        Some(Vec2::new(160.0, 80.0)),
    ))
}

fn part(parts: &[ImagePart], key: u32) -> &ImagePart {
    parts
        .iter()
        .find(|part| part.key == key)
        .expect("projected role")
}

fn bounds(part: &ImagePart) -> [f32; 4] {
    [
        part.node.left,
        part.node.top,
        part.node.width,
        part.node.height,
    ]
    .into_iter()
    .zip([200.0, 100.0, 200.0, 100.0])
    .map(|(value, parent_extent)| match value {
        Val::Px(value) => value,
        Val::Percent(value) => parent_extent * value / 100.0,
        other => panic!("unexpected sizing mode in the 200x100 fixture: {other:?}"),
    })
    .collect::<Vec<_>>()
    .try_into()
    .unwrap()
}

#[test]
fn background_and_status_fill_preserve_local_geometry_and_alpha() {
    let mut frame = frame();
    frame.background_color = Some([0.2, 0.4, 0.6, 0.8]);
    let parts = project_with_loader(&frame, &mut load);
    assert_eq!(bounds(part(&parts, 0)), [0.0, 0.0, 200.0, 100.0]);
    assert_eq!(
        part(&parts, 0).image.color,
        Color::srgba(0.2, 0.4, 0.6, 0.4)
    );
    frame.widget_data = Some(WidgetData::StatusBar(StatusBarData {
        min: 20.0,
        max: 100.0,
        value: 40.0,
        color: [0.3, 0.6, 0.9, 0.8],
        ..default()
    }));
    let parts = project_with_loader(&frame, &mut load);
    assert_eq!(bounds(part(&parts, 0)), [0.0, 0.0, 50.0, 100.0]);
    assert_eq!(
        part(&parts, 0).image.color,
        Color::srgba(0.3, 0.6, 0.9, 0.4)
    );
    assert!(part(&parts, 0).image.rect.is_none());
}

#[test]
fn texture_crop_is_relative_to_atlas_and_rotation_changes_coordinate_handedness() {
    let mut frame = frame();
    frame.widget_data = Some(WidgetData::Texture(TextureData {
        source: TextureSource::File("atlas.png".into()),
        tex_coords: [0.25, 0.75, 0.2, 0.8],
        vertex_color: [0.2, 0.4, 0.6, 0.8],
        desaturated: true,
        rotation: 0.7,
        ..default()
    }));
    let parts = project_with_loader(&frame, &mut load);
    let texture = part(&parts, 0);
    assert_eq!(texture.image.rect, Some(Rect::new(35.0, 30.0, 85.0, 60.0)));
    let luminance = 0.2126 * 0.2 + 0.7152 * 0.4 + 0.0722 * 0.6;
    assert_eq!(
        texture.image.color,
        Color::srgba(luminance, luminance, luminance, 0.4)
    );
    assert!((texture.transform.rotation.as_radians() + 0.7).abs() < 0.00001);
    assert_eq!(bounds(texture), [0.0, 0.0, 200.0, 100.0]);
}

#[test]
fn asymmetric_nine_slice_uses_shared_geometry_and_atlas_uv_edges() {
    let mut frame = frame();
    frame.background_color = Some([0.1, 0.2, 0.3, 1.0]);
    frame.nine_slice = Some(NineSlice {
        edge_sizes: Some([5.0, 7.0, 11.0, 13.0]),
        uv_edge_sizes: Some([2.0, 3.0, 4.0, 5.0]),
        texture: Some(TextureSource::File("panel.png".into())),
        bg_color: [0.2, 0.3, 0.4, 0.8],
        border_color: [1.0, 0.8, 0.2, 1.0],
        ..default()
    });
    let parts = project_with_loader(&frame, &mut load);
    assert_eq!(parts.len(), 14);
    assert_eq!(bounds(part(&parts, 104)), [5.0, 7.0, 184.0, 80.0]);
    assert_eq!(
        part(&parts, 104).image.rect,
        Some(Rect::new(12.0, 23.0, 106.0, 65.0))
    );
    assert_eq!(
        part(&parts, 104).image.color,
        Color::srgba(0.2, 0.3, 0.4, 0.4)
    );
    assert_eq!(bounds(part(&parts, 108)), [189.0, 87.0, 11.0, 13.0]);
    assert_eq!(
        part(&parts, 108).image.rect,
        Some(Rect::new(106.0, 65.0, 110.0, 70.0))
    );
    assert!(part(&parts, 108).z > part(&parts, 104).z);
    assert!(part(&parts, 12).z < part(&parts, 104).z);
}

#[test]
fn per_part_nine_slice_textures_and_three_slice_do_not_add_atlas_cropping() {
    let mut frame = frame();
    frame.nine_slice = Some(NineSlice {
        part_textures: Some(std::array::from_fn(|index| {
            TextureSource::File(format!("{index}.png"))
        })),
        ..default()
    });
    let parts = project_with_loader(&frame, &mut load);
    assert_eq!(parts.len(), 9);
    assert!(parts.iter().all(|part| part.image.rect.is_none()));
    frame.nine_slice = None;
    frame.three_slice = Some(ThreeSlice {
        cap_width: 25.0,
        left: TextureSource::File("left.png".into()),
        center: TextureSource::File("center.png".into()),
        right: TextureSource::File("right.png".into()),
        color: [0.2, 0.5, 0.8, 0.6],
    });
    let parts = project_with_loader(&frame, &mut load);
    assert_eq!(parts.len(), 3);
    assert_eq!(bounds(part(&parts, 200)), [0.0, 0.0, 25.0, 100.0]);
    assert_eq!(bounds(part(&parts, 201)), [25.0, 0.0, 150.0, 100.0]);
    assert_eq!(bounds(part(&parts, 202)), [175.0, 0.0, 25.0, 100.0]);
    assert!(parts.iter().all(|part| part.image.rect.is_none()));
    assert_eq!(
        part(&parts, 201).image.color,
        Color::srgba(0.2, 0.5, 0.8, 0.3)
    );
}

#[test]
fn backdrop_edges_are_outside_and_css_edges_are_inside() {
    let mut frame = frame();
    frame.backdrop = Some(Backdrop {
        border_color: Some([1.0, 0.5, 0.2, 0.8]),
        edge_size: 3.0,
        ..default()
    });
    frame.border = Some(Border {
        width: 2.0,
        color: [0.1, 0.4, 0.7, 1.0],
    });
    let parts = project_with_loader(&frame, &mut load);
    assert_eq!(parts.len(), 8);
    assert_eq!(bounds(part(&parts, 20)), [-3.0, -3.0, 206.0, 3.0]);
    assert_eq!(bounds(part(&parts, 22)), [-3.0, 0.0, 3.0, 100.0]);
    assert_eq!(bounds(part(&parts, 30)), [0.0, 0.0, 200.0, 2.0]);
    assert_eq!(bounds(part(&parts, 31)), [198.0, 0.0, 2.0, 100.0]);
    assert_eq!(
        part(&parts, 20).image.color,
        Color::srgba(1.0, 0.5, 0.2, 0.4)
    );
}

#[test]
fn tiled_fdid_preserves_full_edge_tiles_and_does_not_tile_files() {
    let mut frame = frame();
    frame.widget_data = Some(WidgetData::Texture(TextureData {
        source: TextureSource::FileDataId(120191),
        horiz_tile: true,
        vert_tile: true,
        ..default()
    }));
    let parts = project_with_loader(&frame, &mut load);
    assert_eq!(parts.iter().filter(|part| part.key >= 300).count(), 8);
    assert_eq!(bounds(part(&parts, 307)), [192.0, 64.0, 64.0, 64.0]);
    frame.widget_data = Some(WidgetData::Texture(TextureData {
        source: TextureSource::File("not-tiled.png".into()),
        horiz_tile: true,
        vert_tile: true,
        ..default()
    }));
    let parts = project_with_loader(&frame, &mut load);
    assert_eq!(parts.len(), 1);
}

#[test]
fn button_hover_alpha_is_authored_in_native_image_output() {
    let mut registry = crate::registry::FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("Choice", None);
    let mut authored = frame();
    authored.widget_data = Some(WidgetData::Button(ButtonData {
        hovered: true,
        highlight_texture: Some(TextureSource::File("hover.png".into())),
        use_default_skin: false,
        ..default()
    }));
    *registry.get_mut(id).unwrap() = authored;
    crate::attrs::apply_attribute(
        &mut registry,
        id,
        "button_highlight_alpha",
        "0.15",
        &mut Default::default(),
        &mut Default::default(),
    );
    let output = project_with_loader(registry.get(id).unwrap(), &mut load);
    assert_eq!(
        output.len(),
        1,
        "hover-only choice must not gain a base image"
    );
    assert_eq!(
        part(&output, 1_000_000).image.color,
        Color::srgba(1.0, 1.0, 1.0, 0.075)
    );
    assert_eq!(
        crate::attrs::read_attribute(&registry, id, "button_highlight_alpha").as_deref(),
        Some("0.15")
    );
}

#[test]
fn button_states_and_overlay_keep_existing_selection_rules() {
    let mut frame = frame();
    frame.widget_data = Some(WidgetData::Button(ButtonData {
        normal_texture: Some(TextureSource::File("normal.png".into())),
        highlight_texture: Some(TextureSource::File("highlight.png".into())),
        hovered: true,
        ..default()
    }));
    let parts = project_with_loader(&frame, &mut load);
    assert_eq!(parts.len(), 2);
    assert_eq!(
        part(&parts, 1_000_000).image.color,
        Color::srgba(1.0, 1.0, 1.0, 0.25)
    );
    let Some(WidgetData::Button(button)) = frame.widget_data.as_mut() else {
        unreachable!()
    };
    button.state = crate::widgets::button::ButtonState::Disabled;
    let parts = project_with_loader(&frame, &mut load);
    assert_eq!(parts.len(), 1);
    frame.nine_slice = Some(NineSlice::default());
    let parts = project_with_loader(&frame, &mut load);
    assert!(parts.iter().all(|part| part.key != 1_000_000));
}

#[test]
fn interaction_only_buttons_do_not_cover_their_child_icon_in_any_state() {
    let mut button_frame = frame();
    button_frame.widget_data = Some(WidgetData::Button(ButtonData::default()));
    let mut icon_frame = frame();
    icon_frame.widget_data = Some(WidgetData::Texture(TextureData {
        source: TextureSource::Dynamic(Handle::default()),
        ..default()
    }));
    let icon_parts = project_with_loader(&icon_frame, &mut load);
    assert_eq!(icon_parts.len(), 1, "the child icon must remain projected");
    assert_eq!(bounds(&icon_parts[0]), [0.0, 0.0, 200.0, 100.0]);

    for (state, hovered) in [
        (ButtonState::Normal, false),
        (ButtonState::Normal, true),
        (ButtonState::Pushed, false),
        (ButtonState::Disabled, false),
    ] {
        let Some(WidgetData::Button(button)) = button_frame.widget_data.as_mut() else {
            unreachable!()
        };
        button.state = state;
        button.hovered = hovered;
        assert!(
            project_with_loader(&button_frame, &mut load).is_empty(),
            "unpainted {state:?} button must not cover transparent icon pixels"
        );
    }
}

#[test]
fn prepared_interaction_only_button_has_no_implicit_sliced_background() {
    let mut app = App::new();
    app.insert_resource(crate::plugin::UiState {
        registry: crate::registry::FrameRegistry::new(800.0, 600.0),
        event_bus: crate::event::EventBus::new(),
        focused_frame: None,
    });
    app.add_systems(Update, crate::render_button::sync_button_nine_slices);
    let id = {
        let mut ui = app.world_mut().resource_mut::<crate::plugin::UiState>();
        let id = ui.registry.create_frame("InteractionOnlyButton", None);
        let frame = ui.registry.get_mut(id).unwrap();
        frame.visible = true;
        frame.width = Dimension::Fixed(79.0);
        frame.height = Dimension::Fixed(79.0);
        frame.effective_alpha = 1.0;
        frame.widget_data = Some(WidgetData::Button(ButtonData::default()));
        crate::attrs::apply_attribute(
            &mut ui.registry,
            id,
            "button_default_skin",
            "false",
            &mut Default::default(),
            &mut Default::default(),
        );
        id
    };
    app.update();
    let ui = app.world().resource::<crate::plugin::UiState>();
    let frame = ui.registry.get(id).unwrap();
    let parts = project_with_loader(frame, &mut load);
    assert!(
        parts.is_empty(),
        "an interaction-only button must not paint a default nine-slice behind its icon: keys={:?}",
        parts.iter().map(|part| part.key).collect::<Vec<_>>()
    );
    {
        let mut ui = app.world_mut().resource_mut::<crate::plugin::UiState>();
        let Some(WidgetData::Button(button)) =
            ui.registry.get_mut(id).unwrap().widget_data.as_mut()
        else {
            unreachable!()
        };
        button.highlight_texture = Some(TextureSource::File("hover-ring.png".into()));
        button.hovered = true;
    }
    app.update();
    {
        let ui = app.world().resource::<crate::plugin::UiState>();
        let parts = project_with_loader(ui.registry.get(id).unwrap(), &mut load);
        assert_eq!(
            parts.len(),
            1,
            "hover-only art must not turn into a base slice"
        );
        assert_eq!(parts[0].key, 1_000_000);
    }
    {
        let mut ui = app.world_mut().resource_mut::<crate::plugin::UiState>();
        let Some(WidgetData::Button(button)) =
            ui.registry.get_mut(id).unwrap().widget_data.as_mut()
        else {
            unreachable!()
        };
        button.hovered = false;
        crate::attrs::apply_attribute(
            &mut ui.registry,
            id,
            "button_default_skin",
            "true",
            &mut Default::default(),
            &mut Default::default(),
        );
    }
    app.update();
    let ui = app.world().resource::<crate::plugin::UiState>();
    let parts = project_with_loader(ui.registry.get(id).unwrap(), &mut load);
    assert_eq!(
        parts
            .iter()
            .filter(|part| (100..=108).contains(&part.key))
            .count(),
        9,
        "enabling the ordinary default skin should restore all nine atlas parts"
    );
    {
        let mut ui = app.world_mut().resource_mut::<crate::plugin::UiState>();
        crate::attrs::apply_attribute(
            &mut ui.registry,
            id,
            "button_default_skin",
            "false",
            &mut Default::default(),
            &mut Default::default(),
        );
    }
    app.update();
    let ui = app.world().resource::<crate::plugin::UiState>();
    let frame = ui.registry.get(id).unwrap();
    assert!(
        frame.nine_slice.is_none(),
        "turning off default skin must remove derived atlas state"
    );
    assert!(project_with_loader(frame, &mut load).is_empty());
}

#[test]
fn opted_out_buttons_keep_authored_slices_and_state_textures() {
    let mut app = App::new();
    app.insert_resource(crate::plugin::UiState {
        registry: crate::registry::FrameRegistry::new(800.0, 600.0),
        event_bus: crate::event::EventBus::new(),
        focused_frame: None,
    });
    app.add_systems(Update, crate::render_button::sync_button_nine_slices);
    let id = {
        let mut ui = app.world_mut().resource_mut::<crate::plugin::UiState>();
        let id = ui.registry.create_frame("AuthoredArt", None);
        let frame = ui.registry.get_mut(id).unwrap();
        frame.visible = true;
        frame.width = Dimension::Fixed(120.0);
        frame.height = Dimension::Fixed(40.0);
        frame.effective_alpha = 1.0;
        frame.widget_data = Some(WidgetData::Button(ButtonData {
            normal_texture: Some(TextureSource::File("normal.png".into())),
            pushed_texture: Some(TextureSource::File("pressed.png".into())),
            ..default()
        }));
        crate::attrs::apply_attribute(
            &mut ui.registry,
            id,
            "button_default_skin",
            "false",
            &mut Default::default(),
            &mut Default::default(),
        );
        id
    };
    app.update();
    for state in [
        ButtonState::Normal,
        ButtonState::Pushed,
        ButtonState::Disabled,
    ] {
        {
            let ui = app.world().resource::<crate::plugin::UiState>();
            let frame = ui.registry.get(id).unwrap();
            let parts = project_with_loader(frame, &mut load);
            assert_eq!(
                parts
                    .iter()
                    .filter(|part| (100..=108).contains(&part.key))
                    .count(),
                9,
                "authored base texture must survive {state:?}"
            );
        }
        {
            let mut ui = app.world_mut().resource_mut::<crate::plugin::UiState>();
            let Some(WidgetData::Button(button)) =
                ui.registry.get_mut(id).unwrap().widget_data.as_mut()
            else {
                unreachable!()
            };
            button.state = match state {
                ButtonState::Normal => ButtonState::Pushed,
                ButtonState::Pushed => ButtonState::Disabled,
                ButtonState::Disabled => ButtonState::Normal,
            };
        }
        app.update();
    }
    let authored_nine_slice = NineSlice {
        texture: Some(TextureSource::File("authored-panel.png".into())),
        edge_size: 7.0,
        ..default()
    };
    {
        let mut ui = app.world_mut().resource_mut::<crate::plugin::UiState>();
        let frame = ui.registry.get_mut(id).unwrap();
        frame.nine_slice = Some(authored_nine_slice.clone());
        if let Some(WidgetData::Button(button)) = frame.widget_data.as_mut() {
            button.normal_texture = None;
            button.pushed_texture = None;
        }
    }
    app.update();
    let ui = app.world().resource::<crate::plugin::UiState>();
    assert_eq!(
        ui.registry.get(id).unwrap().nine_slice,
        Some(authored_nine_slice)
    );
    let authored_three_slice = ThreeSlice {
        cap_width: 12.0,
        left: TextureSource::File("left.png".into()),
        center: TextureSource::File("center.png".into()),
        right: TextureSource::File("right.png".into()),
        color: [1.0; 4],
    };
    {
        let mut ui = app.world_mut().resource_mut::<crate::plugin::UiState>();
        let frame = ui.registry.get_mut(id).unwrap();
        frame.nine_slice = None;
        frame.three_slice = Some(authored_three_slice.clone());
    }
    app.update();
    let ui = app.world().resource::<crate::plugin::UiState>();
    let frame = ui.registry.get(id).unwrap();
    assert!(
        frame.nine_slice.is_none(),
        "explicit three-slice must not acquire a default nine-slice"
    );
    assert_eq!(frame.three_slice, Some(authored_three_slice));
    assert_eq!(project_with_loader(frame, &mut load).len(), 3);
}

#[test]
fn highlight_only_button_projects_an_overlay_without_a_base() {
    let mut frame = frame();
    frame.widget_data = Some(WidgetData::Button(ButtonData {
        highlight_texture: Some(TextureSource::File("highlight.png".into())),
        hovered: true,
        ..default()
    }));
    let parts = project_with_loader(&frame, &mut load);
    assert_eq!(parts.len(), 1, "highlight-only button has no authored base");
    assert_eq!(parts[0].key, 1_000_000);
    assert_eq!(parts[0].image.color, Color::srgba(1.0, 1.0, 1.0, 0.25));

    let Some(WidgetData::Button(button)) = frame.widget_data.as_mut() else {
        unreachable!()
    };
    button.state = ButtonState::Disabled;
    assert!(project_with_loader(&frame, &mut load).is_empty());
}

#[test]
fn authored_button_backgrounds_and_textures_survive_without_missing_replacements() {
    let mut frame = frame();
    frame.widget_data = Some(WidgetData::Button(ButtonData::default()));
    frame.background_color = Some([0.2, 0.4, 0.6, 0.8]);
    let parts = project_with_loader(&frame, &mut load);
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].key, 0);
    assert_eq!(parts[0].image.color, Color::srgba(0.2, 0.4, 0.6, 0.4));

    frame.background_color = None;
    frame.backdrop = Some(Backdrop {
        bg_color: Some([0.6, 0.2, 0.1, 1.0]),
        ..default()
    });
    let parts = project_with_loader(&frame, &mut load);
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].image.color, Color::srgba(0.6, 0.2, 0.1, 0.5));

    frame.backdrop = None;
    let Some(WidgetData::Button(button)) = frame.widget_data.as_mut() else {
        unreachable!()
    };
    button.normal_texture = Some(TextureSource::File("normal.png".into()));
    button.pushed_texture = Some(TextureSource::File("pressed.png".into()));
    button.disabled_texture = Some(TextureSource::File("disabled.png".into()));
    for state in [
        ButtonState::Normal,
        ButtonState::Pushed,
        ButtonState::Disabled,
    ] {
        let Some(WidgetData::Button(button)) = frame.widget_data.as_mut() else {
            unreachable!()
        };
        button.state = state;
        let parts = project_with_loader(&frame, &mut load);
        assert_eq!(parts.len(), 1, "authored {state:?} texture must render");
        assert_eq!(parts[0].key, 0);
        assert_eq!(
            parts[0].image.rect,
            Some(Rect::new(10.0, 20.0, 110.0, 70.0))
        );
        assert!(
            project_with_loader(&frame, &mut |_| None).is_empty(),
            "failed authored {state:?} texture must not substitute a white quad"
        );
    }
}

#[test]
fn hidden_frames_and_failed_textures_do_not_emit_white_replacements() {
    let mut frame = frame();
    frame.widget_data = Some(WidgetData::Texture(TextureData {
        source: TextureSource::File("missing.png".into()),
        ..default()
    }));
    assert!(project_with_loader(&frame, &mut |_| None).is_empty());
    frame.visible = false;
    assert!(project_with_loader(&frame, &mut load).is_empty());
}
