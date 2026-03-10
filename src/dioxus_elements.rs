//! Dioxus-compatible element namespace for our WoW-style UI widgets.
//!
//! Structure mirrors dioxus-html: element modules live in `elements` sub-module,
//! then re-exported at the top level. The RSX macro resolves:
//! - TAG_NAME via `dioxus_elements::elements::tag::TAG_NAME`
//! - Attributes via `dioxus_elements::tag::attr_name.0` (name), `.1` (ns), `.2` (volatile)
//! - Completions via `dioxus_elements::elements::completions::CompleteWithBraces::*`
//!
//! Element module names are single lowercase words (no underscores — the RSX
//! parser treats underscored names as components). TAG_NAME is PascalCase
//! so the renderer's `tag_to_widget_type()` maps them correctly.

#![allow(non_upper_case_globals)]

use crate::frame::WidgetType;

/// Attribute descriptor: (name, namespace, volatile).
pub type AttributeDescription = (&'static str, Option<&'static str>, bool);

// -- Shared attributes available on all elements --

macro_rules! shared_attrs {
    () => {
        pub const name: super::super::AttributeDescription = ("name", None, false);
        pub const width: super::super::AttributeDescription = ("width", None, false);
        pub const height: super::super::AttributeDescription = ("height", None, false);
        pub const alpha: super::super::AttributeDescription = ("alpha", None, false);
        pub const shown: super::super::AttributeDescription = ("shown", None, false);
        pub const strata: super::super::AttributeDescription = ("strata", None, false);
        pub const mouse_enabled: super::super::AttributeDescription =
            ("mouse_enabled", None, false);
        pub const movable: super::super::AttributeDescription = ("movable", None, false);
        pub const background_color: super::super::AttributeDescription =
            ("background_color", None, false);
        pub const stretch: super::super::AttributeDescription = ("stretch", None, false);
        pub const draw_layer: super::super::AttributeDescription = ("draw_layer", None, false);
        pub const frame_level: super::super::AttributeDescription = ("frame_level", None, false);
        pub const nine_slice: super::super::AttributeDescription = ("nine_slice", None, false);
    };
}

macro_rules! define_element {
    ($mod_name:ident, $tag:expr) => {
        pub mod $mod_name {
            pub const TAG_NAME: &str = $tag;
            pub const NAME_SPACE: Option<&str> = None;
            shared_attrs!();
        }
    };
    ($mod_name:ident, $tag:expr, { $($attr:ident),* $(,)? }) => {
        pub mod $mod_name {
            pub const TAG_NAME: &str = $tag;
            pub const NAME_SPACE: Option<&str> = None;
            shared_attrs!();
            $(
                pub const $attr: super::super::AttributeDescription =
                    (stringify!($attr), None, false);
            )*
        }
    };
}

