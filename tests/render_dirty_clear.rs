use bevy::prelude::*;
use ui_toolkit::event::EventBus;
use ui_toolkit::plugin::UiState;
use ui_toolkit::registry::FrameRegistry;
use ui_toolkit::render::sync_ui_quads;
use ui_toolkit::render_tiled::sync_ui_tiled_textures;

#[derive(Resource, Default)]
struct Observed {
    before: Vec<u64>,
    after: Vec<u64>,
    changed: bool,
}

fn dirty_ids(state: &UiState) -> Vec<u64> {
    let mut ids: Vec<_> = state.registry.render_dirty.iter().copied().collect();
    ids.sort_unstable();
    ids
}

fn observe_before(state: Res<UiState>, mut observed: ResMut<Observed>) {
    observed.before = dirty_ids(&state);
}

fn observe_after(state: Res<UiState>, mut observed: ResMut<Observed>) {
    observed.after = dirty_ids(&state);
    observed.changed = state.is_changed();
}

fn fixture(tiled: bool) -> App {
    let mut app = App::new();
    app.insert_resource(UiState {
        registry: FrameRegistry::new(800.0, 600.0),
        event_bus: EventBus::new(),
        focused_frame: None,
    });
    app.init_resource::<Observed>();
    if tiled {
        app.add_systems(
            Update,
            (observe_before, sync_ui_tiled_textures, observe_after).chain(),
        );
    } else {
        app.add_systems(
            Update,
            (observe_before, sync_ui_quads, observe_after).chain(),
        );
    }
    // Consume initial insertion before measuring a pass without fixture mutations.
    app.update();
    app
}

fn assert_empty_clear_preserves_resource_tick(tiled: bool) {
    let mut app = fixture(tiled);
    app.update();
    let observed = app.world().resource::<Observed>();
    assert!(observed.before.is_empty());
    assert!(observed.after.is_empty());
    assert!(
        !observed.changed,
        "clearing an already-empty dirty set must not mark UiState changed"
    );
}

fn assert_nonempty_set_drained_before_next_system(tiled: bool) {
    let mut app = fixture(tiled);
    let mut expected = {
        let mut state = app.world_mut().resource_mut::<UiState>();
        let first = state.registry.create_frame("First", None);
        let second = state.registry.create_frame("Second", None);
        vec![first, second]
    };
    expected.sort_unstable();
    app.update();
    let observed = app.world().resource::<Observed>();
    assert_eq!(observed.before, expected);
    assert!(observed.after.is_empty());
    assert!(
        app.world()
            .resource::<UiState>()
            .registry
            .render_dirty
            .is_empty()
    );
    // Do not attribute this pass's resource tick: frame creation also mutated UiState.
}

#[test]
fn quads_empty_dirty_clear_preserves_resource_tick() {
    assert_empty_clear_preserves_resource_tick(false);
}

#[test]
fn tiled_empty_dirty_clear_preserves_resource_tick() {
    assert_empty_clear_preserves_resource_tick(true);
}

#[test]
fn quads_drain_nonempty_dirty_set_before_next_system() {
    assert_nonempty_set_drained_before_next_system(false);
}

#[test]
fn tiled_drain_nonempty_dirty_set_before_next_system() {
    assert_nonempty_set_drained_before_next_system(true);
}
