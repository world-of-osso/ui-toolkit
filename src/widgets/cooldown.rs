#[derive(Debug, Clone)]
pub struct CooldownData {
    pub start: f64,
    pub duration: f64,
    pub mod_rate: f32,
    pub paused: bool,
    pub reverse: bool,
    pub draw_swipe: bool,
    pub draw_edge: bool,
    pub draw_bling: bool,
    pub hide_countdown_numbers: bool,
    pub swipe_color: [f32; 4],
}

impl Default for CooldownData {
    fn default() -> Self {
        Self {
            start: 0.0,
            duration: 0.0,
            mod_rate: 1.0,
            paused: false,
            reverse: false,
            draw_swipe: true,
            draw_edge: false,
            draw_bling: true,
            hide_countdown_numbers: false,
            swipe_color: [0.0, 0.0, 0.0, 0.8],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cooldown_data() {
        let cd = CooldownData::default();
        assert_eq!(cd.mod_rate, 1.0);
        assert!(cd.draw_swipe);
        assert!(cd.draw_bling);
        assert!(!cd.paused);
    }
}
