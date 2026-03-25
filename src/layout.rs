use std::collections::HashSet;

use crate::anchor::{Anchor, AnchorPoint, anchor_position, frame_position_from_anchor};
use crate::frame::{Dimension, FlexAlign, FlexDirection, FlexJustify, FlexLayout};
use crate::registry::FrameRegistry;

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

fn point_to_edge_offsets(point: AnchorPoint) -> (f32, f32) {
    match point {
        AnchorPoint::TopLeft => (0.0, 0.0),
        AnchorPoint::Top => (0.5, 0.0),
        AnchorPoint::TopRight => (1.0, 0.0),
        AnchorPoint::Left => (0.0, 0.5),
        AnchorPoint::Center => (0.5, 0.5),
        AnchorPoint::Right => (1.0, 0.5),
        AnchorPoint::BottomLeft => (0.0, 1.0),
        AnchorPoint::Bottom => (0.5, 1.0),
        AnchorPoint::BottomRight => (1.0, 1.0),
    }
}

fn resolve_target(anchor: &Anchor, parent: &LayoutRect) -> (f32, f32) {
    let (ax, ay) = anchor_position(
        anchor.relative_point,
        parent.x,
        parent.y,
        parent.width,
        parent.height,
    );
    (ax + anchor.x_offset, ay - anchor.y_offset)
}

fn resolve_target_in_rect(anchor: &Anchor, target_rect: &LayoutRect) -> (f32, f32) {
    let (ax, ay) = anchor_position(
        anchor.relative_point,
        target_rect.x,
        target_rect.y,
        target_rect.width,
        target_rect.height,
    );
    (ax + anchor.x_offset, ay - anchor.y_offset)
}

/// Resolve a frame's layout rectangle from its anchors, explicit size, and parent rect.
pub fn resolve_anchors(
    anchors: &[Anchor],
    width: f32,
    height: f32,
    parent_rect: &LayoutRect,
) -> LayoutRect {
    match anchors.len() {
        0 => LayoutRect {
            x: parent_rect.x,
            y: parent_rect.y,
            width,
            height,
        },
        1 => resolve_single_anchor(&anchors[0], width, height, parent_rect),
        _ => resolve_two_anchors(&anchors[0], &anchors[1], width, height, parent_rect),
    }
}

fn fallback_target(registry: &FrameRegistry, frame_id: u64) -> LayoutRect {
    let frame = registry.get(frame_id).unwrap();
    frame
        .parent_id
        .and_then(|pid| registry.get(pid))
        .and_then(|p| p.layout_rect.clone())
        .unwrap_or_else(|| registry.screen_rect())
}

fn resolve_dimension(dim: Dimension, parent_size: f32) -> f32 {
    match dim {
        Dimension::Fixed(v) => v,
        Dimension::Fill => parent_size,
    }
}

fn resolve_no_anchors(registry: &FrameRegistry, frame_id: u64) -> LayoutRect {
    let frame = registry.get(frame_id).unwrap();
    if let Some(existing) = frame.layout_rect.clone() {
        return existing;
    }
    let target = fallback_target(registry, frame_id);
    let w = resolve_dimension(frame.width, target.width);
    let h = resolve_dimension(frame.height, target.height);
    LayoutRect {
        x: target.x,
        y: target.y,
        width: w,
        height: h,
    }
}

fn resolve_one_anchor(registry: &FrameRegistry, frame_id: u64) -> LayoutRect {
    let frame = registry.get(frame_id).unwrap();
    let anchor = &frame.anchors[0];
    let fallback = fallback_target(registry, frame_id);
    let target_rect = anchor_target_rect(registry, anchor, &fallback);
    let (w, h) = resolved_or_auto_size(frame, fallback.width, fallback.height);
    let (tx, ty) = resolve_target_in_rect(anchor, target_rect);
    let (fx, fy) = frame_position_from_anchor(anchor.point, tx, ty, w, h);
    LayoutRect {
        x: fx,
        y: fy,
        width: w,
        height: h,
    }
}

/// Use layout_rect dimensions if the frame has auto-size (0) and was already sized by flex.
fn resolved_or_auto_size(frame: &crate::frame::Frame, parent_w: f32, parent_h: f32) -> (f32, f32) {
    let w = resolve_dimension(frame.width, parent_w);
    let h = resolve_dimension(frame.height, parent_h);
    let w = if w == 0.0 {
        frame.layout_rect.as_ref().map_or(0.0, |r| r.width)
    } else {
        w
    };
    let h = if h == 0.0 {
        frame.layout_rect.as_ref().map_or(0.0, |r| r.height)
    } else {
        h
    };
    (w, h)
}

