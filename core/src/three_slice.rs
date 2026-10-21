//! Shared asymmetric/cropped/vertical geometry for the existing ThreeSlice representation.
use crate::art_layout::{Region, region_element};
use crate::frame::ThreeSlice;
use crate::strata::DrawLayer;
use crate::widget_def::{Attr, Element, WidgetChild};
use crate::widgets::texture::TextureSource;

pub struct SlicePart<'a> {
    pub source: &'a TextureSource,
    pub rect: [f32; 4],
    pub crop: [f32; 4],
}

pub fn slice_parts(slice: &ThreeSlice, width: f32, height: f32) -> [SlicePart<'_>; 3] {
    let [first, last] = slice.cap_widths;
    let extent = if slice.vertical { height } else { width };
    let center = (extent - first - last).max(0.0);
    let sources = [&slice.left, &slice.center, &slice.right];
    let spans = [(0.0, first), (first, center), (first + center, last)];
    std::array::from_fn(|index| {
        let (at, length) = spans[index];
        let rect = if slice.vertical {
            [0.0, at, width, length]
        } else {
            [at, 0.0, length, height]
        };
        SlicePart {
            source: sources[index],
            rect,
            crop: slice.tex_coords[index],
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_template_art_three_slice_preserves_independent_cap_extents() {
        let slice = ThreeSlice {
            cap_widths: [12.0, 20.0],
            ..Default::default()
        };
        let parts = slice_parts(&slice, 100.0, 22.0);
        assert_eq!(parts[0].rect, [0.0, 0.0, 12.0, 22.0]);
        assert_eq!(parts[1].rect, [12.0, 0.0, 68.0, 22.0]);
        assert_eq!(parts[2].rect, [80.0, 0.0, 20.0, 22.0]);
    }

    #[test]
    fn ui_template_art_three_slice_preserves_vertical_track_geometry() {
        let slice = ThreeSlice {
            cap_widths: [6.0, 8.0],
            vertical: true,
            ..Default::default()
        };
        let parts = slice_parts(&slice, 8.0, 100.0);
        assert_eq!(parts[0].rect, [0.0, 0.0, 8.0, 6.0]);
        assert_eq!(parts[1].rect, [0.0, 6.0, 8.0, 86.0]);
        assert_eq!(parts[2].rect, [0.0, 92.0, 8.0, 8.0]);
    }

    #[test]
    fn ui_template_art_three_slice_preserves_each_region_crop() {
        let slice = ThreeSlice {
            tex_coords: [
                [0.1, 0.2, 0.3, 0.4],
                [0.0, 1.0, 0.0, 0.65],
                [1.0, 0.0, 0.0, 1.0],
            ],
            ..Default::default()
        };
        let parts = slice_parts(&slice, 100.0, 22.0);
        for (part, expected) in parts.into_iter().zip(slice.tex_coords) {
            assert_eq!(part.crop, expected);
        }
    }
}

/// Expand a shared ThreeSlice into ordinary texture children, using the same geometry as native.
pub fn slice_art(
    prefix: &str,
    slice: &ThreeSlice,
    [x, y, width, height]: [f32; 4],
    names: [&str; 3],
    layer: DrawLayer,
) -> Element {
    names
        .into_iter()
        .zip(slice_parts(slice, width, height))
        .flat_map(|(name, part)| {
            let [left, top, width, height] = part.rect;
            let mut region = match part.source {
                TextureSource::Atlas(name) => {
                    Region::atlas(name, [x + left, y + top, width, height])
                }
                source => Region {
                    source: source.clone(),
                    rect: [x + left, y + top, width, height],
                    crop: part.crop,
                    tile: [false; 2],
                    rotation: 0.0,
                    layer: layer.as_str(),
                },
            };
            region.crop = part.crop;
            region.layer = layer.as_str();
            let mut element = region_element(format!("{prefix}{name}"), region, 1.0);
            let [WidgetChild::Widget(widget)] = element.as_mut_slice() else {
                unreachable!("one slice region")
            };
            let [r, g, b, a] = slice.color;
            widget.attrs.push(Attr::new_dynamic(
                "vertex_color",
                format!("{r},{g},{b},{a}"),
            ));
            widget.attrs.push(Attr::new_static(
                "blend_mode",
                slice.blend_mode.as_str().into(),
            ));
            element
        })
        .collect()
}
