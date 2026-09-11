use bevy::ui::{PositionType, UiRect, Val2};

use crate::anchor::AnchorTarget;
use crate::layout::LayoutRect;
use crate::strata::{DrawLayer, FrameStrata};
use crate::widgets::button::ButtonData;
use crate::widgets::edit_box::EditBoxData;
use crate::widgets::font_string::FontStringData;
use crate::widgets::slider::{SliderData, StatusBarData};
use crate::widgets::texture::{TextureData, TextureSource};

/// Per-widget-type data attached to a frame.
#[derive(Debug, Clone)]
pub enum WidgetData {
    FontString(FontStringData),
    EditBox(EditBoxData),
    Button(ButtonData),
    Texture(TextureData),
    Slider(SliderData),
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
#[derive(Debug, Clone, PartialEq)]
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

/// Horizontal three-slice frame rendering (left cap, center stretch, right cap).
#[derive(Debug, Clone)]
pub struct ThreeSlice {
    pub cap_width: f32,
    pub left: TextureSource,
    pub center: TextureSource,
    pub right: TextureSource,
    pub color: [f32; 4],
}

impl Default for ThreeSlice {
    fn default() -> Self {
        Self {
            cap_width: 8.0,
            left: TextureSource::None,
            center: TextureSource::None,
            right: TextureSource::None,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
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
    Auto,
}

impl Default for Dimension {
    fn default() -> Self {
        Self::Fixed(0.0)
    }
}

impl Dimension {
    /// Returns the explicit size, or 0.0 for an unresolved Fill/Auto dimension.
    pub fn value(self) -> f32 {
        match self {
            Self::Fixed(v) => v,
            Self::Fill | Self::Auto => 0.0,
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

/// A UI frame in the logical hierarchy, with authored native layout properties.
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
    pub position: UiRect,
    pub position_type: PositionType,
    pub anchor: AnchorTarget,
    pub translation: Val2,
    pub margin: UiRect,
    /// Observational Bevy layout result; never an authored positioning input.
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
    pub three_slice: Option<ThreeSlice>,
    pub border: Option<Border>,
    /// Panel style name (for Panel widget type). Resolved via FrameRegistry::panel_styles.
    pub panel_style: Option<String>,
    /// Three-slice style name. Resolved via FrameRegistry::three_slice_styles.
    pub three_slice_style: Option<String>,

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

impl Default for Frame {
    fn default() -> Self {
        Self {
            id: 0,
            name: None,
            widget_type: WidgetType::default(),
            parent_id: None,
            children: Vec::new(),
            width: Dimension::default(),
            height: Dimension::default(),
            position: UiRect::AUTO,
            position_type: PositionType::Relative,
            anchor: AnchorTarget::Parent,
            translation: Val2::ZERO,
            margin: UiRect::ZERO,
            layout_rect: None,
            hidden: false,
            visible: false,
            alpha: 0.0,
            effective_alpha: 0.0,
            scale: 0.0,
            effective_scale: 0.0,
            strata: FrameStrata::default(),
            frame_level: 0,
            raise_order: 0,
            draw_layer: DrawLayer::default(),
            draw_sub_layer: 0,
            mouse_enabled: false,
            keyboard_enabled: false,
            hit_rect_insets: [0.0; 4],
            background_color: None,
            backdrop: None,
            nine_slice: None,
            three_slice: None,
            border: None,
            panel_style: None,
            three_slice_style: None,
            clamped_to_screen: false,
            movable: false,
            resizable: false,
            onclick: None,
            flex_layout: None,
            widget_data: None,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layout_has_auto_edges_and_zero_explicit_dimensions() {
        for frame in [
            Frame::default(),
            Frame::new(7, Some("Panel".into()), WidgetType::Frame),
        ] {
            assert_eq!(frame.position, UiRect::AUTO);
            assert_eq!(frame.position_type, PositionType::Relative);
            assert_eq!(frame.anchor, AnchorTarget::Parent);
            assert_eq!(frame.translation, Val2::ZERO);
            assert_eq!(frame.margin, UiRect::ZERO);
            assert_eq!(frame.width, Dimension::Fixed(0.0));
            assert_eq!(frame.height, Dimension::Fixed(0.0));
            assert!(frame.layout_rect.is_none());
        }
    }

    #[test]
    fn auto_dimension_is_explicit_and_resolves_from_computed_layout() {
        assert_eq!(Dimension::default(), Dimension::Fixed(0.0));
        assert_eq!(Dimension::Auto.value(), 0.0);
        assert_eq!(Dimension::Fill.value(), 0.0);
        assert_eq!(Dimension::Fixed(32.0).value(), 32.0);
        assert!(!Dimension::Auto.is_fill());
        let mut frame = Frame::new(1, None, WidgetType::Frame);
        frame.width = Dimension::Auto;
        frame.height = Dimension::Auto;
        assert_eq!(
            (frame.resolved_width(), frame.resolved_height()),
            (0.0, 0.0)
        );
        frame.layout_rect = Some(LayoutRect {
            x: 1.0,
            y: 2.0,
            width: 60.0,
            height: 24.0,
        });
        assert_eq!(
            (frame.resolved_width(), frame.resolved_height()),
            (60.0, 24.0)
        );
        assert_eq!(
            (frame.width, frame.height),
            (Dimension::Auto, Dimension::Auto)
        );
    }
}
