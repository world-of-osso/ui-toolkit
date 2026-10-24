# Shared menu auto-grid

Shared menu cell coordinates follow Retail `Blizzard_Menu/Menu.lua:1246–1260,1344–1368,1482–1485` and `MenuConstants.lua`. Skin art does not change placement. [Core integration](../../README.md#portable-core-integration).

## What it must do

- [ ] Choose1/2/3/4 columns for counts≤10/11–24/25–36/>36; rows are ceiling(count/columns).
- [ ] Vertical mode fills final rows before advancing a column; horizontal mode fills columns before advancing a row.
- [ ] Empty input returns no cells and zero rows.
- [ ] Preserve each entry's index exactly once, independent of skin.

## How it works

- [Portable core integration](../../README.md#portable-core-integration).

## Implementation inventory

- `core/src/menu_grid.rs` — shared cell-coordinate model; initially the linear RED-enabling API baseline, not an implemented auto-grid.

## Tests asserting this spec

- `core/tests/menu_auto_grid.rs` — concrete threshold and fill-order cases.

## Known gaps (current cycle)

- [ ] Observe behavioral RED, implement auto-grid and prove scoped GREEN.
- [ ] Engine generic-menu integration and both-skin proof remain separate.

## Out of scope

Housing CAPI identity/order, seven missing labels and native menu art/measurement are not established by cell-coordinate tests. Explicit-column counts, compaction and scrolling are not requested by the current auto-grid consumer.
