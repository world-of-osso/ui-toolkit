# FontString raster line metrics

## What it must do

- Resolve scalable-font default one-to-one line allocation with one pure `resolve_font_lines` function. Inputs are nominal local height, complete physical-pixels-per-UI-unit scale, source spacing, shaped line count and unrounded face ascent/descent.
- Round nominal height times scale to nearest physical pixel, halves upward, with a two-pixel minimum and32-pixel cap; divide by the same scale for local line height. Do not multiply viewport scaling twice.
- Clamp negative source spacing to zero and ceil-round positive spacing in physical pixels. N lines allocate N heights and N−1 gaps; zero lines allocate zero text height.
- Place each baseline at the half-up-rounded face ascender share of the selected raster height, converted back to local units. Baseline is not additional line advance.
- Reject invalid/nonfinite heights, scales and face metrics explicitly. Font loading, shaping, glyph rendering and region clipping remain host responsibilities.

## Concrete contract

At1080 pixels and historical effective scale1, the host supplies p=1080/768. Nominal12 selects17 pixels:12.088889 local units per line; two zero-spacing lines occupy24.177778 units. With face965/−250 the baseline offset is14 physical pixels. At p=1, nominal12 gives12-unit lines. Whole-block alignment uses the allocated block, not glyph ink bounds.

## Evidence and limits

`core/src/font_line_metrics.rs` contains resolver and concrete Ann/ReadyCheck, single-line, half-up/cap, spacing and invalid-input unit fixtures. This implements approved clean-room behavioural prose from historical Cataclysm/Wrath reference builds, not verified Retail70448 behaviour. No cited historical source may be opened, searched or reconstructed by implementers. Allocation is not proof that ink clears adjacent controls.
