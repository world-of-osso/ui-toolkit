pub mod button;
pub mod cooldown;
pub mod edit_box;
pub mod font_string;
pub mod scroll_list;
pub mod slider;
pub mod state_panel;
pub mod tabs;
pub mod texture;
pub mod toggle;
pub mod tooltip;

use crate::widget_def::{Attr, Element, WidgetChild};

/// Runtime frame name for `rsx!` `name:` attributes.
pub(crate) struct DynName(pub String);

/// Append an attribute to the first widget of `element`, for attributes that are
/// present only when the caller supplies art.
pub(crate) fn with_attr(mut element: Element, name: &'static str, value: String) -> Element {
    if let Some(WidgetChild::Widget(def)) = element.first_mut() {
        def.attrs.push(Attr::new_dynamic(name, value));
    }
    element
}
