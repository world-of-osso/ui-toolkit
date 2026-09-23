use super::{DynName, with_attr};
use crate::rsx;
use crate::widget_def::Element;

#[derive(Clone, Copy)]
pub struct Tab<'a> {
    pub label: &'a str,
    /// Onclick action emitted when the tab is clicked.
    pub action: &'a str,
    pub disabled: bool,
}

/// Button atlases for idle and selected tabs. Both are required so a tab that
/// changes selection never keeps the other state's art.
#[derive(Clone, Copy)]
pub struct TabArt<'a> {
    pub idle_atlas: &'a str,
    pub selected_atlas: &'a str,
}

#[derive(Clone, Copy)]
pub struct TabStrip<'a> {
    pub name: &'a str,
    pub tabs: &'a [Tab<'a>],
    pub selected: usize,
    pub tab_width: f32,
    pub tab_height: f32,
    pub gap: f32,
    pub art: Option<TabArt<'a>>,
}

pub fn tab_name(strip: &str, index: usize) -> String {
    format!("{strip}Tab{index}")
}

/// A row of tab buttons named [`tab_name`]. Enabled tabs carry their action as
/// onclick; disabled tabs carry an empty onclick so no stale action survives a rebuild.
pub fn tab_strip(spec: TabStrip<'_>) -> Element {
    let tabs: Element = spec
        .tabs
        .iter()
        .enumerate()
        .flat_map(|(index, tab)| tab_button(&spec, index, tab))
        .collect();
    rsx! {
        r#frame {
            name: DynName(spec.name.to_string()),
            layout: "flex-row",
            gap: {spec.gap},
            {tabs}
        }
    }
}

fn tab_button(spec: &TabStrip<'_>, index: usize, tab: &Tab<'_>) -> Element {
    let action = if tab.disabled { "" } else { tab.action };
    let button = rsx! {
        button {
            name: DynName(tab_name(spec.name, index)),
            width: {spec.tab_width},
            height: {spec.tab_height},
            text: {tab.label},
            onclick: {action},
            disabled: {tab.disabled},
        }
    };
    let Some(art) = spec.art else {
        return button;
    };
    let atlas = if index == spec.selected {
        art.selected_atlas
    } else {
        art.idle_atlas
    };
    with_attr(button, "button_atlas_up", atlas.to_string())
}

#[cfg(test)]
#[path = "tabs_tests.rs"]
mod tests;
