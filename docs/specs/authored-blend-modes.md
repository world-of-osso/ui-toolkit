# Authored blend modes

Preserve Retail XML `alphaMode` through the canonical `blend_mode` attribute into typed region data. Source: the R cache `Blizzard_SharedXML/UI.xsd` default BLEND; `Blizzard_TransmogTemplates.xml`858/872 uses one atlas with different normal/ADD modes. Native projection consumes the data; it must not infer material behavior from names or file IDs.

## What it must do

- [ ] Omitted mode and explicit BLEND use ordinary alpha blending; ADD uses additive blending, independently of source/atlas identity.
- [ ] Reject unsupported values without changing the region. MOD, DISABLE and ALPHAKEY are not advertised as supported modes.
- [ ] Button state art carries explicit source and blend in one shared representation; unset authored mode defaults BLEND, including highlight art. Attribute application order must not discard authored mode.

## How it works

- The core attribute dispatcher populates region data; the consumer's renderer maps it to a material.

## Implementation inventory

- `core/src/widgets/texture.rs` — supported authored modes and texture-region state.
- `core/src/attrs.rs` — canonical attribute propagation.
- `core/src/widgets/button.rs` — state artwork data.

## Tests asserting this spec

- `core/src/attrs_layout_tests.rs` — same-atlas normal/additive regions and unsupported modes.

## Known gaps (current cycle)

- [ ] Complete button-state source/blend propagation and consumer migration.

## Out of scope

- Step2 crop, tint, alpha and Checked region metadata follows in the same representation.
- Additional blend modes require their own explicit semantics and proof; the retired Bevy renderer is not a native Godot parity gate.
