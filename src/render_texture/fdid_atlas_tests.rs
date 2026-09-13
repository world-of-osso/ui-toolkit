use super::*;
use bevy::ecs::system::SystemState;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

struct ExtractedAtlas {
    available: Arc<AtomicBool>,
}

impl BlpLoader for ExtractedAtlas {
    fn ensure_texture(&self, fdid: u32) -> Option<PathBuf> {
        (fdid == 5_648_070 && self.available.load(Ordering::SeqCst))
            .then(|| PathBuf::from("fixture/extracted-character-select.blp"))
    }

    fn load_blp_to_image(&self, path: &Path) -> Result<Image, String> {
        if path != Path::new("fixture/extracted-character-select.blp")
            || !self.available.load(Ordering::SeqCst)
        {
            return Err("atlas is not available at this path".into());
        }
        let mut pixels = Vec::with_capacity(1024 * 1024 * 4);
        for y in 0..1024 {
            for x in 0..1024 {
                pixels.extend_from_slice(&[(x % 251) as u8, (y % 251) as u8, 17, 255]);
            }
        }
        pixels[(446 * 1024 + 1) * 4..(446 * 1024 + 1) * 4 + 4].copy_from_slice(&[240, 180, 120, 0]);
        Ok(Image::new(
            Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            pixels,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        ))
    }

    fn load_blp_gpu_image(&self, _path: &Path) -> Result<Image, String> {
        Err("fixture GPU image is not CPU-croppable".into())
    }
}

#[test]
fn fdid_atlas_loads_authored_crops_and_reuses_cached_pixels() {
    let available = Arc::new(AtomicBool::new(true));
    let loader = BlpLoaderRes(Box::new(ExtractedAtlas {
        available: available.clone(),
    }));
    let mut world = World::new();
    world.init_resource::<Assets<Image>>();
    let mut state = SystemState::<ResMut<Assets<Image>>>::new(&mut world);
    let mut images = Some(state.get_mut(&mut world));
    let mut fdid_cache = HashMap::new();
    let mut file_cache = HashMap::new();
    let mut missing_fdids = HashSet::new();
    let mut missing_files = HashSet::new();
    let mut load = |name: &str, images: &mut Option<ResMut<Assets<Image>>>| {
        load_texture_source(
            &TextureSource::Atlas(name.into()),
            images,
            &mut fdid_cache,
            &mut file_cache,
            &mut missing_fdids,
            &mut missing_files,
            Some(&loader),
        )
        .expect("authored atlas must load through its FDID")
    };
    let card = load("glues-characterselect-card-singles", &mut images);
    assert!(
        card.rect.is_none(),
        "atlas pixels must be materialized to avoid bleed"
    );
    let image = images.as_ref().unwrap().get(&card.handle).unwrap();
    assert_eq!((image.width(), image.height()), (310, 89));
    assert_eq!(
        &image.data.as_ref().unwrap()[..8],
        &[0, 0, 0, 0, 2, 195, 17, 255]
    );

    available.store(false, Ordering::SeqCst);
    let cached = load("glues-characterselect-card-singles", &mut images);
    assert_eq!(cached.handle, card.handle);
    let panel = load("glues-characterselect-card-all-bg", &mut images);
    let panel_image = images.as_ref().unwrap().get(&panel.handle).unwrap();
    assert_eq!((panel_image.width(), panel_image.height()), (60, 60));
    assert_eq!(&panel_image.data.as_ref().unwrap()[..4], &[202, 1, 17, 255]);
    let selected = load("glues-characterselect-card-selected", &mut images);
    let selected_image = images.as_ref().unwrap().get(&selected.handle).unwrap();
    assert_eq!(
        (selected_image.width(), selected_image.height()),
        (342, 122)
    );
    assert_eq!(
        &selected_image.data.as_ref().unwrap()[..4],
        &[98, 1, 17, 255]
    );
}
