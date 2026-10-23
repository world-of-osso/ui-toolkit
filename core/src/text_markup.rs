//! Retail FontString text controls, shared by native text projections.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRun {
    pub text: String,
    /// Explicit RGBA bytes; None keeps the FontString's authored colour.
    pub color: Option<[u8; 4]>,
}

/// Current literal-text baseline. Markup regressions are intentionally RED.
pub fn parse_text_markup(source: &str) -> Vec<TextRun> {
    vec![TextRun {
        text: source.to_owned(),
        color: None,
    }]
}

pub fn display_text(runs: &[TextRun]) -> String {
    runs.iter().map(|run| run.text.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str, color: Option<[u8; 4]>) -> TextRun {
        TextRun {
            text: text.into(),
            color,
        }
    }

    #[test]
    fn text_markup_preserves_plain_unicode_and_brackets() {
        let text = "Kurog’s [elementals] remain alive.";
        assert_eq!(parse_text_markup(text), vec![run(text, None)]);
        assert_eq!(display_text(&parse_text_markup(text)), text);
    }

    #[test]
    #[ignore = "RED: Kurog25033 body still paints color and spell-link controls literally"]
    fn text_markup_kurog_skeletal_fractures_renders_authored_blue_display_text() {
        let source =
            "Earth Breakers continually cast |cFF2959D3|Hspell:377780|h[Skeletal Fractures]|h|r.";
        let runs = parse_text_markup(source);
        assert_eq!(
            runs,
            vec![
                run("Earth Breakers continually cast ", None),
                run("[Skeletal Fractures]", Some([41, 89, 211, 255])),
                run(".", None),
            ]
        );
        assert_eq!(
            display_text(&runs),
            "Earth Breakers continually cast [Skeletal Fractures]."
        );
    }

    #[test]
    #[ignore = "RED: Kurog25035 controls are not decoded into independent color runs"]
    fn text_markup_kurog_tank_links_reset_to_the_authored_font_colour() {
        let source = "|cFF2959D3|Hspell:372158|h[Sundering Strike]|h|r applies stacking Physical damage over time.\nWhile not next to an active Primal Altar, Kurog Grimtotem casts |cFF2959D3|Hspell:371971|h[Elemental Surge]|h|r.";
        assert_eq!(
            parse_text_markup(source),
            vec![
                run("[Sundering Strike]", Some([41, 89, 211, 255])),
                run(
                    " applies stacking Physical damage over time.\nWhile not next to an active Primal Altar, Kurog Grimtotem casts ",
                    None
                ),
                run("[Elemental Surge]", Some([41, 89, 211, 255])),
                run(".", None),
            ]
        );
    }

    #[test]
    #[ignore = "RED: escaped pipes and ARGB alpha are not decoded"]
    fn text_markup_escaped_pipes_stay_literal_inside_and_outside_coloured_links() {
        let source = "||cFF2959D3 |c802959D3|Hspell:377780|h[Skeletal || Fractures]|h|r ||";
        assert_eq!(
            parse_text_markup(source),
            vec![
                run("|cFF2959D3 ", None),
                run("[Skeletal | Fractures]", Some([41, 89, 211, 128])),
                run(" |", None),
            ]
        );
    }

    #[test]
    #[ignore = "RED: standalone hyperlink controls still show their metadata"]
    fn text_markup_link_without_explicit_colour_retains_the_font_colour() {
        assert_eq!(
            parse_text_markup("Use |Hspell:372158|h[Sundering Strike]|h."),
            vec![run("Use [Sundering Strike].", None),]
        );
    }
}
