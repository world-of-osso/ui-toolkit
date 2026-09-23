# ui-toolkit

Registry-authoritative UI frames projected to native Bevy UI. Screens use `rsx!`, `Screen`, and `SharedContext`; the registry owns authored layout, texture, input, visibility, and lifecycle state.

## Hit areas

RSX `hit_rect_insets: "left,right,top,bottom"` adjusts registry hit testing in logical pixels without changing native layout bounds. Positive values shrink the hit area; negative values expand it. Values must be four finite numbers.

## Button highlights

RSX `button_highlight_size: "width,height"` gives the hover overlay an explicit logical-pixel size, centered on the button without changing its layout or hit area. Both dimensions must be finite and positive. Omit the attribute to retain the button-sized overlay. Native UI and legacy sprite projection use the same override; disabled buttons suppress hover as before.

## Atlas sources

`AtlasRegion.source` identifies the backing texture:

- `AtlasSource::File` loads an explicit KTX2/PNG/BLP file.
- `AtlasSource::FileDataId` calls the host `BlpLoader::ensure_texture`, then decodes and caches the resolved BLP.

Character-selection atlas members use `FileDataId(5648070)`, not an absolute WoW install path. DB2 maps `glues-characterselect-card-*` members through `UiTextureAtlasID 2726` to that 1024×1024 atlas. The engine resolves it from the local CASC cache. Character-selection regions are materialized and CPU-decoded so cropped card/panel art preserves transparent-edge pixels.

Character-creation customization arrows and palette regions use `FileDataId(1253496)` (`Interface/GLUES/CHARACTERCREATE/CharacterCreate.BLP`). The host resolves the atlas from local CASC; existing UV regions and artwork are unchanged.

Retail character-creation rings, category/body-type icons, camera controls and dropdown pieces are listed in [`src/atlas/retail.rs`](src/atlas/retail.rs). `atlas::get_name_by_element_id` resolves their DB2 element IDs to supported names; other IDs return `None`. See the [atlas contract](docs/specs/character-creation-atlases.md) for sources, sizing and proof limits.

Adding an atlas: use a file source only for a repository-owned stable asset. Use the authored FileDataID for WoW atlas content; do not add machine-specific install paths or one-off extracted copies.
