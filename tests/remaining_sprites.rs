pub mod support;

use bevy::prelude::*;
use support::*;
use ui_toolkit::frame::{Backdrop, Border, Dimension, ThreeSlice, WidgetData};
use ui_toolkit::plugin::UiState;
use ui_toolkit::widgets::button::ButtonData;
use ui_toolkit::widgets::texture::{DynamicTextureId, TextureSource};

#[derive(Clone, Copy)]
enum Decoration {
    ThreeSlice,
    CssBorder,
    BackdropBorder,
    Highlight,
}

fn fixture(decoration: Decoration) -> (App, u64) {
    let (mut app, window) = native_app();
    if matches!(decoration, Decoration::Highlight) {
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(20.0, 20.0)));
    }
    let image = add_texture(&mut app);
    let id = create_frame(&mut app, "DecorationRegression", 120.0, 60.0);
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let frame = ui.registry.get_mut(id).unwrap();
        frame.mouse_enabled = matches!(decoration, Decoration::Highlight);
        match decoration {
            Decoration::ThreeSlice => {
                frame.three_slice = Some(ThreeSlice {
                    cap_width: 8.0,
                    left: TextureSource::Dynamic(image.clone()),
                    center: TextureSource::Dynamic(image.clone()),
                    right: TextureSource::Dynamic(image),
                    color: [1.0; 4],
                })
            }
            Decoration::CssBorder => {
                frame.border = Some(Border {
                    width: 4.0,
                    color: [1.0; 4],
                })
            }
            Decoration::BackdropBorder => {
                frame.backdrop = Some(Backdrop {
                    bg_color: None,
                    border_color: Some([1.0; 4]),
                    edge_size: 4.0,
                    insets: [0.0; 4],
                })
            }
            // Hover-only authored buttons bypass generated skins. This exercises
            // the overlay via UiPlugin, not the retired standalone renderer.
            Decoration::Highlight => {
                frame.widget_data = Some(WidgetData::Button(ButtonData {
                    hovered: true,
                    use_default_skin: false,
                    highlight_texture: Some(TextureSource::Dynamic(image)),
                    ..default()
                }))
            }
        }
    }
    settle(&mut app);
    (app, id)
}

fn assert_settled(decoration: Decoration) {
    let (mut app, id) = fixture(decoration);
    let entities = image_entities(app.world_mut(), id);
    assert!(!entities.is_empty());
    assert!(
        entities
            .iter()
            .all(|entity| logical_rect(app.world(), *entity).size().min_element() > 0.0)
    );
    app.update();
    assert!(!app.world().resource::<ImageChanges>().0);
}

#[test]
fn settled_three_slice_has_clean_component_ticks() {
    assert_settled(Decoration::ThreeSlice);
}
#[test]
fn settled_css_border_has_clean_component_ticks() {
    assert_settled(Decoration::CssBorder);
}
#[test]
fn settled_backdrop_border_has_clean_component_ticks() {
    assert_settled(Decoration::BackdropBorder);
}
#[test]
fn settled_hover_overlay_has_clean_component_ticks() {
    assert_settled(Decoration::Highlight);
}

fn update_decoration(
    app: &mut App,
    id: u64,
    decoration: Decoration,
    replacement: &DynamicTextureId,
) {
    let mut ui = app.world_mut().resource_mut::<UiState>();
    ui.registry.set_alpha(id, 0.5);
    let frame = ui.registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(200.0);
    frame.height = Dimension::Fixed(100.0);
    match decoration {
        Decoration::ThreeSlice => {
            let slice = frame.three_slice.as_mut().unwrap();
            slice.cap_width = 10.0;
            slice.color = [0.2, 0.4, 0.6, 0.8];
            slice.left = TextureSource::Dynamic(replacement.clone());
            slice.center = TextureSource::Dynamic(replacement.clone());
            slice.right = TextureSource::Dynamic(replacement.clone());
        }
        Decoration::CssBorder => {
            frame.border = Some(Border {
                width: 6.0,
                color: [0.2, 0.4, 0.6, 0.8],
            })
        }
        Decoration::BackdropBorder => {
            let border = frame.backdrop.as_mut().unwrap();
            border.edge_size = 6.0;
            border.border_color = Some([0.2, 0.4, 0.6, 0.8]);
        }
        Decoration::Highlight => {
            let Some(WidgetData::Button(button)) = &mut frame.widget_data else {
                panic!("button")
            };
            button.highlight_texture = Some(TextureSource::Dynamic(replacement.clone()));
        }
    }
}

