use std::collections::BTreeMap;

use bevy::asset::AssetPlugin;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use ui_toolkit::anchor::{Anchor, AnchorPoint};
use ui_toolkit::frame::{Dimension, NineSlice, ThreeSlice, WidgetData};
use ui_toolkit::plugin::{
    UiPlugin, UiProcessingEnabled, UiRenderEnabled, UiRenderSet, UiState, UiTextRenderEnabled,
};
use ui_toolkit::render::{UiQuad, UiText, sync_ui_quads};
use ui_toolkit::render_nine_slice::{UiNineSlicePart, sync_ui_nine_slices};
use ui_toolkit::render_text::sync_ui_text;
use ui_toolkit::render_text_fx::{
    UiTextOutline, UiTextShadow, sync_ui_text_outlines, sync_ui_text_shadows,
};
use ui_toolkit::render_three_slice::{UiThreeSlicePart, sync_ui_three_slices};
use ui_toolkit::strata::FrameStrata;
use ui_toolkit::widgets::font_string::{FontStringData, Outline};
use ui_toolkit::widgets::texture::TextureSource;

#[derive(Clone, Debug, Default, PartialEq)]
struct Snapshot {
    quads: BTreeMap<u64, f32>,
    text: BTreeMap<u64, f32>,
    shadows: BTreeMap<u64, f32>,
    outlines: BTreeMap<u64, Vec<f32>>,
    nine: BTreeMap<u64, f32>,
    three: BTreeMap<u64, f32>,
}

struct Fixture {
    app: App,
    first: u64,
    second: u64,
    text: u64,
    nine: u64,
    three: u64,
    spacer: u64,
}

fn add_frame(ui: &mut UiState, name: &str, x: f32) -> u64 {
    let id = ui.registry.create_frame(name, None);
    let frame = ui.registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(80.0);
    frame.height = Dimension::Fixed(24.0);
    frame.strata = FrameStrata::Medium;
    frame.frame_level = 0;
    frame.raise_order = 0;
    ui.registry
        .set_point(
            id,
            Anchor {
                point: AnchorPoint::TopLeft,
                relative_to: None,
                relative_point: AnchorPoint::TopLeft,
                x_offset: x,
                y_offset: -40.0,
            },
        )
        .unwrap();
    id
}

fn fixture() -> Fixture {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Image>();
    app.init_asset::<Font>();
    app.add_plugins(UiPlugin);
    app.world_mut().spawn((
        Window {
            resolution: (800, 600).into(),
            ..default()
        },
        PrimaryWindow,
    ));
    let (first, second, text, nine, three, spacer) = {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let first = add_frame(&mut ui, "OrderFirst", 10.0);
        ui.registry.get_mut(first).unwrap().background_color = Some([1.0, 0.0, 0.0, 1.0]);
        let second = add_frame(&mut ui, "OrderSecond", 100.0);
        ui.registry.get_mut(second).unwrap().background_color = Some([0.0, 1.0, 0.0, 1.0]);
        let text = add_frame(&mut ui, "OrderText", 190.0);
        ui.registry.get_mut(text).unwrap().widget_data =
            Some(WidgetData::FontString(FontStringData {
                text: "Shared order".into(),
                shadow_color: Some([0.0, 0.0, 0.0, 1.0]),
                ..default()
            }));
        let nine = add_frame(&mut ui, "OrderNine", 280.0);
        ui.registry.get_mut(nine).unwrap().nine_slice = Some(NineSlice::default());
        let three = add_frame(&mut ui, "OrderThree", 370.0);
        ui.registry.get_mut(three).unwrap().three_slice = Some(ThreeSlice {
            left: TextureSource::None,
            center: TextureSource::None,
            right: TextureSource::None,
            ..default()
        });
        let spacer = add_frame(&mut ui, "OrderSpacer", 460.0);
        (first, second, text, nine, three, spacer)
    };
    for _ in 0..3 {
        app.update();
    }
    Fixture {
        app,
        first,
        second,
        text,
        nine,
        three,
        spacer,
    }
}

