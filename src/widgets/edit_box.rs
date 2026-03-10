#[derive(Debug, Clone)]
pub struct EditBoxData {
    pub text: String,
    pub cursor_position: usize,
    pub font: crate::widgets::font_string::GameFont,
    pub font_size: f32,
    pub text_color: [f32; 4],
    pub multi_line: bool,
    pub numeric: bool,
    pub password: bool,
    pub max_letters: Option<u32>,
    pub max_bytes: Option<u32>,
    pub history_lines: u32,
    pub blink_speed: f32,
    pub auto_focus: bool,
    pub text_insets: [f32; 4],
    pub count_invisible_letters: bool,
}

impl Default for EditBoxData {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            font: crate::widgets::font_string::GameFont::default(),
            font_size: 16.0,
            text_color: [1.0, 0.8, 0.2, 1.0],
            multi_line: false,
            numeric: false,
            password: false,
            max_letters: None,
            max_bytes: None,
            history_lines: 0,
            blink_speed: 0.5,
            auto_focus: false,
            text_insets: [0.0; 4],
            count_invisible_letters: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_edit_box_data() {
        let eb = EditBoxData::default();
        assert!(eb.text.is_empty());
        assert_eq!(eb.cursor_position, 0);
        assert!(!eb.multi_line);
        assert_eq!(eb.blink_speed, 0.5);
    }
}
