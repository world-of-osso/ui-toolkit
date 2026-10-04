pub mod support;

use bevy::prelude::*;
use support::*;
use ui_toolkit::plugin::{UiRenderSet, UiState};

#[derive(Resource, Default)]
struct Observed {
    before: Vec<u64>,
    after: Vec<u64>,
    changed: bool,
}

fn dirty_ids(ui: &UiState) -> Vec<u64> {
    let mut ids: Vec<_> = ui.registry.render_dirty.iter().copied().collect();
    ids.sort_unstable();
    ids
}
fn observe_before(ui: Res<UiState>, mut seen: ResMut<Observed>) {
    seen.before = dirty_ids(&ui);
}
fn observe_after(ui: Res<UiState>, mut seen: ResMut<Observed>) {
    seen.after = dirty_ids(&ui);
    seen.changed = ui.is_changed();
}

fn fixture() -> App {
    let (mut app, _) = native_app();
    app.init_resource::<Observed>();
    app.add_systems(PostUpdate, observe_before.before(UiRenderSet::Prepare));
    app.add_systems(
        PostUpdate,
        observe_after
            .after(UiRenderSet::Project)
            .before(bevy::ui::UiSystems::Prepare),
    );
    settle(&mut app);
    app
}

#[test]
fn native_empty_dirty_clear_preserves_resource_tick() {
    let mut app = fixture();
    app.update();
    let seen = app.world().resource::<Observed>();
    assert!(seen.before.is_empty());
    assert!(seen.after.is_empty());
    assert!(
        !seen.changed,
        "empty projection must not mark UiState changed"
    );
}

#[test]
fn native_projection_publishes_current_output_before_draining_dirty_set() {
    let mut app = fixture();
    let first = create_frame(&mut app, "First", 40.0, 20.0);
    let second = create_frame(&mut app, "Second", 60.0, 30.0);
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        ui.registry.get_mut(first).unwrap().background_color = Some([1.0, 0.0, 0.0, 1.0]);
        ui.registry.get_mut(second).unwrap().background_color = Some([0.0, 1.0, 0.0, 1.0]);
    }
    app.update();
    let seen = app.world().resource::<Observed>();
    assert_eq!(seen.before, vec![first, second]);
    assert!(seen.after.is_empty());
    for (id, size, color) in [
        (first, Vec2::new(40.0, 20.0), Color::srgb(1.0, 0.0, 0.0)),
        (second, Vec2::new(60.0, 30.0), Color::srgb(0.0, 1.0, 0.0)),
    ] {
        let image = image_covering(app.world_mut(), id, size / 2.0);
        assert_eq!(logical_rect(app.world(), image).size(), size);
        assert_eq!(app.world().get::<ImageNode>(image).unwrap().color, color);
    }
    settle(&mut app);
    assert!(
        app.world()
            .resource::<UiState>()
            .registry
            .render_dirty
            .is_empty()
    );
    assert!(!app.world().resource::<Observed>().changed);
}
