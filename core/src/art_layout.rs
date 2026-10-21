//! Source-derived named artwork expanded into existing texture declarations.
use crate::rsx;
use crate::widget_def::{Attr, Element, WidgetChild};
use crate::widgets::texture::TextureSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderLayout {
    Inset,
    Dialog,
    Translucent,
}

/// Part order: top-left, top-right, bottom-left, bottom-right, top, bottom, left, right.
pub struct BorderSpec {
    pub layout: BorderLayout,
    pub rect: [f32; 4],
    pub scale: f32,
    pub names: [&'static str; 8],
}

impl BorderSpec {
    pub fn new(layout: BorderLayout, rect: [f32; 4]) -> Self {
        let names = match layout {
            BorderLayout::Inset => [
                "TopLeft",
                "TopRight",
                "BottomLeft",
                "BottomRight",
                "TopEdge",
                "BottomEdge",
                "LeftEdge",
                "RightEdge",
            ],
            BorderLayout::Dialog => [
                "TopLeft",
                "TopRight",
                "BottomLeft",
                "BottomRight",
                "Top",
                "Bottom",
                "Left",
                "Right",
            ],
            BorderLayout::Translucent => [
                "TopLeftCorner",
                "TopRightCorner",
                "BottomLeftCorner",
                "BottomRightCorner",
                "TopBorder",
                "BottomBorder",
                "LeftBorder",
                "RightBorder",
            ],
        };
        Self {
            layout,
            rect,
            scale: 1.0,
            names,
        }
    }
}

struct Region {
    source: TextureSource,
    rect: [f32; 4],
    crop: [f32; 4],
    tile: [bool; 2],
}

impl Region {
    fn atlas(name: &str, rect: [f32; 4]) -> Self {
        let atlas = crate::atlas::get_region(name)
            .unwrap_or_else(|| panic!("missing named artwork atlas {name}"));
        Self {
            source: TextureSource::Atlas(name.into()),
            rect,
            crop: [0.0, 1.0, 0.0, 1.0],
            tile: [atlas.tiles_horizontally, atlas.tiles_vertically],
        }
    }
}

/// NineSlice.lua corner anchors and adjacent-edge endpoints; no source-name dispatch in hosts.
pub fn border_art(prefix: &str, spec: BorderSpec) -> Element {
    let regions = match spec.layout {
        BorderLayout::Inset => inset_regions(spec.rect),
        BorderLayout::Dialog => dialog_regions(spec.rect),
        BorderLayout::Translucent => translucent_regions(spec.rect),
    };
    let mut elements = Vec::new();
    if spec.layout == BorderLayout::Translucent {
        let [x, y, width, height] = spec.rect;
        let background = Region {
            source: TextureSource::SolidColor([0.0, 0.0, 0.0, 0.9]),
            rect: [x + 10.0, y + 10.0, width - 20.0, height - 20.0],
            crop: [0.0, 1.0, 0.0, 1.0],
            tile: [false; 2],
        };
        elements.extend(region_element(
            format!("{prefix}Bg"),
            background,
            spec.scale,
        ));
    }
    for (name, region) in spec.names.into_iter().zip(regions) {
        elements.extend(region_element(
            format!("{prefix}{name}"),
            region,
            spec.scale,
        ));
    }
    elements
}

fn inset_regions([x, y, width, height]: [f32; 4]) -> [Region; 8] {
    // NineSliceLayouts.lua81–91:6px corners,3px edges, bottom corners y−1.
    [
        Region::atlas("UI-Frame-InnerTopLeft", [x, y, 6.0, 6.0]),
        Region::atlas("UI-Frame-InnerTopRight", [x + width - 6.0, y, 6.0, 6.0]),
        Region::atlas(
            "UI-Frame-InnerBotLeftCorner",
            [x, y + height - 5.0, 6.0, 6.0],
        ),
        Region::atlas(
            "UI-Frame-InnerBotRight",
            [x + width - 6.0, y + height - 5.0, 6.0, 6.0],
        ),
        Region::atlas("_UI-Frame-InnerTopTile", [x + 6.0, y, width - 12.0, 3.0]),
        Region::atlas(
            "_UI-Frame-InnerBotTile",
            [x + 6.0, y + height - 2.0, width - 12.0, 3.0],
        ),
        Region::atlas("!UI-Frame-InnerLeftTile", [x, y + 6.0, 3.0, height - 11.0]),
        Region::atlas(
            "!UI-Frame-InnerRightTile",
            [x + width - 3.0, y + 6.0, 3.0, height - 11.0],
        ),
    ]
}

fn dialog_regions([x, y, width, height]: [f32; 4]) -> [Region; 8] {
    // DiamondMetal native corners64px. Short dialogs cross the side-edge anchors.
    let side_top = (height - 64.0).min(64.0);
    let side_height = (height - 128.0).abs();
    [
        Region::atlas("UI-Frame-DiamondMetal-CornerTopLeft", [x, y, 64.0, 64.0]),
        Region::atlas(
            "UI-Frame-DiamondMetal-CornerTopRight",
            [x + width - 64.0, y, 64.0, 64.0],
        ),
        Region::atlas(
            "UI-Frame-DiamondMetal-CornerBottomLeft",
            [x, y + height - 64.0, 64.0, 64.0],
        ),
        Region::atlas(
            "UI-Frame-DiamondMetal-CornerBottomRight",
            [x + width - 64.0, y + height - 64.0, 64.0, 64.0],
        ),
        Region::atlas(
            "_UI-Frame-DiamondMetal-EdgeTop",
            [x + 64.0, y, width - 128.0, 64.0],
        ),
        Region::atlas(
            "_UI-Frame-DiamondMetal-EdgeBottom",
            [x + 64.0, y + height - 64.0, width - 128.0, 64.0],
        ),
        Region::atlas(
            "!UI-Frame-DiamondMetal-EdgeLeft",
            [x, y + side_top, 64.0, side_height],
        ),
        Region::atlas(
            "!UI-Frame-DiamondMetal-EdgeRight",
            [x + width - 64.0, y + side_top, 64.0, side_height],
        ),
    ]
}

fn translucent_regions([x, y, width, height]: [f32; 4]) -> [Region; 8] {
    // UIPanelTemplates.xml984–1043 + Dialog-Border*:32px corners and16px edges.
    let corners = [
        ([x, y, 32.0, 32.0], [0.0, 0.5, 0.0, 0.5]),
        ([x + width - 32.0, y, 32.0, 32.0], [0.5, 1.0, 0.0, 0.5]),
        ([x, y + height - 32.0, 32.0, 32.0], [0.0, 0.5, 0.5, 1.0]),
        (
            [x + width - 32.0, y + height - 32.0, 32.0, 32.0],
            [0.5, 1.0, 0.5, 1.0],
        ),
    ]
    .map(|(rect, crop)| Region {
        source: TextureSource::FileDataId(407329),
        rect,
        crop,
        tile: [false; 2],
    });
    let edges = [
        (
            407332,
            [x + 32.0, y + 1.0, width - 64.0, 16.0],
            [true, false],
        ),
        (
            407328,
            [x + 32.0, y + height - 16.0, width - 64.0, 16.0],
            [true, false],
        ),
        (
            407330,
            [x + 1.0, y + 32.0, 16.0, height - 64.0],
            [false, true],
        ),
        (
            407331,
            [x + width - 16.0, y + 32.0, 16.0, height - 64.0],
            [false, true],
        ),
    ]
    .map(|(fdid, rect, tile)| Region {
        source: TextureSource::FileDataId(fdid),
        rect,
        crop: [0.0, 1.0, 0.0, 1.0],
        tile,
    });
    let [tl, tr, bl, br] = corners;
    let [top, bottom, left, right] = edges;
    [tl, tr, bl, br, top, bottom, left, right]
}

struct DynName(String);

fn region_element(name: String, region: Region, scale: f32) -> Element {
    let [left, top, width, height] = region.rect.map(|value| value * scale);
    let [l, r, t, b] = region.crop;
    let crop = format!("{l},{r},{t},{b}");
    let [horizontal, vertical] = region.tile;
    let layer = if matches!(region.source, TextureSource::SolidColor(_)) {
        "BACKGROUND"
    } else {
        "BORDER"
    };
    let mut element = rsx! { texture { name: {DynName(name)}, left, top, width, height,
        pos_type: "absolute", draw_layer: layer,
        tex_coords: {crop.as_str()}, horiz_tile: horizontal, vert_tile: vertical,
    } };
    let [WidgetChild::Widget(widget)] = element.as_mut_slice() else {
        unreachable!("one texture root")
    };
    let (attribute, value) = match region.source {
        TextureSource::Atlas(name) => ("texture_atlas", name),
        TextureSource::FileDataId(fdid) => ("texture_fdid", fdid.to_string()),
        TextureSource::SolidColor([r, g, b, a]) => ("background_color", format!("{r},{g},{b},{a}")),
        other => panic!("unsupported named artwork source: {other:?}"),
    };
    widget.attrs.push(Attr::new_dynamic(attribute, value));
    element
}