fn resolve_multi_anchor(registry: &FrameRegistry, frame_id: u64) -> LayoutRect {
    let frame = registry.get(frame_id).unwrap();
    let (a, b) = (&frame.anchors[0], &frame.anchors[1]);
    let fallback = fallback_target(registry, frame_id);
    let w = resolve_dimension(frame.width, fallback.width);
    let h = resolve_dimension(frame.height, fallback.height);
    let a_rect = anchor_target_rect(registry, a, &fallback);
    let b_rect = anchor_target_rect(registry, b, &fallback);
    let (t1x, t1y) = resolve_target_in_rect(a, a_rect);
    let (t2x, t2y) = resolve_target_in_rect(b, b_rect);
    let (frac1x, frac1y) = point_to_edge_offsets(a.point);
    let (frac2x, frac2y) = point_to_edge_offsets(b.point);
    let (final_x, final_w) = stretch_or_fixed(t1x, frac1x, t2x, frac2x, w);
    let (final_y, final_h) = stretch_or_fixed(t1y, frac1y, t2y, frac2y, h);
    LayoutRect {
        x: final_x,
        y: final_y,
        width: final_w,
        height: final_h,
    }
}

fn anchor_target_rect<'a>(
    registry: &'a FrameRegistry,
    anchor: &Anchor,
    fallback: &'a LayoutRect,
) -> &'a LayoutRect {
    anchor
        .relative_to
        .and_then(|tid| registry.get(tid))
        .and_then(|t| t.layout_rect.as_ref())
        .unwrap_or(fallback)
}

fn stretch_or_fixed(t1: f32, frac1: f32, t2: f32, frac2: f32, explicit: f32) -> (f32, f32) {
    if (frac1 - frac2).abs() > f32::EPSILON {
        stretch_axis(t1, frac1, t2, frac2)
    } else {
        (t1 - frac1 * explicit, explicit)
    }
}

fn stretch_axis(t1: f32, frac1: f32, t2: f32, frac2: f32) -> (f32, f32) {
    let size = (t2 - t1) / (frac2 - frac1);
    let origin = t1 - frac1 * size;
    (origin, size)
}

pub fn resolve_frame_layout(registry: &FrameRegistry, frame_id: u64) -> Option<LayoutRect> {
    let frame = registry.get(frame_id)?;
    Some(match frame.anchors.len() {
        0 => resolve_no_anchors(registry, frame_id),
        1 => resolve_one_anchor(registry, frame_id),
        _ => resolve_multi_anchor(registry, frame_id),
    })
}

fn resolve_single_anchor(
    anchor: &Anchor,
    width: f32,
    height: f32,
    parent: &LayoutRect,
) -> LayoutRect {
    let (tx, ty) = resolve_target(anchor, parent);
    let (fx, fy) = frame_position_from_anchor(anchor.point, tx, ty, width, height);
    LayoutRect {
        x: fx,
        y: fy,
        width,
        height,
    }
}

fn resolve_two_anchors(
    a: &Anchor,
    b: &Anchor,
    width: f32,
    height: f32,
    parent: &LayoutRect,
) -> LayoutRect {
    let (t1x, t1y) = resolve_target(a, parent);
    let (t2x, t2y) = resolve_target(b, parent);
    let (frac1x, frac1y) = point_to_edge_offsets(a.point);
    let (frac2x, frac2y) = point_to_edge_offsets(b.point);
    let (final_x, final_w) = stretch_or_fixed(t1x, frac1x, t2x, frac2x, width);
    let (final_y, final_h) = stretch_or_fixed(t1y, frac1y, t2y, frac2y, height);
    LayoutRect {
        x: final_x,
        y: final_y,
        width: final_w,
        height: final_h,
    }
}

// --- Layout recomputation ---

pub fn recompute_layouts(registry: &mut FrameRegistry) {
    let dirty_ids: Vec<u64> = if registry.rect_dirty.is_empty() {
        registry.frames_iter().map(|f| f.id).collect()
    } else {
        registry.rect_dirty.iter().copied().collect()
    };
    registry.rect_dirty.clear();
    resolve_dirty_frames(registry, &dirty_ids);
    // Flex auto-sizing may dirty frames (e.g. auto-height changes anchor position).
    // Run one extra pass to settle.
    settle_flex_dirty(registry);
}

fn resolve_dirty_frames(registry: &mut FrameRegistry, ids: &[u64]) {
    let mut visiting = HashSet::new();
    let mut resolved = HashSet::new();
    for &frame_id in ids {
        resolve_frame_recursive(registry, frame_id, &mut visiting, &mut resolved);
    }
}

fn settle_flex_dirty(registry: &mut FrameRegistry) {
    if registry.rect_dirty.is_empty() {
        return;
    }
    let ids: Vec<u64> = registry.rect_dirty.drain().collect();
    resolve_dirty_frames(registry, &ids);
}

