use ui_toolkit_core::menu_grid::{MenuGridDirection, auto_menu_grid};

#[test]
fn menu_auto_grid_uses_retail_count_thresholds() {
    for direction in [MenuGridDirection::Vertical, MenuGridDirection::Horizontal] {
        for (count, columns, rows) in [
            (0, 1, 0),
            (1, 1, 1),
            (10, 1, 10),
            (11, 2, 6),
            (24, 2, 12),
            (25, 3, 9),
            (36, 3, 12),
            (37, 4, 10),
        ] {
            let grid = auto_menu_grid(count, direction);
            assert_eq!(
                (grid.columns, grid.rows),
                (columns, rows),
                "{count} {direction:?}"
            );
            assert_eq!(grid.cells.len(), count);
        }
    }
}

#[test]
fn menu_auto_grid_vertical_fills_final_rows_before_the_next_column() {
    let grid = auto_menu_grid(11, MenuGridDirection::Vertical);
    let cells: Vec<_> = grid
        .cells
        .iter()
        .map(|cell| (cell.column, cell.row))
        .collect();
    assert_eq!(
        cells,
        vec![
            (0, 0),
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            (0, 5),
            (1, 0),
            (1, 1),
            (1, 2),
            (1, 3),
            (1, 4),
        ]
    );
    let larger = auto_menu_grid(25, MenuGridDirection::Vertical);
    assert_eq!((larger.cells[18].column, larger.cells[18].row), (2, 0));
    assert_eq!((larger.cells[24].column, larger.cells[24].row), (2, 6));
}

#[test]
fn menu_auto_grid_horizontal_fills_columns_before_the_next_row() {
    let grid = auto_menu_grid(11, MenuGridDirection::Horizontal);
    let cells: Vec<_> = grid
        .cells
        .iter()
        .map(|cell| (cell.column, cell.row))
        .collect();
    assert_eq!(
        cells,
        vec![
            (0, 0),
            (1, 0),
            (0, 1),
            (1, 1),
            (0, 2),
            (1, 2),
            (0, 3),
            (1, 3),
            (0, 4),
            (1, 4),
            (0, 5),
        ]
    );
}
