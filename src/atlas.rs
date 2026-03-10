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

pub fn get_region(name: &str) -> Option<AtlasRegion> {
    match name.to_ascii_lowercase().as_str() {
        "128-redbutton-up" => Some(AtlasRegion {
            path: "data/ui/128BrownButton9Sliced.ktx2",
            left: 0.001953,
            right: 0.919922,
            top: 0.509766,
            bottom: 0.759766,
            width: 470.0,
            height: 128.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: Some(16.0),
        }),
        "128-redbutton-pressed" => Some(AtlasRegion {
            path: "data/ui/128BrownButton9Sliced.ktx2",
            left: 0.001953,
            right: 0.919922,
            top: 0.255859,
            bottom: 0.505859,
            width: 470.0,
            height: 128.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: Some(16.0),
        }),
        "128-redbutton-disable" => Some(AtlasRegion {
            path: "data/ui/128BrownButton9Sliced.ktx2",
            left: 0.001953,
            right: 0.919922,
            top: 0.001953,
            bottom: 0.251953,
            width: 470.0,
            height: 128.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: Some(16.0),
        }),
        "128-redbutton-highlight" => Some(AtlasRegion {
            path: "data/ui/128BrownButton.ktx2",
            left: 0.001953,
            right: 0.863281,
            top: 0.190918,
            bottom: 0.253418,
            width: 441.0,
            height: 128.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: Some(16.0),
        }),
        "glue-bigbutton-brown-up" => Some(AtlasRegion {
            path: "data/ui/Glues-BigButton-Brown-Up.ktx2",
            left: 0.0,
            right: 1.0,
            top: 0.0,
            bottom: 1.0,
            width: 256.0,
            height: 64.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "glue-bigbutton-brown-down" => Some(AtlasRegion {
            path: "data/ui/Glues-BigButton-Brown-Down.ktx2",
            left: 0.0,
            right: 1.0,
            top: 0.0,
            bottom: 1.0,
            width: 256.0,
            height: 64.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "glue-bigbutton-brown-highlight" => Some(AtlasRegion {
            path: "data/ui/Glues-BigButton-Brown-Highlight.ktx2",
            left: 0.0,
            right: 1.0,
            top: 0.0,
            bottom: 1.0,
            width: 256.0,
            height: 64.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "glue-bigbutton-brown-disable" => Some(AtlasRegion {
            path: "data/ui/Glues-BigButton-Brown-Up.ktx2",
            left: 0.0,
            right: 1.0,
            top: 0.0,
            bottom: 1.0,
            width: 256.0,
            height: 64.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "defaultbutton-nineslice-up" => Some(AtlasRegion {
            path: "data/ui/login-button-generated-regular-normal.ktx2",
            left: 0.0,
            right: 1.0,
            top: 0.0,
            bottom: 1.0,
            width: 500.0,
            height: 132.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: Some(24.0),
        }),
        "defaultbutton-nineslice-pressed" => Some(AtlasRegion {
            path: "data/ui/login-button-generated-regular-pressed.ktx2",
            left: 0.0,
            right: 1.0,
            top: 0.0,
            bottom: 1.0,
            width: 500.0,
            height: 132.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: Some(24.0),
        }),
        "defaultbutton-nineslice-highlight" => Some(AtlasRegion {
            path: "data/ui/login-button-generated-regular-highlight.ktx2",
            left: 0.0,
            right: 1.0,
            top: 0.0,
            bottom: 1.0,
            width: 500.0,
            height: 132.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: Some(24.0),
        }),
        "defaultbutton-nineslice-disabled" => Some(AtlasRegion {
            path: "data/ui/login-button-generated-regular-disabled.ktx2",
            left: 0.0,
            right: 1.0,
            top: 0.0,
            bottom: 1.0,
            width: 500.0,
            height: 132.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: Some(24.0),
        }),
        "glues-characterselect-card-all-bg" => Some(AtlasRegion {
            path: "/home/osso/Projects/wow/Interface/GLUES/CharacterSelect/UICharacterSelectGlues.BLP",
            left: 0.932617,
            right: 0.991211,
            top: 0.246094,
            bottom: 0.304688,
            width: 60.0,
            height: 60.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "glues-characterselect-listrealm-bg" => Some(AtlasRegion {
            path: "/home/osso/Projects/wow/Interface/GLUES/CharacterSelect/UICharacterSelectGlues.BLP",
            left: 0.659180,
            right: 0.933594,
            top: 0.219727,
            bottom: 0.242188,
            width: 281.0,
            height: 23.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "glues-characterselect-card-empty" => Some(AtlasRegion {
            path: "/home/osso/Projects/wow/Interface/GLUES/CharacterSelect/UICharacterSelectGlues.BLP",
            left: 0.622070,
            right: 0.930664,
            top: 0.246094,
            bottom: 0.338867,
            width: 316.0,
            height: 95.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "glues-characterselect-card-empty-hover" => Some(AtlasRegion {
            path: "/home/osso/Projects/wow/Interface/GLUES/CharacterSelect/UICharacterSelectGlues.BLP",
            left: 0.000977,
            right: 0.309570,
            top: 0.340820,
            bottom: 0.433594,
            width: 316.0,
            height: 95.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "glues-characterselect-card-singles" => Some(AtlasRegion {
            path: "/home/osso/Projects/wow/Interface/GLUES/CharacterSelect/UICharacterSelectGlues.BLP",
            left: 0.000977,
            right: 0.303711,
            top: 0.435547,
            bottom: 0.522461,
            width: 310.0,
            height: 89.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "glues-characterselect-card-singles-hover" => Some(AtlasRegion {
            path: "/home/osso/Projects/wow/Interface/GLUES/CharacterSelect/UICharacterSelectGlues.BLP",
            left: 0.305664,
            right: 0.608398,
            top: 0.435547,
            bottom: 0.522461,
            width: 310.0,
            height: 89.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "glues-characterselect-card-selected" => Some(AtlasRegion {
            path: "/home/osso/Projects/wow/Interface/GLUES/CharacterSelect/UICharacterSelectGlues.BLP",
            left: 0.340820,
            right: 0.674805,
            top: 0.000977,
            bottom: 0.120117,
            width: 342.0,
            height: 122.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "glues-characterselect-namebg" => Some(AtlasRegion {
            path: "/home/osso/Projects/wow/Interface/GLUES/CharacterSelect/UICharacterSelectGlues.BLP",
            left: 0.187500,
            right: 0.376953,
            top: 0.562500,
            bottom: 0.622070,
            width: 194.0,
            height: 61.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "glues-characterselect-gs-tophud-left" => Some(AtlasRegion {
            path: "/home/osso/Projects/wow/Interface/GLUES/CharacterSelect/UICharacterSelectGluesGrayscale.BLP",
            left: 0.435547,
            right: 0.865234,
            top: 0.089844,
            bottom: 0.173828,
            width: 220.0,
            height: 43.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "glues-characterselect-gs-tophud-middle" => Some(AtlasRegion {
            path: "/home/osso/Projects/wow/Interface/GLUES/CharacterSelect/UICharacterSelectGluesGrayscale.BLP",
            left: 0.705078,
            right: 0.935547,
            top: 0.441406,
            bottom: 0.525391,
            width: 118.0,
            height: 43.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "glues-characterselect-gs-tophud-right" => Some(AtlasRegion {
            path: "/home/osso/Projects/wow/Interface/GLUES/CharacterSelect/UICharacterSelectGluesGrayscale.BLP",
            left: 0.001953,
            right: 0.431641,
            top: 0.353516,
            bottom: 0.437500,
            width: 220.0,
            height: 43.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "glues-characterselect-gs-tophud-left-selected" => Some(AtlasRegion {
            path: "/home/osso/Projects/wow/Interface/GLUES/CharacterSelect/UICharacterSelectGluesGrayscale.BLP",
            left: 0.435547,
            right: 0.865234,
            top: 0.001953,
            bottom: 0.085938,
            width: 220.0,
            height: 43.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "glues-characterselect-gs-tophud-middle-selected" => Some(AtlasRegion {
            path: "/home/osso/Projects/wow/Interface/GLUES/CharacterSelect/UICharacterSelectGluesGrayscale.BLP",
            left: 0.236328,
            right: 0.466797,
            top: 0.441406,
            bottom: 0.525391,
            width: 118.0,
            height: 43.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        "glues-characterselect-gs-tophud-right-selected" => Some(AtlasRegion {
            path: "/home/osso/Projects/wow/Interface/GLUES/CharacterSelect/UICharacterSelectGluesGrayscale.BLP",
            left: 0.001953,
            right: 0.431641,
            top: 0.265625,
            bottom: 0.349609,
            width: 220.0,
            height: 43.0,
            tiles_horizontally: false,
            tiles_vertically: false,
            nine_slice_edge: None,
        }),
        _ => None,
    }
}

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
}
