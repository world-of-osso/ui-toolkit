# FontString layout attributes

RSX exposes existing FontString wrapping and line-limit render inputs. [Attribute integration](../../README.md#fontstring-layout-attributes).

## What it must do

- [ ] `word_wrap` accepts `true` or `false` and reaches `FontStringData.word_wrap`.
- [ ] `max_lines` accepts a positive integer or `none` and reaches `FontStringData.max_lines`; zero is rejected explicitly.
- [ ] Attribute readback reflects current values; changing or removing attributes restores the declared values or default false/unlimited state.

## How it works

- [Portable core integration](../../README.md#portable-core-integration).

## Implementation inventory

- `core/src/attrs.rs` — attribute application/readback and line-limit validation.
- `core/src/widgets/font_string.rs` — existing render input fields.

## Tests asserting this spec

- `core/tests/font_string_layout.rs` — dynamic apply/readback/removal and invalid zero.

## Known gaps (current cycle)

- [ ] Targeted core gate pending.

## Out of scope

Host glyph drawing, wrapping and centering remain renderer-owned; this contract does not certify either host's pixels.
