/// Core types for the custom RSX widget definition tree.
/// These replace Dioxus's VNode/Template types with a simpler representation
/// that can be produced by both the compile-time rsx! macro and a runtime parser.

/// A widget definition — one UI element with attributes, anchors, and children.
pub struct WidgetDef {
    /// Tag name from compile-time macro (e.g. "Frame", "Button").
    pub tag: &'static str,
    /// Tag name from runtime parser (hot-reload). Takes precedence over `tag` when set.
    pub tag_owned: Option<String>,
    /// Frame name for registry lookup.
    pub name: Option<String>,
    /// Attributes to apply to the frame.
    pub attrs: Vec<Attr>,
    /// Anchor pseudo-elements (positioning, NOT children).
    pub anchors: Vec<AnchorDef>,
    /// Nine-slice backdrop definition (pseudo-element, NOT a child).
    pub nine_slice: Option<NineSliceDef>,
    /// Child widgets.
    pub children: Vec<WidgetChild>,
}

/// A single attribute on a widget.
pub struct Attr {
    /// Attribute name from compile-time macro.
    pub name: &'static str,
    /// Attribute name from runtime parser. Takes precedence over `name` when set.
    pub name_owned: Option<String>,
    /// The attribute value.
    pub value: AttrValue,
}

/// Attribute value — either a static literal (hot-reloadable) or a dynamic expression result.
pub enum AttrValue {
    /// Literal string value (hot-reloadable).
    Static(String),
    /// Value produced by evaluating an expression at runtime (not hot-reloadable).
    Dynamic(String),
}

/// A child in the widget tree.
pub enum WidgetChild {
    /// A concrete widget definition.
    Widget(WidgetDef),
    /// A fragment — multiple children from a sub-function or conditional.
    Fragment(Vec<WidgetChild>),
    /// Placeholder for runtime-opaque dynamic content.
    Dynamic,
}

/// Anchor positioning definition (pseudo-element, not a frame).
pub struct AnchorDef {
    pub point: String,
    pub relative_to: String,
    pub relative_point: String,
    pub x: String,
    pub y: String,
}

/// Declarative nine-slice definition for RSX (converted to `NineSlice` at frame create).
#[derive(Debug, Clone)]
pub struct NineSliceDef {
    pub edge_size: f32,
    pub bg_color: [f32; 4],
    pub border_color: [f32; 4],
    /// Per-part texture paths in TL,T,TR,L,M,R,BL,B,BR order.
    pub textures: Option<[String; 9]>,
}

/// Return type alias for RSX functions.
pub type Element = Vec<WidgetChild>;

impl WidgetDef {
    pub fn new(tag: &'static str) -> Self {
        Self {
            tag,
            tag_owned: None,
            name: None,
            attrs: Vec::new(),
            anchors: Vec::new(),
            nine_slice: None,
            children: Vec::new(),
        }
    }

    /// Get the effective tag name (owned takes precedence).
    pub fn effective_tag(&self) -> &str {
        self.tag_owned.as_deref().unwrap_or(self.tag)
    }
}

impl Attr {
    pub fn new_static(name: &'static str, value: String) -> Self {
        Self {
            name,
            name_owned: None,
            value: AttrValue::Static(value),
        }
    }

    pub fn new_dynamic(name: &'static str, value: String) -> Self {
        Self {
            name,
            name_owned: None,
            value: AttrValue::Dynamic(value),
        }
    }

    /// Get the effective attribute name (owned takes precedence).
    pub fn effective_name(&self) -> &str {
        self.name_owned.as_deref().unwrap_or(self.name)
    }

    /// Get the value as a string regardless of variant.
    pub fn value_str(&self) -> &str {
        match &self.value {
            AttrValue::Static(s) | AttrValue::Dynamic(s) => s,
        }
    }
}

impl Default for AnchorDef {
    fn default() -> Self {
        Self {
            point: "CENTER".to_string(),
            relative_to: "$parent".to_string(),
            relative_point: "CENTER".to_string(),
            x: "0".to_string(),
            y: "0".to_string(),
        }
    }
}
