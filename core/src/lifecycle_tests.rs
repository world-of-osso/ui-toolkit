//! Source behavior controls: GameMenuEsc.lua, UIParentPanelManager.lua,
//! MerchantFrame_OnShow/OnHide, PerksProgram OnShow/OnHide/OnEnter and ReadyCheck.xml.
use crate::lifecycle::{
    EscapeBinding, EscapePriority, HoverEdge, LifecycleSounds, VisibilityEdge, close_group,
    dispatch_escape,
};

#[test]
fn ui_template_lifecycle_visibility_edges_keep_each_observation_boundary_and_reset() {
    let merchant = LifecycleSounds {
        on_show: Some(839),
        on_hide: Some(840),
        ..Default::default()
    };
    let trading_post = LifecycleSounds {
        on_show: Some(218932),
        on_hide: Some(218933),
        ..Default::default()
    };
    let mut session = VisibilityEdge::default();
    let mut mounted = VisibilityEdge::default();
    // Merchant observes its open session before sync; a TP request is not mounted UI.
    assert_eq!(session.observe(true, &merchant), Some(839));
    assert_eq!(mounted.observe(false, &trading_post), None);
    assert_eq!(session.observe(true, &merchant), None);
    assert_eq!(mounted.observe(true, &trading_post), Some(218932));
    assert_eq!(mounted.observe(true, &trading_post), None);
    assert_eq!(session.observe(false, &merchant), Some(840));
    assert_eq!(session.observe(false, &merchant), None);
    assert_eq!(mounted.observe(false, &trading_post), Some(218933));
    assert_eq!(session.observe(true, &merchant), Some(839));
    session.reset(); // Merchant's leave-world reset is silent, not an artificial OnHide.
    assert_eq!(session.observe(false, &merchant), None);
    assert_eq!(session.observe(true, &merchant), Some(839));
}

#[test]
fn ui_template_lifecycle_ready_listener_has_show_only_sound_without_universal_hide() {
    let ready = LifecycleSounds {
        on_show: Some(8960),
        ..Default::default()
    };
    let mut listener = VisibilityEdge::default();
    assert_eq!(listener.observe(false, &ready), None);
    assert_eq!(listener.observe(true, &ready), Some(8960));
    assert_eq!(listener.observe(true, &ready), None);
    assert_eq!(listener.observe(false, &ready), None);
    assert_eq!(listener.observe(true, &ready), Some(8960));
    let mut ordinary = VisibilityEdge::default();
    assert_eq!(ordinary.observe(true, &LifecycleSounds::default()), None);
    assert_eq!(ordinary.observe(false, &LifecycleSounds::default()), None);
}

#[test]
fn ui_template_lifecycle_hover_tracks_logical_owner_across_rebuilds_and_reentry() {
    let product = LifecycleSounds {
        on_enter: Some(218934),
        ..Default::default()
    };
    let mut hover = HoverEdge::default();
    assert_eq!(
        hover.observe(Some("PerksProgramProduct12".into()), &product),
        Some(218934)
    );
    // New native nodes for the same authored owner must not re-fire OnEnter.
    assert_eq!(
        hover.observe(Some(String::from("PerksProgramProduct12")), &product),
        None
    );
    assert_eq!(
        hover.observe(Some("PerksProgramProduct18".into()), &product),
        Some(218934)
    );
    assert_eq!(hover.observe(None, &product), None);
    assert_eq!(
        hover.observe(Some("PerksProgramProduct18".into()), &product),
        Some(218934)
    );
    hover.reset();
    assert_eq!(
        hover.observe(Some("PerksProgramProduct18".into()), &product),
        Some(218934)
    );
}

#[derive(Clone, Copy)]
enum Hook {
    Dialog,
    Menu,
    FirstAddon,
    SecondAddon,
    Windows,
    World,
}

#[derive(Default)]
struct Windows {
    dialog: bool,
    menu: bool,
    target: bool,
    addon: Option<u32>,
    open: Vec<u32>,
}

