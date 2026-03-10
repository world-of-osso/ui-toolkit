#[derive(Debug, Clone)]
pub struct TooltipLine {
    pub left_text: String,
    pub right_text: Option<String>,
    pub left_color: [f32; 3],
    pub right_color: Option<[f32; 3]>,
}

impl Default for TooltipLine {
    fn default() -> Self {
        Self {
            left_text: String::new(),
            right_text: None,
            left_color: [1.0, 1.0, 1.0],
            right_color: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TooltipData {
    pub lines: Vec<TooltipLine>,
    pub owner_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct MessageEntry {
    pub text: String,
    pub color: [f32; 4],
    pub hold_time: f32,
    pub elapsed: f32,
}

impl Default for MessageEntry {
    fn default() -> Self {
        Self {
            text: String::new(),
            color: [1.0, 1.0, 1.0, 1.0],
            hold_time: 5.0,
            elapsed: 0.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MessageFrameData {
    pub messages: Vec<MessageEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tooltip_data() {
        let td = TooltipData::default();
        assert!(td.lines.is_empty());
        assert!(td.owner_id.is_none());
    }

    #[test]
    fn default_message_frame_data() {
        let mf = MessageFrameData::default();
        assert!(mf.messages.is_empty());
    }

    #[test]
    fn default_tooltip_line() {
        let tl = TooltipLine::default();
        assert!(tl.left_text.is_empty());
        assert_eq!(tl.left_color, [1.0, 1.0, 1.0]);
    }
}
