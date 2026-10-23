//! Retail event edges and Escape dispatch, without domain close or audio authority.
//! Source: R GameMenuEsc.lua, UIParentPanelManager.lua and explicitly bound script sounds.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LifecycleSounds {
    pub on_show: Option<u32>,
    pub on_hide: Option<u32>,
    pub on_enter: Option<u32>,
    pub on_click: Option<u32>,
}

/// The caller supplies its source observation: session, mounted canvas or shown listener.
#[derive(Debug, Default)]
pub struct VisibilityEdge {
    shown: bool,
}

impl VisibilityEdge {
    pub fn observe(&mut self, shown: bool, sounds: &LifecycleSounds) -> Option<u32> {
        let was_shown = std::mem::replace(&mut self.shown, shown);
        match (was_shown, shown) {
            (false, true) => sounds.on_show,
            (true, false) => sounds.on_hide,
            _ => None,
        }
    }

    /// Domain-owned teardown may silently reset rather than emit an artificial OnHide.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Logical owner names survive rebuilds; transient native frame IDs do not.
#[derive(Debug, Default)]
pub struct HoverEdge {
    owner: Option<String>,
}

impl HoverEdge {
    pub fn observe(&mut self, owner: Option<String>, sounds: &LifecycleSounds) -> Option<u32> {
        if self.owner == owner {
            return None;
        }
        let entering = owner.is_some();
        self.owner = owner;
        if entering { sounds.on_enter } else { None }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// GameMenuEsc.lua's accepted priorities; declaration order within a priority is stable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum EscapePriority {
    Dialog = 1,
    Menu = 2,
    Casting = 4,
    FrameworkPre = 5,
    Framework = 6,
    FrameworkPost = 7,
    AddOn = 8,
    AddOnPost = 9,
    AddOnPost2 = 10,
    World = 11,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EscapeBinding<H> {
    pub priority: EscapePriority,
    pub hook: H,
}

/// Authored membership distinguishes the bag sweep, panel sweep and UISpecialFrames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseGroup {
    Bags,
    Panels,
    Special,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LifecycleBinding {
    pub sounds: LifecycleSounds,
    pub hover_owner: Option<String>,
    pub close_hook: Option<String>,
    pub close_group: Option<CloseGroup>,
    pub escape_priority: Option<EscapePriority>,
}

impl EscapePriority {
    pub fn from_name(name: &str) -> Self {
        match name {
            "Dialog" => Self::Dialog,
            "Menu" => Self::Menu,
            "Casting" => Self::Casting,
            "FrameworkPre" => Self::FrameworkPre,
            "Framework" => Self::Framework,
            "FrameworkPost" => Self::FrameworkPost,
            "AddOn" => Self::AddOn,
            "AddOnPost" => Self::AddOnPost,
            "AddOnPost2" => Self::AddOnPost2,
            "World" => Self::World,
            _ => panic!("unsupported authored Escape priority: {name}"),
        }
    }
}

impl CloseGroup {
    pub fn from_name(name: &str) -> Self {
        match name {
            "BAGS" => Self::Bags,
            "PANELS" => Self::Panels,
            "SPECIAL" => Self::Special,
            _ => panic!("unsupported authored close group: {name}"),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bags => "BAGS",
            Self::Panels => "PANELS",
            Self::Special => "SPECIAL",
        }
    }
}

/// First handled registration consumes Escape; failed domain actions cannot fall through.
pub fn dispatch_escape<H: Copy, C, E>(
    bindings: &[EscapeBinding<H>],
    context: &mut C,
    mut handle: impl FnMut(&mut C, H) -> Result<bool, E>,
) -> Result<bool, E> {
    let mut ordered: Vec<_> = bindings.iter().collect();
    ordered.sort_by_key(|binding| binding.priority);
    for binding in ordered {
        if handle(context, binding.hook)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// A bag/panel/special-frame sweep invokes every eligible callback, not just one window.
/// Callbacks retain eligibility, network effects and error semantics.
pub fn close_group<H: Copy, C, E>(
    hooks: &[H],
    context: &mut C,
    mut close: impl FnMut(&mut C, H) -> Result<bool, E>,
) -> Result<bool, E> {
    let mut closed = false;
    for hook in hooks.iter().copied() {
        closed |= close(context, hook)?;
    }
    Ok(closed)
}
