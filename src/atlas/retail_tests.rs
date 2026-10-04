use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::render_texture::{BlpLoader, BlpLoaderRes, load_texture_source_pub};
use crate::widgets::texture::TextureSource;

// Golden pixel bounds and logical sizes from the local UiTextureAtlas* CSV join.
// Public names are Element.Name except the Retail red highlight, whose
// canonical name collides with existing project-owned brown art.
const AUTHORED_CROPS: &str = include_str!("retail_fixture.tsv");

struct AuthoredCrop<'a> {
    name: &'a str,
    fdid: u32,
    rect: [u32; 4],
    logical_size: [u32; 2],
    element_id: u32,
}

fn authored_crops() -> Vec<AuthoredCrop<'static>> {
    AUTHORED_CROPS
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| {
            let columns: Vec<_> = line.split('\t').collect();
            assert_eq!(columns.len(), 9, "malformed crop fixture: {line}");
            let number = |index: usize| columns[index].parse().expect("fixture number");
            AuthoredCrop {
                name: columns[0],
                fdid: number(1),
                rect: [number(2), number(3), number(4), number(5)],
                logical_size: [number(6), number(7)],
                element_id: number(8),
            }
        })
        .collect()
}

fn atlas_size(fdid: u32) -> Result<[u32; 2], String> {
    match fdid {
        1_253_496 => Ok([2048, 2048]),
        3_487_944 => Ok([2048, 1024]),
        3_534_438 => Ok([1024, 1024]),
        3_575_404 => Ok([1024, 512]),
        5_390_329 => Ok([512, 256]),
        7_367_529 => Ok([512, 2048]),
        _ => Err(format!("unexpected atlas FDID {fdid}")),
    }
}

fn authored_pixel(fdid: u32, x: u32, y: u32) -> [u8; 4] {
    [
        (x % 251) as u8,
        (y % 241) as u8,
        (fdid % 199) as u8,
        if (x + y) % 17 == 0 { 0 } else { 255 },
    ]
}

struct DecodedAtlasFixture;

impl BlpLoader for DecodedAtlasFixture {
    fn ensure_texture(&self, fdid: u32) -> Option<PathBuf> {
        atlas_size(fdid)
            .ok()
            .map(|_| PathBuf::from(format!("fixture/{fdid}.blp")))
    }

    fn load_blp_to_image(&self, path: &Path) -> Result<Image, String> {
        let fdid = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| format!("invalid fixture path: {}", path.display()))?
            .parse::<u32>()
            .map_err(|error| error.to_string())?;
        let [width, height] = atlas_size(fdid)?;
        let pixels = (0..height)
            .flat_map(|y| (0..width).flat_map(move |x| authored_pixel(fdid, x, y)))
            .collect();
        Ok(Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            pixels,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        ))
    }

    fn load_blp_gpu_image(&self, _path: &Path) -> Result<Image, String> {
        Err("atlas fixture requires CPU-readable pixels".into())
    }
}

fn expected_crop_pixels(crop: &AuthoredCrop<'_>) -> Vec<u8> {
    let [left, top, right, bottom] = crop.rect;
    (top..bottom)
        .flat_map(|y| {
            (left..right).flat_map(move |x| {
                let pixel = authored_pixel(crop.fdid, x, y);
                if pixel[3] == 0 { [0; 4] } else { pixel }
            })
        })
        .collect()
}

#[test]
fn retail_highlight_preserves_legacy_brown_highlight() {
    let retail = super::get_region("retail-128-redbutton-highlight")
        .expect("Retail highlight image is available");
    assert_eq!(retail.source, super::AtlasSource::FileDataId(7_367_529));
    assert_eq!(
        super::get_name_by_element_id(5451),
        Some("retail-128-redbutton-highlight")
    );

    let legacy = super::get_region("128-redbutton-highlight")
        .expect("project-owned brown highlight is still available");
    assert_eq!(
        legacy.source,
        super::AtlasSource::File("data/ui/128BrownButton.ktx2")
    );
}

#[test]
fn retail_character_creation_atlases_crop_authored_pixels_and_keep_logical_sizes() {
    let loader = BlpLoaderRes(Box::new(DecodedAtlasFixture));
    let mut world = World::new();
    world.init_resource::<Assets<Image>>();
    let mut state = SystemState::<ResMut<Assets<Image>>>::new(&mut world);
    let mut images = Some(state.get_mut(&mut world).expect("image assets"));
    let mut texture_cache = HashMap::new();
    let mut file_cache = HashMap::new();
    let mut missing_textures = HashSet::new();
    let mut missing_files = HashSet::new();
    let mut missing_regions = Vec::new();
    let mut missing_elements = Vec::new();

    for crop in authored_crops() {
        if super::get_name_by_element_id(crop.element_id)
            .is_none_or(|name| !name.eq_ignore_ascii_case(crop.name))
        {
            missing_elements.push(crop.element_id);
        }
        let loaded = load_texture_source_pub(
            &TextureSource::Atlas(crop.name.into()),
            &crate::registry::FrameRegistry::new(800.0, 600.0),
            &mut images,
            &mut texture_cache,
            &mut file_cache,
            &mut missing_textures,
            &mut missing_files,
            Some(&loader),
        );
        let Some(loaded) = loaded else {
            missing_regions.push(crop.name);
            continue;
        };
        let image = images.as_ref().unwrap().get(&loaded.handle).unwrap();
        let [left, top, right, bottom] = crop.rect;
        assert_eq!(
            (image.width(), image.height()),
            (right - left, bottom - top),
            "{} must sample the exact authored pixel rectangle",
            crop.name
        );
        assert!(
            loaded.rect.is_none(),
            "{} crop must be materialized",
            crop.name
        );
        assert_eq!(
            image.data.as_deref().unwrap(),
            expected_crop_pixels(&crop),
            "{} must preserve every authored RGBA pixel (transparent RGB sanitized)",
            crop.name
        );
        let region = super::get_region(crop.name).unwrap();
        assert_eq!(
            [region.width, region.height],
            crop.logical_size.map(|value| value as f32),
            "{} must retain DB2 logical override dimensions",
            crop.name
        );
    }
    assert_eq!(super::get_name_by_element_id(0), None);
    assert_eq!(super::get_name_by_element_id(u32::MAX), None);
    assert!(
        missing_regions.is_empty() && missing_elements.is_empty(),
        "retail controls have no loaded atlas images: {missing_regions:?}; unresolved element IDs: {missing_elements:?}"
    );
}
