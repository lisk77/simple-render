use crate::{
    ui::{Bounds, Color, Inset, PaintTransform, Spacing},
    wayland::FrameAction,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Easing {
    #[default]
    Default,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl Easing {
    pub fn apply(self, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);
        match self {
            Self::Default => 1.0 - (1.0 - progress) * (1.0 - progress),
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisualTransition {
    pub animation: Animation,
    pub from_opacity: f32,
    pub to_opacity: f32,
    pub from_scale: f32,
    pub to_scale: f32,
    pub from_translate_x: i32,
    pub to_translate_x: i32,
    pub from_translate_y: i32,
    pub to_translate_y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisualTransitionFrame {
    pub animation: AnimationFrame,
    pub opacity: f32,
    pub transform: PaintTransform,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Offset {
    pub x: i32,
    pub y: i32,
}

impl Offset {
    pub const ZERO: Self = Self { x: 0, y: 0 };

    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VisualEffect {
    Fade {
        from_opacity: f32,
        to_opacity: f32,
    },
    Scale {
        from_scale: f32,
        to_scale: f32,
    },
    Slide {
        from: Offset,
        to: Offset,
    },
    FadeSlide {
        from_opacity: f32,
        to_opacity: f32,
        from: Offset,
        to: Offset,
    },
    FadeScale {
        from_opacity: f32,
        to_opacity: f32,
        from_scale: f32,
        to_scale: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundsTransition {
    pub animation: Animation,
    pub from: Bounds,
    pub to: Bounds,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundsTransitionFrame {
    pub animation: AnimationFrame,
    pub bounds: Bounds,
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

impl Default for Animation {
    fn default() -> Self {
        Self::new(160, Easing::Default)
    }
}

impl VisualTransition {
    pub const fn new(animation: Animation) -> Self {
        Self {
            animation,
            from_opacity: 1.0,
            to_opacity: 1.0,
            from_scale: 1.0,
            to_scale: 1.0,
            from_translate_x: 0,
            to_translate_x: 0,
            from_translate_y: 0,
            to_translate_y: 0,
        }
    }

    pub const fn fade(animation: Animation, from_opacity: f32, to_opacity: f32) -> Self {
        Self {
            from_opacity,
            to_opacity,
            ..Self::new(animation)
        }
    }

    pub const fn fade_scale(
        animation: Animation,
        from_opacity: f32,
        to_opacity: f32,
        from_scale: f32,
        to_scale: f32,
    ) -> Self {
        Self {
            from_opacity,
            to_opacity,
            from_scale,
            to_scale,
            ..Self::new(animation)
        }
    }

    pub const fn fade_slide(
        animation: Animation,
        from_opacity: f32,
        to_opacity: f32,
        from: Offset,
        to: Offset,
    ) -> Self {
        Self {
            from_opacity,
            to_opacity,
            from_translate_x: from.x,
            to_translate_x: to.x,
            from_translate_y: from.y,
            to_translate_y: to.y,
            ..Self::new(animation)
        }
    }

    pub const fn scale(animation: Animation, from_scale: f32, to_scale: f32) -> Self {
        Self {
            from_scale,
            to_scale,
            ..Self::new(animation)
        }
    }

    pub const fn slide(animation: Animation, from: Offset, to: Offset) -> Self {
        Self {
            from_translate_x: from.x,
            to_translate_x: to.x,
            from_translate_y: from.y,
            to_translate_y: to.y,
            ..Self::new(animation)
        }
    }

    pub const fn opacity(mut self, from: f32, to: f32) -> Self {
        self.from_opacity = from;
        self.to_opacity = to;
        self
    }

    pub const fn visual_scale(mut self, from: f32, to: f32) -> Self {
        self.from_scale = from;
        self.to_scale = to;
        self
    }

    pub const fn translation(mut self, from_x: i32, to_x: i32, from_y: i32, to_y: i32) -> Self {
        self.from_translate_x = from_x;
        self.to_translate_x = to_x;
        self.from_translate_y = from_y;
        self.to_translate_y = to_y;
        self
    }

    pub fn frame(self, elapsed_ms: u32) -> VisualTransitionFrame {
        let animation = self.animation.frame(elapsed_ms);
        VisualTransitionFrame {
            animation,
            opacity: lerp_f32(self.from_opacity, self.to_opacity, animation.progress)
                .clamp(0.0, 1.0),
            transform: PaintTransform::new(
                lerp_f32(self.from_scale, self.to_scale, animation.progress),
                lerp_i32(
                    self.from_translate_x,
                    self.to_translate_x,
                    animation.progress,
                ),
                lerp_i32(
                    self.from_translate_y,
                    self.to_translate_y,
                    animation.progress,
                ),
            ),
        }
    }

    pub fn slide_from(
        animation: Animation,
        edge: Edge,
        bounds: Bounds,
        distance: Option<u32>,
    ) -> Self {
        let offset = edge.offset(bounds, distance);
        Self::fade_slide(animation, 0.0, 1.0, offset, Offset::ZERO)
    }

    pub fn slide_to(
        animation: Animation,
        edge: Edge,
        bounds: Bounds,
        distance: Option<u32>,
    ) -> Self {
        let offset = edge.offset(bounds, distance);
        Self::fade_slide(animation, 1.0, 0.0, Offset::ZERO, offset)
    }
}

impl VisualTransitionFrame {
    pub fn frame_action(self) -> FrameAction {
        self.animation.frame_action()
    }

    pub fn compose(self, next: Self) -> Self {
        Self {
            animation: AnimationFrame {
                elapsed_ms: self.animation.elapsed_ms.max(next.animation.elapsed_ms),
                progress: self.animation.progress.max(next.animation.progress),
                complete: self.animation.complete && next.animation.complete,
            },
            opacity: (self.opacity * next.opacity).clamp(0.0, 1.0),
            transform: self.transform.compose(next.transform),
        }
    }
}

impl VisualEffect {
    pub fn transition(self, animation: Animation) -> VisualTransition {
        match self {
            Self::Fade {
                from_opacity,
                to_opacity,
            } => VisualTransition::fade(animation, from_opacity, to_opacity),
            Self::Scale {
                from_scale,
                to_scale,
            } => VisualTransition::scale(animation, from_scale, to_scale),
            Self::Slide { from, to } => VisualTransition::slide(animation, from, to),
            Self::FadeSlide {
                from_opacity,
                to_opacity,
                from,
                to,
            } => VisualTransition::fade_slide(animation, from_opacity, to_opacity, from, to),
            Self::FadeScale {
                from_opacity,
                to_opacity,
                from_scale,
                to_scale,
            } => VisualTransition::fade_scale(
                animation,
                from_opacity,
                to_opacity,
                from_scale,
                to_scale,
            ),
        }
    }
}

impl Edge {
    pub fn offset(self, bounds: Bounds, distance: Option<u32>) -> Offset {
        let distance = distance.unwrap_or_else(|| match self {
            Self::Top | Self::Bottom => bounds.height,
            Self::Left | Self::Right => bounds.width,
        });
        let distance = distance.min(i32::MAX as u32) as i32;
        match self {
            Self::Top => Offset::new(0, -distance),
            Self::Bottom => Offset::new(0, distance),
            Self::Left => Offset::new(-distance, 0),
            Self::Right => Offset::new(distance, 0),
        }
    }
}

impl BoundsTransition {
    pub const fn new(animation: Animation, from: Bounds, to: Bounds) -> Self {
        Self {
            animation,
            from,
            to,
        }
    }

    pub fn frame(self, elapsed_ms: u32) -> BoundsTransitionFrame {
        let animation = self.animation.frame(elapsed_ms);
        BoundsTransitionFrame {
            animation,
            bounds: lerp_bounds(self.from, self.to, animation.progress),
        }
    }
}

impl BoundsTransitionFrame {
    pub fn frame_action(self) -> FrameAction {
        self.animation.frame_action()
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

pub fn lerp_bounds(from: Bounds, to: Bounds, progress: f32) -> Bounds {
    Bounds {
        x: lerp_u32(from.x, to.x, progress),
        y: lerp_u32(from.y, to.y, progress),
        width: lerp_u32(from.width, to.width, progress),
        height: lerp_u32(from.height, to.height, progress),
    }
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
