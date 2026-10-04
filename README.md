# ui-toolkit

Registry-authoritative UI frames projected to native Bevy UI. Screens use `rsx!`, `Screen`, and `SharedContext`; the registry owns authored layout, texture, input, visibility, and lifecycle state.

## Native-only rendering contract

Native Bevy UI is the only root-crate projection. `UiPlugin` already scheduled `native_render::sync_registry`; this retirement removes the unused public Sprite/Text2d projection code, components and stale scheduling enum slots, not a second supported renderer. No legacy compatibility path remains. See [native projection](docs/specs/native-projection.md) for the contract and current inventory.

`render` retains camera setup, frame ordering, shared geometry and `LoadedTexture`; border, nine-/three-slice and tiled modules retain native geometry/source helpers. `render_button` retains button preparation and source selection; `render_text` and `render_text_fx` retain native text properties, layout and outline offsets. Module names do not imply Sprite/Text2d support.

## Portable core integration

`core/` contains Bevy-free `ui-toolkit-core`: shared frame/attribute/registry/screen models, widget wrappers and DB2 atlas skin/canvas resolution. The root crate retains the Bevy projection; Godot projection is engine-owned. Merge `96dbda4` integrated `godot-conversion` and ancestor `testinfra-godot`, including nine-slice attributes and font-directory support. This merge does not extend historical test or runtime proof to the integrated engine.

Canonical engine manifests `game-engine/godot/{rust,ui-model}/Cargo.toml` use `package = "ui-toolkit-core", path = "../../../ui-toolkit/core"`, resolving to `/syncthing/Sync/Projects/world-of-osso/ui-toolkit/core`. Engine build-helper override: `DEPOT_SIBLING_UI_TOOLKIT`. Historical `ui-toolkit-godot-conversion` paths identify earlier evidence only.

## Runtime texture host contract

`TextureSource::Dynamic(DynamicTextureId)` stores a portable ID owned by its `FrameRegistry`, not a Bevy `Handle<Image>`. Register row-major RGBA8 pixels with `registry.create_dynamic_texture(width, height, rgba8)`, replace them with `update_dynamic_texture(id, width, height, rgba8)`, and remove them with `remove_dynamic_texture(id)`. Dimensions must be nonzero and the buffer exactly `width * height * 4` bytes. Updates keep the ID; changed pixels/dimensions and removal mark referencing frames render-dirty. Identical updates leave them unchanged.

The Bevy host in `src/render_texture.rs` owns cached image handles and projects registry pixels into `Rgba8UnormSrgb` images. Shared references reuse a handle; changed pixels or dimensions replace its image in place. A removed ID drops its cache entry and no longer resolves to a projected texture. This does not promise immediate removal of the asset from `Assets<Image>`.

`load_texture_source` and `load_texture_source_pub` require `&FrameRegistry` after the source argument. Native images, nine-/three-slices and button highlights pass the owning registry through their loader calls. Engine handles remain host-owned; there is no Handle-valued dynamic source compatibility path.

### Integration proof boundary

Historical bridge repairs: `cb8f907` and `92c7937`. Reported targeted source proof covers native shared pixels, idle stability, resize and removal, plus legacy sprite pixel updates (2 tests passed); affected fixtures reported 46 passed and 1 failed because DB2 atlas tables were uninitialized. Hotreload also reported passing. Follow-up `a69d62f` initializes self-contained atlas CSV rows in that fixture; its presence alone is not passing proof. Those legacy sprite results describe retired code, not a supported alternate path. Independent whole-root verification remains pending. These results do not extend historical engine/runtime proof or establish rendered screen parity.

Native retirement source revisions: `4026362`, `911e826`, `e682b5f` and `9ce6b0e`. Preservation fixtures at `24502b8` reported 7 passing development cases using actual native layout, images and text. Integration-suite migration and repairs for external-text reconciliation and unowned font weight remain in progress; this documentation records no final new gate pass. Existing portable-core proof (142 tests) is unaffected. Existing engine UI (31 tests), helper (34 tests) and bounded offscreen UI/IPC fixture evidence retain their original scopes; none proves clean-resource shutdown (known leaks) or full Skyborn support (blocked).

## Hit areas

