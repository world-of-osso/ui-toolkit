use std::collections::BTreeMap;

pub mod support;
use bevy::math::Affine2;
use bevy::prelude::*;
use ui_toolkit::frame::{Dimension, NineSlice, ThreeSlice, WidgetData};
use ui_toolkit::native_render::{RegistryNode, RegistryText};
use ui_toolkit::plugin::{
    UiProcessingEnabled, UiRenderEnabled, UiRenderSet, UiState, UiTextRenderEnabled,
};
use ui_toolkit::strata::FrameStrata;
use ui_toolkit::widgets::font_string::{FontStringData, Outline};
use ui_toolkit::widgets::texture::TextureSource;
use ui_toolkit_core::layout_values::PositionType;

#[derive(Clone, Debug, Default, PartialEq)]
struct Snapshot {
    images: BTreeMap<u64, i32>,
    text: BTreeMap<u64, i32>,
    shadows: BTreeMap<u64, i32>,
    outlines: BTreeMap<u64, Vec<i32>>,
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

fn native_app() -> App {
    support::native_app().0
}

fn add_frame(ui: &mut UiState, name: &str, x: f32) -> u64 {
    let id = ui.registry.create_frame(name, None);
    ui.registry
        .set_pos_type(id, PositionType::Absolute)
        .unwrap();
    ui.registry.set_pos(id, x, 40.0).unwrap();
    let frame = ui.registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(80.0);
    frame.height = Dimension::Fixed(24.0);
    frame.strata = FrameStrata::Medium;
    frame.frame_level = 0;
    frame.raise_order = 0;
    id
}

fn settle(app: &mut App) {
    for _ in 0..3 {
        app.update();
    }
}

fn fixture() -> Fixture {
    let mut app = native_app();
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
    settle(&mut app);
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

fn logical_rect(world: &World, entity: Entity) -> Rect {
    let node = world.get::<ComputedNode>(entity).unwrap();
    let transform = Affine2::from(world.get::<UiGlobalTransform>(entity).unwrap());
    Rect::from_center_size(
        transform.translation * node.inverse_scale_factor,
        node.size * node.inverse_scale_factor,
    )
}

fn snapshot(world: &mut World) -> Snapshot {
    let mut result = Snapshot::default();
    let frames: Vec<_> = world
        .query::<(Entity, &RegistryNode)>()
        .iter(world)
        .map(|(entity, id)| (entity, id.0))
        .collect();
    for (entity, id) in frames {
        let center = logical_rect(world, entity).center();
        let images: Vec<_> = world
            .get::<Children>(entity)
            .into_iter()
            .flat_map(|children| children.iter())
            .filter(|child| world.get::<ImageNode>(*child).is_some())
            .collect();
        if images.is_empty() {
            continue;
        }
        let central: Vec<_> = images
            .iter()
            .copied()
            .filter(|child| logical_rect(world, *child).contains(center))
            .collect();
        assert_eq!(
            central.len(),
            1,
            "one image covers center of registry frame {id}"
        );
        let z = world.get::<GlobalZIndex>(central[0]).unwrap().0;
        result.images.insert(id, z);
    }
    for (part, parent) in world.query::<(&RegistryText, &ChildOf)>().iter(world) {
        let z = world.get::<GlobalZIndex>(parent.parent()).unwrap().0;
        match part.key {
            0 => {
                result.text.insert(part.frame_id, z);
            }
            1 => {
                result.shadows.insert(part.frame_id, z);
            }
            _ => result.outlines.entry(part.frame_id).or_default().push(z),
        }
    }
    result
}

fn assert_z(actual: i32, rank: usize, offset: i32) {
    assert_eq!(actual, rank as i32 * 10 + offset);
}

fn assert_order(fixture: &mut Fixture, expected: &[u64]) {
    let result = snapshot(fixture.app.world_mut());
    assert_snapshot_order(fixture, &result, expected);
}

fn assert_snapshot_order(fixture: &Fixture, result: &Snapshot, expected: &[u64]) {
    let image_frames = [fixture.first, fixture.second, fixture.nine, fixture.three];
    let mut expected_images = Vec::new();
    for (rank, &id) in expected.iter().enumerate() {
        if image_frames.contains(&id) {
            assert_z(result.images[&id], rank, 0);
            expected_images.push(id);
        } else if id == fixture.text {
            assert_z(result.text[&id], rank, 7);
            assert_z(result.shadows[&id], rank, 6);
            if let Some(outlines) = result.outlines.get(&id) {
                assert_eq!(outlines, &vec![rank as i32 * 10 + 5; 4]);
            }
        }
    }
    expected_images.sort_unstable();
    assert_eq!(
        result.images.keys().copied().collect::<Vec<_>>(),
        expected_images
    );
    assert_eq!(
        result.text.keys().copied().collect::<Vec<_>>(),
        vec![fixture.text]
    );
    assert_eq!(
        result.shadows.keys().copied().collect::<Vec<_>>(),
        vec![fixture.text]
    );
}

fn change_order(fixture: &mut Fixture) {
    let mut ui = fixture.app.world_mut().resource_mut::<UiState>();
    ui.registry.get_mut(fixture.first).unwrap().frame_level = 5;
    ui.registry.set_hidden(fixture.second, true);
    ui.registry.get_mut(fixture.spacer).unwrap().width = Dimension::Fixed(0.0);
    ui.registry.mark_rect_dirty(fixture.spacer);
}

fn enable_outline(fixture: &mut Fixture) {
    let mut ui = fixture.app.world_mut().resource_mut::<UiState>();
    let Some(WidgetData::FontString(text)) =
        &mut ui.registry.get_mut(fixture.text).unwrap().widget_data
    else {
        panic!("text fixture")
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
    settle(&mut f.app);
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
        settle(&mut f.app);
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
    settle(&mut f.app);
    let disabled = snapshot(f.app.world_mut());
    // Pausing text synchronization must preserve previously rendered text.
    assert_eq!(disabled.text, before.text);
    assert_eq!(disabled.shadows, before.shadows);
    assert!(disabled.outlines.is_empty());
    assert_z(disabled.images[&f.nine], 1, 0);
    assert!(!disabled.images.contains_key(&f.second));
    f.app.world_mut().resource_mut::<UiTextRenderEnabled>().0 = true;
    settle(&mut f.app);
    let expected = [f.text, f.nine, f.three, f.first];
    assert_order(&mut f, &expected);
}

#[test]
fn native_post_update_uses_current_order_without_running_update_schedule() {
    let mut f = fixture();
    change_order(&mut f);
    enable_outline(&mut f);
    // Native synchronization lives in PostUpdate; exercise that boundary directly
    // instead of invoking the retired Sprite/Text2d synchronization functions.
    for _ in 0..3 {
        f.app.world_mut().run_schedule(PostUpdate);
    }
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
fn before_prepare_geometry_updates_reach_layout_and_render_in_same_update() {
    let mut f = fixture();
    let first = f.first;
    f.app.add_systems(
        PostUpdate,
        (move |mut ui: ResMut<UiState>| {
            ui.registry.get_mut(first).unwrap().width = Dimension::Fixed(136.0);
            ui.registry.mark_rect_dirty(first);
        })
        .before(UiRenderSet::Prepare),
    );
    f.app.update();
    let world = f.app.world_mut();
    let entity = world
        .query::<(Entity, &RegistryNode)>()
        .iter(world)
        .find(|(_, id)| id.0 == first)
        .unwrap()
        .0;
    let image = world
        .get::<Children>(entity)
        .unwrap()
        .iter()
        .find(|child| world.get::<ImageNode>(*child).is_some())
        .unwrap();
    assert_eq!(logical_rect(world, image).size(), Vec2::new(136.0, 24.0));
    let frame = world.resource::<UiState>().registry.get(first).unwrap();
    assert_eq!(frame.layout_rect.as_ref().unwrap().width, 136.0);
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
        PostUpdate,
        reorder_before_preparation.before(UiRenderSet::Prepare),
    );
    f.app.add_systems(Last, publish_after_consumers);
    f.app.update();
    let expected = [f.text, f.nine, f.three, f.spacer, f.first, f.second];
    let published = &f.app.world().resource::<PublishedOrder>().0;
    assert_snapshot_order(&f, published, &expected);
}
