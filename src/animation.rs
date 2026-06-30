use crate::{
    ui::{Color, Inset, Spacing},
    wayland::FrameAction,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl Easing {
    pub fn apply(self, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);
        match self {
            Self::Linear => progress,
            Self::EaseIn => progress * progress,
            Self::EaseOut => 1.0 - (1.0 - progress) * (1.0 - progress),
            Self::EaseInOut if progress < 0.5 => 2.0 * progress * progress,
            Self::EaseInOut => 1.0 - (-2.0 * progress + 2.0).powi(2) / 2.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Animation {
    pub duration_ms: u32,
    pub easing: Easing,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationFrame {
    pub elapsed_ms: u32,
    pub progress: f32,
    pub complete: bool,
}

impl AnimationFrame {
    pub fn should_animate(self) -> bool {
        !self.complete
    }

    pub fn frame_action(self) -> FrameAction {
        if self.should_animate() {
            FrameAction::Animate
        } else {
            FrameAction::Wait
        }
    }
}

impl Animation {
    pub const fn new(duration_ms: u32, easing: Easing) -> Self {
        Self {
            duration_ms,
            easing,
        }
    }

    pub fn progress(self, elapsed_ms: u32) -> f32 {
        if self.duration_ms == 0 {
            return 1.0;
        }

        self.easing
            .apply(elapsed_ms as f32 / self.duration_ms as f32)
    }

    pub fn is_complete(self, elapsed_ms: u32) -> bool {
        elapsed_ms >= self.duration_ms
    }

    pub fn frame(self, elapsed_ms: u32) -> AnimationFrame {
        AnimationFrame {
            elapsed_ms,
            progress: self.progress(elapsed_ms),
            complete: self.is_complete(elapsed_ms),
        }
    }
}

pub fn lerp_f32(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress.clamp(0.0, 1.0)
}

pub fn lerp_u32(from: u32, to: u32, progress: f32) -> u32 {
    lerp_f32(from as f32, to as f32, progress).round() as u32
}

pub fn lerp_i32(from: i32, to: i32, progress: f32) -> i32 {
    lerp_f32(from as f32, to as f32, progress).round() as i32
}

pub fn lerp_color(from: Color, to: Color, progress: f32) -> Color {
    Color::rgba(
        lerp_u8(from.red, to.red, progress),
        lerp_u8(from.green, to.green, progress),
        lerp_u8(from.blue, to.blue, progress),
        lerp_u8(from.alpha, to.alpha, progress),
    )
}

pub fn lerp_spacing(from: Spacing, to: Spacing, progress: f32) -> Spacing {
    Spacing {
        top: lerp_u32(from.top, to.top, progress),
        right: lerp_u32(from.right, to.right, progress),
        bottom: lerp_u32(from.bottom, to.bottom, progress),
        left: lerp_u32(from.left, to.left, progress),
    }
}

pub fn lerp_inset(from: Inset, to: Inset, progress: f32) -> Inset {
    Inset {
        top: lerp_optional_u32(from.top, to.top, progress),
        right: lerp_optional_u32(from.right, to.right, progress),
        bottom: lerp_optional_u32(from.bottom, to.bottom, progress),
        left: lerp_optional_u32(from.left, to.left, progress),
    }
}

fn lerp_u8(from: u8, to: u8, progress: f32) -> u8 {
    lerp_f32(from as f32, to as f32, progress)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn lerp_optional_u32(from: Option<u32>, to: Option<u32>, progress: f32) -> Option<u32> {
    match (from, to) {
        (Some(from), Some(to)) => Some(lerp_u32(from, to, progress)),
        (Some(from), None) => (progress < 1.0).then_some(from),
        (None, Some(to)) => (progress > 0.0).then_some(to),
        (None, None) => None,
    }
}