#[derive(Debug, Clone)]
pub enum TextureSource {
    None,
    SolidColor([f32; 4]),
    File(String),
    FileDataId(u32),
    Atlas(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    AlphaKey,
    Additive,
}

#[derive(Debug, Clone)]
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
