use super::HotReloadTemplate;
use crate::widget_def::{AnchorDef, Attr, AttrValue, WidgetChild, WidgetDef};

/// Parse all `rsx! { ... }` blocks in a source file.
pub fn parse_rsx_blocks(source: &str, file_path: &str) -> Vec<HotReloadTemplate> {
    let mut results = Vec::new();
    let mut search_from = 0;
    while let Some(pos) = source[search_from..].find("rsx!") {
        let abs_pos = search_from + pos;
        let after_macro = abs_pos + 4;
        let trimmed = source[after_macro..].trim_start();
        if !trimmed.starts_with('{') {
            search_from = after_macro;
            continue;
        }
        let brace_pos = after_macro + (source[after_macro..].len() - trimmed.len());
        let Some(block) = extract_braced_block(&source[brace_pos..]) else {
            search_from = brace_pos + 1;
            continue;
        };
        let line = source[..abs_pos].matches('\n').count() as u32 + 1;
        let col = (abs_pos - source[..abs_pos].rfind('\n').map(|p| p + 1).unwrap_or(0)) as u32 + 1;
        let consts = parse_const_declarations(source);
        results.push(HotReloadTemplate {
            key: (file_path.to_string(), line, col),
            defs: parse_children(block.trim(), &consts),
        });
        search_from = brace_pos + block.len() + 2;
    }
    results
}

