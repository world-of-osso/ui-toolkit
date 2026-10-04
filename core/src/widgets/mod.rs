pub mod button;
pub mod edit_box;
pub mod font_string;
pub mod scroll_list;
pub mod slider;
pub mod texture;
#[path = "../../../src/widgets/toggle.rs"]
pub mod toggle;
use crate::widget_def::{Attr, Element, WidgetChild};
pub(crate) struct DynName(pub String);
pub(crate) fn with_attr(mut element: Element, name: &'static str, value: String) -> Element {
    if let Some(WidgetChild::Widget(def)) = element.first_mut() {
        def.attrs.push(Attr::new_dynamic(name, value));
    }
    element
}
