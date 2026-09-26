//! Derive native image primitives from the registry's existing visual contracts.

use bevy::prelude::*;

use crate::frame::{Frame, NineSlice, WidgetData};
use crate::render::{
    LoadedTexture, frame_color, frame_sprite_params, frame_transform, texture_tint,
};
use crate::widgets::{button::ButtonState, texture::TextureSource};

use super::{ImagePart, NativeAssets};

#[cfg(test)]
#[path = "images_tests.rs"]
mod tests;

type LoadedImage = (LoadedTexture, Option<Vec2>);

pub(super) fn project_images(frame: &Frame, assets: &mut NativeAssets) -> Vec<ImagePart> {
    let z = assets.frame_z();
    project_at_z(frame, z, &mut |source| {
        let texture = assets.load(source)?;
        let size = assets.image_size(&texture.handle);
        Some((texture, size))
    })
}

fn project_at_z(
    frame: &Frame,
    z: f32,
    load: &mut impl FnMut(&TextureSource) -> Option<LoadedImage>,
) -> Vec<ImagePart> {
    if !frame.visible {
        return Vec::new();
    }
    let mut parts = Vec::new();
    project_background(frame, z, load, &mut parts);
    project_borders(frame, &mut parts);
    project_nine_slice(frame, z, load, &mut parts);
    project_three_slice(frame, z, load, &mut parts);
    project_tiles(frame, load, &mut parts);
    project_highlight(frame, load, &mut parts);
    parts
}

fn project_background(
    frame: &Frame,
    z: f32,
    load: &mut impl FnMut(&TextureSource) -> Option<LoadedImage>,
    parts: &mut Vec<ImagePart>,
) {
    if frame.nine_slice.is_some() && frame.background_color.is_some() {
        project_sliced_background(frame, z, parts);
        return;
    }
    if !has_base_background(frame) {
        return;
    }
    project_base_background(frame, z, load, parts);
}

fn project_sliced_background(frame: &Frame, z: f32, parts: &mut Vec<ImagePart>) {
    for index in 0..5 {
        let (mut transform, size, color) =
            crate::render::backdrop_part_geometry(frame, index, 0, 0.0, 0.0);
        transform.translation.z = z - 0.0002;
        if size.x > 0.0 && size.y > 0.0 {
            parts.push(from_geometry(
                frame,
                10 + u32::from(index),
                transform,
                size,
                solid(color),
            ));
        }
    }
}

fn has_base_background(frame: &Frame) -> bool {
    if frame.nine_slice.is_some() || frame.three_slice.is_some() {
        return frame.background_color.is_some();
    }
    frame.background_color.is_some()
        || frame.backdrop.as_ref().and_then(|b| b.bg_color).is_some()
        || match &frame.widget_data {
            Some(WidgetData::Button(button)) => {
                crate::render_button::select_button_base_texture_source(button).is_some()
            }
            Some(WidgetData::Texture(_) | WidgetData::StatusBar(_)) => true,
            _ => false,
        }
}

fn project_base_background(
    frame: &Frame,
    z: f32,
    load: &mut impl FnMut(&TextureSource) -> Option<LoadedImage>,
    parts: &mut Vec<ImagePart>,
) {
    let Some(image) = base_image(frame, load) else {
        return;
    };
    let (size, offset) = frame_sprite_params(frame);
    let mut transform = frame_transform(frame, 0, 0.0, 0.0);
    transform.translation += offset.extend(z);
    let mut part = from_geometry(frame, 0, transform, size, image);
    part.node.left = px(0);
    part.node.top = px(0);
    part.node.width = match &frame.widget_data {
        Some(WidgetData::StatusBar(bar)) => percent(
            (((bar.value - bar.min) / (bar.max - bar.min).max(f64::EPSILON)).clamp(0.0, 1.0)
                * 100.0) as f32,
        ),
        _ => percent(100),
    };
    part.node.height = percent(100);
    apply_intrinsic_texture_size(frame, &mut part);
    parts.push(part);
}

fn apply_intrinsic_texture_size(frame: &Frame, part: &mut ImagePart) {
    if matches!(frame.widget_data, Some(WidgetData::Texture(_)))
        && (frame.width == crate::frame::Dimension::Auto
            || frame.height == crate::frame::Dimension::Auto)
    {
        part.node.position_type = PositionType::Relative;
        if frame.width == crate::frame::Dimension::Auto {
            part.node.width = Val::Auto;
        }
        if frame.height == crate::frame::Dimension::Auto {
            part.node.height = Val::Auto;
        }
        part.image.image_mode = NodeImageMode::Auto;
    }
}

