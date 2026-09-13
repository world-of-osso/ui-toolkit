# ui-toolkit

Registry-authoritative UI frames projected to native Bevy UI. Screens use `rsx!`, `Screen`, and `SharedContext`; the registry owns authored layout, texture, input, visibility, and lifecycle state.

## Atlas sources

`AtlasRegion.source` identifies the backing texture:

- `AtlasSource::File` loads an explicit KTX2/PNG/BLP file.
- `AtlasSource::FileDataId` calls the host `BlpLoader::ensure_texture`, then decodes and caches the resolved BLP.

Character-selection atlas members use `FileDataId(5648070)`, not an absolute WoW install path. DB2 maps `glues-characterselect-card-*` members through `UiTextureAtlasID 2726` to that 1024×1024 atlas. The engine resolves it from the local CASC cache. Character-selection regions are materialized and CPU-decoded so cropped card/panel art preserves transparent-edge pixels.

Adding an atlas: use a file source only for a repository-owned stable asset. Use the authored FileDataID for WoW atlas content; do not add machine-specific install paths or one-off extracted copies.
