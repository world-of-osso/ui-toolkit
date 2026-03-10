use std::collections::HashSet;

use crate::anchor::{Anchor, AnchorPoint, anchor_position, frame_position_from_anchor};
use crate::registry::FrameRegistry;

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Return the fractional (0..1) position along the X and Y axes for an anchor point.
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
///
/// Follows WoW anchor semantics: no anchors places at parent top-left, one anchor
/// positions the frame, two anchors can stretch the frame along axes where the
/// anchor points differ.
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

pub fn resolve_frame_layout(registry: &FrameRegistry, frame_id: u64) -> Option<LayoutRect> {
    let frame = registry.get(frame_id)?;
    if frame.anchors.is_empty()
        && let Some(existing) = frame.layout_rect.clone()
    {
        return Some(existing);
    }
    let fallback_target = frame
        .parent_id
        .and_then(|parent_id| registry.get(parent_id))
        .and_then(|parent| parent.layout_rect.clone())
        .unwrap_or_else(|| registry.screen_rect());

    Some(match frame.anchors.as_slice() {
        [] => LayoutRect {
            x: fallback_target.x,
            y: fallback_target.y,
            width: frame.width,
            height: frame.height,
        },
        [anchor] => {
            let target_rect = anchor
                .relative_to
                .and_then(|target_id| registry.get(target_id))
                .and_then(|target| target.layout_rect.as_ref())
                .unwrap_or(&fallback_target);
            let (tx, ty) = resolve_target_in_rect(anchor, target_rect);
            let (fx, fy) =
                frame_position_from_anchor(anchor.point, tx, ty, frame.width, frame.height);
            LayoutRect {
                x: fx,
                y: fy,
                width: frame.width,
                height: frame.height,
            }
        }
        [a, b, ..] => {
            let a_target_rect = a
                .relative_to
                .and_then(|target_id| registry.get(target_id))
                .and_then(|target| target.layout_rect.as_ref())
                .unwrap_or(&fallback_target);
            let b_target_rect = b
                .relative_to
                .and_then(|target_id| registry.get(target_id))
                .and_then(|target| target.layout_rect.as_ref())
                .unwrap_or(&fallback_target);
            let (t1x, t1y) = resolve_target_in_rect(a, a_target_rect);
            let (t2x, t2y) = resolve_target_in_rect(b, b_target_rect);

            let (frac1x, frac1y) = point_to_edge_offsets(a.point);
            let (frac2x, frac2y) = point_to_edge_offsets(b.point);

            let (final_x, final_w) = if (frac1x - frac2x).abs() > f32::EPSILON {
                stretch_axis(t1x, frac1x, t2x, frac2x)
            } else {
                let fx = t1x - frac1x * frame.width;
                (fx, frame.width)
            };

            let (final_y, final_h) = if (frac1y - frac2y).abs() > f32::EPSILON {
                stretch_axis(t1y, frac1y, t2y, frac2y)
            } else {
                let fy = t1y - frac1y * frame.height;
                (fy, frame.height)
            };

            LayoutRect {
                x: final_x,
                y: final_y,
                width: final_w,
                height: final_h,
            }
        }
    })
}

pub fn recompute_layouts(registry: &mut FrameRegistry) {
    let frame_ids: Vec<u64> = registry.frames_iter().map(|frame| frame.id).collect();
    let dirty_ids: Vec<u64> = if registry.rect_dirty.is_empty() {
        frame_ids
    } else {
        registry.rect_dirty.iter().copied().collect()
    };

    let mut visiting = HashSet::new();
    let mut resolved = HashSet::new();
    for frame_id in dirty_ids {
        resolve_frame_recursive(registry, frame_id, &mut visiting, &mut resolved);
    }
    registry.rect_dirty.clear();
}

