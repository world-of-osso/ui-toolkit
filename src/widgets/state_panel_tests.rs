use super::*;
use crate::screen::{Screen, SharedContext};

#[derive(Clone, Copy)]
enum View {
    Loading,
    Empty,
    Error,
    Unavailable,
}

fn state(view: View) -> PanelState<'static> {
    match view {
        View::Loading => PanelState::Loading { label: None },
        View::Empty => PanelState::Empty { message: "No mail" },
        View::Error => PanelState::Error {
            message: "Server rejected request",
            retry_action: Some("mail:retry"),
        },
        View::Unavailable => PanelState::Unavailable { message: None },
    }
}

fn setup(view: View) -> (Screen, SharedContext, FrameRegistry) {
    let mut ctx = SharedContext::new();
    ctx.insert(view);
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let mut screen =
        Screen::new(|ctx| state_panel("MailState", state(*ctx.get::<View>().unwrap())));
    screen.sync(&ctx, &mut registry);
    (screen, ctx, registry)
}

fn text(registry: &FrameRegistry) -> String {
    let id = registry.get_by_name(&text_name("MailState")).unwrap();
    match &registry.get(id).unwrap().widget_data {
        Some(WidgetData::FontString(fs)) => fs.text.clone(),
        _ => panic!("state text is not a fontstring"),
    }
}

#[test]
fn empty_and_unavailable_show_messages_without_retry() {
    let (mut screen, mut ctx, mut registry) = setup(View::Empty);
    assert_eq!(text(&registry), "No mail");
    assert!(registry.get_by_name(&retry_name("MailState")).is_none());
    ctx.insert(View::Unavailable);
    screen.sync(&ctx, &mut registry);
    assert_eq!(text(&registry), "Not available yet");
}

#[test]
fn error_retry_button_emits_retry_action_and_leaves_with_state() {
    let (mut screen, mut ctx, mut registry) = setup(View::Error);
    assert_eq!(text(&registry), "Server rejected request");
    let retry = registry.get_by_name(&retry_name("MailState")).unwrap();
    assert_eq!(registry.click_frame(retry).as_deref(), Some("mail:retry"));

    ctx.insert(View::Empty);
    screen.sync(&ctx, &mut registry);
    assert!(registry.get_by_name(&retry_name("MailState")).is_none());
}

#[test]
fn loading_text_animates_until_state_changes() {
    let (mut screen, mut ctx, mut registry) = setup(View::Loading);
    animate_loading_texts_at(&mut registry, 0.1);
    assert_eq!(text(&registry), "Loading");
    animate_loading_texts_at(&mut registry, 0.5);
    assert_eq!(text(&registry), "Loading.");
    animate_loading_texts_at(&mut registry, 1.3);
    assert_eq!(text(&registry), "Loading...");
    animate_loading_texts_at(&mut registry, 1.7);
    assert_eq!(text(&registry), "Loading");

    ctx.insert(View::Empty);
    screen.sync(&ctx, &mut registry);
    animate_loading_texts_at(&mut registry, 0.5);
    assert_eq!(text(&registry), "No mail");
}
