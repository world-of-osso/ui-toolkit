# Native-only Bevy projection

The root crate projects its authoritative frame registry into native Bevy UI only. `src/plugin.rs` schedules `src/native_render/mod.rs`; the user-selected contract excludes legacy Sprite/Text2d projection. See [toolkit rendering](../../README.md#native-only-rendering-contract).

## What it must do

- [ ] Expose no legacy Sprite/Text2d projection systems, components or obsolete scheduling enum slots; retain native projection as the only root-crate renderer.
- [ ] Preserve native solid fills, borders/backdrops, nine-/three-slices, tiling, button states/highlights and text/shadow/outline behavior through registry updates, including updates after idle.
- [ ] Preserve native camera/order preparation, registry-owned texture sources and button preparation rather than deleting helpers still consumed by native rendering.
- [ ] Reconcile native entities with registry visibility and removal; frame ownership and authored properties remain registry-owned.

These bullets await the final native-only gate. Development preservation evidence is not whole-root acceptance.

## How it works

- [Registry projection and proof boundaries](../../README.md)

## Implementation inventory

- `src/plugin.rs` — native synchronization schedule; `UiRenderSet::{Prepare, Project}`.
- `src/native_render/{mod,layout,images,text,caret}.rs` — native reconciliation, layout bounds, visuals, text and carets.
- `src/render.rs` — camera, shared ordering/geometry and `LoadedTexture`.
- `src/render_{border,nine_slice,three_slice,tiled}.rs` and `src/render/backdrop.rs` — retained geometry/source helpers, not legacy entity systems.
- `src/render_button.rs`, `src/render_texture.rs`, `src/render_text.rs`, `src/render_text_fx.rs` — button preparation/source selection, texture loading and native text/layout/offset helpers.

## Tests asserting this spec

- `src/native_render/native_only_tests.rs` — real native layout/image/text preservation; 7 reported development passes at `24502b8`.
- `src/native_render/{tests,images_tests}.rs` — native entity lifecycle and visual policy fixtures.
- `tests/` — existing integration suites being migrated from legacy entity assertions to native behavior; migration is not final passing proof.

## Known gaps (current cycle)

- [ ] Complete integration-suite migration and final native-only verification after external-text reconciliation and unowned font-weight repairs.
- [ ] Clean-resource shutdown remains unproven with known leaks; bounded offscreen UI/IPC evidence does not close this gap.
- [ ] Full Skyborn support remains blocked; existing engine UI/helper evidence does not establish it.

Historical proof and source retirement revisions remain recorded in [README](../../README.md#integration-proof-boundary); portable core is unaffected.

## Out of scope

- Legacy Sprite/Text2d compatibility: explicitly excluded by the user.
- New renderer capabilities or host-screen parity requirements: this change retires unused projection code and preserves existing native behavior.
