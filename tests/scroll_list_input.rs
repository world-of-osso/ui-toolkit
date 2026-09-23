use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use ui_toolkit::event::EventBus;
use ui_toolkit::frame::Dimension;
use ui_toolkit::layout::LayoutRect;
use ui_toolkit::plugin::UiState;
use ui_toolkit::registry::FrameRegistry;
use ui_toolkit::rsx;
use ui_toolkit::screen::{Screen, SharedContext};
use ui_toolkit::scroll_input::sync_scroll_list_input;
use ui_toolkit::widgets::scroll_list::{ScrollList, scroll_list, thumb_name, track_name};

struct DynName(String);

const LIST_X: f32 = 100.0;
const LIST_Y: f32 = 100.0;

fn list_screen() -> Screen {
    Screen::new(|ctx| {
        scroll_list(
            ctx,
            ScrollList {
                name: "Mail",
                width: 200.0,
                height: 100.0,
                row_height: 20.0,
                row_count: 50,
                track_width: 16.0,
                track_fdid: None,
                thumb_fdid: None,
            },
            |index| {
                rsx! {
                    button {
                        name: DynName(format!("MailItem{index}")),
                        width: 184,
                        height: 20,
                        onclick: {format!("mail:open:{index}")},
                    }
                }
            },
        )
    })
}

struct Fixture {
    app: App,
    window: Entity,
    screen: Screen,
    ctx: SharedContext,
}

impl Fixture {
    fn new() -> Self {
        let mut app = App::new();
        app.insert_resource(UiState {
            registry: FrameRegistry::new(800.0, 600.0),
            event_bus: EventBus::new(),
            focused_frame: None,
        });
        app.init_resource::<ButtonInput<MouseButton>>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<AccumulatedMouseScroll>();
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        app.add_systems(Update, sync_scroll_list_input);
        let mut fixture = Self {
            app,
            window,
            screen: list_screen(),
            ctx: SharedContext::new(),
        };
        fixture.sync_screen();
        fixture
    }

    fn registry(&self) -> &FrameRegistry {
        &self.app.world().resource::<UiState>().registry
    }

    /// Rebuild the screen, then stand in for Bevy layout readback of the list rects.
    fn sync_screen(&mut self) {
        let mut ui = self.app.world_mut().resource_mut::<UiState>();
        self.screen.sync(&self.ctx, &mut ui.registry);
        place_layout(&mut ui.registry);
    }

    fn update(&mut self) {
        self.app.update();
        self.sync_screen();
        let world = self.app.world_mut();
        world.resource_mut::<ButtonInput<MouseButton>>().clear();
        world.resource_mut::<ButtonInput<KeyCode>>().clear();
        *world.resource_mut::<AccumulatedMouseScroll>() = AccumulatedMouseScroll::default();
    }

    fn cursor(&mut self, x: f32, y: f32) {
        let mut window = self.app.world_mut().get_mut::<Window>(self.window).unwrap();
        window.set_cursor_position(Some(Vec2::new(x, y)));
    }

    fn wheel_lines(&mut self, lines: f32) {
        *self
            .app
            .world_mut()
            .resource_mut::<AccumulatedMouseScroll>() = AccumulatedMouseScroll {
            unit: MouseScrollUnit::Line,
            delta: Vec2::new(0.0, lines),
        };
        self.update();
    }

    fn mouse(&mut self, pressed: bool) {
        let mut mouse = self
            .app
            .world_mut()
            .resource_mut::<ButtonInput<MouseButton>>();
        if pressed {
            mouse.press(MouseButton::Left);
        } else {
            mouse.release(MouseButton::Left);
        }
        self.update();
    }

    fn key(&mut self, key: KeyCode) {
        let mut keys = self.app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(key);
        self.update();
        self.app
            .world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(key);
    }

    fn first_row(&self) -> usize {
        self.registry().scroll_lists.get("Mail").unwrap().first_row
    }

    fn visible_items(&self) -> Vec<String> {
        (0..50)
            .map(|index| format!("MailItem{index}"))
            .filter(|name| self.registry().get_by_name(name).is_some())
            .collect()
    }
}