fn snapshot(world: &mut World) -> Snapshot {
    let mut result = Snapshot::default();
    for (id, transform) in world.query::<(&UiQuad, &Transform)>().iter(world) {
        result.quads.insert(id.0, transform.translation.z);
    }
    for (id, transform, shadow, outline) in world
        .query::<(
            &UiText,
            &Transform,
            Option<&UiTextShadow>,
            Option<&UiTextOutline>,
        )>()
        .iter(world)
    {
        if shadow.is_some() {
            result.shadows.insert(id.0, transform.translation.z);
        } else if outline.is_some() {
            result
                .outlines
                .entry(id.0)
                .or_default()
                .push(transform.translation.z);
        } else {
            result.text.insert(id.0, transform.translation.z);
        }
    }
    for (part, transform) in world.query::<(&UiNineSlicePart, &Transform)>().iter(world) {
        if part.1 == 4 {
            result.nine.insert(part.0, transform.translation.z);
        }
    }
    for (part, transform) in world.query::<(&UiThreeSlicePart, &Transform)>().iter(world) {
        if part.1 == 1 {
            result.three.insert(part.0, transform.translation.z);
        }
    }
    result
}

fn assert_z(actual: f32, rank: usize, offset: f32) {
    let expected = rank as f32 * 0.001 + offset;
    assert!(
        (actual - expected).abs() < 0.0000001,
        "z={actual}, expected={expected}"
    );
}

fn assert_order(fixture: &mut Fixture, expected: &[u64]) {
    let result = snapshot(fixture.app.world_mut());
    assert_snapshot_order(fixture, &result, expected);
}

fn assert_snapshot_order(fixture: &Fixture, result: &Snapshot, expected: &[u64]) {
    for (rank, &id) in expected.iter().enumerate() {
        if id == fixture.first || id == fixture.second {
            assert_z(result.quads[&id], rank, 0.0);
        } else if id == fixture.text {
            assert_z(result.text[&id], rank, 0.0007);
            assert_z(result.shadows[&id], rank, 0.0006);
            if let Some(outlines) = result.outlines.get(&id) {
                assert_eq!(outlines.len(), 4);
                for &z in outlines {
                    assert_z(z, rank, 0.0005);
                }
            }
        } else if id == fixture.nine {
            assert_z(result.nine[&id], rank, 0.0);
        } else if id == fixture.three {
            assert_z(result.three[&id], rank, 0.0);
        }
    }
    let expected_quads = expected
        .iter()
        .filter(|&&id| id == fixture.first || id == fixture.second)
        .count();
    assert_eq!(result.quads.len(), expected_quads);
    assert_eq!(result.text.len(), 1);
    assert_eq!(result.shadows.len(), 1);
    assert_eq!(result.nine.len(), 1);
    assert_eq!(result.three.len(), 1);
}

fn change_order(fixture: &mut Fixture) {
    let mut ui = fixture.app.world_mut().resource_mut::<UiState>();
    ui.registry.get_mut(fixture.first).unwrap().frame_level = 5;
    ui.registry.set_hidden(fixture.second, true);
    let spacer = ui.registry.get_mut(fixture.spacer).unwrap();
    spacer.width = Dimension::Fixed(0.0);
    spacer.layout_rect.as_mut().unwrap().width = 0.0;
    ui.registry.mark_rect_dirty(fixture.spacer);
}

fn enable_outline(fixture: &mut Fixture) {
    let mut ui = fixture.app.world_mut().resource_mut::<UiState>();
    let Some(WidgetData::FontString(text)) =
        &mut ui.registry.get_mut(fixture.text).unwrap().widget_data
    else {
        panic!("expected text fixture");
    };
    text.outline = Outline::Outline;
}

#[test]
fn plugin_refreshes_order_after_visibility_size_and_ordering_changes() {
    let mut f = fixture();
    let initial = [f.first, f.second, f.text, f.nine, f.three, f.spacer];
    assert_order(&mut f, &initial);
    change_order(&mut f);
    {
        let mut ui = f.app.world_mut().resource_mut::<UiState>();
        ui.registry.get_mut(f.three).unwrap().raise_order = 7;
        ui.registry.get_mut(f.nine).unwrap().strata = FrameStrata::High;
    }
    enable_outline(&mut f);
    f.app.update();
    let expected = [f.text, f.three, f.first, f.nine];
    assert_order(&mut f, &expected);
}