fn base_image(
    frame: &Frame,
    load: &mut impl FnMut(&TextureSource) -> Option<LoadedImage>,
) -> Option<ImageNode> {
    match &frame.widget_data {
        Some(WidgetData::StatusBar(bar)) => {
            let [r, g, b, a] = bar.color;
            let color = Color::srgba(r, g, b, a * frame.effective_alpha);
            let mut image = solid(color);
            if let Some(source) = &bar.texture {
                // Existing status bars sample the full loaded handle, not its atlas rectangle.
                image.image = load(source)?.0.handle;
            }
            Some(image)
        }
        Some(WidgetData::Button(button)) => {
            let Some(source) = crate::render_button::select_button_texture_source(button) else {
                return Some(solid(frame_color(frame)));
            };
            Some(textured(
                load(source)?.0,
                Color::srgba(1.0, 1.0, 1.0, frame.effective_alpha),
            ))
        }
        Some(WidgetData::Texture(texture)) => {
            if matches!(
                texture.source,
                TextureSource::None | TextureSource::SolidColor(_)
            ) {
                // The existing basic renderer uses frame color for these untextured sources.
                return Some(solid(frame_color(frame)));
            }
            let (mut loaded, size) = load(&texture.source)?;
            if texture.tex_coords != [0.0, 1.0, 0.0, 1.0] {
                let source = texture_rect(&loaded, size)?;
                loaded.rect = Some(normalized_crop(source, texture.tex_coords));
            }
            // Additive blending and partial desaturation are not implemented by the old renderer.
            let mut image = textured(loaded, texture_tint(frame));
            // WoW SetTexCoord with left > right or top > bottom mirrors that axis.
            let [left, right, top, bottom] = texture.tex_coords;
            image.flip_x = left > right;
            image.flip_y = top > bottom;
            Some(image)
        }
        _ => Some(solid(frame_color(frame))),
    }
}

fn project_borders(frame: &Frame, parts: &mut Vec<ImagePart>) {
    if let Some(backdrop) = &frame.backdrop
        && backdrop.border_color.is_some()
    {
        for index in 0..4 {
            let (transform, size, color) =
                crate::render_border::edge_geometry(frame, backdrop, index, 0.0, 0.0);
            parts.push(from_geometry(
                frame,
                20 + u32::from(index),
                transform,
                size,
                solid(color),
            ));
        }
    }
    if let Some(border) = &frame.border {
        for index in 0..4 {
            let (transform, size, color) =
                crate::render_border::css_edge_geometry(frame, border, index, 0.0, 0.0);
            parts.push(from_geometry(
                frame,
                30 + u32::from(index),
                transform,
                size,
                solid(color),
            ));
        }
    }
}

fn project_nine_slice(
    frame: &Frame,
    z: f32,
    load: &mut impl FnMut(&TextureSource) -> Option<LoadedImage>,
    parts: &mut Vec<ImagePart>,
) {
    let Some(slice) = &frame.nine_slice else {
        return;
    };
    for index in 0..9 {
        let (transform, size, color) =
            crate::render_nine_slice::part_geometry(frame, slice, index, 0.0, 0.0, z);
        let Some(image) = nine_slice_image(slice, index, color, load) else {
            continue;
        };
        parts.push(from_geometry(
            frame,
            100 + u32::from(index),
            transform,
            size,
            image,
        ));
    }
}

fn nine_slice_image(
    slice: &NineSlice,
    index: u8,
    color: Color,
    load: &mut impl FnMut(&TextureSource) -> Option<LoadedImage>,
) -> Option<ImageNode> {
    let source = slice
        .part_textures
        .as_ref()
        .map(|sources| &sources[index as usize])
        .or(slice.texture.as_ref());
    let Some(source) = source else {
        return Some(solid(color));
    };
    if matches!(source, TextureSource::None) {
        return Some(solid(color));
    }
    let (mut loaded, size) = load(source)?;
    loaded.rect = if slice.part_textures.is_some() {
        // Per-part textures already contain their complete part; old rendering ignores atlas rects.
        None
    } else {
        nine_slice_rect(slice, index, &loaded, size)
    };
    Some(textured(loaded, color))
}

fn nine_slice_rect(
    slice: &NineSlice,
    index: u8,
    loaded: &LoadedTexture,
    size: Option<Vec2>,
) -> Option<Rect> {
    let atlas = texture_rect(loaded, size)?;
    if let Some(rectangles) = &slice.uv_rects {
        return Some(crate::render_nine_slice::explicit_uv_rect_for_part(
            rectangles, index, atlas,
        ));
    }
    let (left, top, right, bottom) = crate::render_nine_slice::uv_edges(slice);
    let mut rect = crate::render_nine_slice::uv_rect_for_part(
        index,
        atlas.width(),
        atlas.height(),
        left,
        top,
        right,
        bottom,
    );
    rect.min += atlas.min;
    rect.max += atlas.min;
    Some(rect)
}

fn project_three_slice(
    frame: &Frame,
    z: f32,
    load: &mut impl FnMut(&TextureSource) -> Option<LoadedImage>,
    parts: &mut Vec<ImagePart>,
) {
    let Some(slice) = &frame.three_slice else {
        return;
    };
    for index in 0..3 {
        let (transform, size, color) =
            crate::render_three_slice::part_geometry(frame, slice, index, 0.0, 0.0, z);
        let source = crate::render_three_slice::part_source(slice, index);
        let mut image = solid(color);
        if !matches!(source, TextureSource::None) {
            let Some((loaded, _)) = load(source) else {
                continue;
            };
            image.image = loaded.handle;
        }
        parts.push(from_geometry(
            frame,
            200 + u32::from(index),
            transform,
            size,
            image,
        ));
    }
}

