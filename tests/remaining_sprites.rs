use bevy::prelude::*;
use ui_toolkit::event::EventBus;
use ui_toolkit::frame::{Backdrop, Border, Dimension, ThreeSlice, WidgetData};
use ui_toolkit::plugin::UiState;
use ui_toolkit::registry::FrameRegistry;
use ui_toolkit::render_border::{sync_css_borders, sync_ui_borders};
use ui_toolkit::render_button::sync_ui_button_highlights;
use ui_toolkit::render_three_slice::sync_ui_three_slices;
use ui_toolkit::widgets::button::ButtonData;
use ui_toolkit::widgets::texture::TextureSource;

#[derive(Clone, Copy)]
enum Renderer {
    ThreeSlice,
    CssBorder,
    BackdropBorder,
    Highlight,
}

#[derive(Resource, Default)]
struct Observed(Vec<(bool, bool)>);

fn observe(visuals: Query<(Ref<Transform>, Ref<Sprite>)>, mut seen: ResMut<Observed>) {
    seen.0 = visuals
        .iter()
        .map(|(transform, sprite)| (transform.is_changed(), sprite.is_changed()))
        .collect();
}

fn fixture(renderer: Renderer) -> (App, u64) {
    let mut app = App::new();
    app.init_resource::<Assets<Image>>();
    let image = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default());
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("RemainingSpriteRegression", None);
    let frame = registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(120.0);
    frame.height = Dimension::Fixed(60.0);
    match renderer {
        Renderer::ThreeSlice => {
            frame.three_slice = Some(ThreeSlice {
                cap_width: 8.0,
                left: TextureSource::Dynamic(image.clone()),
                center: TextureSource::Dynamic(image.clone()),
                right: TextureSource::Dynamic(image.clone()),
                color: [1.0; 4],
            })
        }
        Renderer::CssBorder => {
            frame.border = Some(Border {
                width: 4.0,
                color: [1.0; 4],
            })
        }
        Renderer::BackdropBorder => {
            frame.backdrop = Some(Backdrop {
                bg_color: None,
                border_color: Some([1.0; 4]),
                edge_size: 4.0,
                insets: [0.0; 4],
            })
        }
        Renderer::Highlight => {
            frame.widget_data = Some(WidgetData::Button(ButtonData {
                hovered: true,
                highlight_texture: Some(TextureSource::Dynamic(image)),
                ..default()
            }))
        }
    }
    app.insert_resource(UiState {
        registry,
        event_bus: EventBus::new(),
        focused_frame: None,
    });
    app.init_resource::<Observed>();
    // Highlight is deliberately tested directly: the full plugin converts buttons to nine-slices.
    match renderer {
        Renderer::ThreeSlice => app.add_systems(Update, (sync_ui_three_slices, observe).chain()),
        Renderer::CssBorder => app.add_systems(Update, (sync_css_borders, observe).chain()),
        Renderer::BackdropBorder => app.add_systems(Update, (sync_ui_borders, observe).chain()),
        Renderer::Highlight => {
            app.add_systems(Update, (sync_ui_button_highlights, observe).chain())
        }
    };
    app.update();
    assert_eq!(entities(&mut app).len(), part_count(renderer));
    (app, id)
}

fn part_count(renderer: Renderer) -> usize {
    match renderer {
        Renderer::ThreeSlice => 3,
        Renderer::CssBorder | Renderer::BackdropBorder => 4,
        Renderer::Highlight => 1,
    }
}

fn entities(app: &mut App) -> Vec<Entity> {
    let mut result: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, With<Sprite>>()
        .iter(app.world())
        .collect();
    result.sort();
    result
}

fn assert_settled(renderer: Renderer) {
    let (mut app, _) = fixture(renderer);
    app.update();
    assert_eq!(
        app.world().resource::<Observed>().0,
        vec![(false, false); part_count(renderer)]
    );
}