fn resolve_frame_recursive(
    registry: &mut FrameRegistry,
    frame_id: u64,
    visiting: &mut HashSet<u64>,
    resolved: &mut HashSet<u64>,
) {
    if resolved.contains(&frame_id) || !visiting.insert(frame_id) {
        return;
    }
    resolve_dependencies(registry, frame_id, visiting, resolved);
    if let Some(rect) = resolve_frame_layout(registry, frame_id) {
        if let Some(frame) = registry.get_mut(frame_id) {
            frame.layout_rect = Some(rect);
        }
    }
    apply_flex_to_children(registry, frame_id, visiting, resolved);
    visiting.remove(&frame_id);
    resolved.insert(frame_id);
}

fn resolve_dependencies(
    registry: &mut FrameRegistry,
    frame_id: u64,
    visiting: &mut HashSet<u64>,
    resolved: &mut HashSet<u64>,
) {
    let Some(frame) = registry.get(frame_id) else {
        return;
    };
    let parent_id = frame.parent_id;
    let targets: Vec<u64> = frame.anchors.iter().filter_map(|a| a.relative_to).collect();
    if let Some(pid) = parent_id {
        resolve_frame_recursive(registry, pid, visiting, resolved);
    }
    for tid in targets {
        resolve_frame_recursive(registry, tid, visiting, resolved);
    }
}

fn apply_flex_to_children(
    registry: &mut FrameRegistry,
    frame_id: u64,
    visiting: &mut HashSet<u64>,
    resolved: &mut HashSet<u64>,
) {
    let Some(frame) = registry.get(frame_id) else {
        return;
    };
    let Some(flex) = frame.flex_layout.clone() else {
        return;
    };
    let Some(mut parent_rect) = frame.layout_rect.clone() else {
        return;
    };
    let auto_width = matches!(frame.width, Dimension::Fixed(v) if v == 0.0);
    let auto_height = matches!(frame.height, Dimension::Fixed(v) if v == 0.0);
    let children: Vec<u64> = frame.children.clone();
    for &cid in &children {
        resolve_dependencies(registry, cid, visiting, resolved);
    }
    let rects = compute_flex_rects(&flex, &parent_rect, registry, &children);
    if auto_width || auto_height {
        let old_rect = parent_rect.clone();
        parent_rect =
            auto_size_flex_parent(&parent_rect, &rects, flex.padding, auto_width, auto_height);
        if let Some(frame) = registry.get_mut(frame_id) {
            frame.layout_rect = Some(parent_rect.clone());
        }
        // Dimensions changed — mark dirty so anchors re-resolve next pass
        if (parent_rect.width - old_rect.width).abs() > 0.01
            || (parent_rect.height - old_rect.height).abs() > 0.01
        {
            registry.rect_dirty.insert(frame_id);
        }
    }
    for (&cid, rect) in children.iter().zip(rects) {
        if let Some(child) = registry.get_mut(cid) {
            child.layout_rect = Some(rect);
        }
        resolved.insert(cid);
    }
}

// --- Flex layout computation ---

fn collect_child_sizes(
    parent: &LayoutRect,
    registry: &FrameRegistry,
    children: &[u64],
) -> Vec<(f32, f32)> {
    children
        .iter()
        .filter_map(|&id| registry.get(id))
        .map(|f| {
            (
                resolve_dimension(f.width, parent.width),
                resolve_dimension(f.height, parent.height),
            )
        })
        .collect()
}

fn compute_flex_rects(
    flex: &FlexLayout,
    parent: &LayoutRect,
    registry: &FrameRegistry,
    children: &[u64],
) -> Vec<LayoutRect> {
    let sizes = collect_child_sizes(parent, registry, children);
    if flex.direction == FlexDirection::RowWrap {
        return compute_row_wrap_rects(flex, parent, &sizes);
    }
    layout_linear_children(flex, parent, &sizes)
}

fn layout_linear_children(
    flex: &FlexLayout,
    parent: &LayoutRect,
    sizes: &[(f32, f32)],
) -> Vec<LayoutRect> {
    let total_main: f32 = sizes.iter().map(|s| main_sz(flex.direction, *s)).sum();
    let gap_total = flex.gap * sizes.len().saturating_sub(1) as f32;
    let avail_main = main_extent(flex.direction, parent) - 2.0 * flex.padding;
    let avail_cross = cross_extent(flex.direction, parent) - 2.0 * flex.padding;
    let offset = main_start_offset(flex.justify, avail_main, total_main, gap_total);
    let eff_gap = eff_gap(flex.justify, avail_main, total_main, flex.gap, sizes.len());
    let mut cursor = offset + flex.padding;
    sizes
        .iter()
        .map(|&(w, h)| {
            let ms = main_sz(flex.direction, (w, h));
            let cs = cross_sz(flex.direction, (w, h));
            let fcs = if flex.align == FlexAlign::Stretch {
                avail_cross
            } else {
                cs
            };
            let cp = cross_pos(flex.align, avail_cross, fcs) + flex.padding;
            let rect = build_flex_rect(flex.direction, parent, cursor, cp, ms, fcs);
            cursor += ms + eff_gap;
            rect
        })
        .collect()
}