fn assert_updates(decoration: Decoration) {
    let (mut app, id) = fixture(decoration);
    let original = image_entities(app.world_mut(), id);
    let replacement = add_texture(&mut app);
    update_decoration(&mut app, id, decoration, &replacement);
    settle(&mut app);
    assert_eq!(image_entities(app.world_mut(), id), original);
    let actual: Vec<_> = original
        .iter()
        .map(|entity| logical_rect(app.world(), *entity))
        .collect();
    let expected = match decoration {
        Decoration::ThreeSlice => vec![
            Rect::new(0.0, 0.0, 10.0, 100.0),
            Rect::new(10.0, 0.0, 190.0, 100.0),
            Rect::new(190.0, 0.0, 200.0, 100.0),
        ],
        Decoration::CssBorder => vec![
            Rect::new(0.0, 0.0, 200.0, 6.0),
            Rect::new(194.0, 0.0, 200.0, 100.0),
            Rect::new(0.0, 94.0, 200.0, 100.0),
            Rect::new(0.0, 0.0, 6.0, 100.0),
        ],
        Decoration::BackdropBorder => vec![
            Rect::new(-6.0, -6.0, 206.0, 0.0),
            Rect::new(-6.0, 100.0, 206.0, 106.0),
            Rect::new(-6.0, 0.0, 0.0, 100.0),
            Rect::new(200.0, 0.0, 206.0, 100.0),
        ],
        Decoration::Highlight => vec![Rect::new(0.0, 0.0, 200.0, 100.0)],
    };
    assert_eq!(
        actual.len(),
        expected.len(),
        "no extra visible decoration regions"
    );
    for rect in expected {
        assert!(
            actual.contains(&rect),
            "missing visible region {rect:?} in {actual:?}"
        );
    }
    for entity in original {
        let image = app.world().get::<ImageNode>(entity).unwrap();
        let color = match decoration {
            Decoration::Highlight => Color::srgba(1.0, 1.0, 1.0, 0.25),
            _ => Color::srgba(0.2, 0.4, 0.6, 0.4),
        };
        assert_eq!(image.color, color);
        if matches!(decoration, Decoration::ThreeSlice | Decoration::Highlight) {
            assert_texture(app.world(), &image.image, replacement);
            assert!(
                app.world()
                    .resource::<Assets<Image>>()
                    .get(&image.image)
                    .is_some()
            );
        }
    }
}

#[test]
fn three_slice_geometry_color_and_texture_updates() {
    assert_updates(Decoration::ThreeSlice);
}
#[test]
fn css_border_geometry_and_color_updates() {
    assert_updates(Decoration::CssBorder);
}
#[test]
fn backdrop_border_geometry_and_color_updates() {
    assert_updates(Decoration::BackdropBorder);
}
#[test]
fn hover_overlay_geometry_alpha_and_texture_updates() {
    assert_updates(Decoration::Highlight);
}

fn set_active(app: &mut App, id: u64, decoration: Decoration, active: bool) {
    if matches!(decoration, Decoration::Highlight) {
        let world = app.world_mut();
        let mut windows = world.query_filtered::<&mut Window, With<bevy::window::PrimaryWindow>>();
        windows
            .single_mut(world)
            .unwrap()
            .set_cursor_position(active.then_some(Vec2::new(20.0, 20.0)));
    }
    let mut ui = app.world_mut().resource_mut::<UiState>();
    if matches!(decoration, Decoration::Highlight) {
        let Some(WidgetData::Button(button)) = &mut ui.registry.get_mut(id).unwrap().widget_data
        else {
            panic!("button")
        };
        button.hovered = active;
    } else {
        ui.registry.set_hidden(id, !active);
    }
}

fn assert_repair_and_cleanup(decoration: Decoration) {
    let (mut app, id) = fixture(decoration);
    let original = image_entities(app.world_mut(), id);
    assert_image_repair(&mut app, original[0]);
    assert_eq!(image_entities(app.world_mut(), id), original);
    set_active(&mut app, id, decoration, false);
    settle(&mut app);
    assert!(image_entities(app.world_mut(), id).is_empty());
    for entity in original {
        assert!(app.world().get_entity(entity).is_err());
    }
    set_active(&mut app, id, decoration, true);
    settle(&mut app);
    let restored = image_entities(app.world_mut(), id);
    assert!(!restored.is_empty());
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .remove_frame(id);
    settle(&mut app);
    for entity in restored {
        assert!(app.world().get_entity(entity).is_err());
    }
}

#[test]
fn three_slice_repairs_components_and_removes_stale_parts() {
    assert_repair_and_cleanup(Decoration::ThreeSlice);
}
#[test]
fn css_border_repairs_components_and_removes_stale_parts() {
    assert_repair_and_cleanup(Decoration::CssBorder);
}
#[test]
fn backdrop_border_repairs_components_and_removes_stale_parts() {
    assert_repair_and_cleanup(Decoration::BackdropBorder);
}
#[test]
fn hover_overlay_repairs_components_and_removes_stale_overlay() {
    assert_repair_and_cleanup(Decoration::Highlight);
}