RSX `hit_rect_insets: "left,right,top,bottom"` adjusts registry hit testing in logical pixels without changing native layout bounds. Positive values shrink the hit area; negative values expand it. Values must be four finite numbers.

## Button highlights

RSX `button_highlight_size: "width,height"` gives the hover overlay an explicit logical-pixel size, centered on the button without changing its layout or hit area. Both dimensions must be finite and positive. Omit the attribute to retain the button-sized overlay. Native UI uses the override; disabled buttons suppress hover as before.

## Shared widgets

`ui-toolkit-core` reuses `widgets::toggle::toggle_widget(ToggleWidget { .. })` from the original toolkit. Only the inactive segment emits its action; updating the selected side rebuilds the labels and active segment. Native rendering remains host-owned.

- `widgets::scroll_list::scroll_list(ctx, ScrollList { .. }, row)` builds only viewport rows (`{name}Row{index}`), snapped to whole rows. Positions live in `FrameRegistry::scroll_lists` keyed by list name, so they survive rebuilds and screen recreation; a `Screen` that built a list rebuilds when its position changes. `UiPlugin` handles wheel over the hovered list, dragging `{name}ScrollThumb`, and PgUp/PgDn/Home/End while the list holds `UiState::focused_frame` (set by clicking the list).
- `widgets::tabs::tab_strip(TabStrip { .. })` emits `{name}Tab{index}` buttons whose onclick is the tab action. Disabled tabs get an empty onclick and the disabled button state.
- `widgets::state_panel::state_panel(name, PanelState::..)` fills its parent with Loading (animated dots), Empty, Error (optional `{name}Retry` onclick) or Unavailable (default "Not available yet").

## Atlas sources

`AtlasRegion.source` identifies the backing texture:

- `AtlasSource::File` loads an explicit KTX2/PNG/BLP file.
- `AtlasSource::FileDataId` calls the host `BlpLoader::ensure_texture`, then decodes and caches the resolved BLP.

WoW atlas names resolve from the host's DB2 CSV exports (`UiTextureAtlas`, `UiTextureAtlasElement`, `UiTextureAtlasMember`), loaded by the host before its first DB2 lookup with `atlas::set_atlas_directories(retail_dir, forever_dir)`. The call returns `Result<(), String>`; the host must handle initialization errors. Subsequent calls must specify the same directories. Retail members are `UiTextureAtlasSetID` 0; the Forever export contributes its set-1 (`*c60`) members for the same names. `atlas::set_active_skin(ActiveSkin::Modern | ActiveSkin::Forever)` picks the sets: Modern resolves set 0, Forever set 1 then set 0, as Forever's client does. Within a set the member on `atlas::ATLAS_CANVAS` (`UiCanvas::X1`, `UiCanvasID` 1) wins, then the lowest canvas, then the newest member ID; that constant is the single place to switch to 2x. Sizes keep DB2 `OverrideWidth`/`OverrideHeight`. `atlas::resolve_region(name, skin)` resolves under an explicit skin; `get_region(name)` under the active one. `atlas::get_name_by_element_id` returns the lower-cased Retail element name. Project art and DB2 names are disjoint sources: a name that is neither resolves to `None` (the host reports it as an unknown atlas). Before the tables are loaded only project art resolves.

Project art (`PROJECT_REGIONS` in `core/src/atlas.rs`) wins over a DB2 member of the same name: the brown `128-redbutton-*` buttons, `glue-bigbutton-brown-*`, `defaultbutton-nineslice-*`, `custom-nameplate-bg`, and Retail's red highlight under `retail-128-redbutton-highlight`.

`AtlasRegion::rect_pixels(image_width, image_height)` takes backing-image dimensions and returns portable `PixelRect { min: [x, y], max: [x, y] }` corners. The Bevy host converts these with `Rect::from_corners(Vec2::from_array(rect.min), Vec2::from_array(rect.max))`; logical `AtlasRegion.width`/`height` remain separate from physical crop bounds. FileDataID atlas regions are materialized as cropped images by the host.

Adding an atlas: use a file source only for a repository-owned stable asset. Use the authored FileDataID for WoW atlas content; do not add machine-specific install paths or one-off extracted copies.