#[test]
fn render_and_processing_reenable_use_current_registry_order() {
    for processing in [false, true] {
        let mut f = fixture();
        let before = snapshot(f.app.world_mut());
        if processing {
            f.app.world_mut().resource_mut::<UiProcessingEnabled>().0 = false;
        } else {
            f.app.world_mut().resource_mut::<UiRenderEnabled>().0 = false;
        }
        change_order(&mut f);
        f.app.update();
        assert_eq!(snapshot(f.app.world_mut()), before);
        if processing {
            f.app.world_mut().resource_mut::<UiProcessingEnabled>().0 = true;
        } else {
            f.app.world_mut().resource_mut::<UiRenderEnabled>().0 = true;
        }
        f.app.update();
        let expected = [f.text, f.nine, f.three, f.first];
        assert_order(&mut f, &expected);
    }
}

#[test]
fn text_reenable_uses_new_order_while_nontext_rendering_continues() {
    let mut f = fixture();
    let before = snapshot(f.app.world_mut());
    f.app.world_mut().resource_mut::<UiTextRenderEnabled>().0 = false;
    change_order(&mut f);
    enable_outline(&mut f);
    f.app.update();
    let disabled = snapshot(f.app.world_mut());
    assert_eq!(disabled.text, before.text);
    assert_eq!(disabled.shadows, before.shadows);
    assert!(disabled.outlines.is_empty());
    assert_z(disabled.nine[&f.nine], 1, 0.0);
    assert!(!disabled.quads.contains_key(&f.second));
    f.app.world_mut().resource_mut::<UiTextRenderEnabled>().0 = true;
    f.app.update();
    let expected = [f.text, f.nine, f.three, f.first];
    assert_order(&mut f, &expected);
}

#[test]
fn standalone_renderers_ignore_stale_plugin_preparation() {
    let mut f = fixture();
    f.app.world_mut().resource_mut::<UiProcessingEnabled>().0 = false;
    change_order(&mut f);
    enable_outline(&mut f);
    let world = f.app.world_mut();
    world.run_system_once(sync_ui_quads).unwrap();
    world.run_system_once(sync_ui_text).unwrap();
    world.run_system_once(sync_ui_text_shadows).unwrap();
    world.run_system_once(sync_ui_text_outlines).unwrap();
    world.run_system_once(sync_ui_nine_slices).unwrap();
    world.run_system_once(sync_ui_three_slices).unwrap();
    let expected = [f.text, f.nine, f.three, f.first];
    assert_order(&mut f, &expected);
}

#[derive(Resource)]
struct PendingReorder {
    first: u64,
    second: u64,
    pending: bool,
}

#[derive(Resource, Default)]
struct PublishedOrder(Snapshot);

fn reorder_before_preparation(mut ui: ResMut<UiState>, mut change: ResMut<PendingReorder>) {
    if !change.pending {
        return;
    }
    ui.registry.get_mut(change.first).unwrap().frame_level = 5;
    ui.registry.get_mut(change.second).unwrap().strata = FrameStrata::High;
    change.pending = false;
}

fn publish_after_consumers(world: &mut World) {
    let rendered = snapshot(world);
    world.resource_mut::<PublishedOrder>().0 = rendered;
}

#[test]
fn named_sets_apply_input_before_preparation_and_publish_current_render_order() {
    let mut f = fixture();
    f.app.insert_resource(PendingReorder {
        first: f.first,
        second: f.second,
        pending: true,
    });
    f.app.init_resource::<PublishedOrder>();
    f.app.add_systems(
        Update,
        reorder_before_preparation.before(UiRenderSet::Prepare),
    );
    f.app.add_systems(
        Update,
        publish_after_consumers
            .after(UiRenderSet::Quads)
            .after(UiRenderSet::Text)
            .after(UiRenderSet::Shadows)
            .after(UiRenderSet::Outlines)
            .after(UiRenderSet::NineSlices)
            .after(UiRenderSet::ThreeSlices),
    );
    f.app.update();
    let expected = [f.text, f.nine, f.three, f.spacer, f.first, f.second];
    let published = &f.app.world().resource::<PublishedOrder>().0;
    assert_snapshot_order(&f, published, &expected);
}
