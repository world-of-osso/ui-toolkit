/// A typed frame name for RSX declarations and semantic references.
/// Ensures the same constant is used at both definition and reference sites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameName(pub &'static str);

impl FrameName {
    pub fn as_str(self) -> &'static str {
        self.0
    }
}

impl std::fmt::Display for FrameName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

/// Layout reference space; this does not change logical parentage or ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnchorTarget {
    #[default]
    Parent,
    Screen,
}
