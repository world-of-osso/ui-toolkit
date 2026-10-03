mod db2;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

pub use db2::{ActiveSkin, UiCanvas};

/// The canvas whose atlas members the client draws: 1x, like every Retail crop the
/// client used before the DB2 tables.
pub const ATLAS_CANVAS: UiCanvas = UiCanvas::X1;

/// Pixel-space atlas bounds, ordered as `[x, y]` corners.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelRect {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtlasSource {
    File(&'static str),
    FileDataId(u32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtlasRegion {
    pub source: AtlasSource,
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub width: f32,
    pub height: f32,
    pub tiles_horizontally: bool,
    pub tiles_vertically: bool,
    pub nine_slice_edge: Option<f32>,
}

impl AtlasRegion {
    pub fn rect_pixels(&self, width: u32, height: u32) -> PixelRect {
        let width = width as f32;
        let height = height as f32;
        PixelRect {
            min: [self.left * width, self.top * height],
            max: [self.right * width, self.bottom * height],
        }
    }
}

/// Retail and Forever DB2 export directories the atlas table was loaded from.
struct LoadedTable {
    directories: (PathBuf, PathBuf),
    table: db2::AtlasTable,
}

static TABLE: OnceLock<LoadedTable> = OnceLock::new();
static ACTIVE_SKIN: AtomicU8 = AtomicU8::new(0);

/// Load the atlas tables, once, before the first DB2 atlas lookup: Retail's
/// `UiTextureAtlas*.csv` export in `retail_dir` and Forever's in `forever_dir`.
/// A second call must name the same directories. Until then only project art resolves.
pub fn set_atlas_directories(retail_dir: &Path, forever_dir: &Path) -> Result<(), String> {
    let directories = (retail_dir.to_path_buf(), forever_dir.to_path_buf());
    if TABLE.get().is_none() {
        let table = db2::AtlasTable::load(retail_dir, forever_dir)?;
        let _ = TABLE.set(LoadedTable {
            directories: directories.clone(),
            table,
        });
    }
    let loaded = &TABLE.get().expect("atlas table set above").directories;
    if *loaded == directories {
        Ok(())
    } else {
        Err(format!(
            "Atlas tables are {} and {}, not {} and {}",
            loaded.0.display(),
            loaded.1.display(),
            retail_dir.display(),
            forever_dir.display()
        ))
    }
}

/// Switch the atlas set every later [`get_region`] resolves.
pub fn set_active_skin(skin: ActiveSkin) {
    ACTIVE_SKIN.store(skin as u8, Ordering::Relaxed);
}

pub fn active_skin() -> ActiveSkin {
    match ACTIVE_SKIN.load(Ordering::Relaxed) {
        0 => ActiveSkin::Modern,
        _ => ActiveSkin::Forever,
    }
}

/// Lower-cased name of Retail `UiTextureAtlasElement` `element_id`.
pub fn get_name_by_element_id(element_id: u32) -> Option<&'static str> {
    TABLE.get()?.table.name_of(element_id)
}

pub fn nine_slice_margins(name: &str) -> Option<[f32; 4]> {
    match name.to_ascii_lowercase().as_str() {
        "glues-characterselect-card-all-bg" => Some([14.0, 11.0, 14.0, 17.0]),
        _ => get_region(name)
            .and_then(|region| region.nine_slice_edge)
            .map(|edge| [edge, edge, edge, edge]),
    }
}

/// `name` under the active skin.
pub fn get_region(name: &str) -> Option<AtlasRegion> {
    resolve_region(name, active_skin())
}

/// `name` under `skin`: project art (its own names), else the DB2 atlas members of
/// `skin`'s sets on [`ATLAS_CANVAS`]; a name in neither is `None`.
pub fn resolve_region(name: &str, skin: ActiveSkin) -> Option<AtlasRegion> {
    let key = name.to_ascii_lowercase();
    PROJECT_REGIONS
        .iter()
        .find(|(candidate, _)| *candidate == key)
        .map(|(_, region)| *region)
        .or_else(|| TABLE.get()?.table.resolve(&key, skin, ATLAS_CANVAS))
}

macro_rules! atlas_region {
    ($path:literal, $($args:tt)*) => {
        atlas_region!(AtlasSource::File($path), $($args)*)
    };
    ($path:expr, $left:expr, $right:expr, $top:expr, $bottom:expr, $width:expr, $height:expr, $edge:expr) => {
        AtlasRegion {
            source: $path,
            left: $left,
            right: $right,
            top: $top,
            bottom: $bottom,
            width: $width,
            height: $height,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: $edge,
        }
    };
}

type AtlasRegionEntry = (&'static str, AtlasRegion);

/// Project-owned art: regenerated or recoloured button and nameplate images under
/// `data/ui/`, drawn in place of the DB2 member of the same name, plus the Retail red
/// button highlight under a `retail-` name (`128-redbutton-highlight` is the brown one).
const PROJECT_REGIONS: &[AtlasRegionEntry] = &[
    (
        "128-redbutton-up",
        atlas_region!(
            "data/ui/128BrownButton9Sliced.ktx2",
            0.001953,
            0.919922,
            0.509766,
            0.759766,
            470.0,
            128.0,
            Some(16.0)
        ),
    ),
    (
        "128-redbutton-pressed",
        atlas_region!(
            "data/ui/128BrownButton9Sliced.ktx2",
            0.001953,
            0.919922,
            0.255859,
            0.505859,
            470.0,
            128.0,
            Some(16.0)
        ),
    ),
    (
        "128-redbutton-disable",
        atlas_region!(
            "data/ui/128BrownButton9Sliced.ktx2",
            0.001953,
            0.919922,
            0.001953,
            0.251953,
            470.0,
            128.0,
            Some(16.0)
        ),
    ),
    (
        "128-redbutton-highlight",
        atlas_region!(
            "data/ui/128BrownButton.ktx2",
            0.001953,
            0.863281,
            0.190918,
            0.253418,
            441.0,
            128.0,
            Some(16.0)
        ),
    ),
    (
        "glue-bigbutton-brown-up",
        atlas_region!(
            "data/ui/Glues-BigButton-Brown-Up.ktx2",
            0.0,
            1.0,
            0.0,
            1.0,
            256.0,
            64.0,
            None
        ),
    ),
    (
        "glue-bigbutton-brown-down",
        atlas_region!(
            "data/ui/Glues-BigButton-Brown-Down.ktx2",
            0.0,
            1.0,
            0.0,
            1.0,
            256.0,
            64.0,
            None
        ),
    ),
    (
        "glue-bigbutton-brown-highlight",
        atlas_region!(
            "data/ui/Glues-BigButton-Brown-Highlight.ktx2",
            0.0,
            1.0,
            0.0,
            1.0,
            256.0,
            64.0,
            None
        ),
    ),
    (
        "glue-bigbutton-brown-disable",
        atlas_region!(
            "data/ui/Glues-BigButton-Brown-Up.ktx2",
            0.0,
            1.0,
            0.0,
            1.0,
            256.0,
            64.0,
            None
        ),
    ),
    (
        "defaultbutton-nineslice-up",
        atlas_region!(
            "data/ui/login-button-generated-regular-normal.ktx2",
            0.0,
            1.0,
            0.0,
            1.0,
            500.0,
            132.0,
            Some(24.0)
        ),
    ),
    (
        "defaultbutton-nineslice-pressed",
        atlas_region!(
            "data/ui/login-button-generated-regular-pressed.ktx2",
            0.0,
            1.0,
            0.0,
            1.0,
            500.0,
            132.0,
            Some(24.0)
        ),
    ),
    (
        "defaultbutton-nineslice-highlight",
        atlas_region!(
            "data/ui/login-button-generated-regular-highlight.ktx2",
            0.0,
            1.0,
            0.0,
            1.0,
            500.0,
            132.0,
            Some(24.0)
        ),
    ),
    (
        "defaultbutton-nineslice-disabled",
        atlas_region!(
            "data/ui/login-button-generated-regular-disabled.ktx2",
            0.0,
            1.0,
            0.0,
            1.0,
            500.0,
            132.0,
            Some(24.0)
        ),
    ),
    (
        "custom-nameplate-bg",
        atlas_region!(
            "data/ui/nameplate-bg.ktx2",
            0.0,
            1.0,
            0.0,
            1.0,
            300.0,
            60.0,
            None
        ),
    ),
    (
        "retail-128-redbutton-highlight",
        // UiTextureAtlasMember 34021 on UiTextureAtlas 3556 (FDID 7367529, 512x2048).
        atlas_region!(
            AtlasSource::FileDataId(7_367_529),
            1.0 / 512.0,
            442.0 / 512.0,
            391.0 / 2048.0,
            519.0 / 2048.0,
            441.0,
            128.0,
            None
        ),
    ),
];

#[cfg(test)]
mod tests;
