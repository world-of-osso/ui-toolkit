use std::collections::HashMap;

use super::{DynName, with_attr};
use crate::rsx;
use crate::screen::SharedContext;
use crate::widget_def::Element;

const MIN_THUMB_HEIGHT: f32 = 16.0;
const DEFAULT_TRACK_COLOR: &str = "0,0,0,0.5";
const DEFAULT_THUMB_COLOR: &str = "0.8,0.7,0.5,0.9";

/// Row-snapped scroll geometry shared by the list builder and its input handling.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ScrollGeometry {
    pub row_count: usize,
    pub visible_rows: usize,
    pub row_height: f32,
    pub track_height: f32,
}

impl ScrollGeometry {
    pub fn max_first_row(&self) -> usize {
        self.row_count.saturating_sub(self.visible_rows)
    }

    pub fn clamp(&self, first_row: usize) -> usize {
        first_row.min(self.max_first_row())
    }

    pub fn thumb_height(&self) -> f32 {
        if self.row_count == 0 {
            return self.track_height;
        }
        let ratio = (self.visible_rows as f32 / self.row_count as f32).min(1.0);
        (self.track_height * ratio)
            .max(MIN_THUMB_HEIGHT)
            .min(self.track_height)
    }

    pub fn thumb_top(&self, first_row: usize) -> f32 {
        let max_first = self.max_first_row();
        if max_first == 0 {
            return 0.0;
        }
        self.thumb_travel() * self.clamp(first_row) as f32 / max_first as f32
    }

    /// First row whose thumb position is nearest to `thumb_top` (track-relative pixels).
    pub fn row_at_thumb_top(&self, thumb_top: f32) -> usize {
        let travel = self.thumb_travel();
        if travel <= 0.0 {
            return 0;
        }
        let fraction = (thumb_top / travel).clamp(0.0, 1.0);
        (fraction * self.max_first_row() as f32).round() as usize
    }

    fn thumb_travel(&self) -> f32 {
        self.track_height - self.thumb_height()
    }

    fn to_attr(self) -> String {
        format!(
            "{},{},{},{}",
            self.row_count, self.visible_rows, self.row_height, self.track_height
        )
    }

    /// Parse the `scroll_list` attribute: `row_count,visible_rows,row_height,track_height`.
    pub(crate) fn parse_attr(value: &str) -> Self {
        let parts: Vec<&str> = value.split(',').map(str::trim).collect();
        let parsed = match parts.as_slice() {
            [count, visible, row_height, track_height] => (|| {
                Some(Self {
                    row_count: count.parse().ok()?,
                    visible_rows: visible.parse().ok()?,
                    row_height: row_height.parse().ok()?,
                    track_height: track_height.parse().ok()?,
                })
            })(),
            _ => None,
        };
        parsed.unwrap_or_else(|| {
            panic!(
                "invalid scroll_list '{value}': expected row_count,visible_rows,row_height,track_height"
            )
        })
    }
}

/// Scroll position of one list plus the geometry of its last build.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScrollListState {
    pub geometry: ScrollGeometry,
    pub first_row: usize,
    /// Advances whenever `first_row` changes; screens that built the list rebuild on it.
    pub generation: u64,
    /// Cursor offset from the thumb top while the thumb is being dragged.
    pub drag_grab: Option<f32>,
}

/// Registry-owned scroll lists keyed by list frame name, so positions survive
/// rebuilds and frame recreation.
#[derive(Debug, Default)]
pub struct ScrollLists(HashMap<String, ScrollListState>);

impl ScrollLists {
    pub fn get(&self, name: &str) -> Option<&ScrollListState> {
        self.0.get(name)
    }

    pub fn generation(&self, name: &str) -> u64 {
        self.0.get(name).map_or(0, |state| state.generation)
    }

    pub(crate) fn first_rows(&self) -> HashMap<String, usize> {
        self.0
            .iter()
            .map(|(name, state)| (name.clone(), state.first_row))
            .collect()
    }

    /// Record built geometry. Clamping here does not advance the generation because
    /// the build that supplied the geometry already rendered the clamped position.
    pub(crate) fn configure(&mut self, name: &str, geometry: ScrollGeometry) {
        let state = self.0.entry(name.to_string()).or_default();
        state.geometry = geometry;
        state.first_row = geometry.clamp(state.first_row);
    }

    /// Move a list to `first_row`, clamped to its content. Returns whether it moved.
    pub fn scroll_to(&mut self, name: &str, first_row: usize) -> bool {
        let Some(state) = self.0.get_mut(name) else {
            return false;
        };
        let first_row = state.geometry.clamp(first_row);
        if state.first_row == first_row {
            return false;
        }
        state.first_row = first_row;
        state.generation += 1;
        true
    }

