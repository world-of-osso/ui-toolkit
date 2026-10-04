/// Authored UI measurement; percentages remain unresolved until native layout.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Val {
    #[default]
    Auto,
    Px(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiRect {
    pub left: Val,
    pub right: Val,
    pub top: Val,
    pub bottom: Val,
}

impl UiRect {
    pub const AUTO: Self = Self::all(Val::Auto);
    pub const ZERO: Self = Self::all(Val::Px(0.0));

    pub const fn all(value: Val) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Val2 {
    pub x: Val,
    pub y: Val,
}

impl Val2 {
    pub const ZERO: Self = Self {
        x: Val::Px(0.0),
        y: Val::Px(0.0),
    };
    pub const fn percent(x: f32, y: f32) -> Self {
        Self {
            x: Val::Percent(x),
            y: Val::Percent(y),
        }
    }
    pub const fn px(x: f32, y: f32) -> Self {
        Self {
            x: Val::Px(x),
            y: Val::Px(y),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PositionType {
    #[default]
    Relative,
    Absolute,
}
