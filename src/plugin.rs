use bevy::prelude::*;

use crate::event::EventBus;
use crate::registry::FrameRegistry;

/// Enables synchronization of the UI registry into Bevy render entities.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct UiRenderEnabled(pub bool);

impl Default for UiRenderEnabled {
    fn default() -> Self {
        Self(true)
    }
}

/// Enables UI registry, layout, input, and render synchronization.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct UiProcessingEnabled(pub bool);

impl Default for UiProcessingEnabled {
    fn default() -> Self {
        Self(true)
    }
}

/// Enables synchronization of UI text into Bevy render entities.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct UiTextRenderEnabled(pub bool);

impl Default for UiTextRenderEnabled {
    fn default() -> Self {
        Self(true)
    }
}

/// Central UI state, accessible as a Bevy Resource.
#[derive(Resource)]
pub struct UiState {
    pub registry: FrameRegistry,
    pub event_bus: EventBus,
    /// Currently focused frame (receives keyboard input).
    pub focused_frame: Option<u64>,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        let state = UiState {
            registry: FrameRegistry::new(0.0, 0.0),
            event_bus: EventBus::new(),
            focused_frame: None,
        };
        app.insert_resource(state);
        app.insert_resource(UiRenderEnabled::default());
        app.insert_resource(UiProcessingEnabled::default());
        app.insert_resource(UiTextRenderEnabled::default());
        app.init_resource::<crate::font_registry::FontRegistry>();
        register_ui_startup_systems(app);
        register_ui_update_systems(app);
    }
}

fn register_ui_startup_systems(app: &mut App) {
    app.add_systems(
        Startup,
        (initialize_screen_size, crate::render::setup_ui_camera).chain(),
    );
}

fn register_ui_update_systems(app: &mut App) {
    #[cfg(debug_assertions)]
    app.add_systems(
        Update,
        crate::screen::poll_hot_reload
            .before(sync_screen_size)
            .run_if(bevy::time::common_conditions::on_real_timer(
                std::time::Duration::from_secs(1),
            ))
            .run_if(ui_processing_enabled),
    );
    app.add_systems(
        Update,
        (
            sync_screen_size,
            recompute_layout,
            crate::render_button::sync_button_nine_slices,
            (
                crate::render::sync_ui_quads,
                crate::render_button::sync_ui_button_highlights,
                crate::render_text::sync_ui_text.run_if(ui_text_render_enabled),
                crate::render_border::sync_ui_borders,
                crate::render_border::sync_css_borders,
                crate::render_nine_slice::sync_ui_nine_slices,
                crate::render_three_slice::sync_ui_three_slices,
                crate::render_tiled::sync_ui_tiled_textures,
                (
                    crate::render_text_fx::sync_ui_text_shadows,
                    crate::render_text_fx::sync_ui_text_outlines,
                )
                    .chain()
                    .run_if(ui_text_render_enabled),
            )
                .chain()
                .run_if(ui_render_enabled),
            crate::button_input::sync_button_input,
        )
            .chain()
            .run_if(ui_processing_enabled),
    );
}

fn ui_processing_enabled(enabled: Res<UiProcessingEnabled>) -> bool {
    enabled.0
}

fn ui_render_enabled(enabled: Res<UiRenderEnabled>) -> bool {
    enabled.0
}

fn ui_text_render_enabled(enabled: Res<UiTextRenderEnabled>) -> bool {
    enabled.0
}

pub fn sync_registry_to_primary_window(
    registry: &mut FrameRegistry,
    windows: &Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let (w, h) = (window.width(), window.height());
    if (registry.screen_width - w).abs() > 0.5 || (registry.screen_height - h).abs() > 0.5 {
        registry.screen_width = w;
        registry.screen_height = h;
        registry.mark_all_rects_dirty();
    }
}

fn sync_screen_size(
    mut state: ResMut<UiState>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    sync_registry_to_primary_window(&mut state.registry, &windows);
}

