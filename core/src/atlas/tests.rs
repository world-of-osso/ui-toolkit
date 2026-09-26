use super::*;

#[test]
fn atlas_pixel_rectangle_uses_actual_image_dimensions() {
    let region = get_region("128-redbutton-up").unwrap();
    let rect = region.rect_pixels(512, 256);
    assert!((rect.min[0] - 1.0).abs() < 0.001);
    assert!((rect.min[1] - 130.5).abs() < 0.001);
    assert!((rect.max[0] - 471.0).abs() < 0.001);
    assert!((rect.max[1] - 194.5).abs() < 0.001);
}

#[test]
fn red_button_up_region_exists() {
    let region = get_region("128-redbutton-up").expect("atlas region");
    assert_eq!(
        region.source,
        AtlasSource::File("data/ui/128BrownButton9Sliced.ktx2")
    );
    assert_eq!(region.width, 470.0);
    assert_eq!(region.height, 128.0);
    assert_eq!(region.nine_slice_edge, Some(16.0));
}

#[test]
fn brown_glue_big_button_region_exists() {
    let region = get_region("glue-bigbutton-brown-up").expect("atlas region");
    assert_eq!(
        region.source,
        AtlasSource::File("data/ui/Glues-BigButton-Brown-Up.ktx2")
    );
    assert_eq!(region.width, 256.0);
    assert_eq!(region.height, 64.0);
    assert_eq!(region.nine_slice_edge, None);
}

#[test]
fn login_generated_regular_region_exists() {
    let region = get_region("defaultbutton-nineslice-up").expect("atlas region");
    assert_eq!(
        region.source,
        AtlasSource::File("data/ui/login-button-generated-regular-normal.ktx2")
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
        region.source,
        AtlasSource::File("/home/osso/Projects/wow/Interface/HUD/UIActionBar.BLP")
    );
    assert_eq!(region.width, 51.0);
    assert_eq!(region.height, 51.0);
}

#[test]
fn atlas_lookup_is_case_insensitive() {
    let mixed = get_region("UI-HUD-ActionBar-IconFrame-AddRow-Down").expect("atlas region");
    let lower = get_region("ui-hud-actionbar-iconframe-addrow-down").expect("atlas region");
    assert_eq!(mixed.source, lower.source);
    assert_eq!(mixed.width, lower.width);
    assert_eq!(mixed.height, lower.height);
}

#[test]
fn unknown_region_returns_none() {
    assert!(get_region("this-atlas-does-not-exist").is_none());
}
