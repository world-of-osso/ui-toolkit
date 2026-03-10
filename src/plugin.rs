use bevy::prelude::*;

use crate::event::EventBus;
use crate::registry::FrameRegistry;

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
        app.init_resource::<crate::font_registry::FontRegistry>();
        app.add_systems(
            Startup,
            (initialize_screen_size, crate::render::setup_ui_camera).chain(),
        );
        app.add_systems(
            Update,
            (
                sync_screen_size,
                recompute_layout,
                crate::render_button::sync_button_nine_slices,
                crate::render::sync_ui_quads,
                crate::render_button::sync_ui_button_highlights,
                crate::render_text::sync_ui_text,
                crate::render_border::sync_ui_borders,
                crate::render_nine_slice::sync_ui_nine_slices,
                crate::render_tiled::sync_ui_tiled_textures,
                crate::render_text_fx::sync_ui_text_shadows,
                crate::render_text_fx::sync_ui_text_outlines,
            )
                .chain(),
        );
    }
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
    }
}
