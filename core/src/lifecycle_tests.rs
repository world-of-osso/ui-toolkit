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

#[test]
fn ui_template_lifecycle_binding_roundtrip_preserves_authored_hooks_without_inference() {
    use crate::attrs::{apply_attribute, read_attribute};
    use crate::registry::FrameRegistry;
    let mut registry = FrameRegistry::new(1920.0, 1080.0);
    let frame = registry.create_frame("PerksProgram", None);
    for (name, value) in [
        ("lifecycle_on_show", "218932"),
        ("lifecycle_on_hide", "218933"),
        ("lifecycle_on_enter", "218934"),
        ("lifecycle_on_click", "856"),
        ("lifecycle_hover_owner", "PerksProgramProduct12"),
        ("lifecycle_close_hook", "trading_post"),
        ("lifecycle_close_group", "PANELS"),
        ("lifecycle_escape_priority", "AddOnPost"),
    ] {
        apply_attribute(&mut registry, frame, name, value);
        assert_eq!(
            read_attribute(&registry, frame, name).as_deref(),
            Some(value),
            "{name}"
        );
    }
    let ordinary = registry.create_frame("OrdinaryButton", None);
    assert_eq!(
        read_attribute(&registry, ordinary, "lifecycle_on_click"),
        None
    );
    assert_eq!(
        read_attribute(&registry, ordinary, "lifecycle_close_hook"),
        None
    );
}

#[test]
fn ui_template_lifecycle_rebuild_clears_removed_sound_and_close_descriptors() {
    use crate::attrs::read_attribute;
    use crate::registry::FrameRegistry;
    use crate::widget_def::{Attr, WidgetChild, WidgetDef};
    use crate::widget_def_diff::DiffContext;
    let mut registry = FrameRegistry::new(1920.0, 1080.0);
    let mut diff = DiffContext::new();
    let mut owner = WidgetDef::new("frame");
    owner.name = Some("MerchantFrame".into());
    owner.attrs = vec![
        Attr::new_static("lifecycle_on_show", "839".into()),
        Attr::new_static("lifecycle_close_hook", "merchant".into()),
    ];
    diff.diff_roots(&[WidgetChild::Widget(owner)], None, &mut registry);
    let frame = registry.get_by_name("MerchantFrame").unwrap();
    assert_eq!(
        read_attribute(&registry, frame, "lifecycle_on_show").as_deref(),
        Some("839")
    );
    assert_eq!(
        read_attribute(&registry, frame, "lifecycle_close_hook").as_deref(),
        Some("merchant")
    );
    let mut owner = WidgetDef::new("frame");
    owner.name = Some("MerchantFrame".into());
    diff.diff_roots(&[WidgetChild::Widget(owner)], None, &mut registry);
    assert_eq!(read_attribute(&registry, frame, "lifecycle_on_show"), None);
    assert_eq!(
        read_attribute(&registry, frame, "lifecycle_close_hook"),
        None
    );
}

#[test]
fn ui_template_lifecycle_rejects_unknown_priority_group_and_invalid_sound() {
    use crate::attrs::apply_attribute;
    use crate::registry::FrameRegistry;
    for (name, value) in [
        ("lifecycle_escape_priority", "FirstAvailable"),
        ("lifecycle_close_group", "TopmostOnly"),
        ("lifecycle_on_show", "not-a-sound-kit"),
        ("lifecycle_close_hook", ""),
    ] {
        let mut registry = FrameRegistry::new(1920.0, 1080.0);
        let frame = registry.create_frame("InvalidBinding", None);
        let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            apply_attribute(&mut registry, frame, name, value);
        }));
        assert!(rejected.is_err(), "{name}={value} silently accepted");
    }
}

