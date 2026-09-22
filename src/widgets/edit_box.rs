#[derive(Debug, Clone, PartialEq)]
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

impl EditBoxData {
    /// Insert text at the current cursor position, filtering control chars.
    pub fn insert_at_cursor(&mut self, text: &str) {
        let filtered: String = text.chars().filter(|c| !c.is_control()).collect();
        if filtered.is_empty() {
            return;
        }
        let insert = self.clamp_insert(filtered);
        if insert.is_empty() {
            return;
        }
        self.text.insert_str(self.cursor_position, &insert);
        self.cursor_position += insert.len();
    }

    /// Delete the character before the cursor (backspace).
    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_left();
            self.text.remove(self.cursor_position);
        }
    }

    /// Delete the character after the cursor.
    pub fn delete_forward(&mut self) {
        if self.cursor_position < self.text.len() {
            self.text.remove(self.cursor_position);
        }
    }

    /// Replace a range of text (simulating selection replacement).
    pub fn replace_range(&mut self, start: usize, end: usize, replacement: &str) {
        let start = start.min(self.text.len());
        let end = end.min(self.text.len()).max(start);
        self.text.replace_range(start..end, replacement);
        self.cursor_position = start + replacement.len();
    }

    /// Move cursor left by one character.
    pub fn cursor_left(&mut self) {
        self.cursor_position = self.text[..self.cursor_position]
            .char_indices()
            .next_back()
            .map_or(0, |(index, _)| index);
    }

    /// Move cursor right by one character.
    pub fn cursor_right(&mut self) {
        if let Some(next) = self.text[self.cursor_position..].chars().next() {
            self.cursor_position += next.len_utf8();
        }
    }

    /// Move cursor to start.
    pub fn cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end.
    pub fn cursor_end(&mut self) {
        self.cursor_position = self.text.len();
    }

    fn clamp_insert(&self, text: String) -> String {
        let text = if let Some(max) = self.max_letters {
            let remaining = (max as usize).saturating_sub(self.text.chars().count());
            text.chars().take(remaining).collect()
        } else {
            text
        };
        if let Some(max) = self.max_bytes {
            let remaining = (max as usize).saturating_sub(self.text.len());
            let mut truncated = String::new();
            for ch in text.chars() {
                if truncated.len() + ch.len_utf8() > remaining {
                    break;
                }
                truncated.push(ch);
            }
            truncated
        } else {
            text
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multibyte_cursor_movement_keeps_byte_boundaries() {
        let mut eb = EditBoxData::default();
        eb.insert_at_cursor("aé🦀z");
        eb.cursor_home();
        for expected in [1, 3, 7, 8, 8] {
            eb.cursor_right();
            assert_eq!(eb.cursor_position, expected);
            assert!(eb.text.is_char_boundary(eb.cursor_position));
        }
        for expected in [7, 3, 1, 0, 0] {
            eb.cursor_left();
            assert_eq!(eb.cursor_position, expected);
            assert!(eb.text.is_char_boundary(eb.cursor_position));
        }
    }

    #[test]
    fn multibyte_backspace_removes_whole_characters() {
        let mut eb = EditBoxData::default();
        eb.insert_at_cursor("aé🦀z");
        for (text, cursor) in [("aé🦀", 7), ("aé", 3), ("a", 1), ("", 0), ("", 0)] {
            eb.backspace();
            assert_eq!(eb.text, text);
            assert_eq!(eb.cursor_position, cursor);
        }
    }

    #[test]
    fn multibyte_insertion_after_navigation_uses_the_edit_position() {
        let mut eb = EditBoxData::default();
        eb.insert_at_cursor("aéz");
        eb.cursor_left();
        eb.cursor_left();
        eb.insert_at_cursor("🦀");
        assert_eq!(eb.text, "a🦀éz");
        assert_eq!(eb.cursor_position, 5);
        eb.delete_forward();
        assert_eq!(eb.text, "a🦀z");
        assert_eq!(eb.cursor_position, 5);
    }

    #[test]
    fn multibyte_forward_delete_after_navigation_preserves_the_cursor() {
        let mut eb = EditBoxData::default();
        eb.insert_at_cursor("aé🦀z");
        eb.cursor_home();
        eb.cursor_right();
        eb.cursor_right();
        eb.delete_forward();
        assert_eq!(eb.text, "aéz");
        assert_eq!(eb.cursor_position, 3);
        eb.delete_forward();
        eb.delete_forward();
        assert_eq!(eb.text, "aé");
        assert_eq!(eb.cursor_position, 3);
    }

    #[test]
    fn multibyte_password_edits_preserve_filter_limits_and_byte_offsets() {
        let mut eb = EditBoxData {
            password: true,
            max_letters: Some(3),
            max_bytes: Some(7),
            ..Default::default()
        };
        eb.insert_at_cursor("é\n🦀\tzx");
        assert_eq!(eb.text, "é🦀z");
        assert_eq!(eb.cursor_position, 7);
        eb.backspace();
        eb.insert_at_cursor("é");
        assert_eq!(eb.text, "é🦀");
        assert_eq!(eb.cursor_position, 6);
        eb.insert_at_cursor("x");
        assert_eq!(eb.text, "é🦀x");
        assert_eq!(eb.cursor_position, 7);
        assert!(eb.password);
        assert_eq!(eb.blink_speed, 0.5);
    }

    #[test]
    fn default_edit_box_data() {
        let eb = EditBoxData::default();
        assert!(eb.text.is_empty());
        assert_eq!(eb.cursor_position, 0);
        assert!(!eb.multi_line);
        assert_eq!(eb.blink_speed, 0.5);
    }

    #[test]
    fn insert_at_empty() {
        let mut eb = EditBoxData::default();
        eb.insert_at_cursor("hello");
        assert_eq!(eb.text, "hello");
        assert_eq!(eb.cursor_position, 5);
    }

    #[test]
    fn insert_at_middle() {
        let mut eb = EditBoxData::default();
        eb.text = "helo".into();
        eb.cursor_position = 3;
        eb.insert_at_cursor("l");
        assert_eq!(eb.text, "hello");
        assert_eq!(eb.cursor_position, 4);
    }

    #[test]
    fn insert_filters_control_chars() {
        let mut eb = EditBoxData::default();
        eb.insert_at_cursor("a\nb\tc");
        assert_eq!(eb.text, "abc");
    }

    #[test]
    fn insert_respects_max_letters() {
        let mut eb = EditBoxData {
            max_letters: Some(5),
            ..Default::default()
        };
        eb.insert_at_cursor("abcdefgh");
        assert_eq!(eb.text, "abcde");
    }

    #[test]
    fn insert_respects_max_bytes() {
        let mut eb = EditBoxData {
            max_bytes: Some(3),
            ..Default::default()
        };
        eb.insert_at_cursor("abcde");
        assert_eq!(eb.text, "abc");
    }

    #[test]
    fn replace_range_middle() {
        let mut eb = EditBoxData::default();
        eb.text = "hello world".into();
        eb.replace_range(6, 11, "rust");
        assert_eq!(eb.text, "hello rust");
        assert_eq!(eb.cursor_position, 10);
    }

    #[test]
    fn replace_range_deletion() {
        let mut eb = EditBoxData::default();
        eb.text = "abcdef".into();
        eb.replace_range(2, 4, "");
        assert_eq!(eb.text, "abef");
        assert_eq!(eb.cursor_position, 2);
    }

    #[test]
    fn replace_range_entire_text() {
        let mut eb = EditBoxData::default();
        eb.text = "old".into();
        eb.replace_range(0, 3, "new");
        assert_eq!(eb.text, "new");
        assert_eq!(eb.cursor_position, 3);
    }

    #[test]
    fn backspace_removes_before_cursor() {
        let mut eb = EditBoxData::default();
        eb.text = "hello".into();
        eb.cursor_position = 5;
        eb.backspace();
        assert_eq!(eb.text, "hell");
        assert_eq!(eb.cursor_position, 4);
    }

    #[test]
    fn backspace_at_start_no_op() {
        let mut eb = EditBoxData::default();
        eb.text = "hello".into();
        eb.cursor_position = 0;
        eb.backspace();
        assert_eq!(eb.text, "hello");
    }

    #[test]
    fn delete_forward_at_cursor() {
        let mut eb = EditBoxData::default();
        eb.text = "hello".into();
        eb.cursor_position = 0;
        eb.delete_forward();
        assert_eq!(eb.text, "ello");
    }

    #[test]
    fn delete_forward_at_end_no_op() {
        let mut eb = EditBoxData::default();
        eb.text = "hello".into();
        eb.cursor_position = 5;
        eb.delete_forward();
        assert_eq!(eb.text, "hello");
    }

    #[test]
    fn cursor_movement() {
        let mut eb = EditBoxData::default();
        eb.text = "hello".into();
        eb.cursor_position = 3;
        eb.cursor_left();
        assert_eq!(eb.cursor_position, 2);
        eb.cursor_right();
        assert_eq!(eb.cursor_position, 3);
        eb.cursor_home();
        assert_eq!(eb.cursor_position, 0);
        eb.cursor_end();
        assert_eq!(eb.cursor_position, 5);
    }

    #[test]
    fn cursor_clamped_at_bounds() {
        let mut eb = EditBoxData::default();
        eb.cursor_left();
        assert_eq!(eb.cursor_position, 0);
        eb.text = "ab".into();
        eb.cursor_position = 2;
        eb.cursor_right();
        assert_eq!(eb.cursor_position, 2);
    }
}
