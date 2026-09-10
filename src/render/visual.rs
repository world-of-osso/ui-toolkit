use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::frame::WidgetData;
use crate::render_texture::{BlpLoaderRes, load_texture_source};
use crate::widgets::button::ButtonState;
use crate::widgets::texture::TextureSource;

pub(super) fn frame_visual(
    frame: &crate::frame::Frame,
    images: &mut Option<ResMut<Assets<Image>>>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    missing_file_textures: &mut HashSet<String>,
    blp_loader: Option<&BlpLoaderRes>,
) -> (Color, Handle<Image>, Option<Rect>) {
    let args = (
        images,
        texture_cache,
        file_texture_cache,
        missing_textures,
        missing_file_textures,
        blp_loader,
    );
    let (
        images,
        texture_cache,
        file_texture_cache,
        missing_textures,
        missing_file_textures,
        blp_loader,
    ) = args;
    statusbar_visual(
        frame,
        images,
        texture_cache,
        file_texture_cache,
        missing_textures,
        missing_file_textures,
        blp_loader,
    )
    .or_else(|| {
        frame_button_visual(
            frame,
            images,
            texture_cache,
            file_texture_cache,
            missing_textures,
            missing_file_textures,
            blp_loader,
        )
    })
    .or_else(|| {
        texture_visual(
            frame,
            images,
            texture_cache,
            file_texture_cache,
            missing_textures,
            missing_file_textures,
            blp_loader,
        )
    })
    .unwrap_or_else(|| (super::frame_color(frame), Handle::default(), None))
}

pub(super) fn frame_button_visual(
    frame: &crate::frame::Frame,
    images: &mut Option<ResMut<Assets<Image>>>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    missing_file_textures: &mut HashSet<String>,
    blp_loader: Option<&BlpLoaderRes>,
) -> Option<(Color, Handle<Image>, Option<Rect>)> {
    let WidgetData::Button(btn) = frame.widget_data.as_ref()? else {
        return None;
    };
    button_texture(
        btn,
        frame.effective_alpha,
        images,
        texture_cache,
        file_texture_cache,
        missing_textures,
        missing_file_textures,
        blp_loader,
    )
}

pub(super) fn statusbar_visual(
    frame: &crate::frame::Frame,
    images: &mut Option<ResMut<Assets<Image>>>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    missing_file_textures: &mut HashSet<String>,
    blp_loader: Option<&BlpLoaderRes>,
) -> Option<(Color, Handle<Image>, Option<Rect>)> {
    let WidgetData::StatusBar(sb) = frame.widget_data.as_ref()? else {
        return None;
    };
    let [r, g, b, a] = sb.color;
    if let Some(source) = &sb.texture {
        return Some((
            Color::srgba(r, g, b, a * frame.effective_alpha),
            load_texture_source(
                source,
                images,
                texture_cache,
                file_texture_cache,
                missing_textures,
                missing_file_textures,
                blp_loader,
            )?
            .handle,
            None,
        ));
    }
    Some((
        Color::srgba(r, g, b, a * frame.effective_alpha),
        Handle::default(),
        None,
    ))
}

pub(super) fn texture_visual(
    frame: &crate::frame::Frame,
    images: &mut Option<ResMut<Assets<Image>>>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    missing_file_textures: &mut HashSet<String>,
    blp_loader: Option<&BlpLoaderRes>,
) -> Option<(Color, Handle<Image>, Option<Rect>)> {
    let source = super::frame_texture_source(frame)?;
    // TODO: additive blend requires custom pipeline
    let mut texture = load_texture_source(
        source,
        images,
        texture_cache,
        file_texture_cache,
        missing_textures,
        missing_file_textures,
        blp_loader,
    )?;
    if let Some(WidgetData::Texture(data)) = &frame.widget_data {
        apply_texture_crop(data.tex_coords, &mut texture, images.as_deref())?;
    }
    Some((texture_tint(frame), texture.handle, texture.rect))
}

fn apply_texture_crop(
    coordinates: [f32; 4],
    texture: &mut super::LoadedTexture,
    images: Option<&Assets<Image>>,
) -> Option<()> {
    if coordinates == [0.0, 1.0, 0.0, 1.0] {
        return Some(());
    }
    let source = match texture.rect {
        Some(rect) => rect,
        None => {
            let image = images?.get(&texture.handle)?;
            Rect::from_corners(
                Vec2::ZERO,
                Vec2::new(image.width() as f32, image.height() as f32),
            )
        }
    };
    let [left, right, top, bottom] = coordinates;
    texture.rect = Some(Rect::from_corners(
        source.min + source.size() * Vec2::new(left, top),
        source.min + source.size() * Vec2::new(right, bottom),
    ));
    Some(())
}

#[cfg(test)]
mod crop_tests {
    use super::*;

    #[test]
    fn normalized_crop_is_relative_to_existing_atlas_region() {
        for (coordinates, expected) in [
            ([0.0, 1.0, 0.0, 1.0], Rect::new(16.0, 8.0, 80.0, 40.0)),
            ([0.0, 0.5, 0.0, 1.0], Rect::new(16.0, 8.0, 48.0, 40.0)),
            ([0.25, 0.75, 0.25, 0.75], Rect::new(32.0, 16.0, 64.0, 32.0)),
        ] {
            let mut texture = super::super::LoadedTexture {
                handle: Handle::default(),
                rect: Some(Rect::new(16.0, 8.0, 80.0, 40.0)),
            };
            assert_eq!(
                apply_texture_crop(coordinates, &mut texture, None),
                Some(())
            );
            assert_eq!(texture.rect, Some(expected));
        }
    }
}

pub(super) fn button_texture(
    btn: &crate::widgets::button::ButtonData,
    effective_alpha: f32,
    images: &mut Option<ResMut<Assets<Image>>>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    missing_file_textures: &mut HashSet<String>,
    blp_loader: Option<&BlpLoaderRes>,
) -> Option<(Color, Handle<Image>, Option<Rect>)> {
    let source = select_button_texture_source(btn)?;
    let texture = load_texture_source(
        source,
        images,
        texture_cache,
        file_texture_cache,
        missing_textures,
        missing_file_textures,
        blp_loader,
    )?;
    Some((
        Color::srgba(1.0, 1.0, 1.0, effective_alpha),
        texture.handle,
        texture.rect,
    ))
}

pub(super) fn select_button_texture_source(
    btn: &crate::widgets::button::ButtonData,
) -> Option<&TextureSource> {
    let source = match btn.state {
        ButtonState::Disabled => btn
            .disabled_texture
            .as_ref()
            .or(btn.normal_texture.as_ref()),
        ButtonState::Pushed => btn.pushed_texture.as_ref().or(btn.normal_texture.as_ref()),
        ButtonState::Normal if btn.hovered => btn
            .highlight_texture
            .as_ref()
            .or(btn.normal_texture.as_ref()),
        ButtonState::Normal => btn.normal_texture.as_ref(),
    }?;
    if matches!(source, TextureSource::None) {
        return None;
    }
    Some(source)
}

pub(super) fn texture_tint(frame: &crate::frame::Frame) -> Color {
    let (vertex_color, desaturated) = match &frame.widget_data {
        Some(WidgetData::Texture(tex)) => (tex.vertex_color, tex.desaturated),
        _ => ([1.0, 1.0, 1.0, 1.0], false),
    };
    let [r, g, b, a] = vertex_color;
    if desaturated {
        let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        Color::srgba(lum, lum, lum, a * frame.effective_alpha)
    } else {
        Color::srgba(r, g, b, a * frame.effective_alpha)
    }
}