    pub fn scroll_by(&mut self, name: &str, rows: isize) -> bool {
        let Some(state) = self.0.get(name) else {
            return false;
        };
        let target = state.first_row.saturating_add_signed(rows);
        self.scroll_to(name, target)
    }

    pub(crate) fn set_drag(&mut self, name: &str, grab: Option<f32>) {
        if let Some(state) = self.0.get_mut(name) {
            state.drag_grab = grab;
        }
    }

    pub(crate) fn dragging(&self) -> impl Iterator<Item = (&str, &ScrollListState)> {
        self.0
            .iter()
            .filter(|(_, state)| state.drag_grab.is_some())
            .map(|(name, state)| (name.as_str(), state))
    }
}

pub fn track_name(list: &str) -> String {
    format!("{list}ScrollTrack")
}

pub fn thumb_name(list: &str) -> String {
    format!("{list}ScrollThumb")
}

pub fn row_name(list: &str, index: usize) -> String {
    format!("{list}Row{index}")
}

/// A virtualized list: only rows inside the viewport exist as frames.
#[derive(Clone, Copy)]
pub struct ScrollList<'a> {
    pub name: &'a str,
    pub width: f32,
    pub height: f32,
    pub row_height: f32,
    pub row_count: usize,
    pub track_width: f32,
    /// Optional track/thumb art; plain colors are used when absent.
    pub track_fdid: Option<u32>,
    pub thumb_fdid: Option<u32>,
}

impl ScrollList<'_> {
    fn geometry(&self) -> ScrollGeometry {
        ScrollGeometry {
            row_count: self.row_count,
            visible_rows: (self.height / self.row_height).floor() as usize,
            row_height: self.row_height,
            track_height: self.height,
        }
    }
}

/// Build a scroll list. Rows are wrapped in frames named [`row_name`], laid out
/// from the list top; `row` builds the content of one data row by index.
pub fn scroll_list(
    ctx: &SharedContext,
    spec: ScrollList<'_>,
    row: impl Fn(usize) -> Element,
) -> Element {
    let geometry = spec.geometry();
    let first = geometry.clamp(ctx.scroll_first_row(spec.name));
    let last = (first + geometry.visible_rows).min(spec.row_count);
    let rows: Element = (first..last)
        .flat_map(|index| list_row(&spec, first, index, &row))
        .collect();
    let config = geometry.to_attr();
    rsx! {
        r#frame {
            name: DynName(spec.name.to_string()),
            width: {spec.width},
            height: {spec.height},
            mouse_enabled: true,
            scroll_list: {config},
            {rows}
            {scroll_track(&spec, geometry, first)}
        }
    }
}

fn list_row(
    spec: &ScrollList<'_>,
    first: usize,
    index: usize,
    row: &impl Fn(usize) -> Element,
) -> Element {
    let top = (index - first) as f32 * spec.row_height;
    rsx! {
        r#frame {
            name: DynName(row_name(spec.name, index)),
            width: {spec.width - spec.track_width},
            height: {spec.row_height},
            pos_type: "absolute",
            left: 0,
            top: {top},
            {row(index)}
        }
    }
}

fn scroll_track(spec: &ScrollList<'_>, geometry: ScrollGeometry, first: usize) -> Element {
    let hide = geometry.max_first_row() == 0;
    let color = art_color(spec.track_fdid, DEFAULT_TRACK_COLOR);
    rsx! {
        r#frame {
            name: DynName(track_name(spec.name)),
            width: {spec.track_width},
            height: {spec.height},
            pos_type: "absolute",
            right: 0,
            top: 0,
            hidden: {hide},
            background_color: color,
            {art_layer(&track_name(spec.name), spec.track_fdid)}
            {scroll_thumb(spec, geometry, first)}
        }
    }
}

fn scroll_thumb(spec: &ScrollList<'_>, geometry: ScrollGeometry, first: usize) -> Element {
    let color = art_color(spec.thumb_fdid, DEFAULT_THUMB_COLOR);
    rsx! {
        r#frame {
            name: DynName(thumb_name(spec.name)),
            width: {spec.track_width},
            height: {geometry.thumb_height()},
            pos_type: "absolute",
            left: 0,
            top: {geometry.thumb_top(first)},
            mouse_enabled: true,
            background_color: color,
            {art_layer(&thumb_name(spec.name), spec.thumb_fdid)}
        }
    }
}

fn art_color(fdid: Option<u32>, default: &'static str) -> &'static str {
    if fdid.is_some() { "" } else { default }
}

fn art_layer(owner: &str, fdid: Option<u32>) -> Element {
    let Some(fdid) = fdid else {
        return Vec::new();
    };
    let layer = rsx! {
        texture {
            name: DynName(format!("{owner}Art")),
            stretch: true,
        }
    };
    with_attr(layer, "texture_fdid", fdid.to_string())
}

#[cfg(test)]
#[path = "scroll_list_tests.rs"]
mod tests;