fn compute_row_wrap_rects(
    flex: &FlexLayout,
    parent: &LayoutRect,
    sizes: &[(f32, f32)],
) -> Vec<LayoutRect> {
    let avail_width = parent.width - 2.0 * flex.padding;
    let mut rects = Vec::with_capacity(sizes.len());
    let mut x_cursor = 0.0_f32;
    let mut y_cursor = 0.0_f32;
    let mut row_height = 0.0_f32;

    for &(w, h) in sizes {
        // Wrap to next row if this item doesn't fit (unless it's the first in the row)
        if x_cursor > 0.0 && x_cursor + w > avail_width {
            y_cursor += row_height + flex.gap;
            x_cursor = 0.0;
            row_height = 0.0;
        }
        rects.push(LayoutRect {
            x: parent.x + flex.padding + x_cursor,
            y: parent.y + flex.padding + y_cursor,
            width: w,
            height: h,
        });
        x_cursor += w + flex.gap;
        if h > row_height {
            row_height = h;
        }
    }
    rects
}

fn auto_size_flex_parent(
    parent: &LayoutRect,
    rects: &[LayoutRect],
    padding: f32,
    auto_width: bool,
    auto_height: bool,
) -> LayoutRect {
    let mut sized = parent.clone();
    if rects.is_empty() {
        if auto_width {
            sized.width = 2.0 * padding;
        }
        if auto_height {
            sized.height = 2.0 * padding;
        }
        return sized;
    }

    let max_right = rects
        .iter()
        .map(|rect| rect.x + rect.width)
        .fold(parent.x, f32::max);
    let max_bottom = rects
        .iter()
        .map(|rect| rect.y + rect.height)
        .fold(parent.y, f32::max);

    if auto_width {
        sized.width = (max_right - parent.x) + padding;
    }
    if auto_height {
        sized.height = (max_bottom - parent.y) + padding;
    }
    sized
}

fn main_sz(d: FlexDirection, (w, h): (f32, f32)) -> f32 {
    match d {
        FlexDirection::Column => h,
        FlexDirection::Row | FlexDirection::RowWrap => w,
    }
}

fn cross_sz(d: FlexDirection, (w, h): (f32, f32)) -> f32 {
    match d {
        FlexDirection::Column => w,
        FlexDirection::Row | FlexDirection::RowWrap => h,
    }
}

fn main_extent(d: FlexDirection, r: &LayoutRect) -> f32 {
    match d {
        FlexDirection::Column => r.height,
        FlexDirection::Row | FlexDirection::RowWrap => r.width,
    }
}

fn cross_extent(d: FlexDirection, r: &LayoutRect) -> f32 {
    match d {
        FlexDirection::Column => r.width,
        FlexDirection::Row | FlexDirection::RowWrap => r.height,
    }
}

fn main_start_offset(j: FlexJustify, avail: f32, total: f32, gap_total: f32) -> f32 {
    match j {
        FlexJustify::Start | FlexJustify::SpaceBetween => 0.0,
        FlexJustify::Center => (avail - total - gap_total) / 2.0,
        FlexJustify::End => avail - total - gap_total,
    }
}

fn eff_gap(j: FlexJustify, avail: f32, total: f32, gap: f32, count: usize) -> f32 {
    if j == FlexJustify::SpaceBetween && count > 1 {
        (avail - total) / (count - 1) as f32
    } else {
        gap
    }
}

fn cross_pos(align: FlexAlign, avail: f32, child_size: f32) -> f32 {
    match align {
        FlexAlign::Start | FlexAlign::Stretch => 0.0,
        FlexAlign::Center => (avail - child_size) / 2.0,
        FlexAlign::End => avail - child_size,
    }
}

fn build_flex_rect(
    d: FlexDirection,
    parent: &LayoutRect,
    main_pos: f32,
    cross_p: f32,
    main_s: f32,
    cross_s: f32,
) -> LayoutRect {
    match d {
        FlexDirection::Column => LayoutRect {
            x: parent.x + cross_p,
            y: parent.y + main_pos,
            width: cross_s,
            height: main_s,
        },
        FlexDirection::Row | FlexDirection::RowWrap => LayoutRect {
            x: parent.x + main_pos,
            y: parent.y + cross_p,
            width: main_s,
            height: cross_s,
        },
    }
}