fn close(context: &mut Windows, hook: Hook) -> Result<bool, &'static str> {
    Ok(match hook {
        Hook::Dialog => std::mem::take(&mut context.dialog),
        Hook::Menu => std::mem::take(&mut context.menu),
        Hook::FirstAddon | Hook::SecondAddon => {
            let id = if matches!(hook, Hook::FirstAddon) {
                1
            } else {
                2
            };
            if context.addon == Some(id) {
                context.addon = None;
                true
            } else {
                false
            }
        }
        Hook::Windows => close_group(&[10, 20, 30], context, |context, id| {
            let was_open = context.open.contains(&id);
            context.open.retain(|open| *open != id);
            Ok(was_open)
        })?,
        Hook::World => std::mem::take(&mut context.target),
    })
}

#[test]
fn ui_template_lifecycle_escape_priorities_are_stable_and_first_handled_is_exclusive() {
    let bindings = [
        EscapeBinding {
            priority: EscapePriority::World,
            hook: Hook::World,
        },
        EscapeBinding {
            priority: EscapePriority::AddOnPost,
            hook: Hook::Windows,
        },
        EscapeBinding {
            priority: EscapePriority::AddOn,
            hook: Hook::FirstAddon,
        },
        EscapeBinding {
            priority: EscapePriority::Menu,
            hook: Hook::Menu,
        },
        EscapeBinding {
            priority: EscapePriority::Dialog,
            hook: Hook::Dialog,
        },
        EscapeBinding {
            priority: EscapePriority::AddOn,
            hook: Hook::SecondAddon,
        },
    ];
    let mut windows = Windows {
        dialog: true,
        menu: true,
        target: true,
        addon: Some(1),
        open: vec![10, 20, 30],
    };
    assert_eq!(dispatch_escape(&bindings, &mut windows, close), Ok(true));
    assert!(!windows.dialog);
    assert!(windows.menu && windows.target);
    assert_eq!(windows.open, [10, 20, 30]);
    assert_eq!(dispatch_escape(&bindings, &mut windows, close), Ok(true));
    assert!(!windows.menu);
    assert_eq!(windows.addon, Some(1));
    assert_eq!(dispatch_escape(&bindings, &mut windows, close), Ok(true));
    assert_eq!(windows.addon, None);
    assert_eq!(windows.open, [10, 20, 30]);
    assert_eq!(dispatch_escape(&bindings, &mut windows, close), Ok(true));
    assert!(windows.open.is_empty()); // one all-close sweep, not just the topmost panel
    assert!(windows.target);
    assert_eq!(dispatch_escape(&bindings, &mut windows, close), Ok(true));
    assert!(!windows.target);
    assert_eq!(dispatch_escape(&bindings, &mut windows, close), Ok(false));
}

#[test]
fn ui_template_lifecycle_escape_equal_priority_preserves_registration_order_and_errors() {
    let bindings = [
        EscapeBinding {
            priority: EscapePriority::AddOn,
            hook: Hook::FirstAddon,
        },
        EscapeBinding {
            priority: EscapePriority::AddOn,
            hook: Hook::SecondAddon,
        },
    ];
    let mut remaining = vec![1, 2];
    let handled: Result<bool, &str> =
        dispatch_escape(&bindings, &mut remaining, |remaining, hook| {
            let id = if matches!(hook, Hook::FirstAddon) {
                1
            } else {
                2
            };
            remaining.retain(|open| *open != id);
            Ok(true)
        });
    assert_eq!(handled, Ok(true));
    assert_eq!(remaining, [2]);
    let mut windows = Windows {
        target: true,
        open: vec![10, 20, 30],
        ..Default::default()
    };
    let result = close_group(&[10, 20, 30], &mut windows, |context, id| {
        if id == 20 {
            return Err("trade close request failed");
        }
        context.open.retain(|open| *open != id);
        Ok(true)
    });
    assert_eq!(result, Err("trade close request failed"));
    assert_eq!(windows.open, [20, 30]);
    assert!(windows.target);
}
