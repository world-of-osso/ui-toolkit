//! Distinct Retail PanelTab/PanelTopTab and TabSystem/TabSystemTop art layouts.
use crate::art_layout::{Region, region_element};
use crate::widget_def::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabLayout {
    Panel,
    PanelTop,
    System,
    SystemTop,
}

pub struct TabArtSpec {
    pub layout: TabLayout,
    pub selected: bool,
    pub size: [f32; 2],
    pub scale: f32,
    /// Source-left, middle, source-right. Top System tabs rotate caps onto opposite sides.
    pub names: [&'static str; 3],
}

impl TabArtSpec {
    pub fn new(layout: TabLayout, selected: bool, size: [f32; 2]) -> Self {
        Self {
            layout,
            selected,
            size,
            scale: 1.0,
            names: ["Left", "Middle", "Right"],
        }
    }
}

pub fn tab_art(prefix: &str, spec: TabArtSpec) -> Element {
    let family = if spec.selected { "activetab" } else { "tab" };
    let atlas_names = [
        format!("uiframe-{family}-left"),
        format!("_uiframe-{family}-center"),
        format!("uiframe-{family}-right"),
    ];
    let [left, middle, right] = tab_rects(spec.layout, spec.selected, spec.size);
    let regions = atlas_names
        .into_iter()
        .zip([left, middle, right])
        .map(|(name, rect)| {
            let mut region = Region::atlas(&name, rect);
            region.layer = "BACKGROUND";
            match spec.layout {
                TabLayout::PanelTop => region.crop = [0.0, 1.0, 1.0, 0.25],
                TabLayout::SystemTop => region.rotation = std::f32::consts::PI,
                _ => {}
            }
            region
        });
    spec.names
        .into_iter()
        .zip(regions)
        .flat_map(|(suffix, region)| {
            region_element(format!("{prefix}{suffix}"), region, spec.scale)
        })
        .collect()
}

fn tab_rects(layout: TabLayout, selected: bool, [width, height]: [f32; 2]) -> [[f32; 4]; 3] {
    let native_height = if selected { 42.0 } else { 36.0 };
    match layout {
        TabLayout::SystemTop => {
            let right_x = if selected { -7.0 } else { -6.0 };
            let middle_x = right_x + 37.0;
            let left_x = width - 35.0;
            let y = height - native_height;
            [
                [left_x, y, 35.0, native_height],
                [middle_x, y, left_x - middle_x, native_height],
                [right_x, y, 37.0, native_height],
            ]
        }
        TabLayout::Panel | TabLayout::PanelTop | TabLayout::System => {
            let (left_x, right_offset) = match layout {
                TabLayout::System => (0.0, if selected { 7.0 } else { 6.0 }),
                _ => {
                    if selected {
                        (-1.0, 8.0)
                    } else {
                        (-3.0, 7.0)
                    }
                }
            };
            let (y, art_height) = if layout == TabLayout::PanelTop {
                (height - native_height * 0.75, native_height * 0.75)
            } else {
                (0.0, native_height)
            };
            let right_x = width + right_offset - 37.0;
            [
                [left_x, y, 35.0, art_height],
                [left_x + 35.0, y, right_x - left_x - 35.0, art_height],
                [right_x, y, 37.0, art_height],
            ]
        }
    }
}

pub struct TabWidth {
    pub padding: f32,
    pub minimum: Option<f32>,
    pub maximum: Option<f32>,
}

/// PanelTemplates_TabResize and TabSystemButtonMixin:UpdateTabWidth retain distinct formulas.
pub fn measured_tab_width(layout: TabLayout, text: &str, constraints: TabWidth) -> f32 {
    let font = crate::font_object::resolve_font_object("GameFontNormalSmall");
    let (text_width, _) = crate::text_measure::measure_text(text, font.font, font.font_size)
        .expect("named tab font metrics");
    let (width, default_minimum) = match layout {
        TabLayout::Panel | TabLayout::PanelTop => (text_width + 20.0 + constraints.padding, 72.0),
        TabLayout::System | TabLayout::SystemTop => {
            let text_width = text_width + constraints.padding;
            (
                if 92.0 < text_width {
                    text_width + 10.0
                } else {
                    92.0
                },
                92.0,
            )
        }
    };
    let width = width.max(constraints.minimum.unwrap_or(default_minimum));
    constraints
        .maximum
        .map_or(width, |maximum| width.min(maximum))
}

/// Source GetTextYOffset, in WoW y-up coordinates.
pub fn text_y_offset(layout: TabLayout, selected: bool) -> f32 {
    match (layout, selected) {
        (TabLayout::PanelTop, true) => -4.0,
        (TabLayout::PanelTop, false) => -8.0,
        (TabLayout::SystemTop, true) => 0.0,
        (TabLayout::SystemTop, false) => -3.0,
        (_, true) => -3.0,
        (_, false) => 2.0,
    }
}
