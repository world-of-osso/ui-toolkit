//! Shared menu cell coordinates; RED baseline retains the existing linear menu layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuGridDirection {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuGridCell {
    pub column: usize,
    pub row: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuGrid {
    pub columns: usize,
    pub rows: usize,
    pub cells: Vec<MenuGridCell>,
}

pub fn auto_menu_grid(count: usize, _direction: MenuGridDirection) -> MenuGrid {
    MenuGrid {
        columns: 1,
        rows: count,
        cells: (0..count)
            .map(|row| MenuGridCell { column: 0, row })
            .collect(),
    }
}
