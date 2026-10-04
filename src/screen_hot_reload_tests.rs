use super::*;
use bevy::ecs::schedule::SingleThreadedExecutor;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

use crate::plugin::{UiPlugin, UiRenderEnabled, UiState};
use crate::widget_def::{Attr, AttrValue, WidgetDef};

struct ReloadFixture;

impl ReloadFixture {
    fn install() -> (Self, Sender<HotReloadTemplate>) {
        let (sender, receiver) = mpsc::channel();
        TEST_HOT_RELOAD_RX.with(|slot| {
            assert!(slot.borrow().is_none());
            *slot.borrow_mut() = Some(receiver);
        });
        (Self, sender)
    }
}

impl Drop for ReloadFixture {
    fn drop(&mut self) {
        TEST_HOT_RELOAD_RX.with(|slot| *slot.borrow_mut() = None);
    }
}

fn width_definition(name: &str, width: u32) -> WidgetChild {
    let mut definition = WidgetDef::new("Frame");
    definition.name = Some(name.to_owned());
    definition.attrs = vec![Attr {
        name: "width",
        name_owned: None,
        value: AttrValue::Static(width.to_string()),
    }];
    WidgetChild::Widget(definition)
}

fn enqueue_widths(sender: &Sender<HotReloadTemplate>, first: u32, second: u32) {
    sender
        .send(HotReloadTemplate {
            key: ("cadence.rs".into(), 1, 1),
            defs: vec![
                width_definition("FirstScreen", first),
                width_definition("SecondScreen", second),
            ],
        })
        .unwrap();
}

fn advance_real_time(app: &mut App, milliseconds: u64) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        milliseconds,
    )));
    app.update();
}

fn assert_widths(app: &App, first: f32, second: f32) {
    let registry = &app.world().resource::<UiState>().registry;
    for (name, expected) in [("FirstScreen", first), ("SecondScreen", second)] {
        let id = registry.get_by_name(name).unwrap();
        assert_eq!(registry.get(id).unwrap().width.value(), expected, "{name}");
    }
}

#[test]
fn hot_reload_once_per_second_preserves_immediate_screen_updates() {
    let (_fixture, sender) = ReloadFixture::install();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.init_asset::<bevy::text::Font>();
    app.add_plugins(UiPlugin);
    app.insert_resource(UiRenderEnabled(false));
    // The test receiver is local to this test thread, not the global watcher.
    app.edit_schedule(Update, |schedule| {
        schedule.set_executor(SingleThreadedExecutor::new());
    });
    advance_real_time(&mut app, 0);
    app.world_mut().resource_mut::<Time<Virtual>>().pause();

    let mut context = SharedContext::new();
    context.insert(10_u32);
    let mut first = Screen::new(|context| {
        vec![width_definition(
            "FirstScreen",
            *context.get::<u32>().unwrap(),
        )]
    });
    let mut second = Screen::new(|_| vec![width_definition("SecondScreen", 20)]);
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        first.sync(&context, &mut ui.registry);
        second.sync(&context, &mut ui.registry);
    }
    enqueue_widths(&sender, 30, 40);
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        first.sync(&context, &mut ui.registry);
        second.sync(&context, &mut ui.registry);
    }
    assert_widths(&app, 10.0, 20.0);
    advance_real_time(&mut app, 999);
    assert_widths(&app, 10.0, 20.0);

    context.insert(11_u32);
    first.sync(
        &context,
        &mut app.world_mut().resource_mut::<UiState>().registry,
    );
    assert_widths(&app, 11.0, 20.0);
    advance_real_time(&mut app, 1);
    assert_widths(&app, 30.0, 40.0);

    enqueue_widths(&sender, 50, 60);
    advance_real_time(&mut app, 999);
    assert_widths(&app, 30.0, 40.0);
    advance_real_time(&mut app, 1);
    assert_widths(&app, 50.0, 60.0);
}
