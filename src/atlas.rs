use bevy::prelude::{Image, Rect, Vec2};

#[derive(Debug, Clone, Copy)]
pub struct AtlasRegion {
    pub path: &'static str,
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
    pub fn rect_pixels(&self, image: &Image) -> Rect {
        let width = image.width() as f32;
        let height = image.height() as f32;
        Rect {
            min: Vec2::new(self.left * width, self.top * height),
            max: Vec2::new(self.right * width, self.bottom * height),
        }
    }
}

pub fn nine_slice_margins(name: &str) -> Option<[f32; 4]> {
    match name.to_ascii_lowercase().as_str() {
        "glues-characterselect-card-all-bg" => Some([14.0, 11.0, 14.0, 17.0]),
        _ => get_region(name)
            .and_then(|region| region.nine_slice_edge)
            .map(|edge| [edge, edge, edge, edge]),
    }
}

macro_rules! atlas_region {
    ($path:expr, $left:expr, $right:expr, $top:expr, $bottom:expr, $width:expr, $height:expr, $edge:expr) => {
        AtlasRegion {
            path: $path,
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

const CHARACTER_SELECT_GLUES: &str =
    "/home/osso/Projects/wow/Interface/GLUES/CharacterSelect/UICharacterSelectGlues.BLP";
const CHARACTER_SELECT_GLUES_GRAYSCALE: &str =
    "/home/osso/Projects/wow/Interface/GLUES/CharacterSelect/UICharacterSelectGluesGrayscale.BLP";
const COMMON_DROPDOWN: &str = "/home/osso/Projects/wow/Interface/COMMON/CommonDropdown.BLP";
const CHARACTER_CREATE: &str =
    "/home/osso/Projects/wow/Interface/GLUES/CHARACTERCREATE/CharacterCreate.BLP";
const ACTION_BAR: &str = "/home/osso/Projects/wow/Interface/HUD/UIActionBar.BLP";

pub fn get_region(name: &str) -> Option<AtlasRegion> {
    let key = name.to_ascii_lowercase();
    let key = key.as_str();
    red_button_region(key)
        .or_else(|| glue_big_button_region(key))
        .or_else(|| default_button_nine_slice_region(key))
        .or_else(|| char_select_region(key))
        .or_else(|| char_select_grayscale_region(key))
        .or_else(|| common_dropdown_region(key))
        .or_else(|| character_create_region(key))
        .or_else(|| action_bar_region(key))
}

fn red_button_region(name: &str) -> Option<AtlasRegion> {
    lookup_region(name, RED_BUTTON_REGIONS)
}

fn glue_big_button_region(name: &str) -> Option<AtlasRegion> {
    lookup_region(name, GLUE_BIG_BUTTON_REGIONS)
}

fn default_button_nine_slice_region(name: &str) -> Option<AtlasRegion> {
    lookup_region(name, DEFAULT_BUTTON_NINE_SLICE_REGIONS)
}

fn char_select_region(name: &str) -> Option<AtlasRegion> {
    lookup_region(name, CHAR_SELECT_REGIONS)
}

fn char_select_grayscale_region(name: &str) -> Option<AtlasRegion> {
    lookup_region(name, CHAR_SELECT_GRAYSCALE_REGIONS)
}

fn common_dropdown_region(name: &str) -> Option<AtlasRegion> {
    lookup_region(name, COMMON_DROPDOWN_REGIONS)
}

fn character_create_region(name: &str) -> Option<AtlasRegion> {
    lookup_region(name, CHARACTER_CREATE_REGIONS)
}

fn action_bar_region(name: &str) -> Option<AtlasRegion> {
    lookup_region(name, ACTION_BAR_REGIONS)
}

fn lookup_region(name: &str, regions: &[AtlasRegionEntry]) -> Option<AtlasRegion> {
    regions
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, region)| *region)
}

type AtlasRegionEntry = (&'static str, AtlasRegion);

const RED_BUTTON_REGIONS: &[AtlasRegionEntry] = &[
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
];

const GLUE_BIG_BUTTON_REGIONS: &[AtlasRegionEntry] = &[
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
];

const DEFAULT_BUTTON_NINE_SLICE_REGIONS: &[AtlasRegionEntry] = &[
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
];

const CHAR_SELECT_REGIONS: &[AtlasRegionEntry] = &[
    (
        "glues-characterselect-card-all-bg",
        atlas_region!(
            CHARACTER_SELECT_GLUES,
            0.932617,
            0.991211,
            0.246094,
            0.304688,
            60.0,
            60.0,
            None
        ),
    ),
    (
        "glues-characterselect-listrealm-bg",
        atlas_region!(
            CHARACTER_SELECT_GLUES,
            0.659180,
            0.933594,
            0.219727,
            0.242188,
            281.0,
            23.0,
            None
        ),
    ),
    (
        "glues-characterselect-card-empty",
        atlas_region!(
            CHARACTER_SELECT_GLUES,
            0.622070,
            0.930664,
            0.246094,
            0.338867,
            316.0,
            95.0,
            None
        ),
    ),
    (
        "glues-characterselect-card-empty-hover",
        atlas_region!(
            CHARACTER_SELECT_GLUES,
            0.000977,
            0.309570,
            0.340820,
            0.433594,
            316.0,
            95.0,
            None
        ),
    ),
    (
        "glues-characterselect-card-singles",
        atlas_region!(
            CHARACTER_SELECT_GLUES,
            0.000977,
            0.303711,
            0.435547,
            0.522461,
            310.0,
            89.0,
            None
        ),
    ),
    (
        "glues-characterselect-card-singles-hover",
        atlas_region!(
            CHARACTER_SELECT_GLUES,
            0.305664,
            0.608398,
            0.435547,
            0.522461,
            310.0,
            89.0,
            None
        ),
    ),
    (
        "glues-characterselect-card-selected",
        atlas_region!(
            CHARACTER_SELECT_GLUES,
            0.340820,
            0.674805,
            0.000977,
            0.120117,
            342.0,
            122.0,
            None
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
        "glues-characterselect-namebg",
        atlas_region!(
            CHARACTER_SELECT_GLUES,
            0.187500,
            0.376953,
            0.562500,
            0.622070,
            194.0,
            61.0,
            None
        ),
    ),
    (
        "glues-characterselect-tophud-left-bg",
        atlas_region!(
            CHARACTER_SELECT_GLUES,
            0.378906,
            0.585938,
            0.562500,
            0.612305,
            212.0,
            51.0,
            None
        ),
    ),
    (
        "glues-characterselect-tophud-middle-bg",
        atlas_region!(
            CHARACTER_SELECT_GLUES,
            0.969727,
            0.999023,
            0.125000,
            0.174805,
            30.0,
            51.0,
            None
        ),
    ),
    (
        "glues-characterselect-tophud-right-bg",
        atlas_region!(
            CHARACTER_SELECT_GLUES,
            0.187500,
            0.394531,
            0.624023,
            0.673828,
            212.0,
            51.0,
            None
        ),
    ),
];

const CHAR_SELECT_GRAYSCALE_REGIONS: &[AtlasRegionEntry] = &[
    (
        "glues-characterselect-gs-tophud-left",
        atlas_region!(
            CHARACTER_SELECT_GLUES_GRAYSCALE,
            0.435547,
            0.865234,
            0.089844,
            0.173828,
            220.0,
            43.0,
            None
        ),
    ),
    (
        "glues-characterselect-gs-tophud-middle",
        atlas_region!(
            CHARACTER_SELECT_GLUES_GRAYSCALE,
            0.705078,
            0.935547,
            0.441406,
            0.525391,
            118.0,
            43.0,
            None
        ),
    ),
    (
        "glues-characterselect-gs-tophud-right",
        atlas_region!(
            CHARACTER_SELECT_GLUES_GRAYSCALE,
            0.001953,
            0.431641,
            0.353516,
            0.437500,
            220.0,
            43.0,
            None
        ),
    ),
    (
        "glues-characterselect-gs-tophud-left-selected",
        atlas_region!(
            CHARACTER_SELECT_GLUES_GRAYSCALE,
            0.435547,
            0.865234,
            0.001953,
            0.085938,
            220.0,
            43.0,
            None
        ),
    ),
    (
        "glues-characterselect-gs-tophud-middle-selected",
        atlas_region!(
            CHARACTER_SELECT_GLUES_GRAYSCALE,
            0.236328,
            0.466797,
            0.441406,
            0.525391,
            118.0,
            43.0,
            None
        ),
    ),
    (
        "glues-characterselect-gs-tophud-right-selected",
        atlas_region!(
            CHARACTER_SELECT_GLUES_GRAYSCALE,
            0.001953,
            0.431641,
            0.265625,
            0.349609,
            220.0,
            43.0,
            None
        ),
    ),
];

const COMMON_DROPDOWN_REGIONS: &[AtlasRegionEntry] = &[
    (
        "common-dropdown-c-button",
        atlas_region!(
            COMMON_DROPDOWN,
            0.001953,
            0.078125,
            0.804688,
            0.957031,
            39.0,
            39.0,
            None
        ),
    ),
    (
        "common-dropdown-icon-back",
        atlas_region!(
            COMMON_DROPDOWN,
            0.955078,
            0.988281,
            0.003906,
            0.070312,
            17.0,
            17.0,
            None
        ),
    ),
    (
        "common-dropdown-icon-next",
        atlas_region!(
            COMMON_DROPDOWN,
            0.605469,
            0.638672,
            0.113281,
            0.179688,
            17.0,
            17.0,
            None
        ),
    ),
];

const CHARACTER_CREATE_REGIONS: &[AtlasRegionEntry] = &[
    (
        "charactercreate-customize-backbutton",
        atlas_region!(
            CHARACTER_CREATE,
            0.841309,
            0.878418,
            0.204590,
            0.241699,
            38.0,
            38.0,
            None
        ),
    ),
    (
        "charactercreate-customize-backbutton-down",
        atlas_region!(
            CHARACTER_CREATE,
            0.918457,
            0.955566,
            0.204590,
            0.241699,
            38.0,
            38.0,
            None
        ),
    ),
    (
        "charactercreate-customize-backbutton-disabled",
        atlas_region!(
            CHARACTER_CREATE,
            0.879395,
            0.916504,
            0.204590,
            0.241699,
            38.0,
            38.0,
            None
        ),
    ),
    (
        "charactercreate-customize-nextbutton",
        atlas_region!(
            CHARACTER_CREATE,
            0.956543,
            0.993652,
            0.204590,
            0.241699,
            38.0,
            38.0,
            None
        ),
    ),
    (
        "charactercreate-customize-nextbutton-down",
        atlas_region!(
            CHARACTER_CREATE,
            0.175293,
            0.212402,
            0.918457,
            0.955566,
            38.0,
            38.0,
            None
        ),
    ),
    (
        "charactercreate-customize-nextbutton-disabled",
        atlas_region!(
            CHARACTER_CREATE,
            0.137207,
            0.174316,
            0.918457,
            0.955566,
            38.0,
            38.0,
            None
        ),
    ),
    (
        "charactercreate-customize-palette",
        atlas_region!(
            CHARACTER_CREATE,
            0.938965,
            0.979980,
            0.104004,
            0.113770,
            42.0,
            10.0,
            None
        ),
    ),
    (
        "charactercreate-customize-palette-selected",
        atlas_region!(
            CHARACTER_CREATE,
            0.888184,
            0.937988,
            0.104004,
            0.123535,
            51.0,
            20.0,
            None
        ),
    ),
];

const ACTION_BAR_REGIONS: &[AtlasRegionEntry] = &[
    (
        "ui-hud-actionbar-iconframe",
        atlas_region!(
            ACTION_BAR, 0.707031, 0.886719, 0.248047, 0.291992, 46.0, 45.0, None
        ),
    ),
    (
        "ui-hud-actionbar-iconframe-addrow",
        atlas_region!(
            ACTION_BAR, 0.707031, 0.906250, 0.297852, 0.347656, 51.0, 51.0, None
        ),
    ),
    (
        "ui-hud-actionbar-iconframe-down",
        atlas_region!(
            ACTION_BAR, 0.707031, 0.886719, 0.508789, 0.552734, 46.0, 45.0, None
        ),
    ),
    (
        "ui-hud-actionbar-iconframe-addrow-down",
        atlas_region!(
            ACTION_BAR, 0.707031, 0.906250, 0.349609, 0.399414, 51.0, 51.0, None
        ),
    ),
    (
        "ui-hud-actionbar-iconframe-mouseover",
        atlas_region!(
            ACTION_BAR, 0.707031, 0.886719, 0.627930, 0.671875, 46.0, 45.0, None
        ),
    ),
    (
        "ui-hud-actionbar-iconframe-border",
        atlas_region!(
            ACTION_BAR, 0.707031, 0.886719, 0.462891, 0.506836, 46.0, 45.0, None
        ),
    ),
    (
        "ui-hud-actionbar-iconframe-flash",
        atlas_region!(
            ACTION_BAR, 0.707031, 0.886719, 0.554688, 0.598633, 46.0, 45.0, None
        ),
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn red_button_up_region_exists() {
        let region = get_region("128-redbutton-up").expect("atlas region");
        assert_eq!(region.path, "data/ui/128BrownButton9Sliced.ktx2");
        assert_eq!(region.width, 470.0);
        assert_eq!(region.height, 128.0);
        assert_eq!(region.nine_slice_edge, Some(16.0));
    }

    #[test]
    fn brown_glue_big_button_region_exists() {
        let region = get_region("glue-bigbutton-brown-up").expect("atlas region");
        assert_eq!(region.path, "data/ui/Glues-BigButton-Brown-Up.ktx2");
        assert_eq!(region.width, 256.0);
        assert_eq!(region.height, 64.0);
        assert_eq!(region.nine_slice_edge, None);
    }

    #[test]
    fn login_generated_regular_region_exists() {
        let region = get_region("defaultbutton-nineslice-up").expect("atlas region");
        assert_eq!(
            region.path,
            "data/ui/login-button-generated-regular-normal.ktx2"
        );
        assert_eq!(region.width, 500.0);
        assert_eq!(region.height, 132.0);
        assert_eq!(region.nine_slice_edge, Some(24.0));
    }

    #[test]
    fn char_select_list_backdrop_has_asymmetric_slice_margins() {
        assert_eq!(
            nine_slice_margins("glues-characterselect-card-all-bg"),
            Some([14.0, 11.0, 14.0, 17.0])
        );
    }

    #[test]
    fn wow_action_bar_regions_exist() {
        let region = get_region("ui-hud-actionbar-iconframe-addrow-down").expect("atlas region");
        assert_eq!(
            region.path,
            "/home/osso/Projects/wow/Interface/HUD/UIActionBar.BLP"
        );
        assert_eq!(region.width, 51.0);
        assert_eq!(region.height, 51.0);
    }

    #[test]
    fn atlas_lookup_is_case_insensitive() {
        let mixed = get_region("UI-HUD-ActionBar-IconFrame-AddRow-Down").expect("atlas region");
        let lower = get_region("ui-hud-actionbar-iconframe-addrow-down").expect("atlas region");
        assert_eq!(mixed.path, lower.path);
        assert_eq!(mixed.width, lower.width);
        assert_eq!(mixed.height, lower.height);
    }

    #[test]
    fn unknown_region_returns_none() {
        assert!(get_region("this-atlas-does-not-exist").is_none());
    }
}
