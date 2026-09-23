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

impl UiState {
    /// Drop stale ownership for removed frames: unregister their event listeners
    /// so they cannot fire and clear keyboard focus held on them.
    pub fn resolve_removed_frames(&mut self) {
        for id in self.registry.drain_removed_frames() {
            self.event_bus.unregister_all(id);
            if self.focused_frame == Some(id) {
                self.focused_frame = None;
            }
        }
    }
}

/// Ordering points for the plugin's shared-order render pipeline.
///
/// Registry changes affecting render order must run before `Prepare`, which
/// includes window sizing, layout, button derivation, and order preparation.
/// Standalone renderer functions retain their independent ordering behavior.
#[derive(SystemSet, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum UiRenderSet {
    Prepare,
    Quads,
    Text,
    Shadows,
    Outlines,
    NineSlices,
    ThreeSlices,
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
        app.init_resource::<crate::render::UiFrameOrder>();
        register_ui_startup_systems(app);
        register_ui_update_systems(app);
        app.init_resource::<crate::native_render::caret::UiCaretBlocked>();
        app.add_systems(
            PostUpdate,
            crate::native_render::layout::read_bounds
                .after(bevy::ui::UiSystems::PostLayout)
                .before(crate::native_render::caret::sync_carets)
                .run_if(ui_processing_enabled)
                .run_if(ui_render_enabled),
        );
        app.add_systems(
            PostUpdate,
            crate::native_render::caret::sync_carets
                .after(bevy::ui::UiSystems::PostLayout)
                .run_if(ui_processing_enabled)
                .run_if(ui_render_enabled)
                .run_if(ui_text_render_enabled),
        );
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
        PreUpdate,
        crate::scroll_input::sync_scroll_list_input
            .after(bevy::input::InputSystems)
            .run_if(ui_processing_enabled),
    );
    app.add_systems(
        Update,
        crate::widgets::state_panel::animate_loading_texts.run_if(ui_processing_enabled),
    );
    #[cfg(debug_assertions)]
    app.add_systems(
        Update,
        crate::screen::poll_hot_reload
            .run_if(bevy::time::common_conditions::on_real_timer(
                std::time::Duration::from_secs(1),
            ))
            .run_if(ui_processing_enabled),
    );
    app.add_systems(
        PostUpdate,
        (
            apply_frame_removals,
            sync_screen_size.in_set(UiRenderSet::Prepare),
            crate::render_button::sync_button_nine_slices.in_set(UiRenderSet::Prepare),
            (
                crate::render::prepare_ui_frame_order.in_set(UiRenderSet::Prepare),
                crate::native_render::sync_registry.in_set(UiRenderSet::Quads),
            )
                .chain()
                .run_if(ui_render_enabled),
            crate::button_input::sync_button_input,
        )
            .chain()
            .after(bevy::camera::CameraUpdateSystems)
            .before(bevy::ui::UiSystems::Prepare)
            .before(bevy::ui::UiSystems::Stack)
            .run_if(ui_processing_enabled),
    );
}

fn ui_processing_enabled(enabled: Res<UiProcessingEnabled>) -> bool {
    enabled.0
}

fn apply_frame_removals(mut state: ResMut<UiState>) {
    if !state.registry.removed_frames.is_empty() {
        state.resolve_removed_frames();
    }
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
    if let Some((width, height)) = changed_window_size(registry, windows) {
        resize_registry(registry, width, height);
    }
}

fn changed_window_size(
    registry: &FrameRegistry,
    windows: &Query<&Window, With<bevy::window::PrimaryWindow>>,
) -> Option<(f32, f32)> {
    let window = windows.single().ok()?;
    let (width, height) = (window.width(), window.height());
    ((registry.screen_width - width).abs() > 0.5 || (registry.screen_height - height).abs() > 0.5)
        .then_some((width, height))
}

fn resize_registry(registry: &mut FrameRegistry, width: f32, height: f32) {
    registry.screen_width = width;
    registry.screen_height = height;
    registry.mark_all_rects_dirty();
}

fn sync_screen_size(
    mut state: ResMut<UiState>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    if let Some((width, height)) = changed_window_size(&state.registry, &windows) {
        resize_registry(&mut state.registry, width, height);
    }
}

fn initialize_screen_size(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut state: ResMut<UiState>,
) {
    sync_registry_to_primary_window(&mut state.registry, &windows);
}

#[cfg(test)]
#[path = "plugin_tests.rs"]
mod tests;
