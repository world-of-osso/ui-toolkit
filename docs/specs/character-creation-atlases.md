# Retail character-creation atlas controls

`src/atlas/retail.rs` supplies named artwork for registry-authored character creation. Rendering uses the existing host FileDataID loader; see [atlas source behavior](../../README.md#atlas-sources).

## What it must do

- [x] Resolve 72 reference control/category atlas entries and their `UiTextureAtlasElement.ID` values to identical cropped images, including Mirror (`18308`/`18307`) and all red three-slice states plus highlight.
- [x] Resolve `common-dropdown-icon-back`, `common-dropdown-icon-next` and their `-disabled` states through canonical element IDs `25734`–`25737` and the 1× dropdown sheet FDID `5390329`, preserving each 17×17 crop.
- [x] Resolve menu-style-2 background `common-dropdown-c-bg` (ID `25590`, 90×90 crop) and the dropdown choice hover art `common-dropdown-customize-mouseover` (ID `25927`, 20×20 crop) from FDID `5390329`.
- [x] Preserve exact physical crop bounds and logical override dimensions from the local `UiTextureAtlasElement` → `UiTextureAtlasMember` → `UiTextureAtlas` join.
- [x] Resolve character artwork through FDID `1253496`, common icons through `3487944`, gray square buttons through `3534438`, dropdown panel pieces through `3575404`, and current Retail red-button slices through `7367529`.
- [x] Preserve source pixels through the existing atlas crop path, except transparent RGB sanitization already required by that path.
- [x] Return `None` for unsupported element IDs; retain existing named atlas sources unchanged.

## How it works

- [Host loader and atlas sources](../../README.md#atlas-sources).
- Reference templates: local `Blizzard_CharacterCreate`, `Blizzard_CharacterCustomize`, `Blizzard_CustomizationUI`, shared `RingedFrameTemplate.xml`, and `NineSliceLayouts.lua`.
- Public names come from `UiTextureAtlasElement.Name` except Retail `128-RedButton-Highlight` (ID `5451`): its canonical name collides with a brown project-owned lookup, so its public key is `retail-128-redbutton-highlight`. The nine left/center/right state names stay canonical and the existing brown highlight lookup is unchanged. Dropdown member `CommittedName` values use old `UI-Frame-CharacterCreateDropdown-*` names; the canonical reference names are `CharacterCreateDropdown-NineSlice-*` (including `_`/`!` edge prefixes).
| Element ID | Canonical DB2 name | Public Retail name |
| --- | --- | --- |
| 5451 | `128-RedButton-Highlight` | `retail-128-redbutton-highlight` |

- Crop sizes are not UI sizes: reference templates explicitly size rings, while some DB2 members override physical dimensions. The red navigation button's authored left/center/right widths are `114`/`64`/`292` at height `128`; its template scales both caps by button height and stretches the center. No uniform atlas-edge thickness is invented. The menu-style-2 popup uses the single authored `common-dropdown-c-bg`; older character-create dropdown nine-slice pieces remain available for their original consumers.

## Implementation inventory

- `src/atlas.rs`: named lookup and bounded public element-ID lookup.
- `src/atlas/retail.rs`: six FileDataID sources, pixel rectangles and logical sizes; enabled/disabled dropdown arrows share the element-ID table rather than duplicate file-backed entries.
- `src/atlas/retail_fixture.tsv`: independent golden crop rectangles from local DB2 CSV metadata.
- `src/atlas/retail_tests.rs`: decoded-image fixture exercising production crop output and lookup.

## Tests asserting this spec

- `atlas::retail_tests::retail_character_creation_atlases_crop_authored_pixels_and_keep_logical_sizes`: all 72 registered crops, complete coordinate-encoded RGBA payloads, transparent pixels, logical dimensions and element-ID/public-name mapping.

## Known gaps (current cycle)

- Targeted proof at toolkit `3fb0138`: `cargo test --lib atlas:: -- --nocapture` passes 8/8 (54-region crop test plus seven existing atlas regressions). The decoded atlas fixture encodes coordinates and source identity; it is not proprietary artwork.
- Dropdown-arrow follow-up at `8cad72b`: the single exact crop regression passes 1/1 across all 58 entries after failing specifically on the four missing arrows and element IDs. It uses the same decoded-image fixture, not original BLP artwork.
- [x] Retail highlight ID `5451` resolves as `retail-128-redbutton-highlight` to FDID `7367529`; legacy `128-redbutton-highlight` keeps its brown source. No fallback or lookup precedence change.
- [ ] Host integration must prove all six original BLP files decode through local CASC and establish rendered screen parity. This slice does not claim those results.

## Out of scope

- Circular icon masking, additive glow blending, category/control authoring and whole-screen layout: this slice supplies exact atlas artwork, not a new rendering capability.
- Full atlas-catalog import: only the reference controls required by this screen are registered.