#[test]
fn ui_template_lifecycle_hover_children_share_the_nearest_authored_logical_owner() {
    use crate::attrs::apply_attribute;
    use crate::lifecycle::hover_owner;
    use crate::registry::FrameRegistry;
    let mut registry = FrameRegistry::new(1920.0, 1080.0);
    let panel = registry.create_frame("PerksProgramFrame", None);
    let product = registry.create_frame("PerksProgramProduct12", Some(panel));
    let toggle = registry.create_frame("PerksProgramCartToggle12", Some(product));
    let refund = registry.create_frame("PerksProgramRefundIcon12", Some(product));
    apply_attribute(&mut registry, panel, "lifecycle_on_show", "218932");
    apply_attribute(&mut registry, product, "lifecycle_on_enter", "218934");
    apply_attribute(
        &mut registry,
        product,
        "lifecycle_hover_owner",
        "PerksProgramProduct12",
    );
    apply_attribute(&mut registry, toggle, "lifecycle_on_click", "856");
    let mut hover = HoverEdge::default();
    for (hit, expected) in [(product, Some(218934)), (toggle, None), (refund, None)] {
        let (owner, binding) = hover_owner(&registry, hit).unwrap();
        assert_eq!(owner.id, product);
        assert_eq!(
            hover.observe(binding.hover_owner.clone(), &binding.sounds),
            expected
        );
    }
    assert!(hover_owner(&registry, panel).is_none());
    registry.set_hidden(product, true);
    assert!(hover_owner(&registry, refund).is_none());
    registry.remove_frame(product);
    assert!(hover_owner(&registry, product).is_none());
}

