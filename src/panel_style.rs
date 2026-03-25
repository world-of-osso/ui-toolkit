use crate::frame::NineSlice;
use crate::registry::FrameRegistry;

impl FrameRegistry {
    /// Register a named panel style. Panels referencing this name get its NineSlice applied.
    pub fn register_panel_style(&mut self, name: impl Into<String>, nine_slice: NineSlice) {
        self.panel_styles.insert(name.into(), nine_slice);
    }

    /// Look up a panel style by name.
    pub fn panel_style(&self, name: &str) -> Option<&NineSlice> {
        self.panel_styles.get(name)
    }

    /// Apply a named panel style to a frame, setting its nine_slice.
    pub fn apply_panel_style(&mut self, frame_id: u64, style_name: &str) {
        let ns = self.panel_styles.get(style_name).cloned();
        if let Some(frame) = self.get_mut(frame_id) {
            frame.panel_style = Some(style_name.to_string());
            if let Some(ns) = ns {
                frame.nine_slice = Some(ns);
            }
        }
    }

    /// Apply the default panel style if one is registered.
    pub fn apply_default_panel_style(&mut self, frame_id: u64) {
        let Some(ns) = self.panel_styles.get("default").cloned() else {
            // Mark as default so it gets applied when the style is registered later.
            if let Some(frame) = self.get_mut(frame_id) {
                frame.panel_style = Some("default".to_string());
            }
            return;
        };
        if let Some(frame) = self.get_mut(frame_id) {
            frame.panel_style = Some("default".to_string());
            frame.nine_slice = Some(ns);
        }
    }

    /// Re-apply panel styles to all frames that reference them (call after registering styles).
    pub fn refresh_panel_styles(&mut self) {
        let ids: Vec<(u64, String)> = self
            .frames_iter()
            .filter_map(|f| f.panel_style.as_ref().map(|s| (f.id, s.clone())))
            .collect();
        for (id, style_name) in ids {
            if let Some(ns) = self.panel_styles.get(&style_name).cloned() {
                if let Some(frame) = self.get_mut(id) {
                    frame.nine_slice = Some(ns);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frame::NineSlice;
    use crate::registry::FrameRegistry;
    use crate::widgets::texture::TextureSource;

    #[test]
    fn register_and_apply_panel_style() {
        let mut reg = FrameRegistry::new(1920.0, 1080.0);
        reg.register_panel_style(
            "gold",
            NineSlice {
                edge_size: 12.0,
                texture: Some(TextureSource::File("panel.png".to_string())),
                ..Default::default()
            },
        );
        let id = reg.create_frame("TestPanel", None);
        reg.apply_panel_style(id, "gold");
        let frame = reg.get(id).unwrap();
        assert_eq!(frame.panel_style.as_deref(), Some("gold"));
        assert!(frame.nine_slice.is_some());
    }

    #[test]
    fn default_panel_style_defers_when_no_style_registered() {
        let mut reg = FrameRegistry::new(1920.0, 1080.0);
        let id = reg.create_frame("TestPanel", None);
        reg.apply_default_panel_style(id);
        let frame = reg.get(id).unwrap();
        assert!(
            frame.nine_slice.is_none(),
            "no nine_slice until style is registered"
        );
        assert_eq!(frame.panel_style.as_deref(), Some("default"));
    }

    #[test]
    fn refresh_applies_deferred_style() {
        let mut reg = FrameRegistry::new(1920.0, 1080.0);
        let id = reg.create_frame("TestPanel", None);
        reg.apply_default_panel_style(id);
        reg.register_panel_style("default", NineSlice::default());
        reg.refresh_panel_styles();
        let frame = reg.get(id).unwrap();
        assert!(frame.nine_slice.is_some());
    }
}
