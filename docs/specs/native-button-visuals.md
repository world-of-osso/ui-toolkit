# Native button visuals

Registry buttons may own art or serve only as input targets for child textures. Native Bevy UI must project authored button visuals without inventing a background behind interaction-only artwork. See [toolkit rendering](../../README.md).

## What it must do

- [x] `button_default_skin` defaults to `true`, preserving the existing generated nine-slice and its normal, hovered, pushed and disabled states for ordinary buttons.
- [x] `button_default_skin: false` omits the generated default skin when no authored base texture exists; a hover-only highlight remains an overlay, not a base image.
- [x] Turning the default skin off removes only its previously generated nine-slice; authored nine- and three-slices remain.
- [x] Authored background/backdrop colors, base textures and state textures still project. Failed authored textures never become white substitutes.
- [x] A child icon with transparent pixels retains its native image entity after an authored parent background is removed.
- [x] `button_highlight_alpha` accepts only finite `0..=1`, defaults to `0.5`, and multiplies inherited alpha on the native hover overlay without changing ordinary buttons.

### Portable texture projection

- [ ] Authored dynamic base, state, slice and highlight sources resolve through their owning registry's portable ID and RGBA8 pixels; Bevy image handles remain host-owned. Updates and removal must reach each projection path. Targeted native/legacy pixel tests pass as reported below, but they do not establish complete button-path coverage.

## How it works

- [Toolkit registry and projection](../../README.md)

## Implementation inventory

- `core/src/widgets/button.rs` — authored default-skin policy and button states.
- `core/src/attrs.rs` — RSX/runtime attribute application and readback.
- `src/render_button.rs` — default/explicit button nine-slice preparation and generated-state ownership.
- `src/native_render/images.rs` — native base, slice and highlight projection.
- `core/src/registry.rs` — dynamic pixel ownership and referencing-frame invalidation.
- `src/render_texture.rs` — registry-to-Bevy image projection and loader boundary.

## Tests asserting this spec

- `src/native_render/images_tests.rs` — emitted image roles, state transitions, authored visuals and explicit load failures.
- `src/native_render/tests.rs` — native entity lifecycle and transparent child image.
- `core/src/attrs_layout_tests.rs` — authored attribute roundtrip.
- `tests/button_nine_slice.rs` — ordinary default-skin compatibility.

## Known gaps (current cycle)

- [ ] Independent whole-root gate for bridge repairs `cb8f907` / `92c7937` and fixture follow-up `a69d62f` remains pending; see [integration proof boundary](../../README.md#integration-proof-boundary). Existing checked policy bullets retain their historical attribution, not a claim of renewed whole-root proof.

- [ ] Screen-specific artwork and rendered screenshot comparison belong to each host screen, not this toolkit policy.

## Out of scope

Character-creation navigation artwork and other screen styling.