#[test]
fn ui_template_lifecycle_registered_group_closes_pending_domains_without_a_mounted_frame() {
    use crate::lifecycle::{CloseGroup, CloseRegistration, close_registered_group};
    let registrations = [
        CloseRegistration {
            hook: "bags",
            group: CloseGroup::Bags,
            priority: EscapePriority::AddOnPost,
        },
        CloseRegistration {
            hook: "merchant",
            group: CloseGroup::Panels,
            priority: EscapePriority::AddOnPost,
        },
        CloseRegistration {
            hook: "trading_post",
            group: CloseGroup::Panels,
            priority: EscapePriority::AddOnPost,
        },
        CloseRegistration {
            hook: "item_ref",
            group: CloseGroup::Special,
            priority: EscapePriority::AddOnPost,
        },
    ];
    struct Domains {
        bags: Vec<u8>,
        merchant_npc: Option<u64>,
        trading_request: bool,
        trading_ui_mounted: bool,
        item_ref: bool,
        sent_closes: Vec<u64>,
    }
    let mut state = Domains {
        bags: vec![0, 1],
        merchant_npc: Some(185468),
        trading_request: true,
        trading_ui_mounted: false,
        item_ref: true,
        sent_closes: Vec::new(),
    };
    let mut close = |state: &mut Domains, hook| -> Result<bool, &'static str> {
        match hook {
            "bags" => Ok(!std::mem::take(&mut state.bags).is_empty()),
            "merchant" => {
                if let Some(npc) = state.merchant_npc.take() {
                    state.sent_closes.push(npc);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            "trading_post" => Ok(std::mem::take(&mut state.trading_request)),
            "item_ref" => Ok(std::mem::take(&mut state.item_ref)),
            _ => Err("unknown domain close hook"),
        }
    };
    assert!(
        !state.trading_ui_mounted,
        "Trading Post request deliberately has no mounted UI"
    );
    assert_eq!(
        close_registered_group(&registrations, CloseGroup::Panels, &mut state, &mut close),
        Ok(true)
    );
    assert_eq!(state.merchant_npc, None);
    assert!(!state.trading_request);
    assert_eq!(state.sent_closes, [185468]);
    assert_eq!(state.bags, [0, 1]);
    assert!(state.item_ref);
    assert_eq!(
        close_registered_group(&registrations, CloseGroup::Panels, &mut state, &mut close),
        Ok(false)
    );
    assert_eq!(
        state.sent_closes,
        [185468],
        "repeat sweep cannot resend a domain close"
    );
    assert_eq!(
        close_registered_group(&registrations, CloseGroup::Bags, &mut state, &mut close),
        Ok(true)
    );
    assert!(state.bags.is_empty());
    assert_eq!(
        close_registered_group(&registrations, CloseGroup::Special, &mut state, &mut close),
        Ok(true)
    );
    assert!(!state.item_ref);
}

#[test]
fn ui_template_lifecycle_registration_feeds_root_attrs_without_a_second_policy_copy() {
    use crate::lifecycle::{CloseGroup, CloseRegistration, bind_close_registration};
    use crate::registry::FrameRegistry;
    use crate::widget_def::{WidgetChild, WidgetDef};
    use crate::widget_def_diff::DiffContext;
    let registration = CloseRegistration {
        hook: "merchant",
        group: CloseGroup::Panels,
        priority: EscapePriority::AddOnPost,
    };
    let mut owner = WidgetDef::new("frame");
    owner.name = Some("MerchantFrame".into());
    let element = bind_close_registration(vec![WidgetChild::Widget(owner)], &registration);
    let mut registry = FrameRegistry::new(1920.0, 1080.0);
    DiffContext::new().diff_roots(&element, None, &mut registry);
    let frame = registry
        .get(registry.get_by_name("MerchantFrame").unwrap())
        .unwrap();
    let binding = frame.lifecycle.as_ref().unwrap();
    assert_eq!(binding.close_hook.as_deref(), Some("merchant"));
    assert_eq!(binding.close_group, Some(CloseGroup::Panels));
    assert_eq!(binding.escape_priority, Some(EscapePriority::AddOnPost));
}

#[test]
fn ui_template_lifecycle_escape_registration_feeds_root_without_inventing_a_close_group() {
    use crate::lifecycle::bind_escape_registration;
    use crate::registry::FrameRegistry;
    use crate::widget_def::{WidgetChild, WidgetDef};
    use crate::widget_def_diff::DiffContext;
    let registration = EscapeBinding {
        hook: "game_menu",
        priority: EscapePriority::Framework,
    };
    let mut owner = WidgetDef::new("frame");
    owner.name = Some("GameMenuRoot".into());
    let element = bind_escape_registration(vec![WidgetChild::Widget(owner)], &registration);
    let mut registry = FrameRegistry::new(1920.0, 1080.0);
    DiffContext::new().diff_roots(&element, None, &mut registry);
    let frame = registry
        .get(registry.get_by_name("GameMenuRoot").unwrap())
        .unwrap();
    let binding = frame.lifecycle.as_ref().unwrap();
    assert_eq!(binding.close_hook.as_deref(), Some("game_menu"));
    assert_eq!(binding.escape_priority, Some(EscapePriority::Framework));
    assert_eq!(
        binding.close_group, None,
        "Framework handler is not a UISpecialFrames member"
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
            Ok::<bool, &'static str>(was_open)
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
    let failing_bindings = [
        EscapeBinding {
            priority: EscapePriority::Dialog,
            hook: Hook::Dialog,
        },
        EscapeBinding {
            priority: EscapePriority::World,
            hook: Hook::World,
        },
    ];
    let result = dispatch_escape(&failing_bindings, &mut windows, |context, hook| {
        if matches!(hook, Hook::Dialog) {
            return Err("invitation dismissal failed");
        }
        close(context, hook)
    });
    assert_eq!(result, Err("invitation dismissal failed"));
    assert!(windows.target); // a failed domain action must not fall through into World.
}

#[test]
fn ui_template_lifecycle_click_requires_an_explicit_enabled_visible_owner() {
    use crate::frame::WidgetData;
    use crate::lifecycle::{LifecycleBinding, click_sound};
    use crate::registry::FrameRegistry;
    use crate::widgets::button::{ButtonData, ButtonState};
    let mut registry = FrameRegistry::new(1920.0, 1080.0);
    let id = registry.create_frame("AuthoredButton", None);
    let frame = registry.get_mut(id).unwrap();
    frame.widget_data = Some(WidgetData::Button(ButtonData::default()));
    assert_eq!(
        click_sound(frame),
        None,
        "ordinary buttons do not imply UIButtonMixin"
    );
    frame.lifecycle = Some(LifecycleBinding {
        sounds: LifecycleSounds {
            on_show: Some(839),
            ..Default::default()
        },
        ..Default::default()
    });
    assert_eq!(
        click_sound(frame),
        None,
        "show hooks do not create click hooks"
    );
    frame.lifecycle.as_mut().unwrap().sounds.on_click = Some(856);
    assert_eq!(click_sound(frame), Some(856));
    frame.lifecycle.as_mut().unwrap().sounds.on_click = Some(286137);
    assert_eq!(
        click_sound(frame),
        Some(286137),
        "explicit kit replaces the default declaration"
    );
    frame.widget_data = Some(WidgetData::Button(ButtonData {
        enabled: false,
        ..Default::default()
    }));
    assert_eq!(click_sound(frame), None);
    frame.widget_data = Some(WidgetData::Button(ButtonData {
        enabled: true,
        state: ButtonState::Disabled,
        ..Default::default()
    }));
    assert_eq!(click_sound(frame), None);
    frame.widget_data = Some(WidgetData::Button(ButtonData::default()));
    frame.visible = false;
    assert_eq!(click_sound(frame), None);
    frame.visible = true;
    assert_eq!(click_sound(frame), Some(286137));
}

#[test]
fn ui_template_lifecycle_catalog_keeps_group_and_exclusive_dispatch_independent() {
    use crate::lifecycle::{CloseGroup, LifecycleRegistration, close_registered_group};
    use std::collections::BTreeSet;
    let registrations = [
        LifecycleRegistration {
            hook: "inactive",
            group: None,
            escape: None,
        },
        LifecycleRegistration {
            hook: "panel",
            group: Some(CloseGroup::Panels),
            escape: None,
        },
        LifecycleRegistration {
            hook: "professions",
            group: Some(CloseGroup::Panels),
            escape: Some(EscapeBinding {
                hook: "professions_escape",
                priority: EscapePriority::AddOn,
            }),
        },
        LifecycleRegistration {
            hook: "menu",
            group: None,
            escape: Some(EscapeBinding {
                hook: "menu_escape",
                priority: EscapePriority::Framework,
            }),
        },
    ];
    let mut shown = BTreeSet::from(["inactive", "panel", "professions", "menu"]);
    let mut escape = |shown: &mut BTreeSet<&str>, hook| {
        let window = match hook {
            "menu_escape" => "menu",
            "professions_escape" => "professions",
            _ => panic!("group membership must not fabricate an exclusive handler: {hook}"),
        };
        Ok::<_, &str>(shown.remove(window))
    };
    assert_eq!(
        dispatch_escape(&registrations, &mut shown, &mut escape),
        Ok(true)
    );
    assert_eq!(shown, BTreeSet::from(["inactive", "panel", "professions"]));
    assert_eq!(
        dispatch_escape(&registrations, &mut shown, &mut escape),
        Ok(true)
    );
    assert_eq!(shown, BTreeSet::from(["inactive", "panel"]));
    assert_eq!(
        dispatch_escape(&registrations, &mut shown, &mut escape),
        Ok(false)
    );

    let mut shown = BTreeSet::from(["inactive", "panel", "professions", "menu"]);
    assert_eq!(
        close_registered_group(
            &registrations,
            CloseGroup::Panels,
            &mut shown,
            |shown, hook| { Ok::<_, &str>(shown.remove(hook)) }
        ),
        Ok(true)
    );
    assert_eq!(shown, BTreeSet::from(["inactive", "menu"]));
}

#[test]
fn ui_template_lifecycle_catalog_root_rebuild_preserves_separate_hooks_and_absence() {
    use crate::lifecycle::{CloseGroup, LifecycleRegistration, bind_registration};
    use crate::registry::FrameRegistry;
    use crate::widget_def::{WidgetChild, WidgetDef};
    use crate::widget_def_diff::DiffContext;
    let cases = [
        (
            LifecycleRegistration {
                hook: "professions",
                group: Some(CloseGroup::Panels),
                escape: Some(EscapeBinding {
                    hook: "professions_escape",
                    priority: EscapePriority::AddOn,
                }),
            },
            Some((
                Some("professions"),
                Some(CloseGroup::Panels),
                Some("professions_escape"),
                Some(EscapePriority::AddOn),
            )),
        ),
        (
            LifecycleRegistration {
                hook: "panel",
                group: Some(CloseGroup::Panels),
                escape: None,
            },
            Some((Some("panel"), Some(CloseGroup::Panels), None, None)),
        ),
        (
            LifecycleRegistration {
                hook: "menu",
                group: None,
                escape: Some(EscapeBinding {
                    hook: "menu_escape",
                    priority: EscapePriority::Framework,
                }),
            },
            Some((
                None,
                None,
                Some("menu_escape"),
                Some(EscapePriority::Framework),
            )),
        ),
        (
            LifecycleRegistration {
                hook: "inactive",
                group: None,
                escape: None,
            },
            None,
        ),
    ];
    let mut registry = FrameRegistry::new(1920.0, 1080.0);
    let mut diff = DiffContext::new();
    for (registration, expected) in cases {
        let mut owner = WidgetDef::new("frame");
        owner.name = Some("LifecycleRoot".into());
        let element = bind_registration(vec![WidgetChild::Widget(owner)], &registration);
        diff.diff_roots(&element, None, &mut registry);
        let frame = registry
            .get(registry.get_by_name("LifecycleRoot").unwrap())
            .unwrap();
        let actual = frame.lifecycle.as_ref().map(|binding| {
            (
                binding.close_hook.as_deref(),
                binding.close_group,
                binding.escape_hook.as_deref(),
                binding.escape_priority,
            )
        });
        assert_eq!(
            actual, expected,
            "root policy after rebuild: {}",
            registration.hook
        );
    }
}
