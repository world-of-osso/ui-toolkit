use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::atlas;
use crate::render::LoadedTexture;
use crate::widgets::texture::TextureSource;

/// Trait for loading BLP textures and resolving FDIDs to file paths.
/// Game-engine implements this to bridge the asset pipeline.
pub trait BlpLoader: Send + Sync + 'static {
    fn load_blp_to_image(&self, path: &Path) -> Result<Image, String>;
    fn load_blp_gpu_image(&self, path: &Path) -> Result<Image, String>;
    fn ensure_texture(&self, fdid: u32) -> Option<std::path::PathBuf>;
}

/// Bevy resource holding the BLP loader implementation.
#[derive(Resource)]
pub struct BlpLoaderRes(pub Box<dyn BlpLoader>);

pub fn load_texture_source_pub(
    source: &TextureSource,
    images: &mut Option<ResMut<Assets<Image>>>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    missing_file_textures: &mut HashSet<String>,
    blp_loader: Option<&BlpLoaderRes>,
) -> Option<LoadedTexture> {
    load_texture_source(
        source,
        images,
        texture_cache,
        file_texture_cache,
        missing_textures,
        missing_file_textures,
        blp_loader,
    )
}

pub fn load_texture_source(
    source: &TextureSource,
    images: &mut Option<ResMut<Assets<Image>>>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    missing_file_textures: &mut HashSet<String>,
    blp_loader: Option<&BlpLoaderRes>,
) -> Option<LoadedTexture> {
    match source {
        TextureSource::FileDataId(fdid) => {
            load_fdid_texture(*fdid, images, texture_cache, missing_textures, blp_loader)
                .map(|handle| LoadedTexture { handle, rect: None })
        }
        TextureSource::File(path) => {
            load_file_texture(path, images, file_texture_cache, missing_file_textures, blp_loader)
                .map(|handle| LoadedTexture { handle, rect: None })
        }
        TextureSource::Atlas(name) => {
            load_atlas_texture(name, images, file_texture_cache, missing_file_textures, blp_loader)
        }
        _ => None,
    }
}

fn load_atlas_texture(
    name: &str,
    images: &mut Option<ResMut<Assets<Image>>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_file_textures: &mut HashSet<String>,
    blp_loader: Option<&BlpLoaderRes>,
) -> Option<LoadedTexture> {
    let region = atlas::get_region(name)?;
    if should_materialize_atlas_region(region.path) {
        return load_materialized_atlas_region(
            name,
            region.path,
            region.left,
            region.right,
            region.top,
            region.bottom,
            images,
            file_texture_cache,
            missing_file_textures,
            blp_loader,
        );
    }
    let handle = load_file_texture(
        region.path,
        images,
        file_texture_cache,
        missing_file_textures,
        blp_loader,
    )?;
    let rect = images
        .as_ref()
        .and_then(|assets| assets.get(&handle))
        .map(|image| region.rect_pixels(image));
    Some(LoadedTexture { handle, rect })
}

fn load_materialized_atlas_region(
    name: &str,
    path: &str,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    images: &mut Option<ResMut<Assets<Image>>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_file_textures: &mut HashSet<String>,
    blp_loader: Option<&BlpLoaderRes>,
) -> Option<LoadedTexture> {
    let cache_key = format!("atlas::{name}");
    if let Some(handle) = file_texture_cache.get(&cache_key) {
        return Some(LoadedTexture {
            handle: handle.clone(),
            rect: None,
        });
    }

    let base_handle = load_file_texture(path, images, file_texture_cache, missing_file_textures, blp_loader)?;
    let assets = images.as_mut().map(|images| &mut **images)?;
    let base = assets.get(&base_handle)?;
    let cropped = crop_image_region(base, left, right, top, bottom)?;
    let handle = assets.add(cropped);
    file_texture_cache.insert(cache_key, handle.clone());
    Some(LoadedTexture { handle, rect: None })
}