/// Element modules. RSX element names must be single lowercase words (no
/// underscores) because the Dioxus RSX parser reserves underscored names
/// for components.
pub mod elements {
    define_element!(r#frame, "Frame");

    define_element!(button, "Button", {
        text,
        font_size,
        button_atlas_up,
        button_atlas_pressed,
        button_atlas_highlight,
        button_atlas_disabled,
    });

    define_element!(checkbutton, "CheckButton", {
        text,
        font_size,
    });

    define_element!(texture, "Texture", {
        texture_file,
        texture_fdid,
        texture_atlas,
    });

    define_element!(fontstring, "FontString", {
        text,
        font,
        font_size,
        font_color,
        justify_h,
    });

    define_element!(editbox, "EditBox", {
        text,
        font,
        font_size,
        font_color,
        password,
    });

    define_element!(line, "Line");
    define_element!(scrollframe, "ScrollFrame");
    define_element!(slider, "Slider");
    define_element!(statusbar, "StatusBar");
    define_element!(cooldown, "Cooldown");
    define_element!(model, "Model");
    define_element!(playermodel, "PlayerModel");
    define_element!(modelscene, "ModelScene");
    define_element!(colorselect, "ColorSelect");
    define_element!(messageframe, "MessageFrame");
    define_element!(simplehtml, "SimpleHTML");
    define_element!(gametooltip, "GameTooltip");
    define_element!(minimap, "Minimap");

    /// Anchor pseudo-element — not a frame; intercepted by the renderer to apply
    /// anchor data to the parent frame.
    pub mod anchor {
        pub const TAG_NAME: &str = "Anchor";
        pub const NAME_SPACE: Option<&str> = None;
        pub const point: super::super::AttributeDescription = ("point", None, false);
        pub const relative_to: super::super::AttributeDescription = ("relative_to", None, false);
        pub const relative_point: super::super::AttributeDescription =
            ("relative_point", None, false);
        pub const x: super::super::AttributeDescription = ("x", None, false);
        pub const y: super::super::AttributeDescription = ("y", None, false);
    }

    /// Completions module for IDE autocompletion (referenced by RSX macro).
    #[doc(hidden)]
    pub mod completions {
        #[allow(non_camel_case_types)]
        pub enum CompleteWithBraces {
            r#frame {},
            button {},
            checkbutton {},
            texture {},
            fontstring {},
            editbox {},
            line {},
            scrollframe {},
            slider {},
            statusbar {},
            cooldown {},
            model {},
            playermodel {},
            modelscene {},
            colorselect {},
            messageframe {},
            simplehtml {},
            gametooltip {},
            minimap {},
            anchor {},
        }
    }
}

// Re-export all element modules at the top level (attributes resolve via dioxus_elements::tag::attr).
pub use elements::*;

/// Stub events module (RSX macro references dioxus_elements::events for event handlers).
pub mod events {}

/// Maps a Dioxus element tag name to our WidgetType.
pub fn tag_to_widget_type(tag: &str) -> Option<WidgetType> {
    match tag {
        "Frame" | "frame" => Some(WidgetType::Frame),
        "Button" | "button" => Some(WidgetType::Button),
        "CheckButton" => Some(WidgetType::CheckButton),
        "Texture" | "texture" => Some(WidgetType::Texture),
        "FontString" | "fontstring" | "label" => Some(WidgetType::FontString),
        "Line" | "line" => Some(WidgetType::Line),
        "EditBox" | "editbox" => Some(WidgetType::EditBox),
        "ScrollFrame" => Some(WidgetType::ScrollFrame),
        "Slider" | "slider" => Some(WidgetType::Slider),
        "StatusBar" => Some(WidgetType::StatusBar),
        "Cooldown" => Some(WidgetType::Cooldown),
        "Model" => Some(WidgetType::Model),
        "PlayerModel" => Some(WidgetType::PlayerModel),
        "ModelScene" => Some(WidgetType::ModelScene),
        "ColorSelect" => Some(WidgetType::ColorSelect),
        "MessageFrame" => Some(WidgetType::MessageFrame),
        "SimpleHTML" => Some(WidgetType::SimpleHTML),
        "GameTooltip" => Some(WidgetType::GameTooltip),
        "Minimap" => Some(WidgetType::Minimap),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::*;

    #[allow(unused_imports)]
    use crate::dioxus_elements;

    #[test]
    fn maps_all_pascal_case_tags() {
        assert_eq!(tag_to_widget_type("Frame"), Some(WidgetType::Frame));
        assert_eq!(tag_to_widget_type("Button"), Some(WidgetType::Button));
        assert_eq!(
            tag_to_widget_type("CheckButton"),
            Some(WidgetType::CheckButton)
        );
        assert_eq!(tag_to_widget_type("Texture"), Some(WidgetType::Texture));
        assert_eq!(
            tag_to_widget_type("FontString"),
            Some(WidgetType::FontString)
        );
        assert_eq!(tag_to_widget_type("Line"), Some(WidgetType::Line));
        assert_eq!(tag_to_widget_type("EditBox"), Some(WidgetType::EditBox));
        assert_eq!(
            tag_to_widget_type("ScrollFrame"),
            Some(WidgetType::ScrollFrame)
        );
        assert_eq!(tag_to_widget_type("Slider"), Some(WidgetType::Slider));
        assert_eq!(tag_to_widget_type("StatusBar"), Some(WidgetType::StatusBar));
        assert_eq!(tag_to_widget_type("Cooldown"), Some(WidgetType::Cooldown));
        assert_eq!(tag_to_widget_type("Model"), Some(WidgetType::Model));
        assert_eq!(
            tag_to_widget_type("PlayerModel"),
            Some(WidgetType::PlayerModel)
        );
        assert_eq!(
            tag_to_widget_type("ModelScene"),
            Some(WidgetType::ModelScene)
        );
        assert_eq!(
            tag_to_widget_type("ColorSelect"),
            Some(WidgetType::ColorSelect)
        );
        assert_eq!(
            tag_to_widget_type("MessageFrame"),
            Some(WidgetType::MessageFrame)
        );
        assert_eq!(
            tag_to_widget_type("SimpleHTML"),
            Some(WidgetType::SimpleHTML)
        );
        assert_eq!(
            tag_to_widget_type("GameTooltip"),
            Some(WidgetType::GameTooltip)
        );
        assert_eq!(tag_to_widget_type("Minimap"), Some(WidgetType::Minimap));
    }

    #[test]
    fn maps_lowercase_aliases() {
        assert_eq!(tag_to_widget_type("frame"), Some(WidgetType::Frame));
        assert_eq!(tag_to_widget_type("button"), Some(WidgetType::Button));
        assert_eq!(tag_to_widget_type("texture"), Some(WidgetType::Texture));
        assert_eq!(tag_to_widget_type("fontstring"), Some(WidgetType::FontString));
        assert_eq!(tag_to_widget_type("label"), Some(WidgetType::FontString));
        assert_eq!(tag_to_widget_type("line"), Some(WidgetType::Line));
        assert_eq!(tag_to_widget_type("editbox"), Some(WidgetType::EditBox));
        assert_eq!(tag_to_widget_type("slider"), Some(WidgetType::Slider));
    }

    #[test]
    fn unknown_tag_returns_none() {
        assert_eq!(tag_to_widget_type("div"), None);
        assert_eq!(tag_to_widget_type("span"), None);
        assert_eq!(tag_to_widget_type(""), None);
    }

    #[test]
    fn rsx_compiles_with_custom_frame() {
        fn test_component() -> Element {
            rsx! {
                r#frame { name: "TestRoot", width: 100.0, height: 50.0 }
            }
        }

        let mut dom = dioxus_core::VirtualDom::new(test_component);
        let mut registry = crate::registry::FrameRegistry::new(1024.0, 768.0);
        let mut renderer = crate::dioxus_renderer::GameUiRenderer::new();
        let mut applier =
            crate::dioxus_renderer::MutationApplier::new(&mut renderer, &mut registry);
        dom.rebuild(&mut applier);

        let root_id = registry
            .get_by_name("TestRoot")
            .expect("frame named TestRoot should exist");
        let frame = registry.get(root_id).unwrap();
        assert!((frame.width - 100.0).abs() < f32::EPSILON);
        assert!((frame.height - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn rsx_compiles_with_nested_elements() {
        fn test_component() -> Element {
            rsx! {
                r#frame { name: "Parent", width: 800.0, height: 600.0,
                    button {
                        name: "MyButton",
                        width: 200.0,
                        height: 32.0,
                        text: "Click me",
                    }
                    fontstring { name: "MyLabel", text: "Hello", font_size: 14.0 }
                    texture { name: "MyTex", texture_file: "test.blp" }
                    editbox {
                        name: "MyInput",
                        width: 250.0,
                        height: 32.0,
                        password: true,
                    }
                }
            }
        }

        let mut dom = dioxus_core::VirtualDom::new(test_component);
        let mut registry = crate::registry::FrameRegistry::new(1024.0, 768.0);
        let mut renderer = crate::dioxus_renderer::GameUiRenderer::new();
        let mut applier =
            crate::dioxus_renderer::MutationApplier::new(&mut renderer, &mut registry);
        dom.rebuild(&mut applier);

        assert!(registry.get_by_name("Parent").is_some());
        assert!(registry.get_by_name("MyButton").is_some());
        assert!(registry.get_by_name("MyLabel").is_some());
        assert!(registry.get_by_name("MyTex").is_some());
        assert!(registry.get_by_name("MyInput").is_some());

        let parent_id = registry.get_by_name("Parent").unwrap();
        let button_id = registry.get_by_name("MyButton").unwrap();
        let button = registry.get(button_id).unwrap();
        assert_eq!(button.parent_id, Some(parent_id));
    }
}
