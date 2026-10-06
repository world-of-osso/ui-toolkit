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
fn atlas_lookup_is_case_insensitive() {
    let mixed = get_region("128-RedButton-Up").expect("atlas region");
    let lower = get_region("128-redbutton-up").expect("atlas region");
    assert_eq!(mixed, lower);
}

#[test]
fn unknown_region_returns_none() {
    assert!(get_region("this-atlas-does-not-exist").is_none());
}

/// One element drawn by four members: Retail 1x and 2x (set 0), a newer Retail 1x
/// duplicate, and a Forever set-1 1x member.
fn write_tables(dir: &std::path::Path) {
    std::fs::create_dir_all(dir.join("retail")).unwrap();
    std::fs::create_dir_all(dir.join("forever")).unwrap();
    let write = |file: &str, text: &str| std::fs::write(dir.join(file), text).unwrap();
    write(
        "retail/UiTextureAtlas.csv",
        "ID,FileDataID,AtlasWidth,AtlasHeight,UiCanvasID\n10,100,256,128,1\n11,110,512,256,2\n12,120,64,64,1\n",
    );
    write(
        "retail/UiTextureAtlasElement.csv",
        "Name,ID\n\"Glow Frame \",7\n",
    );
    write(
        "retail/UiTextureAtlasMember.csv",
        "CommittedName,ID,UiTextureAtlasID,Width,Height,CommittedLeft,CommittedRight,CommittedTop,CommittedBottom,UiTextureAtlasElementID,OverrideWidth,OverrideHeight,CommittedFlags,UiCanvasID\n\
         a,1,10,32,16,0,32,0,16,7,0,0,0,0\n\
         a,2,11,64,32,0,64,0,32,7,0,0,0,0\n\
         a,3,12,32,16,32,64,16,32,7,16,8,0,0\n",
    );
    write(
        "forever/UiTextureAtlas.csv",
        "ID,FileDataID,UiTextureAtlasSetID,AtlasWidth,AtlasHeight,UiCanvasID\n10,100,0,256,128,1\n20,200,1,128,128,1\n",
    );
    write(
        "forever/UiTextureAtlasElement.csv",
        "Name,ID\n\"Glow Frame \",7\n",
    );
    write(
        "forever/UiTextureAtlasMember.csv",
        "CommittedName,ID,UiTextureAtlasID,Width,Height,CommittedLeft,CommittedRight,CommittedTop,CommittedBottom,UiTextureAtlasElementID,OverrideWidth,OverrideHeight,CommittedFlags,UiCanvasID\n\
         a,1,10,32,16,0,32,0,16,7,0,0,0,0\n\
         a-c60,4,20,32,16,64,96,0,16,7,0,0,0,0\n",
    );
}

#[test]
fn concurrent_thread_atlas_lookups_keep_their_own_skin() {
    use std::sync::Barrier;

    let dir = std::env::temp_dir().join(format!("atlas-thread-skins-{}", std::process::id()));
    write_tables(&dir);
    set_atlas_directories(&dir.join("retail"), &dir.join("forever")).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    let modern_set = Barrier::new(2);
    let both_set = Barrier::new(2);
    std::thread::scope(|scope| {
        let resolve_skin = |skin, expected_fdid| {
            assert_eq!(thread_skin(), ActiveSkin::Modern);
            if skin == ActiveSkin::Modern {
                set_thread_skin(skin);
                modern_set.wait();
            } else {
                modern_set.wait();
                set_thread_skin(skin);
            }
            both_set.wait();
            assert_eq!(thread_skin(), skin);
            assert_eq!(
                get_region("glow frame ").unwrap().source,
                AtlasSource::FileDataId(expected_fdid)
            );
        };
        let modern = scope.spawn(move || resolve_skin(ActiveSkin::Modern, 120));
        let forever = scope.spawn(move || resolve_skin(ActiveSkin::Forever, 200));
        modern.join().unwrap();
        forever.join().unwrap();
    });
    assert_eq!(thread_skin(), ActiveSkin::Modern);
}

#[test]
fn db2_member_selection_prefers_newest_member_on_the_canvas_within_the_skin_set() {
    let dir = std::env::temp_dir().join(format!("atlas-db2-{}", std::process::id()));
    write_tables(&dir);
    let table = db2::AtlasTable::load(&dir.join("retail"), &dir.join("forever")).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();

    let modern = table
        .resolve("glow frame ", ActiveSkin::Modern, UiCanvas::X1)
        .unwrap();
    assert_eq!(modern.source, AtlasSource::FileDataId(120));
    assert_eq!(
        (modern.left, modern.right, modern.top, modern.bottom),
        (0.5, 1.0, 0.25, 0.5)
    );
    assert_eq!((modern.width, modern.height), (16.0, 8.0));

    let forever = table
        .resolve("glow frame ", ActiveSkin::Forever, UiCanvas::X1)
        .unwrap();
    assert_eq!(forever.source, AtlasSource::FileDataId(200));
    assert_eq!(
        (forever.left, forever.right, forever.top, forever.bottom),
        (0.5, 0.75, 0.0, 0.125)
    );
    assert_eq!(table.name_of(7), Some("glow frame "));

    let modern_2x = table
        .resolve("glow frame ", ActiveSkin::Modern, UiCanvas(2))
        .unwrap();
    assert_eq!(modern_2x.source, AtlasSource::FileDataId(110));
    assert_eq!((modern_2x.width, modern_2x.height), (64.0, 32.0));
    // Forever has no 2x set-1 member here: its 1x set-1 member still wins over set 0.
    let forever_2x = table
        .resolve("glow frame ", ActiveSkin::Forever, UiCanvas(2))
        .unwrap();
    assert_eq!(forever_2x.source, AtlasSource::FileDataId(200));
}
