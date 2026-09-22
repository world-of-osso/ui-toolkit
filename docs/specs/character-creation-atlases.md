# Retail character-creation atlas controls

`src/atlas/retail.rs` supplies named artwork for registry-authored character creation. Rendering uses the existing host FileDataID loader; see [atlas source behavior](../../README.md#atlas-sources).

## What it must do

- [x] Resolve the 54 reference control/category atlas names and their `UiTextureAtlasElement.ID` values to identical cropped images.
- [x] Resolve `common-dropdown-icon-back`, `common-dropdown-icon-next` and their `-disabled` states through canonical element IDs `25734`–`25737` and the 1× dropdown sheet FDID `5390329`, preserving each 17×17 crop.
- [x] Preserve exact physical crop bounds and logical override dimensions from the local `UiTextureAtlasElement` → `UiTextureAtlasMember` → `UiTextureAtlas` join.
- [x] Resolve character artwork through FDID `1253496`, common icons through `3487944`, gray square buttons through `3534438`, and dropdown panel pieces through `3575404`.
- [x] Preserve source pixels through the existing atlas crop path, except transparent RGB sanitization already required by that path.
- [x] Return `None` for unsupported element IDs; retain existing named atlas sources unchanged.

## How it works

- [Host loader and atlas sources](../../README.md#atlas-sources).
- Reference templates: local `Blizzard_CharacterCreate`, `Blizzard_CharacterCustomize`, `Blizzard_CustomizationUI`, shared `RingedFrameTemplate.xml`, and `NineSliceLayouts.lua`.
- Public names come from `UiTextureAtlasElement.Name`. Dropdown member `CommittedName` values use old `UI-Frame-CharacterCreateDropdown-*` names; the canonical reference names are `CharacterCreateDropdown-NineSlice-*` (including `_`/`!` edge prefixes).
- Crop sizes are not UI sizes: reference templates explicitly size rings, while some DB2 members override physical dimensions. No uniform atlas-edge thickness is invented; the dropdown panel exposes its nine individual pieces.

## Implementation inventory

- `src/atlas.rs`: named lookup and bounded public element-ID lookup.
- `src/atlas/retail.rs`: five FileDataID sources, pixel rectangles and logical sizes; enabled/disabled dropdown arrows share the element-ID table rather than duplicate file-backed entries.
- `src/atlas/retail_fixture.tsv`: independent golden crop rectangles from local DB2 CSV metadata.
- `src/atlas/retail_tests.rs`: decoded-image fixture exercising production crop output and lookup.

## Tests asserting this spec

- `atlas::retail_tests::retail_character_creation_atlases_crop_authored_pixels_and_keep_logical_sizes`: all 58 crops, complete coordinate-encoded RGBA payloads, transparent pixels, logical dimensions and element-name mapping.

## Known gaps (current cycle)

- Targeted proof at toolkit `3fb0138`: `cargo test --lib atlas:: -- --nocapture` passes 8/8 (54-region crop test plus seven existing atlas regressions). The decoded atlas fixture encodes coordinates and source identity; it is not proprietary artwork.
- Dropdown-arrow follow-up at `8cad72b`: the single exact crop regression passes 1/1 across all 58 entries after failing specifically on the four missing arrows and element IDs. It uses the same decoded-image fixture, not original BLP artwork.
- [ ] Host integration must prove all five original BLP files decode through local CASC and establish rendered screen parity. This slice does not claim those results.

## Out of scope

- Circular icon masking, additive glow blending, category/control authoring and whole-screen layout: this slice supplies exact atlas artwork, not a new rendering capability.
- Full atlas-catalog import: only the reference controls required by this screen are registered.
