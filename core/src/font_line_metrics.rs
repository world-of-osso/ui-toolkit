//! Raster-quantized line allocation from the approved historical behaviour prose.
//! Retail 70448 remains unverified; no font I/O or renderer state belongs here.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontLineRequest {
    pub nominal_height: f32,
    /// Complete physical pixels per local UI unit, including viewport conversion once.
    pub pixels_per_ui: f32,
    pub spacing: f32,
    pub line_count: usize,
    pub face_ascent: f32,
    pub face_descent: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontLineMetrics {
    pub raster_pixels: u8,
    pub line_height_ui: f32,
    pub spacing_ui: f32,
    pub advance_ui: f32,
    pub block_height_ui: f32,
    pub baseline_ui: f32,
}

pub fn resolve_font_lines(request: FontLineRequest) -> Result<FontLineMetrics, String> {
    validate_font_line_request(request)?;
    let pixels_per_ui = request.pixels_per_ui;
    let raster_pixels = (request.nominal_height * pixels_per_ui)
        .round()
        .clamp(2.0, 32.0) as u8;
    let line_height_ui = f32::from(raster_pixels) / pixels_per_ui;
    let spacing_ui = (request.spacing.max(0.0) * pixels_per_ui).ceil() / pixels_per_ui;
    let advance_ui = line_height_ui + spacing_ui;
    let gap_count = request.line_count.saturating_sub(1);
    let block_height_ui =
        request.line_count as f32 * line_height_ui + gap_count as f32 * spacing_ui;
    let face_height = request.face_ascent + request.face_descent.abs();
    let baseline_pixels = (request.face_ascent / face_height * f32::from(raster_pixels)).round();
    Ok(FontLineMetrics {
        raster_pixels,
        line_height_ui,
        spacing_ui,
        advance_ui,
        block_height_ui,
        baseline_ui: baseline_pixels / pixels_per_ui,
    })
}

fn validate_font_line_request(request: FontLineRequest) -> Result<(), String> {
    for (name, value) in [
        ("nominal height", request.nominal_height),
        ("pixels per UI unit", request.pixels_per_ui),
        ("face ascent", request.face_ascent),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(format!(
                "FontString {name} must be finite and positive, got {value}"
            ));
        }
    }
    if !request.spacing.is_finite() {
        return Err(format!(
            "FontString spacing must be finite, got {}",
            request.spacing
        ));
    }
    if !request.face_descent.is_finite() {
        return Err(format!(
            "FontString face descent must be finite, got {}",
            request.face_descent
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> FontLineRequest {
        FontLineRequest {
            nominal_height: 12.0,
            pixels_per_ui: 1080.0 / 768.0,
            spacing: 0.0,
            line_count: 2,
            face_ascent: 965.0,
            face_descent: -250.0,
        }
    }

    fn close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.00001,
            "{actual} != {expected}"
        );
    }

    #[test]
    fn raster_lines_ann_readycheck_1080_scale_one_positions() {
        let text = "Ann has initiated a ready check.\nRaid Difficulty: 10 Player";
        let mut input = request();
        input.line_count = text.lines().count();
        let metrics = resolve_font_lines(input).unwrap();
        assert_eq!(metrics.raster_pixels, 17);
        close(metrics.line_height_ui, 12.088889);
        close(metrics.block_height_ui, 24.177778);
        close(metrics.baseline_ui, 9.955556);
        close(37.0 + metrics.advance_ui, 49.088889);
        close(37.0 + metrics.baseline_ui, 46.955555);
        close(37.0 + metrics.advance_ui + metrics.baseline_ui, 59.044444);
        // Source button top remains60; allocated height is NOT an ink-clearance claim.
        close(37.0 + metrics.block_height_ui - 60.0, 1.177778);
    }

    #[test]
    fn raster_lines_normalized_pixel_scale_has_twelve_unit_lines() {
        let mut input = request();
        input.pixels_per_ui = (768.0 / 1080.0) * (1080.0 / 768.0);
        let metrics = resolve_font_lines(input).unwrap();
        assert_eq!(metrics.raster_pixels, 12);
        close(metrics.line_height_ui, 12.0);
        close(metrics.block_height_ui, 24.0);
    }

    #[test]
    fn raster_lines_single_line_nominal_twelve_has_rule_defined_middle_baseline() {
        let mut input = request();
        input.line_count = 1;
        let metrics = resolve_font_lines(input).unwrap();
        close(metrics.block_height_ui, 12.088889);
        close(
            37.0 + (20.0 - metrics.block_height_ui) / 2.0 + metrics.baseline_ui,
            50.91111,
        );
    }

    #[test]
    fn raster_lines_round_half_up_and_enforce_two_to_thirty_two_pixels() {
        for (height, expected) in [
            (0.25, 2),
            (2.49, 2),
            (2.5, 3),
            (31.49, 31),
            (31.5, 32),
            (100.0, 32),
        ] {
            let metrics = resolve_font_lines(FontLineRequest {
                nominal_height: height,
                pixels_per_ui: 1.0,
                ..request()
            })
            .unwrap();
            assert_eq!(metrics.raster_pixels, expected, "nominal {height}");
        }
    }

    #[test]
    fn raster_lines_spacing_is_ceil_rounded_between_lines_only() {
        let metrics = resolve_font_lines(FontLineRequest {
            spacing: 1.0,
            line_count: 3,
            ..request()
        })
        .unwrap();
        close(metrics.spacing_ui, 2.0 / (1080.0 / 768.0));
        close(metrics.advance_ui, 19.0 / (1080.0 / 768.0));
        close(metrics.block_height_ui, 55.0 / (1080.0 / 768.0));
        let negative = resolve_font_lines(FontLineRequest {
            spacing: -4.0,
            ..request()
        })
        .unwrap();
        close(negative.spacing_ui, 0.0);
        close(negative.block_height_ui, 24.177778);
    }

    #[test]
    fn raster_lines_empty_text_allocates_no_lines() {
        let metrics = resolve_font_lines(FontLineRequest {
            line_count: 0,
            spacing: 2.0,
            ..request()
        })
        .unwrap();
        close(metrics.block_height_ui, 0.0);
    }

    #[test]
    fn raster_lines_reject_invalid_units_and_face_metrics() {
        for input in [
            FontLineRequest {
                pixels_per_ui: 0.0,
                ..request()
            },
            FontLineRequest {
                nominal_height: 0.0,
                ..request()
            },
            FontLineRequest {
                spacing: f32::NAN,
                ..request()
            },
            FontLineRequest {
                face_ascent: 0.0,
                face_descent: 0.0,
                ..request()
            },
        ] {
            assert!(resolve_font_lines(input).is_err());
        }
    }
}
