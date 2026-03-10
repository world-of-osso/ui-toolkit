pub mod anchor;
pub mod animation;
pub mod atlas;
mod dioxus_anchor;
pub mod dioxus_attrs;
pub mod dioxus_elements;
pub(crate) mod dioxus_hotreload_diff;
#[cfg(test)]
mod dioxus_hotreload_diff_tests;
pub mod dioxus_renderer;
mod dioxus_renderer_anchor;
mod dioxus_renderer_tree;
pub mod dioxus_screen;
pub mod event;
pub mod plugin;
pub mod font_registry;
pub mod frame;
pub mod input;
pub mod layout;
pub mod registry;
pub mod render;
pub mod render_border;
pub mod render_button;
pub mod render_nine_slice;
pub mod render_text;
pub mod render_text_fx;
pub mod render_texture;
pub mod render_tiled;
pub mod strata;
pub mod text_measure;
pub mod widgets;

#[cfg(test)]
mod panel_tests;
#[cfg(test)]
mod render_tests;

#[cfg(test)]
mod dioxus_renderer_tests;
