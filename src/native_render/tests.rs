use super::*;
use crate::anchor::{AnchorPoint, AnchorTarget};
use crate::frame::{Dimension, WidgetType};
use crate::widgets::{edit_box::EditBoxData, font_string::FontStringData};
use bevy::asset::{AssetApp, AssetPlugin};
use bevy::camera::{CameraPlugin, CameraUpdateSystems, ComputedCameraValues, RenderTargetInfo};
use bevy::math::Affine2;
use bevy::time::TimeUpdateStrategy;
use bevy::window::PrimaryWindow;
use std::time::Duration;

pub(crate) fn app_with_real_fonts(scale: f32) -> App {
    assert!(std::path::Path::new(GameFont::ArialNarrow.path()).is_file());
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        bevy::image::ImagePlugin::default(),
        bevy::mesh::MeshPlugin,
        bevy::window::WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        },
        bevy::input::InputPlugin,
        bevy::transform::TransformPlugin,
        CameraPlugin,
        bevy::text::TextPlugin,
        bevy::picking::DefaultPickingPlugins,
        bevy::ui::UiPlugin,
        crate::plugin::UiPlugin,
    ));
    app.init_asset::<TextureAtlasLayout>();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    let mut window = Window {
        resolution: ((800.0 * scale) as u32, (600.0 * scale) as u32).into(),
        ..default()
    };
    window.resolution.set_scale_factor_override(Some(scale));
    app.world_mut().spawn((window, PrimaryWindow));
    app.add_systems(
        PostUpdate,
        update_camera_target
            .after(CameraUpdateSystems)
            .before(bevy::ui::UiSystems::Prepare),
    );
    app.finish();
    app.cleanup();
    app
}

// Supply the render-target metadata normally provided by a windowed renderer;
// layout, shaping, transforms, clipping and registry synchronization remain real systems.
fn update_camera_target(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<UiCamera>>,
) {
    let window = windows.single().unwrap();
    for mut camera in &mut cameras {
        camera.computed = ComputedCameraValues {
            target_info: Some(RenderTargetInfo {
                physical_size: UVec2::new(window.physical_width(), window.physical_height()),
                scale_factor: window.scale_factor(),
            }),
            ..default()
        };
    }
}

fn create_frame(
    world: &mut World,
    name: &str,
    parent: Option<u64>,
    width: f32,
    height: f32,
) -> u64 {
    let mut ui = world.resource_mut::<UiState>();
    let id = ui.registry.create_frame(name, parent);
    let frame = ui.registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(width);
    frame.height = Dimension::Fixed(height);
    id
}

fn anchor(world: &mut World, id: u64, point: AnchorPoint, target: Option<u64>, x: f32, y: f32) {
    let mut ui = world.resource_mut::<UiState>();
    assert_eq!(ui.registry.get(id).unwrap().parent_id, target);
    ui.registry
        .set_pos_type(id, PositionType::Absolute)
        .unwrap();
    ui.registry.set_anchor(id, AnchorTarget::Parent).unwrap();
    let (horizontal, vertical) = crate::anchor::anchor_position(point, 0.0, 0.0, 1.0, 1.0);
    let frame = ui.registry.get_mut(id).unwrap();
    frame.position.left = percent(horizontal * 100.0);
    frame.position.top = percent(vertical * 100.0);
    frame.translation = Val2::percent(-horizontal * 100.0, -vertical * 100.0);
    frame.margin.left = px(x);
    frame.margin.top = px(-y);
}

fn projected_frame(world: &mut World, id: u64) -> Entity {
    let found: Vec<_> = world
        .query::<(Entity, &RegistryNode)>()
        .iter(world)
        .filter(|(_, frame)| frame.0 == id)
        .map(|(entity, _)| entity)
        .collect();
    assert_eq!(found.len(), 1, "one native entity for registry frame {id}");
    found[0]
}

fn projected_text(world: &mut World, id: u64) -> (Entity, Entity) {
    world
        .query::<(Entity, &RegistryText)>()
        .iter(world)
        .find(|(_, text)| text.frame_id == id && text.key == 0)
        .map(|(entity, text)| (entity, text.bounds))
        .expect("projected text")
}

