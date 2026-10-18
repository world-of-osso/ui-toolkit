# FontString layout attributes

RSX exposes existing FontString wrapping and line-limit render inputs. [Attribute integration](../../README.md#fontstring-layout-attributes).

## What it must do

- [x] `word_wrap` accepts `true` or `false` and reaches `FontStringData.word_wrap`.
- [x] `max_lines` accepts a positive integer or `none` and reaches `FontStringData.max_lines`; zero is rejected explicitly.
- [x] Optional `line_spacing` accepts an integer and reaches `FontStringData.line_spacing`; omitted values remain `None` so hosts retain their existing theme. Explicit zero survives rebuild/readback. Hosts apply it to both text measurement and drawing.
- [x] Attribute readback reflects current values; explicit changes, including false/none, update the existing render inputs on Screen rebuilds.

## How it works

- [Portable core integration](../../README.md#portable-core-integration).

## Implementation inventory

- `core/src/attrs.rs` — attribute application/readback and line-limit validation.
- `core/src/widgets/font_string.rs` — existing render input fields.

## Tests asserting this spec

- `core/tests/font_string_layout.rs` — dynamic apply/readback/explicit clearing and invalid zero.

## Known gaps (current cycle)

Core attribute gate passes; host pixel acceptance is separate.

## Out of scope

Host glyph drawing, wrapping and centering remain renderer-owned; this contract does not certify either host's pixels.
