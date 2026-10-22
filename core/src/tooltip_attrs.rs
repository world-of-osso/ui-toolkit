//! Attribute round-trip for source tooltip keys; no panel/source-name classification.
use crate::frame::Frame;
use crate::tooltip::TooltipAnchor;

pub(crate) fn read(frame: &Frame, name: &str) -> Option<String> {
    let binding = frame.tooltip.as_ref()?;
    match name {
        "tooltip_provider" => binding.provider.clone(),
        "tooltip_title" => binding.title.clone(),
        "tooltip_text" => binding.text.clone(),
        "tooltip_title_color" => binding.title_color.map(color_string),
        "tooltip_text_color" => binding.text_color.map(color_string),
        "tooltip_anchor" => Some(binding.anchor.as_str().into()),
        "tooltip_offset_x" => Some(binding.offset[0].to_string()),
        "tooltip_offset_y" => Some(binding.offset[1].to_string()),
        "tooltip_wrap" => Some(binding.wrap.to_string()),
        "disabled_tooltip" => binding.disabled_text.clone(),
        "disabled_tooltip_anchor" => Some(binding.disabled_anchor.as_str().into()),
        "disabled_tooltip_offset_x" => Some(binding.disabled_offset[0].to_string()),
        "disabled_tooltip_offset_y" => Some(binding.disabled_offset[1].to_string()),
        _ => None,
    }
}

pub(crate) fn apply(frame: &mut Frame, name: &str, value: &str) -> bool {
    if !name.starts_with("tooltip_") && !name.starts_with("disabled_tooltip") {
        return false;
    }
    let binding = frame.tooltip.get_or_insert_with(Default::default);
    match name {
        "tooltip_provider" => {
            assert!(
                !value.trim().is_empty(),
                "tooltip_provider requires a content key"
            );
            binding.provider = Some(value.into());
        }
        "tooltip_title" => binding.title = Some(value.into()),
        "tooltip_text" => binding.text = Some(value.into()),
        "tooltip_title_color" => binding.title_color = Some(parse_color(value)),
        "tooltip_text_color" => binding.text_color = Some(parse_color(value)),
        "tooltip_anchor" => binding.anchor = TooltipAnchor::from_name(value),
        "tooltip_offset_x" => binding.offset[0] = parse_offset(value),
        "tooltip_offset_y" => binding.offset[1] = parse_offset(value),
        "tooltip_wrap" => {
            binding.wrap = value.parse().expect("tooltip_wrap requires true or false")
        }
        "disabled_tooltip" => binding.disabled_text = Some(value.into()),
        "disabled_tooltip_anchor" => binding.disabled_anchor = TooltipAnchor::from_name(value),
        "disabled_tooltip_offset_x" => binding.disabled_offset[0] = parse_offset(value),
        "disabled_tooltip_offset_y" => binding.disabled_offset[1] = parse_offset(value),
        _ => panic!("unsupported tooltip attribute: {name}"),
    }
    true
}

fn parse_offset(value: &str) -> f32 {
    let offset: f32 = value.parse().expect("tooltip offset requires a number");
    assert!(offset.is_finite(), "tooltip offset must be finite");
    offset
}

fn parse_color(value: &str) -> [f32; 4] {
    let color = crate::attrs::parse_color(value).expect("tooltip color requires four RGBA numbers");
    assert!(
        color.iter().all(|value| value.is_finite()),
        "tooltip color must be finite"
    );
    color
}

fn color_string([r, g, b, a]: [f32; 4]) -> String {
    format!("{r},{g},{b},{a}")
}
