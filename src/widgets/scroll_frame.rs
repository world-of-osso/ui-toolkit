#[derive(Debug, Clone)]
pub struct ScrollFrameData {
    pub scroll_child_id: Option<u64>,
    pub h_scroll: f32,
    pub v_scroll: f32,
}

impl Default for ScrollFrameData {
    fn default() -> Self {
        Self {
            scroll_child_id: None,
            h_scroll: 0.0,
            v_scroll: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scroll_frame_data() {
        let sf = ScrollFrameData::default();
        assert!(sf.scroll_child_id.is_none());
        assert_eq!(sf.h_scroll, 0.0);
        assert_eq!(sf.v_scroll, 0.0);
    }
}
