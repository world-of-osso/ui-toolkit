# Authored blend modes

Preserve Retail XML `alphaMode` through the canonical `blend_mode` attribute into typed region data. Source: the R cache `Blizzard_SharedXML/UI.xsd` default BLEND; `Blizzard_TransmogTemplates.xml`858/872 uses one atlas with different normal/ADD modes. Native projection consumes the data; it must not infer material behavior from names or file IDs.

## What it must do

- [x] Omitted mode and explicit BLEND use ordinary alpha blending; ADD uses additive blending, independently of source/atlas identity.
- [x] Reject unsupported values without changing the region. MOD, DISABLE and ALPHAKEY are not advertised as supported modes.
- [x] Button state art carries explicit source and blend in one shared representation; unset authored mode defaults BLEND, including highlight art. Attribute application order must not discard authored mode.

## How it works

- The core attribute dispatcher populates region data; the consumer's renderer maps it to a material.
- Button state slots contain optional `ButtonTexture { source, blend_mode }` records. Source and mode attributes update independent members, so either attribute order preserves both. `button_normal_blend_mode`, `button_pushed_blend_mode`, `button_disabled_blend_mode` and `button_highlight_blend_mode` accept the same strict BLEND/ADD spellings.

## Implementation inventory

- `core/src/widgets/texture.rs` — supported authored modes and texture-region state.
- `core/src/attrs.rs` — canonical attribute propagation.
- `core/src/widgets/button.rs` — state artwork data.

## Tests asserting this spec

- `core/src/attrs_layout_tests.rs` — same-atlas normal/additive regions, unsupported modes, button source/mode order and default BLEND highlight.

## Known gaps (current cycle)

- [x] Complete button-state source/blend propagation and native Godot consumer migration.
- [ ] Coordinated engine/toolkit lead acceptance; offline native proof does not establish retired Bevy renderer parity.

## Out of scope

- Step2 crop, tint, alpha and Checked region metadata follows in the same representation.
- Additional blend modes require their own explicit semantics and proof; the retired Bevy renderer is not a native Godot parity gate.
