use super::texture::{BlendMode, TextureSource};

/// Authored artwork for one button state, independent of its source identity.
#[derive(Debug, Clone, PartialEq)]
pub struct ButtonTexture {
    pub source: TextureSource,
    pub blend_mode: BlendMode,
    pub tex_coords: [f32; 4],
    pub vertex_color: [f32; 4],
    pub alpha: f32,
    pub desaturated: bool,
    /// Region size and screen-space offset, independent of the button hit area.
    pub size: Option<[f32; 2]>,
    pub offset: [f32; 2],
}

impl ButtonTexture {
    pub fn new(source: TextureSource) -> Self {
        Self {
            source,
            ..Self::default()
        }
    }
}

impl Default for ButtonTexture {
    fn default() -> Self {
        Self {
            source: TextureSource::None,
            blend_mode: BlendMode::Blend,
            tex_coords: [0.0, 1.0, 0.0, 1.0],
            vertex_color: [1.0; 4],
            alpha: 1.0,
            desaturated: false,
            size: None,
            offset: [0.0; 2],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Pushed,
    Disabled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ButtonData {
    pub template: Option<crate::button_art::ButtonTemplate>,
    pub state: ButtonState,
    pub enabled: bool,
    pub hovered: bool,
    /// Whether an untextured button receives the toolkit's default nine-slice skin.
    pub use_default_skin: bool,
    pub normal_texture: Option<ButtonTexture>,
    pub pushed_texture: Option<ButtonTexture>,
    pub highlight_texture: Option<ButtonTexture>,
    pub disabled_texture: Option<ButtonTexture>,
    pub checked: bool,
    pub checked_texture: Option<ButtonTexture>,
    pub disabled_checked_texture: Option<ButtonTexture>,
    pub text: String,
    pub font_size: f32,
    pub pushed_text_offset: [f32; 2],
    pub highlight_locked: bool,
}

impl Default for ButtonData {
    fn default() -> Self {
        Self {
            template: None,
            state: ButtonState::Normal,
            enabled: true,
            hovered: false,
            use_default_skin: true,
            normal_texture: None,
            pushed_texture: None,
            highlight_texture: None,
            disabled_texture: None,
            checked: false,
            checked_texture: None,
            disabled_checked_texture: None,
            text: String::new(),
            font_size: 14.0,
            pushed_text_offset: [0.0, 0.0],
            highlight_locked: false,
        }
    }
}

impl ButtonData {
    /// A metadata-only slot never replaces drawable artwork.
    pub fn base_texture(&self) -> Option<&ButtonTexture> {
        let normal = sourced(self.normal_texture.as_ref());
        match self.state {
            ButtonState::Normal => normal,
            ButtonState::Pushed => sourced(self.pushed_texture.as_ref()).or(normal),
            ButtonState::Disabled => sourced(self.disabled_texture.as_ref()).or(normal),
        }
    }

    pub fn checked_region(&self) -> Option<&ButtonTexture> {
        if !self.checked {
            return None;
        }
        let slot = if self.state == ButtonState::Disabled {
            &self.disabled_checked_texture
        } else {
            &self.checked_texture
        };
        sourced(slot.as_ref())
    }

    pub fn highlight_region(&self) -> Option<&ButtonTexture> {
        let highlighted = self.hovered || self.highlight_locked;
        if highlighted && self.state != ButtonState::Disabled {
            sourced(self.highlight_texture.as_ref())
        } else {
            None
        }
    }
}

fn sourced(texture: Option<&ButtonTexture>) -> Option<&ButtonTexture> {
    texture.filter(|texture| !matches!(texture.source, TextureSource::None))
}

#[derive(Debug, Clone, Default)]
pub struct CheckButtonData {
    pub button: ButtonData,
    pub checked: bool,
    pub checked_texture: Option<ButtonTexture>,
    pub disabled_checked_texture: Option<ButtonTexture>,
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
