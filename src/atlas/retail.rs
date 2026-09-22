//! Retail control crops joined from UiTextureAtlasElement, UiTextureAtlasMember,
//! and UiTextureAtlas. Bounds are physical pixels; sizes retain DB2 overrides.

use super::{AtlasRegion, AtlasSource};

const fn pixels(fdid: u32, sheet: [u32; 2], rect: [u32; 4], size: [u32; 2]) -> AtlasRegion {
    AtlasRegion {
        source: AtlasSource::FileDataId(fdid),
        left: rect[0] as f32 / sheet[0] as f32,
        right: rect[2] as f32 / sheet[0] as f32,
        top: rect[1] as f32 / sheet[1] as f32,
        bottom: rect[3] as f32 / sheet[1] as f32,
        width: size[0] as f32,
        height: size[1] as f32,
        tiles_horizontally: false,
        tiles_vertically: false,
        nine_slice_edge: None,
    }
}

pub(super) const REGIONS: &[(u32, &str, AtlasRegion)] = &[
    (
        11949,
        "128-redbutton-left",
        pixels(7367529, [512, 2048], [391, 911, 505, 1039], [114, 128]),
    ),
    (
        11948,
        "128-redbutton-left-pressed",
        pixels(7367529, [512, 2048], [391, 1171, 505, 1299], [114, 128]),
    ),
    (
        11947,
        "128-redbutton-left-disabled",
        pixels(7367529, [512, 2048], [391, 1041, 505, 1169], [114, 128]),
    ),
    (
        11955,
        "_128-redbutton-center",
        pixels(7367529, [512, 2048], [0, 1, 64, 129], [64, 128]),
    ),
    (
        11954,
        "_128-redbutton-center-pressed",
        pixels(7367529, [512, 2048], [0, 261, 64, 389], [64, 128]),
    ),
    (
        11953,
        "_128-redbutton-center-disabled",
        pixels(7367529, [512, 2048], [0, 131, 64, 259], [64, 128]),
    ),
    (
        11952,
        "128-redbutton-right",
        pixels(7367529, [512, 2048], [1, 521, 293, 649], [292, 128]),
    ),
    (
        11951,
        "128-redbutton-right-pressed",
        pixels(7367529, [512, 2048], [1, 781, 293, 909], [292, 128]),
    ),
    (
        11950,
        "128-redbutton-right-disabled",
        pixels(7367529, [512, 2048], [1, 651, 293, 779], [292, 128]),
    ),
    (
        12706,
        "!charactercreatedropdown-nineslice-edgeleft",
        pixels(3575404, [1024, 512], [253, 1, 377, 409], [62, 204]),
    ),
    (
        12707,
        "!charactercreatedropdown-nineslice-edgeright",
        pixels(3575404, [1024, 512], [127, 1, 251, 409], [62, 204]),
    ),
    (
        12713,
        "_charactercreatedropdown-nineslice-edgebottom",
        pixels(3575404, [1024, 512], [379, 1, 811, 145], [216, 72]),
    ),
    (
        12714,
        "_charactercreatedropdown-nineslice-edgetop",
        pixels(3575404, [1024, 512], [379, 147, 811, 251], [216, 52]),
    ),
    (
        21086,
        "charactercreate-customize-dropdown-icon-lock",
        pixels(1253496, [2048, 2048], [389, 1777, 419, 1820], [8, 12]),
    ),
    (
        12618,
        "charactercreate-customize-dropdown-linemouseover-middle",
        pixels(1253496, [2048, 2048], [2043, 1, 2044, 41], [1, 20]),
    ),
    (
        12619,
        "charactercreate-customize-dropdown-linemouseover-side",
        pixels(1253496, [2048, 2048], [2029, 1, 2041, 41], [6, 20]),
    ),
    (
        17859,
        "charactercreate-customize-dropdown-newtagglow",
        pixels(1253496, [2048, 2048], [1149, 455, 1189, 495], [40, 40]),
    ),
    (
        12616,
        "charactercreate-customize-dropdownbox",
        pixels(1253496, [2048, 2048], [439, 497, 739, 573], [150, 38]),
    ),
    (
        12629,
        "charactercreate-customize-dropdownbox-hover",
        pixels(1253496, [2048, 2048], [741, 497, 1041, 573], [150, 38]),
    ),
    (
        12630,
        "charactercreate-customize-dropdownbox-open",
        pixels(1253496, [2048, 2048], [1043, 497, 1343, 573], [150, 38]),
    ),
    (
        16265,
        "charactercreate-customize-palette-glow",
        pixels(1253496, [2048, 2048], [1923, 235, 2007, 255], [42, 10]),
    ),
    (
        16304,
        "charactercreate-customize-palette-half",
        pixels(1253496, [2048, 2048], [519, 471, 603, 491], [42, 10]),
    ),
    (
        12072,
        "charactercreate-gendericon-female",
        pixels(1253496, [2048, 2048], [281, 1, 537, 257], [256, 256]),
    ),
    (
        16965,
        "charactercreate-gendericon-female-selected",
        pixels(1253496, [2048, 2048], [539, 1, 795, 257], [256, 256]),
    ),
    (
        12071,
        "charactercreate-gendericon-male",
        pixels(1253496, [2048, 2048], [797, 1, 1053, 257], [256, 256]),
    ),
    (
        16966,
        "charactercreate-gendericon-male-selected",
        pixels(1253496, [2048, 2048], [1055, 1, 1311, 257], [256, 256]),
    ),
    (
        11983,
        "charactercreate-icon-alliance",
        pixels(1253496, [2048, 2048], [1351, 259, 1535, 459], [92, 100]),
    ),
    (
        11985,
        "charactercreate-icon-customize-accessories",
        pixels(1253496, [2048, 2048], [1723, 259, 1879, 417], [104, 105]),
    ),
    (
        11984,
        "charactercreate-icon-customize-accessories-selected",
        pixels(1253496, [2048, 2048], [1881, 259, 2037, 417], [104, 105]),
    ),
    (
        11987,
        "charactercreate-icon-customize-body",
        pixels(1253496, [2048, 2048], [281, 497, 437, 655], [104, 105]),
    ),
    (
        11986,
        "charactercreate-icon-customize-body-selected",
        pixels(1253496, [2048, 2048], [281, 657, 437, 815], [104, 105]),
    ),
    (
        11989,
        "charactercreate-icon-customize-hair",
        pixels(1253496, [2048, 2048], [1819, 1, 2027, 211], [104, 105]),
    ),
    (
        11988,
        "charactercreate-icon-customize-hair-selected",
        pixels(1253496, [2048, 2048], [519, 259, 727, 469], [104, 105]),
    ),
    (
        11991,
        "charactercreate-icon-customize-head",
        pixels(1253496, [2048, 2048], [281, 817, 437, 975], [104, 105]),
    ),
    (
        11990,
        "charactercreate-icon-customize-head-selected",
        pixels(1253496, [2048, 2048], [281, 977, 437, 1135], [104, 105]),
    ),
    (
        18308,
        "charactercreate-icon-customize-mirror",
        pixels(1253496, [2048, 2048], [281, 1137, 437, 1295], [156, 158]),
    ),
    (
        18307,
        "charactercreate-icon-customize-mirror-selected",
        pixels(1253496, [2048, 2048], [281, 1297, 437, 1455], [156, 158]),
    ),
    (
        11993,
        "charactercreate-icon-customize-torso",
        pixels(1253496, [2048, 2048], [729, 259, 937, 469], [104, 105]),
    ),
    (
        11992,
        "charactercreate-icon-customize-torso-selected",
        pixels(1253496, [2048, 2048], [939, 259, 1147, 469], [104, 105]),
    ),
    (
        11994,
        "charactercreate-icon-dice",
        pixels(1253496, [2048, 2048], [1313, 1, 1569, 257], [24, 24]),
    ),
    (
        11995,
        "charactercreate-icon-horde",
        pixels(1253496, [2048, 2048], [1537, 259, 1721, 459], [92, 100]),
    ),
    (
        20111,
        "charactercreate-icon-requiredarrow",
        pixels(1253496, [2048, 2048], [439, 575, 519, 641], [40, 33]),
    ),
    (
        11996,
        "charactercreate-ring-alliance",
        pixels(1253496, [2048, 2048], [1, 1, 279, 281], [278, 280]),
    ),
    (
        12081,
        "charactercreate-ring-alliance-disabled",
        pixels(1253496, [2048, 2048], [1, 283, 279, 563], [278, 280]),
    ),
    (
        11997,
        "charactercreate-ring-customizebackground",
        pixels(1253496, [2048, 2048], [1571, 1, 1817, 247], [123, 123]),
    ),
    (
        11998,
        "charactercreate-ring-horde",
        pixels(1253496, [2048, 2048], [1, 565, 279, 845], [278, 280]),
    ),
    (
        12083,
        "charactercreate-ring-horde-disabled",
        pixels(1253496, [2048, 2048], [1, 847, 279, 1127], [278, 280]),
    ),
    (
        11999,
        "charactercreate-ring-metaldark",
        pixels(1253496, [2048, 2048], [1, 1129, 279, 1409], [278, 280]),
    ),
    (
        12082,
        "charactercreate-ring-metaldark-disabled",
        pixels(1253496, [2048, 2048], [1, 1411, 279, 1691], [278, 280]),
    ),
    (
        12000,
        "charactercreate-ring-metallight",
        pixels(1253496, [2048, 2048], [1, 1693, 279, 1973], [278, 280]),
    ),
    (
        12001,
        "charactercreate-ring-select",
        pixels(1253496, [2048, 2048], [281, 259, 517, 495], [236, 236]),
    ),
    (
        12708,
        "charactercreatedropdown-nineslice-center",
        pixels(3575404, [1024, 512], [1, 399, 2, 400], [1, 1]),
    ),
    (
        12709,
        "charactercreatedropdown-nineslice-cornerbottomleft",
        pixels(3575404, [1024, 512], [1, 1, 125, 145], [62, 72]),
    ),
    (
        12710,
        "charactercreatedropdown-nineslice-cornerbottomright",
        pixels(3575404, [1024, 512], [1, 253, 125, 397], [62, 72]),
    ),
    (
        12711,
        "charactercreatedropdown-nineslice-cornertopleft",
        pixels(3575404, [1024, 512], [1, 147, 125, 251], [62, 52]),
    ),
    (
        12712,
        "charactercreatedropdown-nineslice-cornertopright",
        pixels(3575404, [1024, 512], [813, 1, 937, 105], [62, 52]),
    ),
    (
        12242,
        "common-button-square-gray-down",
        pixels(3534438, [1024, 1024], [1, 291, 257, 547], [42, 42]),
    ),
    (
        12243,
        "common-button-square-gray-up",
        pixels(3534438, [1024, 1024], [1, 549, 257, 805], [42, 42]),
    ),
    (
        25735,
        "common-dropdown-icon-back",
        pixels(5390329, [512, 256], [489, 1, 506, 18], [17, 17]),
    ),
    (
        25734,
        "common-dropdown-icon-back-disabled",
        pixels(5390329, [512, 256], [291, 29, 308, 46], [17, 17]),
    ),
    (
        25737,
        "common-dropdown-icon-next",
        pixels(5390329, [512, 256], [310, 29, 327, 46], [17, 17]),
    ),
    (
        25736,
        "common-dropdown-icon-next-disabled",
        pixels(5390329, [512, 256], [329, 29, 346, 46], [17, 17]),
    ),
    (
        11967,
        "common-icon-rotateleft",
        pixels(3487944, [2048, 1024], [259, 775, 359, 875], [20, 20]),
    ),
    (
        11968,
        "common-icon-rotateright",
        pixels(3487944, [2048, 1024], [259, 877, 359, 977], [20, 20]),
    ),
    (
        11969,
        "common-icon-undo",
        pixels(3487944, [2048, 1024], [775, 259, 1031, 515], [25, 25]),
    ),
    (
        11970,
        "common-icon-zoomin",
        pixels(3487944, [2048, 1024], [1033, 1, 1289, 257], [25, 25]),
    ),
    (
        16958,
        "common-icon-zoomin-disable",
        pixels(3487944, [2048, 1024], [1291, 1, 1547, 257], [25, 25]),
    ),
    (
        11971,
        "common-icon-zoomout",
        pixels(3487944, [2048, 1024], [1549, 1, 1805, 257], [25, 25]),
    ),
    (
        16959,
        "common-icon-zoomout-disable",
        pixels(3487944, [2048, 1024], [1033, 259, 1289, 515], [25, 25]),
    ),
];
