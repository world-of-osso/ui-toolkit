use bevy::prelude::*;

use super::DynName;
use crate::frame::WidgetData;
use crate::plugin::UiState;
use crate::registry::FrameRegistry;
use crate::rsx;
use crate::widget_def::Element;

pub const LOADING_TEXT: &str = "Loading";
pub const UNAVAILABLE_TEXT: &str = "Not available yet";
pub const RETRY_TEXT: &str = "Retry";
const DOT_PERIOD_SECS: f32 = 0.4;

/// Non-content state of a data-backed region.
#[derive(Clone, Copy, Debug)]
pub enum PanelState<'a> {
    Loading {
        label: Option<&'a str>,
    },
    Empty {
        message: &'a str,
    },
    Error {
        message: &'a str,
        retry_action: Option<&'a str>,
    },
    Unavailable {
        message: Option<&'a str>,
    },
}

pub fn text_name(panel: &str) -> String {
    format!("{panel}Text")
}

pub fn retry_name(panel: &str) -> String {
    format!("{panel}Retry")
}

/// A panel filling its parent that shows `state` in place of the region's content.
pub fn state_panel(name: &str, state: PanelState<'_>) -> Element {
    let (message, loading) = match state {
        PanelState::Loading { label } => {
            let label = label.unwrap_or(LOADING_TEXT);
            (label, label)
        }
        PanelState::Empty { message } | PanelState::Error { message, .. } => (message, ""),
        PanelState::Unavailable { message } => (message.unwrap_or(UNAVAILABLE_TEXT), ""),
    };
    let retry = match state {
        PanelState::Error {
            retry_action: Some(action),
            ..
        } => retry_button(name, action),
        _ => Vec::new(),
    };
    rsx! {
        r#frame {
            name: DynName(name.to_string()),
            stretch: true,
            fontstring {
                name: DynName(text_name(name)),
                text: {message},
                loading_text: {loading},
                justify_h: "CENTER",
                height: 20,
                pos_type: "absolute",
                left: 0,
                right: 0,
                top: "50%",
                translate_y: "-50%",
            }
            {retry}
        }
    }
}

fn retry_button(panel: &str, action: &str) -> Element {
    rsx! {
        button {
            name: DynName(retry_name(panel)),
            width: 100,
            height: 24,
            text: RETRY_TEXT,
            onclick: {action},
            pos_type: "absolute",
            left: "50%",
            top: "50%",
            translate_x: "-50%",
            margin_top: 20,
        }
    }
}

/// Loading text at `elapsed_secs`: the base followed by zero to three cycling dots.
pub fn loading_text_at(base: &str, elapsed_secs: f32) -> String {
    let dots = (elapsed_secs / DOT_PERIOD_SECS) as usize % 4;
    format!("{base}{}", ".".repeat(dots))
}

/// Advance every loading fontstring to its text at `elapsed_secs`.
pub fn animate_loading_texts_at(registry: &mut FrameRegistry, elapsed_secs: f32) {
    let updates: Vec<(u64, String)> = registry
        .loading_texts
        .iter()
        .filter_map(|(&id, base)| {
            let text = loading_text_at(base, elapsed_secs);
            let Some(WidgetData::FontString(fs)) = registry.get(id)?.widget_data.as_ref() else {
                return None;
            };
            (fs.text != text).then_some((id, text))
        })
        .collect();
    for (id, text) in updates {
        if let Some(WidgetData::FontString(fs)) =
            registry.get_mut(id).and_then(|f| f.widget_data.as_mut())
        {
            fs.text = text;
        }
    }
}

pub fn animate_loading_texts(mut ui: ResMut<UiState>, time: Res<Time>) {
    if ui.registry.loading_texts.is_empty() {
        return;
    }
    let elapsed = time.elapsed_secs();
    animate_loading_texts_at(&mut ui.registry, elapsed);
}

#[cfg(test)]
#[path = "state_panel_tests.rs"]
mod tests;