/// Extract content between matched braces (excluding the braces themselves).
fn extract_braced_block(s: &str) -> Option<&str> {
    if !s.starts_with('{') {
        return None;
    }
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&s[1..i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Parse `const NAME: Type = Type("value")` or `const NAME: &str = "value"` declarations.
pub fn parse_const_declarations(source: &str) -> Vec<(String, String)> {
    let mut consts = Vec::new();
    let mut rest = source;
    while let Some(pos) = rest.find("const ") {
        rest = &rest[pos + 6..];
        let name_end = rest
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(rest.len());
        let name = &rest[..name_end];
        if name.is_empty() {
            continue;
        }
        let Some(eq_pos) = rest.find('=') else {
            continue;
        };
        let rhs = rest[eq_pos + 1..].trim_start();
        let value = extract_const_value(rhs);
        if let Some(v) = value {
            consts.push((name.to_string(), v));
        }
    }
    consts
}

fn extract_const_value(rhs: &str) -> Option<String> {
    if rhs.starts_with('"') {
        return extract_string_literal(rhs);
    }
    let paren_pos = rhs.find('(')?;
    let after_paren = rhs[paren_pos + 1..].trim_start();
    if after_paren.starts_with('"') {
        extract_string_literal(after_paren)
    } else {
        None
    }
}

/// Extract the string content from a `"..."` literal at the start of `s`.
fn extract_string_literal(s: &str) -> Option<String> {
    if !s.starts_with('"') {
        return None;
    }
    let mut chars = s[1..].chars();
    let mut result = String::new();
    loop {
        match chars.next()? {
            '\\' => match chars.next()? {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                c => {
                    result.push('\\');
                    result.push(c);
                }
            },
            '"' => return Some(result),
            c => result.push(c),
        }
    }
}

fn is_ws_or_comma(b: u8) -> bool {
    matches!(b, b' ' | b'\n' | b'\r' | b'\t' | b',')
}

fn skip_whitespace(input: &str, pos: usize) -> usize {
    let bytes = input.as_bytes();
    let mut p = pos;
    while p < input.len()
        && (bytes[p] == b' ' || bytes[p] == b'\n' || bytes[p] == b'\r' || bytes[p] == b'\t')
    {
        p += 1;
    }
    p
}

fn read_ident(input: &str, pos: usize) -> (usize, &str) {
    let bytes = input.as_bytes();
    let start = pos;
    let mut p = pos;
    while p < input.len() && (bytes[p].is_ascii_alphanumeric() || bytes[p] == b'_') {
        p += 1;
    }
    (p, &input[start..p])
}

/// Parse children from RSX block content (recursive descent).
fn parse_children(input: &str, consts: &[(String, String)]) -> Vec<WidgetChild> {
    let mut children = Vec::new();
    let mut pos = 0;
    let bytes = input.as_bytes();
    while pos < input.len() {
        pos = skip_child_delimiters(input, pos, bytes);
        if pos >= input.len() {
            break;
        }
        let (new_pos, child) = parse_child_at(input, pos, consts, bytes);
        pos = new_pos;
        if let Some(child) = child {
            children.push(child);
        }
    }
    children
}

fn skip_child_delimiters(input: &str, mut pos: usize, bytes: &[u8]) -> usize {
    while pos < input.len() && is_ws_or_comma(bytes[pos]) {
        pos += 1;
    }
    pos
}

fn parse_child_at(
    input: &str,
    pos: usize,
    consts: &[(String, String)],
    bytes: &[u8],
) -> (usize, Option<WidgetChild>) {
    if input[pos..].starts_with("if ") || bytes[pos] == b'{' {
        let (new_pos, child) = parse_dynamic_block(input, pos);
        return (new_pos, Some(child));
    }

    let (new_pos, ident) = read_ident(input, pos);
    if new_pos == pos {
        return (pos + 1, None);
    }

    let attr_pos = skip_whitespace(input, new_pos);
    if attr_pos < input.len() && bytes[attr_pos] == b':' {
        return (skip_attr_value_line(input, attr_pos), None);
    }

    parse_named_child(input, attr_pos, ident, consts, bytes)
}

fn parse_named_child(
    input: &str,
    pos: usize,
    ident: &str,
    consts: &[(String, String)],
    bytes: &[u8],
) -> (usize, Option<WidgetChild>) {
    if pos >= input.len() || bytes[pos] != b'{' {
        return (pos, None);
    }
    let Some(block) = extract_braced_block(&input[pos..]) else {
        return (pos + 1, None);
    };
    let consumed = block.len() + 2;
    let child = dispatch_element(ident, block.trim(), consts);
    (pos + consumed, Some(child))
}

fn skip_attr_value_line(input: &str, pos: usize) -> usize {
    if let Some(nl) = input[pos..].find('\n') {
        pos + nl + 1
    } else {
        input.len()
    }
}

fn parse_dynamic_block(input: &str, pos: usize) -> (usize, WidgetChild) {
    let brace_start = skip_to_brace(&input[pos..]);
    let brace_abs = pos + (input[pos..].len() - brace_start.len());
    let block = extract_braced_block(&input[brace_abs..]);
    let after = brace_abs + block.map(|b| b.len() + 2).unwrap_or(1);
    let after = skip_else_block(input, after);
    (after, WidgetChild::Dynamic)
}

fn skip_else_block(input: &str, pos: usize) -> usize {
    let rest = input[pos..].trim_start();
    if !rest.starts_with("else") {
        return pos;
    }
    let else_start = pos + (input[pos..].len() - rest.len()) + 4;
    let rest2 = input[else_start..].trim_start();
    let brace2 = else_start + (input[else_start..].len() - rest2.len());
    match extract_braced_block(&input[brace2..]) {
        Some(b) => brace2 + b.len() + 2,
        None => pos,
    }
}

fn skip_to_brace(s: &str) -> &str {
    if let Some(p) = s.find('{') {
        &s[p..]
    } else {
        s
    }
}

fn dispatch_element(tag: &str, block: &str, consts: &[(String, String)]) -> WidgetChild {
    if tag == "anchor" {
        parse_anchor_element(block, consts)
    } else {
        parse_element(tag, block, consts)
    }
}

fn parse_anchor_element(block: &str, consts: &[(String, String)]) -> WidgetChild {
    let mut def = AnchorDef::default();
    let attrs = collect_attrs(block, consts);
    for a in &attrs {
        match a.effective_name() {
            "point" => def.point = a.value_str().to_string(),
            "relative_to" => def.relative_to = a.value_str().to_string(),
            "relative_point" => def.relative_point = a.value_str().to_string(),
            "x" => def.x = a.value_str().to_string(),
            "y" => def.y = a.value_str().to_string(),
            _ => {}
        }
    }
    let mut widget = WidgetDef::new("");
    widget.tag_owned = Some("anchor".to_string());
    widget.attrs = attrs;
    let _ = def;
    WidgetChild::Widget(widget)
}

fn parse_element(tag: &str, block: &str, consts: &[(String, String)]) -> WidgetChild {
    let mut widget = WidgetDef::new("");
    widget.tag_owned = Some(tag.to_string());
    widget.attrs = collect_attrs(block, consts);
    if let Some(name_attr) = widget.attrs.iter().find(|a| a.effective_name() == "name") {
        if let AttrValue::Static(ref s) = name_attr.value {
            widget.name = Some(s.clone());
        }
    }
    attach_children_and_anchors(&mut widget, block, consts);
    WidgetChild::Widget(widget)
}

fn attach_children_and_anchors(widget: &mut WidgetDef, block: &str, consts: &[(String, String)]) {
    for child in parse_children(block, consts) {
        match child {
            WidgetChild::Widget(ref w) if w.tag_owned.as_deref() == Some("anchor") => {
                widget.anchors.push(anchor_from_widget(w));
            }
            other => widget.children.push(other),
        }
    }
}

fn anchor_from_widget(w: &WidgetDef) -> AnchorDef {
    let mut a = AnchorDef::default();
    for attr in &w.attrs {
        match attr.effective_name() {
            "point" => a.point = attr.value_str().to_string(),
            "relative_to" => a.relative_to = attr.value_str().to_string(),
            "relative_point" => a.relative_point = attr.value_str().to_string(),
            "x" => a.x = attr.value_str().to_string(),
            "y" => a.y = attr.value_str().to_string(),
            _ => {}
        }
    }
    a
}

/// Collect all `name: value,` attribute pairs from a block.
fn collect_attrs(block: &str, consts: &[(String, String)]) -> Vec<Attr> {
    let mut attrs = Vec::new();
    let mut pos = 0;
    let bytes = block.as_bytes();
    while pos < block.len() {
        pos = skip_attr_delimiters(block, pos, bytes);
        if pos >= block.len() {
            break;
        }
        match collect_attr_step(block, pos, consts, bytes) {
            AttrStep::Stop => break,
            AttrStep::Continue { next_pos, attr } => {
                pos = next_pos;
                if let Some(attr) = attr {
                    attrs.push(attr);
                }
            }
        }
    }
    attrs
}

fn skip_attr_delimiters(block: &str, mut pos: usize, bytes: &[u8]) -> usize {
    while pos < block.len() && is_ws_or_comma(bytes[pos]) {
        pos += 1;
    }
    pos
}

enum AttrStep {
    Stop,
    Continue { next_pos: usize, attr: Option<Attr> },
}

fn collect_attr_step(
    block: &str,
    pos: usize,
    consts: &[(String, String)],
    bytes: &[u8],
) -> AttrStep {
    let (name_end, name) = read_ident(block, pos);
    if name_end == pos {
        return AttrStep::Continue {
            next_pos: pos + 1,
            attr: None,
        };
    }

    let key_pos = skip_whitespace(block, name_end);
    if key_pos < block.len() && bytes[key_pos] == b'{' {
        return AttrStep::Stop;
    }
    if key_pos >= block.len() || bytes[key_pos] != b':' {
        return AttrStep::Continue {
            next_pos: skip_attr_value_line(block, key_pos),
            attr: None,
        };
    }

    let value_pos = skip_whitespace(block, key_pos + 1);
    if value_pos >= block.len() {
        return AttrStep::Stop;
    }

    let (value_opt, consumed) = parse_attr_value(&block[value_pos..], consts);
    AttrStep::Continue {
        next_pos: value_pos + consumed,
        attr: value_opt.map(|value| build_attr(name, value)),
    }
}

fn build_attr(name: &str, value: String) -> Attr {
    let mut attr = Attr::new_static("", value);
    attr.name_owned = Some(name.to_string());
    attr
}

fn parse_attr_value(s: &str, consts: &[(String, String)]) -> (Option<String>, usize) {
    let trimmed = s.trim_start();
    let leading = s.len() - trimmed.len();
    if let Some((val, n)) = try_parse_string_literal(trimmed) {
        return (Some(val), leading + n);
    }
    if let Some((val, n)) = try_parse_number(trimmed) {
        return (Some(val), leading + n);
    }
    if let Some((val, n)) = try_parse_bool(trimmed) {
        return (Some(val), leading + n);
    }
    let (ident, ident_len) = read_qualified_ident(trimmed);
    if let Some((val, n)) = try_parse_known_ident(&ident, ident_len, trimmed, consts) {
        return (Some(val), leading + n);
    }
    (None, leading + skip_value(trimmed))
}

fn try_parse_string_literal(s: &str) -> Option<(String, usize)> {
    if !s.starts_with('"') {
        return None;
    }
    let val = extract_string_literal(s)?;
    let n = scan_string_literal_len(s);
    Some((val, n))
}

fn try_parse_number(s: &str) -> Option<(String, usize)> {
    let end = s
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
        .unwrap_or(s.len());
    if end == 0 {
        return None;
    }
    let first = s.as_bytes()[0];
    if !first.is_ascii_digit() && !(first == b'-' && end > 1 && s.as_bytes()[1].is_ascii_digit()) {
        return None;
    }
    let num_str = s[..end].trim_end_matches('.');
    Some((num_str.to_string(), end))
}

fn try_parse_bool(s: &str) -> Option<(String, usize)> {
    let not_ident = |c: char| !c.is_alphanumeric() && c != '_';
    if s.starts_with("true") && s[4..].starts_with(not_ident) {
        return Some(("true".to_string(), 4));
    }
    if s.starts_with("false") && s[5..].starts_with(not_ident) {
        return Some(("false".to_string(), 5));
    }
    None
}

fn try_parse_known_ident(
    ident: &str,
    ident_len: usize,
    s: &str,
    consts: &[(String, String)],
) -> Option<(String, usize)> {
    if ident == "FontColor::new" {
        let content = extract_paren_content(&s[ident_len..])?;
        let paren_total = 1 + content.len() + 1;
        return Some((content.to_string(), ident_len + paren_total));
    }
    if let Some(v) = ident.strip_prefix("FrameStrata::") {
        return Some((v.to_string(), ident_len));
    }
    if let Some(v) = ident.strip_prefix("DrawLayer::") {
        return Some((v.to_string(), ident_len));
    }
    if let Some(v) = ident.strip_prefix("GameFont::") {
        return Some((v.to_string(), ident_len));
    }
    if let Some(v) = ident.strip_prefix("AnchorPoint::") {
        return Some((map_anchor_point(v).to_string(), ident_len));
    }
    if let Some(v) = ident.strip_prefix("JustifyH::") {
        return Some((map_justify_h(v).to_string(), ident_len));
    }
    if !ident.contains("::") && !ident.is_empty() {
        if let Some((_, val)) = consts.iter().find(|(k, _)| k == ident) {
            return Some((val.clone(), ident_len));
        }
    }
    None
}

fn read_qualified_ident(s: &str) -> (String, usize) {
    let mut pos = 0;
    let bytes = s.as_bytes();
    loop {
        let seg_start = pos;
        while pos < s.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
            pos += 1;
        }
        if pos == seg_start {
            break;
        }
        if pos + 1 < s.len() && bytes[pos] == b':' && bytes[pos + 1] == b':' {
            pos += 2;
        } else {
            break;
        }
    }
    (s[..pos].to_string(), pos)
}

fn extract_paren_content(s: &str) -> Option<&str> {
    let s = s.trim_start();
    if !s.starts_with('(') {
        return None;
    }
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&s[1..i]);
                }
            }
            _ => {}
        }
    }
    None
}