fn place_layout(registry: &mut FrameRegistry) {
    set_rect(registry, "Mail", LIST_X, LIST_Y, 200.0, 100.0);
    set_rect(
        registry,
        &track_name("Mail"),
        LIST_X + 184.0,
        LIST_Y,
        16.0,
        100.0,
    );
    let thumb = registry.get_by_name(&thumb_name("Mail")).unwrap();
    let frame = registry.get(thumb).unwrap();
    let Val::Px(top) = frame.position.top else {
        panic!("thumb top is not pixels");
    };
    let Dimension::Fixed(height) = frame.height else {
        panic!("thumb height is not fixed");
    };
    set_rect(
        registry,
        &thumb_name("Mail"),
        LIST_X + 184.0,
        LIST_Y + top,
        16.0,
        height,
    );
    let rows: Vec<(u64, Val)> = registry
        .children_of(registry.get_by_name("Mail").unwrap())
        .into_iter()
        .filter_map(|id| {
            let frame = registry.get(id)?;
            frame.name.as_deref()?.strip_prefix("MailRow")?;
            Some((id, frame.position.top))
        })
        .collect();
    for (row, top) in rows {
        let Val::Px(top) = top else { continue };
        let rect = LayoutRect {
            x: LIST_X,
            y: LIST_Y + top,
            width: 184.0,
            height: 20.0,
        };
        registry.set_computed_layout(row, rect.clone()).unwrap();
        for item in registry.children_of(row) {
            registry.set_computed_layout(item, rect.clone()).unwrap();
        }
    }
}

fn set_rect(registry: &mut FrameRegistry, name: &str, x: f32, y: f32, width: f32, height: f32) {
    let id = registry.get_by_name(name).unwrap();
    registry
        .set_computed_layout(
            id,
            LayoutRect {
                x,
                y,
                width,
                height,
            },
        )
        .unwrap();
}

#[test]
fn wheel_over_list_scrolls_rows_and_wheel_elsewhere_does_not() {
    let mut f = Fixture::new();
    f.cursor(150.0, 130.0);
    f.wheel_lines(-2.0);
    assert_eq!(f.first_row(), 2);
    assert_eq!(
        f.visible_items(),
        [
            "MailItem2",
            "MailItem3",
            "MailItem4",
            "MailItem5",
            "MailItem6"
        ]
    );
    f.wheel_lines(5.0);
    assert_eq!(f.first_row(), 0, "wheel up clamps at the top");

    f.cursor(500.0, 500.0);
    f.wheel_lines(-3.0);
    assert_eq!(f.first_row(), 0);
}

#[test]
fn dragging_thumb_maps_cursor_to_rows() {
    let mut f = Fixture::new();
    // Thumb is 16px tall at the track top; grab it 4px below its top.
    f.cursor(LIST_X + 190.0, LIST_Y + 4.0);
    f.mouse(true);
    f.cursor(LIST_X + 190.0, LIST_Y + 4.0 + 42.0);
    f.update();
    // Half of the 84px travel over 45 scrollable rows.
    assert_eq!(f.first_row(), 23);
    assert_eq!(f.visible_items()[0], "MailItem23");
    f.cursor(LIST_X + 190.0, 590.0);
    f.update();
    assert_eq!(
        f.first_row(),
        45,
        "dragging past the track clamps to the end"
    );

    f.mouse(false);
    f.cursor(LIST_X + 190.0, LIST_Y + 4.0);
    f.update();
    assert_eq!(
        f.first_row(),
        45,
        "released thumb no longer follows the cursor"
    );
}

#[test]
fn page_keys_scroll_only_after_list_gains_focus() {
    let mut f = Fixture::new();
    f.key(KeyCode::PageDown);
    assert_eq!(f.first_row(), 0, "unfocused list ignores keys");

    f.cursor(150.0, 130.0);
    f.mouse(true);
    f.mouse(false);
    f.key(KeyCode::PageDown);
    assert_eq!(f.first_row(), 5);
    f.key(KeyCode::End);
    assert_eq!(f.first_row(), 45);
    f.key(KeyCode::PageUp);
    assert_eq!(f.first_row(), 40);
    f.key(KeyCode::Home);
    assert_eq!(f.first_row(), 0);

    f.cursor(500.0, 500.0);
    f.mouse(true);
    f.mouse(false);
    f.key(KeyCode::End);
    assert_eq!(f.first_row(), 0, "clicking elsewhere drops list focus");
}
