use super::*;
use crate::frame::WidgetData;
use crate::registry::FrameRegistry;
use crate::screen::{Screen, SharedContext};
use crate::widgets::button::{ButtonData, ButtonState};
use crate::widgets::texture::TextureSource;

/// Selected tab and whether the third tab is disabled.
struct View(usize, bool);

fn tab_screen(art: Option<TabArt<'static>>) -> Screen {
    Screen::new(move |ctx| {
        let View(selected, third_disabled) = *ctx.get::<View>().unwrap();
        let tabs = [
            Tab {
                label: "Quests",
                action: "tab:quests",
                disabled: false,
            },
            Tab {
                label: "Map",
                action: "tab:map",
                disabled: false,
            },
            Tab {
                label: "Pets",
                action: "tab:pets",
                disabled: third_disabled,
            },
        ];
        tab_strip(TabStrip {
            name: "LogTabs",
            tabs: &tabs,
            selected,
            tab_width: 80.0,
            tab_height: 24.0,
            gap: 4.0,
            art,
        })
    })
}

fn setup(art: Option<TabArt<'static>>, view: View) -> (Screen, SharedContext, FrameRegistry) {
    let mut ctx = SharedContext::new();
    ctx.insert(view);
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let mut screen = tab_screen(art);
    screen.sync(&ctx, &mut registry);
    (screen, ctx, registry)
}

fn click(registry: &mut FrameRegistry, index: usize) -> Option<String> {
    let id = registry.get_by_name(&tab_name("LogTabs", index)).unwrap();
    registry.click_frame(id)
}

fn button(registry: &FrameRegistry, index: usize) -> &ButtonData {
    let id = registry.get_by_name(&tab_name("LogTabs", index)).unwrap();
    match &registry.get(id).unwrap().widget_data {
        Some(WidgetData::Button(button)) => button,
        _ => panic!("tab is not a button"),
    }
}

#[test]
fn clicking_tab_emits_its_action() {
    let (_, _, mut registry) = setup(None, View(0, false));
    assert_eq!(click(&mut registry, 1).as_deref(), Some("tab:map"));
    assert_eq!(click(&mut registry, 2).as_deref(), Some("tab:pets"));
    assert_eq!(button(&registry, 0).text, "Quests");
}

#[test]
fn disabling_a_tab_drops_its_action_across_rebuild() {
    let (mut screen, mut ctx, mut registry) = setup(None, View(0, false));
    ctx.insert(View(0, true));
    screen.sync(&ctx, &mut registry);
    assert_eq!(click(&mut registry, 2).as_deref(), Some(""));
    assert_eq!(button(&registry, 2).state, ButtonState::Disabled);

    ctx.insert(View(0, false));
    screen.sync(&ctx, &mut registry);
    assert_eq!(click(&mut registry, 2).as_deref(), Some("tab:pets"));
    assert_eq!(button(&registry, 2).state, ButtonState::Normal);
}

#[test]
fn selected_tab_uses_selected_art_and_follows_selection() {
    let art = TabArt {
        idle_atlas: "tab-idle",
        selected_atlas: "tab-selected",
    };
    let atlas = |registry: &FrameRegistry, index| button(registry, index).normal_texture.clone();
    let (mut screen, mut ctx, mut registry) = setup(Some(art), View(0, false));
    assert_eq!(
        atlas(&registry, 0),
        Some(TextureSource::Atlas("tab-selected".into()))
    );
    assert_eq!(
        atlas(&registry, 1),
        Some(TextureSource::Atlas("tab-idle".into()))
    );

    ctx.insert(View(1, false));
    screen.sync(&ctx, &mut registry);
    assert_eq!(
        atlas(&registry, 0),
        Some(TextureSource::Atlas("tab-idle".into()))
    );
    assert_eq!(
        atlas(&registry, 1),
        Some(TextureSource::Atlas("tab-selected".into()))
    );
}
