# Native button visuals

Registry buttons may own art or serve only as input targets for child textures. Native Bevy UI must project authored button visuals without inventing a background behind interaction-only artwork. See [toolkit rendering](../../README.md).

## What it must do

- [x] `button_default_skin` defaults to `true`, preserving the existing generated nine-slice and its normal, hovered, pushed and disabled states for ordinary buttons.
- [x] `button_default_skin: false` omits the generated default skin when no authored base texture exists; a hover-only highlight remains an overlay, not a base image.
- [x] Turning the default skin off removes only its previously generated nine-slice; authored nine- and three-slices remain.
- [x] Authored background/backdrop colors, base textures and state textures still project. Failed authored textures never become white substitutes.
- [x] A child icon with transparent pixels retains its native image entity after an authored parent background is removed.

## How it works

- [Toolkit registry and projection](../../README.md)

## Implementation inventory

- `src/widgets/button.rs` — authored default-skin policy and button states.
- `src/attrs.rs` — RSX/runtime attribute application and readback.
- `src/render_button.rs` — default/explicit button nine-slice preparation and generated-state ownership.
- `src/native_render/images.rs` — native base, slice and highlight projection.

## Tests asserting this spec

- `src/native_render/images_tests.rs` — emitted image roles, state transitions, authored visuals and explicit load failures.
- `src/native_render/tests.rs` — native entity lifecycle and transparent child image.
- `src/attrs_layout_tests.rs` — authored attribute roundtrip.
- `tests/button_nine_slice.rs` — ordinary default-skin compatibility.

## Known gaps (current cycle)

- [ ] Screen-specific artwork and rendered screenshot comparison belong to each host screen, not this toolkit policy.

## Out of scope

Character-creation navigation artwork and other screen styling.
