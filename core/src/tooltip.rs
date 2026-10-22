//! Authored tooltip bindings. Hosts resolve live content and convert owner anchors to pixels.
use crate::{frame::Frame, registry::FrameRegistry};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipAnchor {
    Default,
    Cursor,
    CursorRight,
    #[default]
    Right,
    Left,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    /// ContainerFrameItemButton_CalculateItemTooltipAnchors.
    BagSlot,
    /// TargetFrame aura owner-side selection.
    ByCenter,
}

impl TooltipAnchor {
    pub fn from_name(name: &str) -> Self {
        match name {
            "DEFAULT" => Self::Default,
            "ANCHOR_CURSOR" => Self::Cursor,
            "ANCHOR_CURSOR_RIGHT" => Self::CursorRight,
            "ANCHOR_RIGHT" => Self::Right,
            "ANCHOR_LEFT" => Self::Left,
            "ANCHOR_TOP" => Self::Top,
            "ANCHOR_BOTTOM" => Self::Bottom,
            "ANCHOR_TOPLEFT" => Self::TopLeft,
            "ANCHOR_TOPRIGHT" => Self::TopRight,
            "ANCHOR_BOTTOMLEFT" => Self::BottomLeft,
            "ANCHOR_BOTTOMRIGHT" => Self::BottomRight,
            "BAG_SLOT" => Self::BagSlot,
            "BY_CENTER" => Self::ByCenter,
            _ => panic!("unsupported authored tooltip anchor: {name}"),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "DEFAULT",
            Self::Cursor => "ANCHOR_CURSOR",
            Self::CursorRight => "ANCHOR_CURSOR_RIGHT",
            Self::Right => "ANCHOR_RIGHT",
            Self::Left => "ANCHOR_LEFT",
            Self::Top => "ANCHOR_TOP",
            Self::Bottom => "ANCHOR_BOTTOM",
            Self::TopLeft => "ANCHOR_TOPLEFT",
            Self::TopRight => "ANCHOR_TOPRIGHT",
            Self::BottomLeft => "ANCHOR_BOTTOMLEFT",
            Self::BottomRight => "ANCHOR_BOTTOMRIGHT",
            Self::BagSlot => "BAG_SLOT",
            Self::ByCenter => "BY_CENTER",
        }
    }
}

/// UIButtonMixin enabled/disabled policy; provider identity binds live domain content.
#[derive(Clone, Debug, PartialEq)]
pub struct TooltipBinding {
    pub provider: Option<String>,
    pub title: Option<String>,
    pub text: Option<String>,
    pub title_color: Option<[f32; 4]>,
    pub text_color: Option<[f32; 4]>,
    pub anchor: TooltipAnchor,
    /// Retail SetOwner offsets: x right, y up. Hosts convert y once.
    pub offset: [f32; 2],
    pub wrap: bool,
    pub disabled_text: Option<String>,
    pub disabled_anchor: TooltipAnchor,
    pub disabled_offset: [f32; 2],
}

impl Default for TooltipBinding {
    fn default() -> Self {
        Self {
            provider: None,
            title: None,
            text: None,
            title_color: None,
            text_color: None,
            anchor: TooltipAnchor::Right,
            offset: [0.0; 2],
            wrap: true,
            disabled_text: None,
            disabled_anchor: TooltipAnchor::Right,
            disabled_offset: [0.0; 2],
        }
    }
}

impl TooltipBinding {
    pub fn has_content(&self) -> bool {
        self.provider.is_some()
            || self.title.is_some()
            || self.text.is_some()
            || self.disabled_text.is_some()
    }
}

/// Resolve the nearest visible bound owner, not a list of panel-name predicates.
/// Missing/removed or hidden ancestors cannot supply a live hover owner.
pub fn tooltip_owner(registry: &FrameRegistry, mut id: u64) -> Option<(&Frame, &TooltipBinding)> {
    loop {
        let frame = registry.get(id)?;
        if !frame.visible {
            return None;
        }
        if let Some(binding) = &frame.tooltip {
            if binding.has_content() {
                return Some((frame, binding));
            }
        }
        id = frame.parent_id?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attrs::apply_attribute;

    #[test]
    fn ui_template_behavior_child_art_resolves_the_nearest_declared_live_owner() {
        let mut registry = FrameRegistry::new(800.0, 600.0);
        let window = registry.create_frame("Window", None);
        let product = registry.create_frame("Product42", Some(window));
        let icon = registry.create_frame("Icon", Some(product));
        apply_attribute(&mut registry, window, "tooltip_title", "Window help");
        apply_attribute(&mut registry, product, "tooltip_provider", "product");
        let (owner, binding) = tooltip_owner(&registry, icon).unwrap();
        assert_eq!(owner.id, product);
        assert_eq!(binding.provider.as_deref(), Some("product"));
        registry.get_mut(product).unwrap().tooltip = None;
        let (owner, binding) = tooltip_owner(&registry, icon).unwrap();
        assert_eq!(owner.id, window);
        assert_eq!(binding.title.as_deref(), Some("Window help"));
    }

    #[test]
    fn ui_template_behavior_hidden_and_removed_owners_cannot_survive_hover_refresh() {
        let mut registry = FrameRegistry::new(800.0, 600.0);
        let owner = registry.create_frame("Owner", None);
        let icon = registry.create_frame("Icon", Some(owner));
        apply_attribute(&mut registry, owner, "tooltip_text", "Current content");
        registry.set_hidden(owner, true);
        assert!(tooltip_owner(&registry, icon).is_none());
        registry.set_hidden(owner, false);
        assert_eq!(tooltip_owner(&registry, icon).unwrap().0.id, owner);
        registry.remove_frame_tree(owner);
        assert!(tooltip_owner(&registry, icon).is_none());
    }
}
