use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BorderWidth {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl BorderWidth {
    pub const ZERO: Self = Self::all(0);

    pub const fn new() -> Self {
        Self::ZERO
    }

    pub const fn all(width: u32) -> Self {
        Self {
            top: width,
            right: width,
            bottom: width,
            left: width,
        }
    }

    pub const fn top(mut self, width: u32) -> Self {
        self.top = width;
        self
    }

    pub const fn right(mut self, width: u32) -> Self {
        self.right = width;
        self
    }

    pub const fn bottom(mut self, width: u32) -> Self {
        self.bottom = width;
        self
    }

    pub const fn left(mut self, width: u32) -> Self {
        self.left = width;
        self
    }

    pub(in crate::ui) fn is_zero(self) -> bool {
        self.top == 0 && self.right == 0 && self.bottom == 0 && self.left == 0
    }

    pub(in crate::ui) fn scaled(self, scale: u32) -> Self {
        Self {
            top: self.top.saturating_mul(scale),
            right: self.right.saturating_mul(scale),
            bottom: self.bottom.saturating_mul(scale),
            left: self.left.saturating_mul(scale),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Border {
    pub width: u32,
    pub widths: BorderWidth,
    pub color: Paint,
    pub gradient: GradientDirection,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            width: 1,
            widths: BorderWidth::ZERO,
            color: Paint::solid(Color::WHITE),
            gradient: GradientDirection::default(),
        }
    }
}

impl Border {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn widths(mut self, widths: BorderWidth) -> Self {
        self.widths = widths;
        self
    }

    pub fn color(mut self, color: impl Into<Paint>) -> Self {
        self.color = color.into();
        self
    }

    pub fn gradient(mut self, gradient: GradientDirection) -> Self {
        self.gradient = gradient;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    pub background: Option<Paint>,
    pub border: Option<Border>,
    pub corner_radius: u32,
    pub corner_radii: CornerRadius,
    pub gradient: GradientDirection,
    pub opacity: f32,
    pub transform: PaintTransform,
    pub anti_alias: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: None,
            border: None,
            corner_radius: 0,
            corner_radii: CornerRadius::ZERO,
            gradient: GradientDirection::default(),
            opacity: 1.0,
            transform: PaintTransform::IDENTITY,
            anti_alias: true,
        }
    }
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn background(mut self, color: impl Into<Paint>) -> Self {
        self.background = Some(color.into());
        self
    }

    pub fn border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }

    pub fn corner_radius(mut self, radius: u32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn corner_radii(mut self, radii: CornerRadius) -> Self {
        self.corner_radii = radii;
        self
    }

    pub fn gradient(mut self, gradient: GradientDirection) -> Self {
        self.gradient = gradient;
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn transform(mut self, transform: PaintTransform) -> Self {
        self.transform = transform;
        self
    }

    pub fn anti_alias(mut self, anti_alias: bool) -> Self {
        self.anti_alias = anti_alias;
        self
    }

    pub fn translated(mut self, x: i32, y: i32) -> Self {
        self.transform.translate_x = x;
        self.transform.translate_y = y;
        self
    }

    pub fn scaled(mut self, scale: f32) -> Self {
        self.transform.scale = scale;
        self
    }
}
