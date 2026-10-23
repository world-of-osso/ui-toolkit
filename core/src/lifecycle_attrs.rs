//! Strict authored lifecycle descriptors, independent of frame names and native nodes.
use crate::frame::Frame;
use crate::lifecycle::{CloseGroup, EscapePriority};

pub(crate) fn read(frame: &Frame, name: &str) -> Option<String> {
    let binding = frame.lifecycle.as_ref()?;
    match name {
        "lifecycle_on_show" => binding.sounds.on_show.map(|kit| kit.to_string()),
        "lifecycle_on_hide" => binding.sounds.on_hide.map(|kit| kit.to_string()),
        "lifecycle_on_enter" => binding.sounds.on_enter.map(|kit| kit.to_string()),
        "lifecycle_on_click" => binding.sounds.on_click.map(|kit| kit.to_string()),
        "lifecycle_hover_owner" => binding.hover_owner.clone(),
        "lifecycle_close_hook" => binding.close_hook.clone(),
        "lifecycle_escape_hook" => binding.escape_hook.clone(),
        "lifecycle_close_group" => binding.close_group.map(|group| group.as_str().into()),
        "lifecycle_escape_priority" => binding
            .escape_priority
            .map(|priority| format!("{priority:?}")),
        _ => None,
    }
}

pub(crate) fn apply(frame: &mut Frame, name: &str, value: &str) -> bool {
    if !name.starts_with("lifecycle_") {
        return false;
    }
    let binding = frame.lifecycle.get_or_insert_with(Default::default);
    match name {
        "lifecycle_on_show" => binding.sounds.on_show = Some(parse_sound(value)),
        "lifecycle_on_hide" => binding.sounds.on_hide = Some(parse_sound(value)),
        "lifecycle_on_enter" => binding.sounds.on_enter = Some(parse_sound(value)),
        "lifecycle_on_click" => binding.sounds.on_click = Some(parse_sound(value)),
        "lifecycle_hover_owner" => binding.hover_owner = Some(parse_hook(value)),
        "lifecycle_close_hook" => binding.close_hook = Some(parse_hook(value)),
        "lifecycle_escape_hook" => binding.escape_hook = Some(parse_hook(value)),
        "lifecycle_close_group" => binding.close_group = Some(CloseGroup::from_name(value)),
        "lifecycle_escape_priority" => {
            binding.escape_priority = Some(EscapePriority::from_name(value))
        }
        _ => panic!("unsupported lifecycle attribute: {name}"),
    }
    true
}

fn parse_sound(value: &str) -> u32 {
    value
        .parse()
        .expect("lifecycle sound requires an unsigned sound-kit ID")
}

fn parse_hook(value: &str) -> String {
    assert!(
        !value.trim().is_empty(),
        "lifecycle hook/owner requires an identity"
    );
    value.to_owned()
}
