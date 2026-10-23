//! Retail FontString text controls, shared by native text projections.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRun {
    pub text: String,
    /// Explicit RGBA bytes; None keeps the FontString's authored colour.
    pub color: Option<[u8; 4]>,
}

/// Decode supported FontString controls without interpreting display text as another markup language.
/// Unknown controls remain literal; hyperlink targets are metadata, not displayed text.
pub fn parse_text_markup(mut source: &str) -> Vec<TextRun> {
    let mut runs = Vec::new();
    let mut color = None;
    let mut hyperlink = false;
    while let Some(pipe) = source.find('|') {
        append_run(&mut runs, &source[..pipe], color);
        source = &source[pipe..];
        let Some((control, consumed)) = read_control(source, hyperlink) else {
            append_run(&mut runs, "|", color);
            source = &source[1..];
            continue;
        };
        match control {
            Control::Pipe => append_run(&mut runs, "|", color),
            Control::Color(value) => color = Some(value),
            Control::Reset => color = None,
            Control::OpenLink => hyperlink = true,
            Control::CloseLink => hyperlink = false,
        }
        source = &source[consumed..];
    }
    append_run(&mut runs, source, color);
    runs
}

enum Control {
    Pipe,
    Color([u8; 4]),
    Reset,
    OpenLink,
    CloseLink,
}

fn read_control(source: &str, hyperlink: bool) -> Option<(Control, usize)> {
    if source.starts_with("||") {
        return Some((Control::Pipe, 2));
    }
    if source.starts_with("|c") {
        let argb = u32::from_str_radix(source.get(2..10)?, 16).ok()?;
        let [alpha, red, green, blue] = argb.to_be_bytes();
        return Some((Control::Color([red, green, blue, alpha]), 10));
    }
    if source.starts_with("|r") {
        return Some((Control::Reset, 2));
    }
    if let Some(target) = source.strip_prefix("|H") {
        let end = target.find("|h")?;
        return Some((Control::OpenLink, end + 4));
    }
    if hyperlink && source.starts_with("|h") {
        return Some((Control::CloseLink, 2));
    }
    None
}

fn append_run(runs: &mut Vec<TextRun>, text: &str, color: Option<[u8; 4]>) {
    if text.is_empty() {
        return;
    }
    match runs.last_mut() {
        Some(previous) if previous.color == color => previous.text.push_str(text),
        _ => runs.push(TextRun {
            text: text.into(),
            color,
        }),
    }
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
    fn text_markup_link_without_explicit_colour_retains_the_font_colour() {
        assert_eq!(
            parse_text_markup("Use |Hspell:372158|h[Sundering Strike]|h."),
            vec![run("Use [Sundering Strike].", None),]
        );
    }
}
