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

/// Enables synchronization of UI text into Bevy render entities.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct UiTextRenderEnabled(pub bool);

impl Default for UiTextRenderEnabled {
    fn default() -> Self {
        Self(true)
    }
}

/// Enables synchronization of UI text shadows into Bevy render entities.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct UiTextShadowRenderEnabled(pub bool);

impl Default for UiTextShadowRenderEnabled {
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
        app.insert_resource(UiTextRenderEnabled::default());
        app.insert_resource(UiTextShadowRenderEnabled::default());
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
                    crate::render_text_fx::sync_ui_text_shadows
                        .run_if(ui_text_shadow_render_enabled),
                    crate::render_text_fx::sync_ui_text_outlines,
                )
                    .chain()
                    .run_if(ui_text_render_enabled),
            )
                .chain()
                .run_if(ui_render_enabled),
            crate::button_input::sync_button_input,
        )
            .chain(),
    );
}

fn ui_render_enabled(enabled: Res<UiRenderEnabled>) -> bool {
    enabled.0
}

fn ui_text_render_enabled(enabled: Res<UiTextRenderEnabled>) -> bool {
    enabled.0
}

fn ui_text_shadow_render_enabled(enabled: Res<UiTextShadowRenderEnabled>) -> bool {
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
    crate::layout::recompute_layouts(&mut state.registry);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_adds_ui_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_asset::<bevy::text::Font>();
        app.add_plugins(UiPlugin);
        app.update();
        assert!(app.world().get_resource::<UiState>().is_some());
        assert_eq!(
            app.world().resource::<UiTextShadowRenderEnabled>(),
            &UiTextShadowRenderEnabled(true)
        );
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

    #[test]
    fn disabled_ui_text_shadows_keep_main_text_and_outlines() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_asset::<bevy::image::Image>();
        app.init_asset::<bevy::text::Font>();
        app.add_plugins(UiPlugin);

        let frame_id = {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let frame_id = ui.registry.create_frame("EffectText", None);
            let frame = ui.registry.get_mut(frame_id).unwrap();
            frame.width = crate::frame::Dimension::Fixed(100.0);
            frame.height = crate::frame::Dimension::Fixed(40.0);
            frame.widget_data = Some(crate::frame::WidgetData::FontString(
                crate::widgets::font_string::FontStringData {
                    text: "Hello".into(),
                    shadow_color: Some([0.0, 0.0, 0.0, 1.0]),
                    outline: crate::widgets::font_string::Outline::Outline,
                    ..Default::default()
                },
            ));
            frame_id
        };
        app.world_mut()
            .resource_mut::<UiTextShadowRenderEnabled>()
            .0 = false;

        app.update();

        assert_eq!(ui_text_count(&mut app, frame_id), 1);
        assert_eq!(ui_shadow_count(&mut app, frame_id), 0);
        assert_eq!(ui_outline_count(&mut app, frame_id), 4);

        app.world_mut()
            .resource_mut::<UiTextShadowRenderEnabled>()
            .0 = true;
        app.update();

        assert_eq!(ui_shadow_count(&mut app, frame_id), 1);
        assert_eq!(ui_outline_count(&mut app, frame_id), 4);
    }

    fn ui_shadow_count(app: &mut App, frame_id: u64) -> usize {
        let mut query = app
            .world_mut()
            .query::<&crate::render_text_fx::UiTextShadow>();
        query
            .iter(app.world())
            .filter(|shadow| shadow.0 == frame_id)
            .count()
    }

    fn ui_outline_count(app: &mut App, frame_id: u64) -> usize {
        let mut query = app
            .world_mut()
            .query::<&crate::render_text_fx::UiTextOutline>();
        query
            .iter(app.world())
            .filter(|outline| outline.0 == frame_id)
            .count()
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
