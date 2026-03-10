use crate::layout::LayoutRect;
use crate::registry::FrameRegistry;

/// Test whether a screen-space point is inside a layout rect, shrunk by insets.
///
/// Insets order: [left, right, top, bottom].
pub fn hit_test(x: f32, y: f32, rect: &LayoutRect, insets: &[f32; 4]) -> bool {
    x >= rect.x + insets[0]
        && x <= rect.x + rect.width - insets[1]
        && y >= rect.y + insets[2]
        && y <= rect.y + rect.height - insets[3]
}

/// Find the topmost frame under screen-space point (x, y).
///
/// Walks all visible, mouse-enabled frames in reverse strata order
/// (highest strata first, then highest frame_level, then highest raise_order).
/// Returns the first frame whose layout rect (with insets) contains the point.
pub fn find_frame_at(registry: &FrameRegistry, x: f32, y: f32) -> Option<u64> {
    let mut candidates: Vec<_> = registry
        .frames_iter()
        .filter(|f| f.visible && f.mouse_enabled && f.layout_rect.is_some())
        .collect();

    candidates.sort_by(|a, b| {
        b.strata
            .cmp(&a.strata)
            .then(b.frame_level.cmp(&a.frame_level))
            .then(b.raise_order.cmp(&a.raise_order))
    });

    for frame in candidates {
        let rect = frame.layout_rect.as_ref().unwrap();
        if hit_test(x, y, rect, &frame.hit_rect_insets) {
            return Some(frame.id);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::LayoutRect;
    use crate::registry::FrameRegistry;

    #[test]
    fn hit_test_inside() {
        let rect = LayoutRect {
            x: 100.0,
            y: 100.0,
            width: 200.0,
            height: 100.0,
        };
        assert!(hit_test(150.0, 150.0, &rect, &[0.0; 4]));
    }

    #[test]
    fn hit_test_outside() {
        let rect = LayoutRect {
            x: 100.0,
            y: 100.0,
            width: 200.0,
            height: 100.0,
        };
        assert!(!hit_test(50.0, 50.0, &rect, &[0.0; 4]));
    }

    #[test]
    fn hit_test_with_insets() {
        let rect = LayoutRect {
            x: 100.0,
            y: 100.0,
            width: 200.0,
            height: 100.0,
        };
        let insets = [10.0, 10.0, 10.0, 10.0];
        // inside rect but in inset margin
        assert!(!hit_test(105.0, 105.0, &rect, &insets));
        // inside effective area
        assert!(hit_test(115.0, 115.0, &rect, &insets));
    }

    #[test]
    fn find_topmost_frame() {
        let mut reg = FrameRegistry::new(800.0, 600.0);
        let id1 = reg.create_frame("bg", None);
        {
            let f = reg.get_mut(id1).unwrap();
            f.width = 800.0;
            f.height = 600.0;
            f.mouse_enabled = true;
            f.layout_rect = Some(LayoutRect {
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
            });
        }
        let id2 = reg.create_frame("button", None);
        {
            let f = reg.get_mut(id2).unwrap();
            f.width = 100.0;
            f.height = 50.0;
            f.mouse_enabled = true;
            f.frame_level = 1; // higher level = on top
            f.layout_rect = Some(LayoutRect {
                x: 50.0,
                y: 50.0,
                width: 100.0,
                height: 50.0,
            });
        }

        // Click on the button area should find the button (higher level)
        assert_eq!(find_frame_at(&reg, 75.0, 75.0), Some(id2));
        // Click outside button but inside bg
        assert_eq!(find_frame_at(&reg, 400.0, 400.0), Some(id1));
        // Click outside both
        assert_eq!(find_frame_at(&reg, 900.0, 900.0), None);
    }
}
