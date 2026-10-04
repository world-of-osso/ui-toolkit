/// Stable identity for a runtime RGBA8 image stored in the frame registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DynamicTextureId(pub u64);

/// Pixel data in row-major RGBA8 order; updates keep the same identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicTexture {
    pub width: u32,
    pub height: u32,
    pub rgba8: Vec<u8>,
}

impl DynamicTexture {
    pub fn new(width: u32, height: u32, rgba8: Vec<u8>) -> Result<Self, &'static str> {
        let size = usize::try_from(width)
            .ok()
            .and_then(|width| width.checked_mul(height as usize))
            .and_then(|pixels| pixels.checked_mul(4));
        if width == 0 || height == 0 || size != Some(rgba8.len()) {
            return Err(
                "dynamic texture must have nonzero dimensions and exactly width * height * 4 RGBA8 bytes",
            );
        }
        Ok(Self {
            width,
            height,
            rgba8,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextureSource {
    None,
    SolidColor([f32; 4]),
    File(String),
    FileDataId(u32),
    Atlas(String),
    /// Runtime image stored in the frame registry (e.g. minimap composite).
    Dynamic(DynamicTextureId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    AlphaKey,
    Additive,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureData {
    pub source: TextureSource,
    pub tex_coords: [f32; 4],
    pub horiz_tile: bool,
    pub vert_tile: bool,
    pub blend_mode: BlendMode,
    pub vertex_color: [f32; 4],
    pub desaturated: bool,
    pub desaturation: f32,
    pub rotation: f32,
}

impl Default for TextureData {
    fn default() -> Self {
        Self {
            source: TextureSource::None,
            tex_coords: [0.0, 1.0, 0.0, 1.0],
            horiz_tile: false,
            vert_tile: false,
            blend_mode: BlendMode::AlphaKey,
            vertex_color: [1.0, 1.0, 1.0, 1.0],
            desaturated: false,
            desaturation: 0.0,
            rotation: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_texture_data() {
        let td = TextureData::default();
        assert!(matches!(td.source, TextureSource::None));
        assert_eq!(td.tex_coords, [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(td.blend_mode, BlendMode::AlphaKey);
        assert!(!td.desaturated);
    }
}