#[test]
fn settled_three_slice_has_clean_component_ticks() {
    assert_settled(Renderer::ThreeSlice);
}
#[test]
fn settled_css_border_has_clean_component_ticks() {
    assert_settled(Renderer::CssBorder);
}
#[test]
fn settled_backdrop_border_has_clean_component_ticks() {
    assert_settled(Renderer::BackdropBorder);
}
#[test]
fn settled_direct_highlight_has_clean_component_ticks() {
    assert_settled(Renderer::Highlight);
}

fn assert_updates(renderer: Renderer) {
    let (mut app, id) = fixture(renderer);
    let original = entities(&mut app);
    let replacement = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default());
    {
        let mut state = app.world_mut().resource_mut::<UiState>();
        let frame = state.registry.get_mut(id).unwrap();
        frame.width = Dimension::Fixed(200.0);
        frame.height = Dimension::Fixed(100.0);
        frame.effective_alpha = 0.5;
        match renderer {
            Renderer::ThreeSlice => {
                let slice = frame.three_slice.as_mut().unwrap();
                slice.cap_width = 10.0;
                slice.color = [0.2, 0.4, 0.6, 0.8];
                slice.left = TextureSource::Dynamic(replacement.clone());
                slice.center = TextureSource::Dynamic(replacement.clone());
                slice.right = TextureSource::Dynamic(replacement.clone());
            }
            Renderer::CssBorder => {
                frame.border = Some(Border {
                    width: 6.0,
                    color: [0.2, 0.4, 0.6, 0.8],
                });
            }
            Renderer::BackdropBorder => {
                let border = frame.backdrop.as_mut().unwrap();
                border.edge_size = 6.0;
                border.border_color = Some([0.2, 0.4, 0.6, 0.8]);
            }
            Renderer::Highlight => {
                let Some(WidgetData::Button(button)) = &mut frame.widget_data else {
                    panic!("button fixture")
                };
                button.highlight_texture = Some(TextureSource::Dynamic(replacement.clone()));
            }
        }
    }
    app.update();
    assert_eq!(entities(&mut app), original);
    let mut geometry = Vec::new();
    for entity in original {
        let sprite = app.world().get::<Sprite>(entity).unwrap();
        let transform = app.world().get::<Transform>(entity).unwrap();
        geometry.push((
            transform.translation.x,
            transform.translation.y,
            sprite.custom_size.unwrap(),
        ));
        let expected_color = match renderer {
            Renderer::Highlight => Color::srgba(1.0, 1.0, 1.0, 0.25),
            _ => Color::srgba(0.2, 0.4, 0.6, 0.4),
        };
        assert_eq!(sprite.color, expected_color);
        if matches!(renderer, Renderer::ThreeSlice | Renderer::Highlight) {
            assert_eq!(sprite.image, replacement);
            assert!(
                app.world()
                    .resource::<Assets<Image>>()
                    .get(&sprite.image)
                    .is_some()
            );
        }
    }
    let expected = match renderer {
        Renderer::ThreeSlice => vec![
            (-395.0, 250.0, Vec2::new(10.0, 100.0)),
            (-300.0, 250.0, Vec2::new(180.0, 100.0)),
            (-205.0, 250.0, Vec2::new(10.0, 100.0)),
        ],
        Renderer::CssBorder => vec![
            (-300.0, 297.0, Vec2::new(200.0, 6.0)),
            (-203.0, 250.0, Vec2::new(6.0, 100.0)),
            (-300.0, 203.0, Vec2::new(200.0, 6.0)),
            (-397.0, 250.0, Vec2::new(6.0, 100.0)),
        ],
        Renderer::BackdropBorder => vec![
            (-300.0, 303.0, Vec2::new(212.0, 6.0)),
            (-300.0, 197.0, Vec2::new(212.0, 6.0)),
            (-403.0, 250.0, Vec2::new(6.0, 100.0)),
            (-197.0, 250.0, Vec2::new(6.0, 100.0)),
        ],
        Renderer::Highlight => vec![(-300.0, 250.0, Vec2::new(200.0, 100.0))],
    };
    for value in expected {
        assert!(
            geometry.contains(&value),
            "missing geometry {value:?} in {geometry:?}"
        );
    }
}

