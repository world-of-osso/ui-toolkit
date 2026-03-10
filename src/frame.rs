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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WidgetType {
    Frame,
    Button,
    CheckButton,
    Texture,
    FontString,
    Line,
    EditBox,
    ScrollFrame,
    Slider,
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
    /// Edge size in texture pixel space for UV sampling. Falls back to `edge_size` when `None`.
    pub uv_edge_size: Option<f32>,
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
            uv_edge_size: None,
            bg_color: [0.0, 0.0, 0.0, 0.8],
            border_color: [1.0, 1.0, 1.0, 1.0],
            texture: None,
            part_textures: None,
            uv_rects: None,
        }
    }
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

/// A UI frame in the WoW frame hierarchy.
pub struct Frame {
    pub id: u64,
    pub name: Option<String>,
    pub widget_type: WidgetType,

    // Hierarchy
    pub parent_id: Option<u64>,
    pub children: Vec<u64>,

    // Layout
    pub width: f32,
    pub height: f32,
    pub anchors: Vec<Anchor>,
    pub layout_rect: Option<LayoutRect>,

    // Visibility
    pub shown: bool,
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

    // Behavior
    pub clamped_to_screen: bool,
    pub movable: bool,
    pub resizable: bool,

    // Widget-specific data
    pub widget_data: Option<WidgetData>,
}

impl Frame {
    pub fn new(id: u64, name: Option<String>, widget_type: WidgetType) -> Self {
        Self {
            id,
            name,
            widget_type,
            parent_id: None,
            children: Vec::new(),
            width: 0.0,
            height: 0.0,
            anchors: Vec::new(),
            layout_rect: None,
            shown: true,
            visible: true,
            alpha: 1.0,
            effective_alpha: 1.0,
            scale: 1.0,
            effective_scale: 1.0,
            strata: FrameStrata::default(),
            frame_level: 0,
            raise_order: 0,
            draw_layer: DrawLayer::default(),
            draw_sub_layer: 0,
            mouse_enabled: true,
            keyboard_enabled: false,
            hit_rect_insets: [0.0; 4],
            background_color: None,
            backdrop: None,
            nine_slice: None,
            clamped_to_screen: false,
            movable: false,
            resizable: false,
            widget_data: None,
        }
    }

    #[cfg(test)]
    pub fn default_for_test() -> Self {
        Self::new(0, None, WidgetType::Frame)
    }
}
