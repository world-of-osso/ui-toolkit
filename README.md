# ui-toolkit

Registry-authoritative UI frames projected to native Bevy UI. Screens use `rsx!`, `Screen`, and `SharedContext`; the registry owns authored layout, texture, input, visibility, and lifecycle state.

## Portable core integration

`core/` contains Bevy-free `ui-toolkit-core`: shared frame/attribute/registry/screen models, widget wrappers and DB2 atlas skin/canvas resolution. The root crate retains the Bevy projection; Godot projection is engine-owned. Merge `96dbda4` integrated `godot-conversion` and ancestor `testinfra-godot`, including nine-slice attributes and font-directory support. This merge does not extend historical test or runtime proof to the integrated engine.

Canonical engine manifests `game-engine/godot/{rust,ui-model}/Cargo.toml` use `package = "ui-toolkit-core", path = "../../../ui-toolkit/core"`, resolving to `/syncthing/Sync/Projects/world-of-osso/ui-toolkit/core`. Engine build-helper override: `DEPOT_SIBLING_UI_TOOLKIT`. Historical `ui-toolkit-godot-conversion` paths identify earlier evidence only.

## Hit areas

RSX `hit_rect_insets: "left,right,top,bottom"` adjusts registry hit testing in logical pixels without changing native layout bounds. Positive values shrink the hit area; negative values expand it. Values must be four finite numbers.

## Button highlights

RSX `button_highlight_size: "width,height"` gives the hover overlay an explicit logical-pixel size, centered on the button without changing its layout or hit area. Both dimensions must be finite and positive. Omit the attribute to retain the button-sized overlay. Native UI and legacy sprite projection use the same override; disabled buttons suppress hover as before.

## Shared widgets

`ui-toolkit-core` reuses `widgets::toggle::toggle_widget(ToggleWidget { .. })` from the original toolkit. Only the inactive segment emits its action; updating the selected side rebuilds the labels and active segment. Native rendering remains host-owned.

- `widgets::scroll_list::scroll_list(ctx, ScrollList { .. }, row)` builds only viewport rows (`{name}Row{index}`), snapped to whole rows. Positions live in `FrameRegistry::scroll_lists` keyed by list name, so they survive rebuilds and screen recreation; a `Screen` that built a list rebuilds when its position changes. `UiPlugin` handles wheel over the hovered list, dragging `{name}ScrollThumb`, and PgUp/PgDn/Home/End while the list holds `UiState::focused_frame` (set by clicking the list).
- `widgets::tabs::tab_strip(TabStrip { .. })` emits `{name}Tab{index}` buttons whose onclick is the tab action. Disabled tabs get an empty onclick and the disabled button state.
- `widgets::state_panel::state_panel(name, PanelState::..)` fills its parent with Loading (animated dots), Empty, Error (optional `{name}Retry` onclick) or Unavailable (default "Not available yet").

## Atlas sources

`AtlasRegion.source` identifies the backing texture:

- `AtlasSource::File` loads an explicit KTX2/PNG/BLP file.
- `AtlasSource::FileDataId` calls the host `BlpLoader::ensure_texture`, then decodes and caches the resolved BLP.

WoW atlas names resolve from the host's DB2 CSV exports (`UiTextureAtlas`, `UiTextureAtlasElement`, `UiTextureAtlasMember`), loaded once by `atlas::set_atlas_directories(retail_dir, forever_dir)`. Retail members are `UiTextureAtlasSetID` 0; the Forever export contributes its set-1 (`*c60`) members for the same names. `atlas::set_active_skin(ActiveSkin::Modern | ActiveSkin::Forever)` picks the sets: Modern resolves set 0, Forever set 1 then set 0, as Forever's client does. Within a set the member on `atlas::ATLAS_CANVAS` (`UiCanvas::X1`, `UiCanvasID` 1) wins, then the lowest canvas, then the newest member ID; that constant is the single place to switch to 2x. Sizes keep DB2 `OverrideWidth`/`OverrideHeight`. `atlas::resolve_region(name, skin)` resolves under an explicit skin; `get_region(name)` under the active one. `atlas::get_name_by_element_id` returns the lower-cased Retail element name. Project art and DB2 names are disjoint sources: a name that is neither resolves to `None` (the host reports it as an unknown atlas). Before the tables are loaded only project art resolves.

Project art (`PROJECT_REGIONS` in `core/src/atlas.rs`) wins over a DB2 member of the same name: the brown `128-redbutton-*` buttons, `glue-bigbutton-brown-*`, `defaultbutton-nineslice-*`, `custom-nameplate-bg`, and Retail's red highlight under `retail-128-redbutton-highlight`.

Adding an atlas: use a file source only for a repository-owned stable asset. Use the authored FileDataID for WoW atlas content; do not add machine-specific install paths or one-off extracted copies.