#[test]
fn three_slice_geometry_color_and_texture_updates() {
    assert_updates(Renderer::ThreeSlice);
}
#[test]
fn css_border_geometry_and_color_updates() {
    assert_updates(Renderer::CssBorder);
}
#[test]
fn backdrop_border_geometry_and_color_updates() {
    assert_updates(Renderer::BackdropBorder);
}
#[test]
fn direct_highlight_geometry_alpha_and_texture_updates() {
    assert_updates(Renderer::Highlight);
}

fn assert_sprite(actual: &Sprite, expected: &Sprite) {
    assert_eq!(actual.image, expected.image);
    assert_eq!(actual.texture_atlas, expected.texture_atlas);
    assert_eq!(actual.color, expected.color);
    assert_eq!(actual.flip_x, expected.flip_x);
    assert_eq!(actual.flip_y, expected.flip_y);
    assert_eq!(actual.custom_size, expected.custom_size);
    assert_eq!(actual.rect, expected.rect);
    assert_eq!(actual.image_mode, expected.image_mode);
}

fn assert_repair_and_cleanup(renderer: Renderer) {
    let (mut app, id) = fixture(renderer);
    let original = entities(&mut app);
    let entity = original[0];
    let expected_transform = *app.world().get::<Transform>(entity).unwrap();
    let expected_sprite = app.world().get::<Sprite>(entity).unwrap().clone();
    app.world_mut().entity_mut(entity).insert((
        Transform::from_xyz(1.0, 2.0, 3.0),
        Sprite {
            color: Color::BLACK,
            flip_x: true,
            flip_y: true,
            rect: Some(Rect::new(0.0, 0.0, 0.5, 0.5)),
            ..default()
        },
    ));
    app.update();
    assert_eq!(
        *app.world().get::<Transform>(entity).unwrap(),
        expected_transform
    );
    assert_sprite(app.world().get::<Sprite>(entity).unwrap(), &expected_sprite);
    app.world_mut().entity_mut(entity).remove::<Sprite>();
    app.update();
    assert_eq!(entities(&mut app), original);
    assert_sprite(app.world().get::<Sprite>(entity).unwrap(), &expected_sprite);
    app.world_mut().entity_mut(entity).remove::<Transform>();
    app.update();
    assert_eq!(entities(&mut app), original);
    assert_eq!(
        *app.world().get::<Transform>(entity).unwrap(),
        expected_transform
    );
    set_active(&mut app, id, renderer, false);
    app.update();
    assert!(entities(&mut app).is_empty());
    set_active(&mut app, id, renderer, true);
    app.update();
    assert_eq!(entities(&mut app).len(), part_count(renderer));
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .remove_frame(id);
    app.update();
    assert!(entities(&mut app).is_empty());
}

fn set_active(app: &mut App, id: u64, renderer: Renderer, active: bool) {
    let mut state = app.world_mut().resource_mut::<UiState>();
    if matches!(renderer, Renderer::Highlight) {
        let Some(WidgetData::Button(button)) = &mut state.registry.get_mut(id).unwrap().widget_data
        else {
            panic!("button fixture")
        };
        button.hovered = active;
    } else {
        state.registry.set_hidden(id, !active);
    }
}

#[test]
fn three_slice_repairs_components_and_removes_stale_parts() {
    assert_repair_and_cleanup(Renderer::ThreeSlice);
}
#[test]
fn css_border_repairs_components_and_removes_stale_parts() {
    assert_repair_and_cleanup(Renderer::CssBorder);
}
#[test]
fn backdrop_border_repairs_components_and_removes_stale_parts() {
    assert_repair_and_cleanup(Renderer::BackdropBorder);
}
#[test]
fn direct_highlight_repairs_components_and_removes_stale_overlay() {
    assert_repair_and_cleanup(Renderer::Highlight);
}