fn logical_rect(world: &World, entity: Entity) -> Rect {
    let node = world.get::<ComputedNode>(entity).unwrap();
    let transform = Affine2::from(world.get::<UiGlobalTransform>(entity).unwrap());
    Rect::from_center_size(
        transform.translation * node.inverse_scale_factor,
        node.size * node.inverse_scale_factor,
    )
}

fn assert_registry_bounds(world: &mut World, id: u64) {
    let entity = projected_frame(world, id);
    let actual = logical_rect(world, entity);
    let expected = world
        .resource::<UiState>()
        .registry
        .get(id)
        .unwrap()
        .layout_rect
        .as_ref()
        .unwrap();
    assert!(
        (actual.min - Vec2::new(expected.x, expected.y))
            .abs()
            .max_element()
            <= 1.0,
        "{actual:?} vs {expected:?}"
    );
    assert!(
        (actual.size() - Vec2::new(expected.width, expected.height))
            .abs()
            .max_element()
            <= 1.0
    );
}

fn settle(app: &mut App) {
    app.update();
    app.update();
}

#[test]
fn registry_anchors_size_and_resize_match_native_bounds_without_replacing_frames() {
    for scale in [1.0, 2.0] {
        let mut app = app_with_real_fonts(scale);
        let root = create_frame(app.world_mut(), "Panel", None, 240.0, 120.0);
        anchor(app.world_mut(), root, AnchorPoint::Center, None, 0.0, 0.0);
        let child = create_frame(app.world_mut(), "Child", Some(root), 80.0, 24.0);
        anchor(
            app.world_mut(),
            child,
            AnchorPoint::TopLeft,
            Some(root),
            12.0,
            -14.0,
        );
        settle(&mut app);
        let root_entity = projected_frame(app.world_mut(), root);
        let child_entity = projected_frame(app.world_mut(), child);
        assert_registry_bounds(app.world_mut(), root);
        assert_registry_bounds(app.world_mut(), child);
        assert_eq!(
            app.world().get::<ChildOf>(child_entity).unwrap().parent(),
            root_entity
        );
        assert!(
            (logical_rect(app.world(), child_entity).min
                - logical_rect(app.world(), root_entity).min
                - Vec2::new(12.0, 14.0))
            .length()
                < 1.0
        );
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let frame = ui.registry.get_mut(root).unwrap();
            frame.width = Dimension::Fixed(320.0);
            frame.height = Dimension::Fixed(160.0);
            ui.registry.mark_rect_dirty(root);
        }
        anchor(
            app.world_mut(),
            child,
            AnchorPoint::TopLeft,
            Some(root),
            30.0,
            -20.0,
        );
        {
            let world = app.world_mut();
            let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
            windows
                .single_mut(world)
                .unwrap()
                .resolution
                .set_physical_resolution((1000.0 * scale) as u32, (700.0 * scale) as u32);
        }
        settle(&mut app);
        assert_eq!(projected_frame(app.world_mut(), root), root_entity);
        assert_eq!(projected_frame(app.world_mut(), child), child_entity);
        assert_registry_bounds(app.world_mut(), root);
        assert_registry_bounds(app.world_mut(), child);
        assert!(
            (logical_rect(app.world(), root_entity).center() - Vec2::new(500.0, 350.0)).length()
                <= 1.0
        );
    }
}