fn crop_image_region(image: &Image, left: f32, right: f32, top: f32, bottom: f32) -> Option<Image> {
    let is_rgba8 = matches!(image.texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm);
    if !is_rgba8 { return None; }
    let data = image.data.as_ref()?;
    let (w, h) = (image.width() as usize, image.height() as usize);
    let x0 = (left * w as f32).round().clamp(0.0, w as f32) as usize;
    let x1 = (right * w as f32).round().clamp(0.0, w as f32) as usize;
    let y0 = (top * h as f32).round().clamp(0.0, h as f32) as usize;
    let y1 = (bottom * h as f32).round().clamp(0.0, h as f32) as usize;
    if x1 <= x0 || y1 <= y0 { return None; }
    let pixels = crop_rgba_pixels(data, w, x0, x1, y0, y1);
    Some(Image::new(
        Extent3d { width: (x1 - x0) as u32, height: (y1 - y0) as u32, depth_or_array_layers: 1 },
        TextureDimension::D2, pixels, image.texture_descriptor.format, RenderAssetUsages::default(),
    ))
}

fn crop_rgba_pixels(data: &[u8], stride: usize, x0: usize, x1: usize, y0: usize, y1: usize) -> Vec<u8> {
    let crop_w = x1 - x0;
    let mut out = vec![0u8; crop_w * (y1 - y0) * 4];
    for row in 0..(y1 - y0) {
        let src = ((y0 + row) * stride + x0) * 4;
        let dst = row * crop_w * 4;
        out[dst..dst + crop_w * 4].copy_from_slice(&data[src..src + crop_w * 4]);
    }
    out
}

pub fn load_file_texture(
    path: &str,
    images: &mut Option<ResMut<Assets<Image>>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_file_textures: &mut HashSet<String>,
    blp_loader: Option<&BlpLoaderRes>,
) -> Option<Handle<Image>> {
    if let Some(handle) = file_texture_cache.get(path) {
        return Some(handle.clone());
    }
    if missing_file_textures.contains(path) {
        return None;
    }
    let assets = images.as_mut().map(|images| &mut **images)?;
    let image = match load_ui_file_texture(path, blp_loader) {
        Ok(image) => image,
        Err(err) => {
            eprintln!("[UI] Failed to load texture {path}: {err}");
            missing_file_textures.insert(path.to_string());
            return None;
        }
    };
    let handle = assets.add(image);
    file_texture_cache.insert(path.to_string(), handle.clone());
    Some(handle)
}

fn load_image_from_buffer(path: &str, ext: &str) -> Result<Image, String> {
    let bytes = fs::read(path).map_err(|err| format!("Failed to read {ext}: {err}"))?;
    Image::from_buffer(
        &bytes,
        ImageType::Extension(ext),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::default(),
        RenderAssetUsages::default(),
    )
    .map_err(|err| format!("Failed to decode {ext}: {err}"))
}

fn load_ui_file_texture(path: &str, blp_loader: Option<&BlpLoaderRes>) -> Result<Image, String> {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".ktx2") {
        load_image_from_buffer(path, "ktx2")
    } else if lower.ends_with(".png") {
        load_image_from_buffer(path, "png")
    } else if let Some(loader) = blp_loader {
        if should_cpu_decode_ui_texture(path) {
            loader.0.load_blp_to_image(Path::new(path))
        } else {
            loader.0.load_blp_gpu_image(Path::new(path))
        }
    } else {
        Err("No BLP loader configured".to_string())
    }
}

fn should_cpu_decode_ui_texture(path: &str) -> bool {
    path.contains("/Interface/GLUES/CharacterSelect/")
        || path.contains("/Interface/CharacterSelection/")
}

fn should_materialize_atlas_region(path: &str) -> bool {
    path.contains("/Interface/GLUES/CharacterSelect/")
        || path.contains("/Interface/CharacterSelection/")
}

pub fn load_fdid_texture(
    fdid: u32,
    images: &mut Option<ResMut<Assets<Image>>>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    blp_loader: Option<&BlpLoaderRes>,
) -> Option<Handle<Image>> {
    if let Some(handle) = texture_cache.get(&fdid) {
        return Some(handle.clone());
    }
    if missing_textures.contains(&fdid) {
        return None;
    }
    let loader = blp_loader?;
    let assets = images.as_mut().map(|images| &mut **images)?;
    let path = loader.0.ensure_texture(fdid)?;
    let image = match loader.0.load_blp_gpu_image(&path) {
        Ok(image) => image,
        Err(_) => {
            missing_textures.insert(fdid);
            return None;
        }
    };
    let handle = assets.add(image);
    texture_cache.insert(fdid, handle.clone());
    Some(handle)
}
