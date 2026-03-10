#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopType {
    None,
    Repeat,
    Bounce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Smoothing {
    None,
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone)]
pub enum AnimationType {
    Alpha {
        from: f32,
        to: f32,
    },
    Translation {
        from: [f32; 2],
        to: [f32; 2],
    },
    Scale {
        from: [f32; 2],
        to: [f32; 2],
        origin: [f32; 2],
    },
    Rotation {
        from_deg: f32,
        to_deg: f32,
    },
    VertexColor {
        from: [f32; 4],
        to: [f32; 4],
    },
    FlipBook {
        frame_count: u32,
        frame_rate: f32,
    },
    TexCoordTranslation {
        from: [f32; 2],
        to: [f32; 2],
    },
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub anim_type: AnimationType,
    pub duration: f32,
    pub start_delay: f32,
    pub smoothing: Smoothing,
    pub order: u32,
}

#[derive(Debug, Clone)]
pub struct AnimationGroup {
    pub id: u64,
    pub frame_id: u64,
    pub animations: Vec<Animation>,
    pub looping: LoopType,
    pub playing: bool,
    pub paused: bool,
    pub elapsed: f32,
    forward: bool,
}

impl AnimationGroup {
    pub fn new(id: u64, frame_id: u64) -> Self {
        Self {
            id,
            frame_id,
            animations: Vec::new(),
            looping: LoopType::None,
            playing: false,
            paused: false,
            elapsed: 0.0,
            forward: true,
        }
    }

