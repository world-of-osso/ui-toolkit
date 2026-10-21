//! Retail button presets and authored three-slice geometry, independent of source identity.
use crate::frame::{Frame, WidgetData};
use crate::widgets::button::{ButtonData, ButtonState, ButtonTexture};
use crate::widgets::texture::{BlendMode, TextureSource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonTemplate {
    SharedRed,
    Panel,
    Close,
}

impl ButtonTemplate {
    pub fn from_name(name: &str) -> Self {
        match name {
            "SharedButtonTemplate"
            | "SharedButtonLargeTemplate"
            | "SharedButtonSmallTemplate"
            | "SharedButtonExtraSmallTemplate" => Self::SharedRed,
            "UIPanelButtonTemplate" => Self::Panel,
            "UIPanelCloseButton" => Self::Close,
            _ => panic!("unsupported Retail button template: {name}"),
        }
    }
}

/// One drawable state region in local screen coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct ButtonArtPart {
    pub rect: [f32; 4],
    pub texture: ButtonTexture,
    pub crop: [f32; 4],
}

/// Declaration inheritance precedes local state-region overrides, as with font objects.
pub(crate) fn apply_button_template(frame: &mut Frame, name: Option<&str>) {
    let template = name.map(ButtonTemplate::from_name);
    let Some(WidgetData::Button(button)) = &mut frame.widget_data else {
        assert!(template.is_none(), "button_template requires a Button");
        return;
    };
    if template.is_none() && button.template.is_none() {
        return;
    }
    button.template = template;
    let inherited = template.map(template_states).unwrap_or_default();
    button.normal_texture = inherited[0].clone();
    button.pushed_texture = inherited[1].clone();
    button.disabled_texture = inherited[2].clone();
    button.highlight_texture = inherited[3].clone();
    button.use_default_skin = template.is_none();
}

fn template_states(template: ButtonTemplate) -> [Option<ButtonTexture>; 4] {
    let sources = match template {
        ButtonTemplate::SharedRed => [
            atlas("128-redbutton-left"),
            atlas("128-redbutton-left-pressed"),
            atlas("128-redbutton-left-disabled"),
            atlas("retail-128-redbutton-highlight"),
        ],
        ButtonTemplate::Panel => [
            TextureSource::FileDataId(130828),
            TextureSource::FileDataId(130825),
            TextureSource::FileDataId(130824),
            TextureSource::FileDataId(130826),
        ],
        ButtonTemplate::Close => [
            atlas("RedButton-Exit"),
            atlas("RedButton-exit-pressed"),
            atlas("RedButton-Exit-Disabled"),
            atlas("RedButton-Highlight"),
        ],
    };
    let mut states = sources.map(|source| Some(ButtonTexture::new(source)));
    // SharedButton InitButton omits a blend argument; legacy Panel/Close explicitly ADD.
    if template != ButtonTemplate::SharedRed {
        states[3].as_mut().unwrap().blend_mode = BlendMode::Additive;
    }
    states
}

fn atlas(name: &str) -> TextureSource {
    TextureSource::Atlas(name.into())
}

/// Resolve authored geometry once; plain texture sources never opt into a template.
pub fn button_art_parts(
    button: &ButtonData,
    width: f32,
    height: f32,
) -> Option<Vec<ButtonArtPart>> {
    let template = button.template?;
    let mut parts = match (template, button.base_texture()) {
        (_, None) => Vec::new(),
        (ButtonTemplate::SharedRed, Some(texture)) => {
            shared_red_parts(texture, button.state, width, height)
        }
        (ButtonTemplate::Panel, Some(texture)) => panel_parts(texture, width, height),
        (ButtonTemplate::Close, Some(texture)) => vec![part(
            texture.clone(),
            [0.0, 0.0, width, height],
            [0.0, 1.0, 0.0, 1.0],
        )],
    };
    if let Some(texture) = button.highlight_region() {
        let crop = if template == ButtonTemplate::Panel {
            [0.0, 0.625, 0.0, 0.6875]
        } else {
            [0.0, 1.0, 0.0, 1.0]
        };
        parts.push(part(texture.clone(), [0.0, 0.0, width, height], crop));
    }
    Some(parts)
}

/// ThreeSliceButtonMixin:UpdateScale: trim the larger cap, then equalize if needed.
fn cropped_cap_widths(width: f32, left: f32, right: f32) -> [f32; 2] {
    let excess = left + right - width;
    if excess <= 0.0 {
        [left, right]
    } else if left - excess > right {
        [left - excess, right]
    } else if right - excess > left {
        [left, right - excess]
    } else {
        [width / 2.0, width / 2.0]
    }
}

fn shared_red_parts(
    texture: &ButtonTexture,
    state: ButtonState,
    width: f32,
    height: f32,
) -> Vec<ButtonArtPart> {
    // The Retail128px left/right members are114/292 wide in both approved skins.
    let scale = height / 128.0;
    let original = [114.0 * scale, 292.0 * scale];
    let [left, right] = cropped_cap_widths(width, original[0], original[1]);
    let suffix = match state {
        ButtonState::Normal => "",
        ButtonState::Pushed => "-pressed",
        ButtonState::Disabled => "-disabled",
    };
    [
        (
            "128-redbutton-left",
            [0.0, 0.0, left, height],
            [0.0, left / original[0], 0.0, 1.0],
        ),
        (
            "_128-redbutton-center",
            [left, 0.0, width - left - right, height],
            [0.0, 1.0, 0.0, 1.0],
        ),
        (
            "128-redbutton-right",
            [width - right, 0.0, right, height],
            [1.0 - right / original[1], 1.0, 0.0, 1.0],
        ),
    ]
    .into_iter()
    .map(|(name, rect, crop)| {
        let region = ButtonTexture {
            source: atlas(&format!("{name}{suffix}")),
            ..texture.clone()
        };
        part(region, rect, crop)
    })
    .collect()
}

fn panel_parts(texture: &ButtonTexture, width: f32, height: f32) -> Vec<ButtonArtPart> {
    // SecureUIPanelTemplates.xml:39–78: fixed12px caps, cropped80×22 strip.
    [
        ([0.0, 0.0, 12.0, height], [0.0, 0.09375, 0.0, 0.6875]),
        (
            [12.0, 0.0, width - 24.0, height],
            [0.09375, 0.53125, 0.0, 0.6875],
        ),
        (
            [width - 12.0, 0.0, 12.0, height],
            [0.53125, 0.625, 0.0, 0.6875],
        ),
    ]
    .into_iter()
    .map(|(rect, crop)| part(texture.clone(), rect, crop))
    .collect()
}

fn part(texture: ButtonTexture, rect: [f32; 4], crop: [f32; 4]) -> ButtonArtPart {
    ButtonArtPart {
        rect,
        texture,
        crop,
    }
}