fn resolve_frame_recursive(
    registry: &mut FrameRegistry,
    frame_id: u64,
    visiting: &mut HashSet<u64>,
    resolved: &mut HashSet<u64>,
) {
    if resolved.contains(&frame_id) {
        return;
    }
    if !visiting.insert(frame_id) {
        return;
    }

    let Some((parent_id, anchors)) = registry
        .get(frame_id)
        .map(|frame| (frame.parent_id, frame.anchors.clone()))
    else {
        visiting.remove(&frame_id);
        return;
    };

    if let Some(parent_id) = parent_id {
        resolve_frame_recursive(registry, parent_id, visiting, resolved);
    }
    for target_id in anchors.iter().filter_map(|anchor| anchor.relative_to) {
        resolve_frame_recursive(registry, target_id, visiting, resolved);
    }

    if let Some(layout_rect) = resolve_frame_layout(registry, frame_id)
        && let Some(frame) = registry.get_mut(frame_id)
    {
        frame.layout_rect = Some(layout_rect);
    }

    visiting.remove(&frame_id);
    resolved.insert(frame_id);
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

    let (final_x, final_w) = if (frac1x - frac2x).abs() > f32::EPSILON {
        stretch_axis(t1x, frac1x, t2x, frac2x)
    } else {
        let fx = t1x - frac1x * width;
        (fx, width)
    };

    let (final_y, final_h) = if (frac1y - frac2y).abs() > f32::EPSILON {
        stretch_axis(t1y, frac1y, t2y, frac2y)
    } else {
        let fy = t1y - frac1y * height;
        (fy, height)
    };

    LayoutRect {
        x: final_x,
        y: final_y,
        width: final_w,
        height: final_h,
    }
}

