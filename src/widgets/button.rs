use super::texture::TextureSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Pushed,
    Disabled,
}

#[derive(Debug, Clone)]
pub struct ButtonData {
    pub state: ButtonState,
    pub enabled: bool,
    pub hovered: bool,
    pub normal_texture: Option<TextureSource>,
    pub pushed_texture: Option<TextureSource>,
    pub highlight_texture: Option<TextureSource>,
    pub disabled_texture: Option<TextureSource>,
    pub text: String,
    pub font_size: f32,
    pub pushed_text_offset: [f32; 2],
    pub highlight_locked: bool,
}

impl Default for ButtonData {
    fn default() -> Self {
        Self {
            state: ButtonState::Normal,
            enabled: true,
            hovered: false,
            normal_texture: None,
            pushed_texture: None,
            highlight_texture: None,
            disabled_texture: None,
            text: String::new(),
            font_size: 14.0,
            pushed_text_offset: [0.0, 0.0],
            highlight_locked: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CheckButtonData {
    pub button: ButtonData,
    pub checked: bool,
    pub checked_texture: Option<TextureSource>,
    pub disabled_checked_texture: Option<TextureSource>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_button_data() {
        let b = ButtonData::default();
        assert_eq!(b.state, ButtonState::Normal);
        assert!(b.enabled);
        assert!(b.text.is_empty());
    }

    #[test]
    fn default_check_button_data() {
        let cb = CheckButtonData::default();
        assert!(!cb.checked);
        assert!(cb.button.enabled);
    }
}