fn project_tiles(
    frame: &Frame,
    load: &mut impl FnMut(&TextureSource) -> Option<LoadedImage>,
    parts: &mut Vec<ImagePart>,
) {
    let Some((horizontal, vertical)) = crate::render_tiled::frame_tiling(frame) else {
        return;
    };
    let Some(fdid) = crate::render_tiled::frame_tiled_fdid(frame) else {
        return;
    };
    let Some((loaded, _)) = load(&TextureSource::FileDataId(fdid)) else {
        return;
    };
    let size = crate::render_tiled::tile_size(frame, horizontal, vertical);
    for (index, center) in crate::render_tiled::tile_positions(frame, horizontal, vertical)
        .into_iter()
        .enumerate()
    {
        let transform = Transform::from_xyz(center.x, -center.y, 0.001);
        let image = textured(
            LoadedTexture {
                handle: loaded.handle.clone(),
                rect: None,
            },
            texture_tint(frame),
        );
        parts.push(from_geometry(
            frame,
            300 + index as u32,
            transform,
            size,
            image,
        ));
    }
}

fn project_highlight(
    frame: &Frame,
    load: &mut impl FnMut(&TextureSource) -> Option<LoadedImage>,
    parts: &mut Vec<ImagePart>,
) {
    let Some(source) = crate::render_button::button_highlight_source(frame) else {
        return;
    };
    let Some(WidgetData::Button(button)) = &frame.widget_data else {
        return;
    };
    if !button.hovered || button.state == ButtonState::Disabled {
        return;
    }
    let Some((loaded, _)) = load(source) else {
        return;
    };
    let origin = frame_origin(frame);
    let button_size = Vec2::new(frame.resolved_width(), frame.resolved_height());
    let size = button.highlight_size.map_or(button_size, Vec2::from_array);
    let button_center = origin + button_size / 2.0;
    let center_x = if button.highlight_size.is_some() {
        button_center.x
    } else {
        // Preserve ordinary button projection when no size is authored.
        origin.x + frame.width.value() / 2.0
    };
    let transform = Transform::from_xyz(center_x, -button_center.y, 500.0);
    let image = textured(
        loaded,
        Color::srgba(
            1.0,
            1.0,
            1.0,
            frame.effective_alpha * button.highlight_alpha,
        ),
    );
    parts.push(from_geometry(frame, 1_000_000, transform, size, image));
}

fn frame_origin(frame: &Frame) -> Vec2 {
    frame
        .layout_rect
        .as_ref()
        .map_or(Vec2::ZERO, |rect| Vec2::new(rect.x, rect.y))
}

fn from_geometry(
    frame: &Frame,
    key: u32,
    transform: Transform,
    size: Vec2,
    image: ImageNode,
) -> ImagePart {
    let origin = Vec2::new(transform.translation.x, -transform.translation.y)
        - frame_origin(frame)
        - size / 2.0;
    ImagePart {
        key,
        node: Node {
            position_type: PositionType::Absolute,
            left: px(origin.x),
            top: px(origin.y),
            width: px(size.x),
            height: px(size.y),
            ..default()
        },
        image,
        // Registry sprites rotate counter-clockwise in Y-up space; native UI is Y-down.
        transform: UiTransform::from_rotation(Rot2::radians(
            -transform.rotation.to_euler(EulerRot::XYZ).2,
        )),
        z: (transform.translation.z * 10_000.0).round() as i32,
    }
}

fn solid(color: Color) -> ImageNode {
    ImageNode {
        image: Handle::default(),
        color,
        image_mode: NodeImageMode::Stretch,
        ..default()
    }
}

fn textured(texture: LoadedTexture, color: Color) -> ImageNode {
    ImageNode {
        image: texture.handle,
        rect: texture.rect,
        color,
        image_mode: NodeImageMode::Stretch,
        ..default()
    }
}

fn texture_rect(texture: &LoadedTexture, size: Option<Vec2>) -> Option<Rect> {
    texture
        .rect
        .or_else(|| size.map(|size| Rect::from_corners(Vec2::ZERO, size)))
}

fn normalized_crop(source: Rect, [left, right, top, bottom]: [f32; 4]) -> Rect {
    Rect::from_corners(
        source.min + source.size() * Vec2::new(left, top),
        source.min + source.size() * Vec2::new(right, bottom),
    )
}

#[cfg(test)]
fn project_with_loader(
    frame: &Frame,
    load: &mut impl FnMut(&TextureSource) -> Option<LoadedImage>,
) -> Vec<ImagePart> {
    project_at_z(frame, 0.0, load)
}