/// Given two target positions along an axis and their fractional offsets,
/// compute the origin and size of the frame along that axis.
///
/// frac represents where the anchor sits within the frame (0=start, 1=end).
/// target = origin + frac * size, so with two equations we solve for origin and size.
fn stretch_axis(t1: f32, frac1: f32, t2: f32, frac2: f32) -> (f32, f32) {
    let size = (t2 - t1) / (frac2 - frac1);
    let origin = t1 - frac1 * size;
    (origin, size)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parent() -> LayoutRect {
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        }
    }

    fn anchor(
        point: AnchorPoint,
        relative_point: AnchorPoint,
        x_offset: f32,
        y_offset: f32,
    ) -> Anchor {
        Anchor {
            point,
            relative_to: None,
            relative_point,
            x_offset,
            y_offset,
        }
    }

    fn approx_eq(a: &LayoutRect, b: &LayoutRect) -> bool {
        (a.x - b.x).abs() < 0.01
            && (a.y - b.y).abs() < 0.01
            && (a.width - b.width).abs() < 0.01
            && (a.height - b.height).abs() < 0.01
    }

    #[test]
    fn no_anchors_at_parent_topleft() {
        let result = resolve_anchors(&[], 100.0, 50.0, &parent());
        assert_eq!(
            result,
            LayoutRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
            }
        );
    }

    #[test]
    fn single_center_to_center() {
        let anchors = [anchor(AnchorPoint::Center, AnchorPoint::Center, 0.0, 0.0)];
        let result = resolve_anchors(&anchors, 200.0, 100.0, &parent());
        // Frame centered: (800-200)/2=300, (600-100)/2=250
        assert_eq!(
            result,
            LayoutRect {
                x: 300.0,
                y: 250.0,
                width: 200.0,
                height: 100.0,
            }
        );
    }

    #[test]
    fn single_topleft_with_offset() {
        // x_offset=10, y_offset=-5 means target_y = parent_y - (-5) = 5
        let anchors = [anchor(
            AnchorPoint::TopLeft,
            AnchorPoint::TopLeft,
            10.0,
            -5.0,
        )];
        let result = resolve_anchors(&anchors, 100.0, 50.0, &parent());
        assert_eq!(
            result,
            LayoutRect {
                x: 10.0,
                y: 5.0,
                width: 100.0,
                height: 50.0,
            }
        );
    }

    #[test]
    fn two_anchors_horizontal_stretch() {
        // LEFT-to-LEFT with 20px inset, RIGHT-to-RIGHT with -20px inset
        let anchors = [
            anchor(AnchorPoint::Left, AnchorPoint::Left, 20.0, 0.0),
            anchor(AnchorPoint::Right, AnchorPoint::Right, -20.0, 0.0),
        ];
        let result = resolve_anchors(&anchors, 100.0, 50.0, &parent());
        // Horizontal: LEFT target = (0+20, 300), RIGHT target = (800-20, 300)
        // frac1x=0, frac2x=1 → size = (780-20)/(1-0) = 760, origin = 20
        // Vertical: same frac (0.5, 0.5) → use first anchor, fy = 300 - 0.5*50 = 275
        let expected = LayoutRect {
            x: 20.0,
            y: 275.0,
            width: 760.0,
            height: 50.0,
        };
        assert!(
            approx_eq(&result, &expected),
            "got {result:?}, expected {expected:?}"
        );
    }

    #[test]
    fn two_anchors_vertical_stretch() {
        // TOP-to-TOP and BOTTOM-to-BOTTOM with 10px insets
        let anchors = [
            anchor(AnchorPoint::Top, AnchorPoint::Top, 0.0, -10.0),
            anchor(AnchorPoint::Bottom, AnchorPoint::Bottom, 0.0, 10.0),
        ];
        let result = resolve_anchors(&anchors, 200.0, 100.0, &parent());
        // TOP target = (400, 0 - (-10)) = (400, 10)
        // BOTTOM target = (400, 600 - 10) = (400, 590)
        // X fracs both 0.5 → use first anchor: fx = 400 - 0.5*200 = 300
        // Y fracs 0.0 and 1.0 → size = (590-10)/(1-0) = 580, origin = 10
        let expected = LayoutRect {
            x: 300.0,
            y: 10.0,
            width: 200.0,
            height: 580.0,
        };
        assert!(
            approx_eq(&result, &expected),
            "got {result:?}, expected {expected:?}"
        );
    }

    #[test]
    fn single_bottomright_anchor() {
        let anchors = [anchor(
            AnchorPoint::BottomRight,
            AnchorPoint::BottomRight,
            0.0,
            0.0,
        )];
        let result = resolve_anchors(&anchors, 100.0, 50.0, &parent());
        // Target = parent bottom-right = (800, 600)
        // frame_position_from_anchor(BottomRight, 800, 600, 100, 50) = (700, 550)
        assert_eq!(
            result,
            LayoutRect {
                x: 700.0,
                y: 550.0,
                width: 100.0,
                height: 50.0,
            }
        );
    }

    #[test]
    fn resolve_frame_layout_uses_relative_frame_rect() {
        let mut registry = FrameRegistry::new(800.0, 600.0);
        let target = registry.create_frame("Target", None);
        let child = registry.create_frame("Child", None);

        registry.get_mut(target).unwrap().layout_rect = Some(LayoutRect {
            x: 100.0,
            y: 80.0,
            width: 200.0,
            height: 120.0,
        });

        let frame = registry.get_mut(child).unwrap();
        frame.width = 50.0;
        frame.height = 30.0;
        frame.anchors.push(Anchor {
            point: AnchorPoint::TopLeft,
            relative_to: Some(target),
            relative_point: AnchorPoint::BottomRight,
            x_offset: 10.0,
            y_offset: -5.0,
        });

        let rect = resolve_frame_layout(&registry, child).unwrap();
        assert_eq!(
            rect,
            LayoutRect {
                x: 310.0,
                y: 205.0,
                width: 50.0,
                height: 30.0,
            }
        );
    }

    #[test]
    fn resolve_frame_layout_falls_back_to_screen_for_root_frame() {
        let mut registry = FrameRegistry::new(800.0, 600.0);
        let child = registry.create_frame("Child", None);
        let frame = registry.get_mut(child).unwrap();
        frame.width = 100.0;
        frame.height = 40.0;
        frame.anchors.push(Anchor {
            point: AnchorPoint::Center,
            relative_to: None,
            relative_point: AnchorPoint::Center,
            x_offset: 0.0,
            y_offset: 0.0,
        });

        let rect = resolve_frame_layout(&registry, child).unwrap();
        assert_eq!(
            rect,
            LayoutRect {
                x: 350.0,
                y: 280.0,
                width: 100.0,
                height: 40.0,
            }
        );
    }

    #[test]
    fn recompute_layouts_updates_anchored_children() {
        let mut registry = FrameRegistry::new(800.0, 600.0);
        let parent = registry.create_frame("Parent", None);
        let child = registry.create_frame("Child", Some(parent));

        {
            let frame = registry.get_mut(parent).unwrap();
            frame.width = 300.0;
            frame.height = 200.0;
            frame.layout_rect = Some(LayoutRect {
                x: 40.0,
                y: 50.0,
                width: 300.0,
                height: 200.0,
            });
        }

        {
            let frame = registry.get_mut(child).unwrap();
            frame.width = 100.0;
            frame.height = 40.0;
            frame.anchors.push(Anchor {
                point: AnchorPoint::TopLeft,
                relative_to: Some(parent),
                relative_point: AnchorPoint::BottomRight,
                x_offset: 5.0,
                y_offset: -10.0,
            });
        }

        recompute_layouts(&mut registry);

        assert_eq!(
            registry.get(child).unwrap().layout_rect,
            Some(LayoutRect {
                x: 345.0,
                y: 260.0,
                width: 100.0,
                height: 40.0,
            })
        );
    }
}
