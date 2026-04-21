extern crate self as ui_toolkit;

pub mod anchor;
pub mod anchor_resolve;
pub mod animation;
pub mod atlas;
pub mod attrs;
pub mod button_input;
pub mod event;
pub mod font_registry;
pub mod frame;
pub mod hotreload;
pub mod input;
pub mod layout;
pub mod panel_style;
pub mod plugin;
pub mod registry;
pub mod render;
pub mod render_border;
pub mod render_button;
pub mod render_nine_slice;
pub mod render_text;
pub mod render_text_fx;
pub mod render_texture;
pub mod render_three_slice;
pub mod render_tiled;
pub mod screen;
pub mod strata;
pub mod text_measure;
pub mod widget_def;
pub mod widget_def_diff;
pub mod widgets;

pub use ui_toolkit_macros::rsx;

#[cfg(test)]
mod layout_tests;
#[cfg(test)]
mod panel_tests;
#[cfg(test)]
mod render_tests;