#[test]
fn registry_text_colors_alpha_and_visibility_reach_native_outputs_without_idle_writes() {
    for scale in [1.0, 2.0] {
        let mut app = app_with_real_fonts(scale);
        let panel = create_frame(app.world_mut(), "Panel", None, 240.0, 80.0);
        let label = create_frame(app.world_mut(), "Label", Some(panel), 180.0, 24.0);
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            ui.registry.get_mut(panel).unwrap().background_color = Some([0.2, 0.4, 0.6, 0.8]);
            let frame = ui.registry.get_mut(label).unwrap();
            frame.widget_type = WidgetType::FontString;
            frame.widget_data = Some(WidgetData::FontString(FontStringData {
                text: "Before".into(),
                font: GameFont::ArialNarrow,
                color: [1.0, 0.5, 0.25, 0.8],
                ..default()
            }));
        }
        settle(&mut app);
        let panel_entity = projected_frame(app.world_mut(), panel);
        let (text_entity, _) = projected_text(app.world_mut(), label);
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let Some(WidgetData::FontString(data)) =
                &mut ui.registry.get_mut(label).unwrap().widget_data
            else {
                panic!("font string");
            };
            data.text = "Changed by registry".into();
            ui.registry.set_alpha(panel, 0.5);
        }
        settle(&mut app);
        assert_eq!(projected_text(app.world_mut(), label).0, text_entity);
        assert_eq!(
            app.world().get::<Text>(text_entity).unwrap().0,
            "Changed by registry"
        );
        assert!((app.world().get::<TextColor>(text_entity).unwrap().0.alpha() - 0.4).abs() < 0.001);
        let image = app
            .world_mut()
            .query::<(Entity, &RegistryImage)>()
            .iter(app.world())
            .find(|(_, part)| part.frame_id == panel && part.key == 0)
            .unwrap()
            .0;
        assert!((app.world().get::<ImageNode>(image).unwrap().color.alpha() - 0.4).abs() < 0.001);
        app.world_mut().clear_trackers();
        app.update();
        assert!(
            !app.world()
                .entity(text_entity)
                .get_ref::<Text>()
                .unwrap()
                .is_changed()
        );
        assert!(
            !app.world()
                .entity(image)
                .get_ref::<ImageNode>()
                .unwrap()
                .is_changed()
        );
        app.world_mut()
            .resource_mut::<UiState>()
            .registry
            .set_hidden(panel, true);
        settle(&mut app);
        assert_eq!(projected_frame(app.world_mut(), panel), panel_entity);
        assert_eq!(logical_rect(app.world(), panel_entity).size(), Vec2::ZERO);
        assert!(
            !app.world()
                .resource::<UiState>()
                .registry
                .get(label)
                .unwrap()
                .visible
        );
        app.world_mut()
            .resource_mut::<UiState>()
            .registry
            .set_hidden(panel, false);
        settle(&mut app);
        assert_registry_bounds(app.world_mut(), panel);
        let (restored, _) = projected_text(app.world_mut(), label);
        assert_eq!(
            app.world().get::<Text>(restored).unwrap().0,
            "Changed by registry"
        );
    }
}

#[test]
fn registry_reparent_and_removal_preserve_logical_identity_and_clean_native_subtrees() {
    let mut app = app_with_real_fonts(1.0);
    let first = create_frame(app.world_mut(), "First", None, 200.0, 80.0);
    let second = create_frame(app.world_mut(), "Second", None, 200.0, 80.0);
    anchor(
        app.world_mut(),
        second,
        AnchorPoint::TopLeft,
        None,
        300.0,
        -100.0,
    );
    let child = create_frame(app.world_mut(), "Moved", Some(first), 50.0, 20.0);
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let frame = ui.registry.get_mut(child).unwrap();
        frame.widget_type = WidgetType::FontString;
        frame.widget_data = Some(WidgetData::FontString(FontStringData {
            text: "Owned child".into(),
            font: GameFont::ArialNarrow,
            ..default()
        }));
    }
    settle(&mut app);
    let entity = projected_frame(app.world_mut(), child);
    let (child_text, child_bounds) = projected_text(app.world_mut(), child);
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        ui.registry
            .get_mut(first)
            .unwrap()
            .children
            .retain(|id| *id != child);
        ui.registry.get_mut(second).unwrap().children.push(child);
        ui.registry.get_mut(child).unwrap().parent_id = Some(second);
        ui.registry.mark_rect_dirty(child);
    }
    anchor(
        app.world_mut(),
        child,
        AnchorPoint::TopLeft,
        Some(second),
        7.0,
        -9.0,
    );
    settle(&mut app);
    assert_eq!(projected_frame(app.world_mut(), child), entity);
    let second_entity = projected_frame(app.world_mut(), second);
    assert_eq!(
        app.world().get::<ChildOf>(entity).unwrap().parent(),
        second_entity
    );
    assert_registry_bounds(app.world_mut(), child);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .remove_frame_tree(second);
    settle(&mut app);
    assert!(app.world().get_entity(entity).is_err());
    assert!(app.world().get_entity(second_entity).is_err());
    assert!(app.world().get_entity(child_text).is_err());
    assert!(app.world().get_entity(child_bounds).is_err());
    assert_eq!(
        app.world_mut()
            .query::<&RegistryNode>()
            .iter(app.world())
            .count(),
        1
    );
    assert_eq!(
        app.world()
            .resource::<UiState>()
            .registry
            .get_by_name("Moved"),
        None
    );
    assert_registry_bounds(app.world_mut(), first);
}