    pub fn play(&mut self) {
        self.playing = true;
        self.paused = false;
        self.elapsed = 0.0;
        self.forward = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.elapsed = 0.0;
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn tick(&mut self, dt: f32) {
        if !self.playing || self.paused {
            return;
        }

        let total = self.total_duration();
        if total <= 0.0 {
            self.stop();
            return;
        }

        self.advance_elapsed(dt, total);
    }

    fn advance_elapsed(&mut self, dt: f32, total: f32) {
        match self.looping {
            LoopType::None => {
                self.elapsed += dt;
                if self.elapsed >= total {
                    self.elapsed = total;
                    self.stop();
                }
            }
            LoopType::Repeat => {
                self.elapsed += dt;
                if self.elapsed >= total {
                    self.elapsed %= total;
                }
            }
            LoopType::Bounce => {
                self.advance_bounce(dt, total);
            }
        }
    }

    fn advance_bounce(&mut self, dt: f32, total: f32) {
        if self.forward {
            self.elapsed += dt;
            if self.elapsed >= total {
                self.elapsed = total - (self.elapsed - total);
                self.forward = false;
            }
        } else {
            self.elapsed -= dt;
            if self.elapsed <= 0.0 {
                self.elapsed = -self.elapsed;
                self.forward = true;
            }
        }
    }

    pub fn total_duration(&self) -> f32 {
        self.animations
            .iter()
            .map(|a| a.start_delay + a.duration)
            .fold(0.0_f32, f32::max)
    }

    pub fn is_finished(&self) -> bool {
        !self.playing
    }
}

pub fn evaluate_progress(elapsed: f32, delay: f32, duration: f32) -> Option<f32> {
    if elapsed < delay {
        return None;
    }
    if duration <= 0.0 {
        return Some(1.0);
    }
    let t = ((elapsed - delay) / duration).clamp(0.0, 1.0);
    Some(t)
}

pub fn apply_smoothing(t: f32, smoothing: Smoothing) -> f32 {
    match smoothing {
        Smoothing::None => t,
        Smoothing::In => t * t,
        Smoothing::Out => {
            let inv = 1.0 - t;
            1.0 - inv * inv
        }
        Smoothing::InOut => 3.0 * t * t - 2.0 * t * t * t,
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn lerp4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
        lerp(a[3], b[3], t),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_animation(duration: f32, delay: f32) -> Animation {
        Animation {
            anim_type: AnimationType::Alpha { from: 0.0, to: 1.0 },
            duration,
            start_delay: delay,
            smoothing: Smoothing::None,
            order: 0,
        }
    }

    #[test]
    fn play_stop_pause_lifecycle() {
        let mut group = AnimationGroup::new(1, 10);
        assert!(!group.playing);
        assert!(group.is_finished());

        group.play();
        assert!(group.playing);
        assert!(!group.paused);
        assert!(!group.is_finished());

        group.pause();
        assert!(group.paused);

        group.resume();
        assert!(!group.paused);

        group.stop();
        assert!(!group.playing);
        assert_eq!(group.elapsed, 0.0);
        assert!(group.is_finished());
    }

    #[test]
    fn tick_advances_elapsed_when_playing() {
        let mut group = AnimationGroup::new(1, 10);
        group.animations.push(make_animation(2.0, 0.0));
        group.play();

        group.tick(0.5);
        assert!((group.elapsed - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_does_not_advance_when_paused() {
        let mut group = AnimationGroup::new(1, 10);
        group.animations.push(make_animation(2.0, 0.0));
        group.play();
        group.tick(0.5);
        group.pause();
        group.tick(1.0);
        assert!((group.elapsed - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn loop_none_stops_at_total_duration() {
        let mut group = AnimationGroup::new(1, 10);
        group.animations.push(make_animation(1.0, 0.0));
        group.looping = LoopType::None;
        group.play();

        group.tick(1.5);
        assert!(!group.playing);
        assert!(group.is_finished());
    }

    #[test]
    fn loop_repeat_wraps_elapsed() {
        let mut group = AnimationGroup::new(1, 10);
        group.animations.push(make_animation(1.0, 0.0));
        group.looping = LoopType::Repeat;
        group.play();

        group.tick(1.5);
        assert!(group.playing);
        assert!((group.elapsed - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn loop_bounce_reverses_direction() {
        let mut group = AnimationGroup::new(1, 10);
        group.animations.push(make_animation(1.0, 0.0));
        group.looping = LoopType::Bounce;
        group.play();

        group.tick(1.2);
        assert!(group.playing);
        // Should have bounced: 1.2 overshoots by 0.2, so elapsed = 1.0 - 0.2 = 0.8
        assert!((group.elapsed - 0.8).abs() < 0.01);
    }

    #[test]
    fn total_duration_with_mixed_delays() {
        let mut group = AnimationGroup::new(1, 10);
        group.animations.push(make_animation(1.0, 0.5)); // 1.5
        group.animations.push(make_animation(0.5, 2.0)); // 2.5
        group.animations.push(make_animation(2.0, 0.0)); // 2.0
        assert!((group.total_duration() - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn evaluate_progress_none_before_delay() {
        assert!(evaluate_progress(0.3, 0.5, 1.0).is_none());
    }

    #[test]
    fn evaluate_progress_returns_normalized() {
        let p = evaluate_progress(1.0, 0.5, 1.0).unwrap();
        assert!((p - 0.5).abs() < f32::EPSILON);

        let p = evaluate_progress(1.5, 0.5, 1.0).unwrap();
        assert!((p - 1.0).abs() < f32::EPSILON);

        let p = evaluate_progress(0.5, 0.5, 1.0).unwrap();
        assert!((p - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn smoothing_none_is_identity() {
        assert!((apply_smoothing(0.5, Smoothing::None) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn smoothing_in_out_at_boundaries() {
        assert!((apply_smoothing(0.0, Smoothing::InOut) - 0.0).abs() < f32::EPSILON);
        assert!((apply_smoothing(1.0, Smoothing::InOut) - 1.0).abs() < f32::EPSILON);
        // Midpoint of hermite: 3*(0.5)^2 - 2*(0.5)^3 = 0.5
        assert!((apply_smoothing(0.5, Smoothing::InOut) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn lerp_midpoint() {
        assert!((lerp(0.0, 10.0, 0.5) - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lerp4_interpolates_all_channels() {
        let a = [0.0, 1.0, 2.0, 3.0];
        let b = [10.0, 11.0, 12.0, 13.0];
        let result = lerp4(a, b, 0.5);
        assert!((result[0] - 5.0).abs() < f32::EPSILON);
        assert!((result[1] - 6.0).abs() < f32::EPSILON);
        assert!((result[2] - 7.0).abs() < f32::EPSILON);
        assert!((result[3] - 8.0).abs() < f32::EPSILON);
    }
}