fn map_anchor_point(v: &str) -> &str {
    match v {
        "TopLeft" => "TOPLEFT",
        "Top" => "TOP",
        "TopRight" => "TOPRIGHT",
        "Left" => "LEFT",
        "Center" => "CENTER",
        "Right" => "RIGHT",
        "BottomLeft" => "BOTTOMLEFT",
        "Bottom" => "BOTTOM",
        "BottomRight" => "BOTTOMRIGHT",
        other => other,
    }
}

fn map_justify_h(v: &str) -> &str {
    match v {
        "Left" => "LEFT",
        "Right" => "RIGHT",
        "Center" => "CENTER",
        other => other,
    }
}

fn scan_string_literal_len(s: &str) -> usize {
    if !s.starts_with('"') {
        return 0;
    }
    let mut pos = 1;
    let bytes = s.as_bytes();
    while pos < s.len() {
        if bytes[pos] == b'\\' {
            pos += 2;
            continue;
        }
        if bytes[pos] == b'"' {
            return pos + 1;
        }
        pos += 1;
    }
    pos
}

fn skip_value(s: &str) -> usize {
    let mut depth = 0i32;
    let mut pos = 0;
    let bytes = s.as_bytes();
    while pos < s.len() {
        match bytes[pos] {
            b'"' => {
                let n = scan_string_literal_len(&s[pos..]);
                pos += if n == 0 { 1 } else { n };
                continue;
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                if depth == 0 {
                    return pos;
                }
                depth -= 1;
            }
            b',' | b'\n' if depth == 0 => return pos,
            _ => {}
        }
        pos += 1;
    }
    pos
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget_def::WidgetChild;

    fn find_attr<'a>(attrs: &'a [Attr], name: &str) -> Option<&'a str> {
        attrs
            .iter()
            .find(|a| a.effective_name() == name)
            .map(|a| a.value_str())
    }

    #[test]
    fn parse_simple_element() {
        let source = r#"rsx! { frame { name: "Foo", width: 320.0 } }"#;
        let templates = parse_rsx_blocks(source, "test.rs");
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].defs.len(), 1);
        let WidgetChild::Widget(ref w) = templates[0].defs[0] else {
            panic!("expected Widget")
        };
        assert_eq!(w.effective_tag(), "frame");
        assert_eq!(find_attr(&w.attrs, "name"), Some("Foo"));
        assert_eq!(find_attr(&w.attrs, "width"), Some("320.0"));
    }

    #[test]
    fn parse_nested_children() {
        let source = r#"rsx! {
            frame {
                name: "Outer",
                frame {
                    name: "Inner",
                }
            }
        }"#;
        let templates = parse_rsx_blocks(source, "test.rs");
        assert_eq!(templates.len(), 1);
        let WidgetChild::Widget(ref outer) = templates[0].defs[0] else {
            panic!()
        };
        assert_eq!(outer.children.len(), 1);
        let WidgetChild::Widget(ref inner) = outer.children[0] else {
            panic!()
        };
        assert_eq!(find_attr(&inner.attrs, "name"), Some("Inner"));
    }

    #[test]
    fn parse_anchor_pseudo_element() {
        let source = r#"rsx! {
            frame {
                name: "Anchored",
                anchor {
                    point: AnchorPoint::Center,
                    relative_to: "$parent",
                    relative_point: AnchorPoint::Center,
                    x: 0.0,
                    y: 0.0,
                }
            }
        }"#;
        let templates = parse_rsx_blocks(source, "test.rs");
        let WidgetChild::Widget(ref frame) = templates[0].defs[0] else {
            panic!()
        };
        assert_eq!(frame.anchors.len(), 1);
        assert_eq!(frame.anchors[0].point, "CENTER");
        assert_eq!(frame.anchors[0].relative_to, "$parent");
        assert_eq!(frame.anchors[0].relative_point, "CENTER");
    }

    #[test]
    fn parse_const_resolution() {
        let source = r#"
const MY_FRAME: FrameName = FrameName("LoginScreen");
fn build() -> Element { rsx! { frame { name: MY_FRAME } } }
"#;
        let templates = parse_rsx_blocks(source, "test.rs");
        assert_eq!(templates.len(), 1);
        let WidgetChild::Widget(ref w) = templates[0].defs[0] else {
            panic!()
        };
        assert_eq!(find_attr(&w.attrs, "name"), Some("LoginScreen"));
    }

    #[test]
    fn parse_enum_variants() {
        let source = r#"rsx! { frame { strata: FrameStrata::Dialog, justify_h: JustifyH::Left } }"#;
        let templates = parse_rsx_blocks(source, "test.rs");
        let WidgetChild::Widget(ref w) = templates[0].defs[0] else {
            panic!()
        };
        assert_eq!(find_attr(&w.attrs, "strata"), Some("Dialog"));
        assert_eq!(find_attr(&w.attrs, "justify_h"), Some("LEFT"));
    }

    #[test]
    fn parse_bool_and_number() {
        let source =
            r#"rsx! { frame { hidden: false, mouse_enabled: false, alpha: 0.5, frame_level: 3 } }"#;
        let templates = parse_rsx_blocks(source, "test.rs");
        let WidgetChild::Widget(ref w) = templates[0].defs[0] else {
            panic!()
        };
        assert_eq!(find_attr(&w.attrs, "hidden"), Some("false"));
        assert_eq!(find_attr(&w.attrs, "mouse_enabled"), Some("false"));
        assert_eq!(find_attr(&w.attrs, "alpha"), Some("0.5"));
        assert_eq!(find_attr(&w.attrs, "frame_level"), Some("3"));
    }

    #[test]
    fn dynamic_expressions_produce_dynamic_child() {
        let source = r#"rsx! { frame { { some_var } } }"#;
        let templates = parse_rsx_blocks(source, "test.rs");
        let WidgetChild::Widget(ref w) = templates[0].defs[0] else {
            panic!()
        };
        assert_eq!(w.children.len(), 1);
        assert!(matches!(w.children[0], WidgetChild::Dynamic));
    }
}
