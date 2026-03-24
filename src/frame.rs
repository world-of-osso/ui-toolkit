use crate::anchor::Anchor;
use crate::layout::LayoutRect;
use crate::strata::{DrawLayer, FrameStrata};
use crate::widgets::button::ButtonData;
use crate::widgets::edit_box::EditBoxData;
use crate::widgets::font_string::FontStringData;
use crate::widgets::slider::StatusBarData;
use crate::widgets::texture::{TextureData, TextureSource};

/// Per-widget-type data attached to a frame.
#[derive(Debug, Clone)]
pub enum WidgetData {
    FontString(FontStringData),
    EditBox(EditBoxData),
    Button(ButtonData),
    Texture(TextureData),
    StatusBar(StatusBarData),
}

/// WoW widget types corresponding to frame XML element names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WidgetType {
    #[default]
    Frame,
    Button,
    CheckButton,
    Texture,
    FontString,
    Line,
    EditBox,
    ScrollFrame,
    Slider,
    Panel,
    StatusBar,
    Cooldown,
    Model,
    PlayerModel,
    ModelScene,
    ColorSelect,
    MessageFrame,
    SimpleHTML,
    GameTooltip,
    Minimap,
}

/// Nine-slice frame rendering (solid color corners/edges/center, or textured).
#[derive(Debug, Clone)]
pub struct NineSlice {
    pub edge_size: f32,
    /// Vertical edge size (top/bottom). Falls back to `edge_size` when `None`.
    pub edge_size_v: Option<f32>,
    /// Optional per-side edge sizes in screen space: `[left, top, right, bottom]`.
    pub edge_sizes: Option<[f32; 4]>,
    /// Edge size in texture pixel space for UV sampling. Falls back to `edge_size` when `None`.
    pub uv_edge_size: Option<f32>,
    /// Optional per-side edge sizes in texture space: `[left, top, right, bottom]`.
    pub uv_edge_sizes: Option<[f32; 4]>,
    pub bg_color: [f32; 4],
    pub border_color: [f32; 4],
    /// Optional texture applied to all 9 parts with UV sub-rects.
    pub texture: Option<TextureSource>,
    /// Optional per-part textures in TL,T,TR,L,C,R,BL,B,BR order.
    pub part_textures: Option<[TextureSource; 9]>,
    /// Optional normalized UV rects per part: [left, right, top, bottom].
    pub uv_rects: Option<[[f32; 4]; 9]>,
}

impl Default for NineSlice {
    fn default() -> Self {
        Self {
            edge_size: 4.0,
            edge_size_v: None,
            edge_sizes: None,
            uv_edge_size: None,
            uv_edge_sizes: None,
            bg_color: [0.0, 0.0, 0.0, 0.8],
            border_color: [1.0, 1.0, 1.0, 1.0],
            texture: None,
            part_textures: None,
            uv_rects: None,
        }
    }
}

/// CSS-like border for a frame (4 solid-color edge sprites).
#[derive(Debug, Clone, PartialEq)]
pub struct Border {
    pub width: f32,
    pub color: [f32; 4],
}

/// Backdrop decoration for a frame (background fill + border).
#[derive(Debug, Clone)]
pub struct Backdrop {
    pub bg_color: Option<[f32; 4]>,
    pub border_color: Option<[f32; 4]>,
    pub edge_size: f32,
    pub insets: [f32; 4], // left, right, top, bottom
}

impl Default for Backdrop {
    fn default() -> Self {
        Self {
            bg_color: None,
            border_color: None,
            edge_size: 1.0,
            insets: [0.0; 4],
        }
    }
}

/// Sizing mode for a frame dimension.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Fixed(f32),
    Fill,
}

impl Default for Dimension {
    fn default() -> Self {
        Self::Fixed(0.0)
    }
}

impl Dimension {
    /// Returns the explicit size, or 0.0 for Fill (resolved later by layout).
    pub fn value(self) -> f32 {
        match self {
            Self::Fixed(v) => v,
            Self::Fill => 0.0,
        }
    }

    pub fn is_fill(self) -> bool {
        matches!(self, Self::Fill)
    }
}

/// Flex layout direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexDirection {
    #[default]
    Column,
    Row,
    RowWrap,
}

/// Alignment along the cross axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexAlign {
    Start,
    #[default]
    Center,
    End,
    Stretch,
}

/// Justification along the main axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexJustify {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
}

/// Flex layout mode for a container frame.
#[derive(Debug, Clone, Default)]
pub struct FlexLayout {
    pub direction: FlexDirection,
    pub gap: f32,
    pub justify: FlexJustify,
    pub align: FlexAlign,
    pub padding: f32,
}

/// A UI frame in the WoW frame hierarchy.
#[derive(Default)]
pub struct Frame {
    pub id: u64,
    pub name: Option<String>,
    pub widget_type: WidgetType,

    // Hierarchy
    pub parent_id: Option<u64>,
    pub children: Vec<u64>,

    // Layout
    pub width: Dimension,
    pub height: Dimension,
    pub anchors: Vec<Anchor>,
    pub layout_rect: Option<LayoutRect>,

    // Visibility
    pub hidden: bool,
    pub visible: bool,

    // Alpha
    pub alpha: f32,
    pub effective_alpha: f32,

    // Scale
    pub scale: f32,
    pub effective_scale: f32,

    // Strata and layering
    pub strata: FrameStrata,
    pub frame_level: i32,
    pub raise_order: i32,
    pub draw_layer: DrawLayer,
    pub draw_sub_layer: i32,

    // Input
    pub mouse_enabled: bool,
    pub keyboard_enabled: bool,
    pub hit_rect_insets: [f32; 4],

    // Appearance
    pub background_color: Option<[f32; 4]>,
    pub backdrop: Option<Backdrop>,
    pub nine_slice: Option<NineSlice>,
    pub border: Option<Border>,
    /// Panel style name (for Panel widget type). Resolved via FrameRegistry::panel_styles.
    pub panel_style: Option<String>,

    // Behavior
    pub clamped_to_screen: bool,
    pub movable: bool,
    pub resizable: bool,

    // Events
    pub onclick: Option<String>,

    // Layout mode
    pub flex_layout: Option<FlexLayout>,

    // Widget-specific data
    pub widget_data: Option<WidgetData>,
}

impl Frame {
    pub fn new(id: u64, name: Option<String>, widget_type: WidgetType) -> Self {
        Self {
            id,
            name,
            widget_type,
            visible: true,
            alpha: 1.0,
            effective_alpha: 1.0,
            scale: 1.0,
            effective_scale: 1.0,
            mouse_enabled: false,
            ..Self::default()
        }
    }

    /// Resolved width: from layout_rect if available, otherwise from the dimension spec.
    pub fn resolved_width(&self) -> f32 {
        self.layout_rect
            .as_ref()
            .map_or(self.width.value(), |r| r.width)
    }

    /// Resolved height: from layout_rect if available, otherwise from the dimension spec.
    pub fn resolved_height(&self) -> f32 {
        self.layout_rect
            .as_ref()
            .map_or(self.height.value(), |r| r.height)
    }

    pub fn is_editbox(&self) -> bool {
        matches!(self.widget_data, Some(WidgetData::EditBox(_)))
    }

    #[cfg(test)]
    pub fn default_for_test() -> Self {
        Self::new(0, None, WidgetType::Frame)
    }
}