fn caret_entity(world: &mut World, bounds: Entity) -> Entity {
    world
        .query::<(Entity, &ChildOf, &BackgroundColor)>()
        .iter(world)
        .find(|(_, parent, _)| parent.parent() == bounds)
        .map(|(entity, _, _)| entity)
        .expect("caret visual")
}

#[test]
fn registry_password_cursor_and_blink_drive_shaped_caret_at_both_scales() {
    for scale in [1.0, 2.0] {
        let mut app = app_with_real_fonts(scale);
        let field = create_frame(app.world_mut(), "Password", None, 300.0, 40.0);
        anchor(
            app.world_mut(),
            field,
            AnchorPoint::TopLeft,
            None,
            100.0,
            -50.0,
        );
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let frame = ui.registry.get_mut(field).unwrap();
            frame.widget_type = WidgetType::EditBox;
            frame.widget_data = Some(WidgetData::EditBox(EditBoxData {
                text: "éx".into(),
                cursor_position: 2,
                password: true,
                font: GameFont::ArialNarrow,
                font_size: 20.0,
                blink_speed: 0.25,
                text_insets: [8.0, 8.0, 4.0, 4.0],
                ..default()
            }));
            ui.focused_frame = Some(field);
        }
        settle(&mut app);
        let (text, bounds) = projected_text(app.world_mut(), field);
        let caret = caret_entity(app.world_mut(), bounds);
        assert_eq!(app.world().get::<Text>(text).unwrap().0, "***");
        let block = app
            .world()
            .get::<bevy::text::ComputedTextBlock>(text)
            .unwrap();
        assert!(
            block.buffer().width() > 0.0,
            "real font shaping must produce width"
        );
        let caret_rect = logical_rect(app.world(), caret);
        let text_rect = logical_rect(app.world(), text);
        assert!(
            caret_rect.min.x > text_rect.min.x
                && caret_rect.min.x < text_rect.min.x + block.buffer().width() / scale
        );
        assert!((caret_rect.width() - 2.0).abs() < 0.01);
        assert!(app.world().get::<BackgroundColor>(caret).unwrap().0.alpha() > 0.0);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            300,
        )));
        app.update();
        assert_eq!(
            app.world().get::<BackgroundColor>(caret).unwrap().0.alpha(),
            0.0
        );
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let Some(WidgetData::EditBox(edit)) =
                &mut ui.registry.get_mut(field).unwrap().widget_data
            else {
                panic!("edit box");
            };
            edit.cursor_right();
        }
        app.update();
        assert!(app.world().get::<BackgroundColor>(caret).unwrap().0.alpha() > 0.0);
        assert!(logical_rect(app.world(), caret).min.x > caret_rect.min.x);
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let Some(WidgetData::EditBox(edit)) =
                &mut ui.registry.get_mut(field).unwrap().widget_data
            else {
                panic!("edit box");
            };
            assert_eq!(edit.text, "éx");
            edit.insert_at_cursor("Q");
        }
        app.update();
        assert_eq!(app.world().get::<Text>(text).unwrap().0, "****");
        assert!(app.world().get::<BackgroundColor>(caret).unwrap().0.alpha() > 0.0);
        app.world_mut().resource_mut::<caret::UiCaretBlocked>().0 = true;
        app.update();
        assert_eq!(
            app.world().get::<BackgroundColor>(caret).unwrap().0.alpha(),
            0.0
        );
        app.world_mut().resource_mut::<caret::UiCaretBlocked>().0 = false;
        app.update();
        assert!(app.world().get::<BackgroundColor>(caret).unwrap().0.alpha() > 0.0);
        app.world_mut().resource_mut::<UiState>().focused_frame = None;
        app.update();
        assert_eq!(
            app.world().get::<BackgroundColor>(caret).unwrap().0.alpha(),
            0.0
        );
        let ui = app.world().resource::<UiState>();
        let Some(WidgetData::EditBox(edit)) = &ui.registry.get(field).unwrap().widget_data else {
            panic!("edit box");
        };
        assert_eq!(edit.text, "éxQ");
        assert_eq!(edit.cursor_position, 4);
    }
}
