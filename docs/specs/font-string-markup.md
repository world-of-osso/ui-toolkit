# FontString text markup

Retail FontStrings interpret inline text controls; callers retain their original authored strings. The shared core exposes display runs for host drawing and display text for measurement.

## What it must do

- Decode `|cAARRGGBB` into explicit RGBA runs; `|r` restores the FontString's authored colour. Preserve alpha and Unicode display text.
- Render `|Htarget|hdisplay|h` as its display text in the surrounding explicit colour, or the authored font colour when none is specified. Target metadata is not displayed; do not invent a link colour.
- Decode `||` once into a literal pipe, including inside link display text; escaped control-looking text does not become formatting.
- Preserve brackets and unknown controls literally. Adjacent text with the same colour remains one run. Measurement excludes supported control bytes.
- Keep parsing independent of panel names, catalogs, fonts and rendering hosts.

## Out of scope

Texture/atlas controls (`|T`, `|A`), hyperlink activation, spell-description variable evaluation and native EJ title synthesis. Native glyphs, wrapping and node lifetime remain host-owned; parser tests do not certify host pixels.

## Proof

Concrete Kurog JournalEncounterSection25033/25035 strings exercise blue spell links and restoration of authored font colour. Independent tests cover escaped pipes, ARGB alpha, standalone links and plain Unicode/brackets.