fn initialize_screen_size(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut state: ResMut<UiState>,
) {
    sync_registry_to_primary_window(&mut state.registry, &windows);
}

fn recompute_layout(mut state: ResMut<UiState>) {
    if state.registry.rect_dirty.is_empty() {
        return;
    }
    crate::layout::recompute_layouts(&mut state.registry);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct LayoutCallerChanged(bool);

    fn observe_layout_caller_change(
        state: Res<UiState>,
        mut observed: ResMut<LayoutCallerChanged>,
    ) {
        observed.0 = state.is_changed();
    }

    fn layout_caller_app() -> (App, u64) {
        use crate::anchor::{Anchor, AnchorPoint};
        use crate::frame::Dimension;

        let mut registry = FrameRegistry::new(800.0, 600.0);
        let id = registry.create_frame("LayoutCaller", None);
        let frame = registry.get_mut(id).unwrap();
        frame.width = Dimension::Fixed(80.0);
        frame.height = Dimension::Fixed(30.0);
        registry
            .set_point(
                id,
                Anchor {
                    point: AnchorPoint::TopLeft,
                    relative_to: None,
                    relative_point: AnchorPoint::TopLeft,
                    x_offset: 12.0,
                    y_offset: 0.0,
                },
            )
            .unwrap();
        crate::layout::recompute_layouts(&mut registry);
        registry.render_dirty.clear();
        let mut app = App::new();
        app.insert_resource(UiState {
            registry,
            event_bus: EventBus::new(),
            focused_frame: None,
        });
        app.init_resource::<LayoutCallerChanged>();
        app.add_systems(
            Update,
            (recompute_layout, observe_layout_caller_change).chain(),
        );
        app.update(); // Consume the initial resource insertion notification.
        (app, id)
    }

    #[test]
    fn layout_caller_clean_updates_do_not_mark_ui_state_changed() {
        let (mut app, id) = layout_caller_app();
        let expected = app
            .world()
            .resource::<UiState>()
            .registry
            .get(id)
            .unwrap()
            .layout_rect
            .clone();
        for _ in 0..3 {
            app.update();
            assert!(!app.world().resource::<LayoutCallerChanged>().0);
            let state = app.world().resource::<UiState>();
            assert!(state.registry.rect_dirty.is_empty());
            assert!(state.registry.render_dirty.is_empty());
            assert_eq!(state.registry.get(id).unwrap().layout_rect, expected);
        }
    }

    #[test]
    fn layout_caller_dirty_update_changes_geometry_and_ui_state() {
        let (mut app, id) = layout_caller_app();
        {
            let mut state = app.world_mut().resource_mut::<UiState>();
            // Observe the production callback's notification, not fixture mutation.
            let state = state.bypass_change_detection();
            state.registry.get_mut(id).unwrap().width = crate::frame::Dimension::Fixed(150.0);
            state.registry.mark_rect_dirty(id);
            state.registry.render_dirty.clear();
        }
        app.update();
        assert!(app.world().resource::<LayoutCallerChanged>().0);
        let state = app.world().resource::<UiState>();
        let rect = state
            .registry
            .get(id)
            .unwrap()
            .layout_rect
            .as_ref()
            .unwrap();
        assert_eq!((rect.x, rect.width, rect.height), (12.0, 150.0, 30.0));
        assert!(state.registry.rect_dirty.is_empty());
        assert!(state.registry.render_dirty.contains(&id));
        app.update();
        assert!(!app.world().resource::<LayoutCallerChanged>().0);
    }

    #[test]
    fn plugin_adds_ui_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_asset::<bevy::text::Font>();
        app.add_plugins(UiPlugin);
        app.update();
        assert!(app.world().get_resource::<UiState>().is_some());
    }

    #[test]
    fn disabled_ui_processing_pauses_update_chain_until_reenabled() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_asset::<bevy::image::Image>();
        app.init_asset::<bevy::text::Font>();
        app.add_plugins(UiPlugin);
        app.world_mut().spawn((
            Window {
                resolution: bevy::window::WindowResolution::new(800, 600),
                ..Default::default()
            },
            bevy::window::PrimaryWindow,
        ));
        app.update();

        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            ui.registry.screen_width = 123.0;
            ui.registry.screen_height = 456.0;
        }
        app.world_mut().resource_mut::<UiProcessingEnabled>().0 = false;
        app.update();

        let ui = app.world().resource::<UiState>();
        assert_eq!(ui.registry.screen_width, 123.0);
        assert_eq!(ui.registry.screen_height, 456.0);

        app.world_mut().resource_mut::<UiProcessingEnabled>().0 = true;
        app.update();

        let ui = app.world().resource::<UiState>();
        assert_eq!(ui.registry.screen_width, 800.0);
        assert_eq!(ui.registry.screen_height, 600.0);
    }

    #[test]
    fn disabled_ui_render_keeps_registry_frame_until_reenabled() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_asset::<bevy::image::Image>();
        app.init_asset::<bevy::text::Font>();
        app.add_plugins(UiPlugin);

        let frame_id = {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let frame_id = ui.registry.create_frame("VisiblePanel", None);
            let frame = ui.registry.get_mut(frame_id).unwrap();
            frame.width = crate::frame::Dimension::Fixed(100.0);
            frame.height = crate::frame::Dimension::Fixed(40.0);
            frame.background_color = Some([1.0, 0.0, 0.0, 1.0]);
            frame_id
        };
        app.world_mut().resource_mut::<UiRenderEnabled>().0 = false;

        app.update();

        assert!(
            app.world()
                .resource::<UiState>()
                .registry
                .get(frame_id)
                .is_some()
        );
        assert_eq!(ui_quad_count(&mut app, frame_id), 0);

        app.world_mut().resource_mut::<UiRenderEnabled>().0 = true;
        app.update();

        assert_eq!(ui_quad_count(&mut app, frame_id), 1);
    }

    #[test]
    fn disabled_ui_text_render_keeps_quad_until_reenabled() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_asset::<bevy::image::Image>();
        app.init_asset::<bevy::text::Font>();
        app.add_plugins(UiPlugin);

        let frame_id = {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let frame_id = ui.registry.create_frame("TextPanel", None);
            let frame = ui.registry.get_mut(frame_id).unwrap();
            frame.width = crate::frame::Dimension::Fixed(100.0);
            frame.height = crate::frame::Dimension::Fixed(40.0);
            frame.background_color = Some([1.0, 0.0, 0.0, 1.0]);
            frame.widget_data = Some(crate::frame::WidgetData::FontString(
                crate::widgets::font_string::FontStringData {
                    text: "Hello".into(),
                    ..Default::default()
                },
            ));
            frame_id
        };
        app.world_mut().resource_mut::<UiTextRenderEnabled>().0 = false;

        app.update();

        assert_eq!(ui_quad_count(&mut app, frame_id), 1);
        assert_eq!(ui_text_count(&mut app, frame_id), 0);

        app.world_mut().resource_mut::<UiTextRenderEnabled>().0 = true;
        app.update();

        assert_eq!(ui_text_count(&mut app, frame_id), 1);
    }

    fn ui_text_count(app: &mut App, frame_id: u64) -> usize {
        let mut query = app.world_mut().query_filtered::<&crate::render::UiText, (
            Without<crate::render_text_fx::UiTextShadow>,
            Without<crate::render_text_fx::UiTextOutline>,
        )>();
        query
            .iter(app.world())
            .filter(|text| text.0 == frame_id)
            .count()
    }

    fn ui_quad_count(app: &mut App, frame_id: u64) -> usize {
        let mut query = app.world_mut().query::<&crate::render::UiQuad>();
        query
            .iter(app.world())
            .filter(|quad| quad.0 == frame_id)
            .count()
    }
}
