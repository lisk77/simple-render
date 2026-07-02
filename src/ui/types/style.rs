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

    pub const fn all(width: u32) -> Self {
        Self {
            top: width,
            right: width,
            bottom: width,
            left: width,
        }
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

#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    pub background: Option<Paint>,
    pub border: Option<Border>,
    pub corner_radius: u32,
    pub corner_radii: CornerRadius,
    pub gradient: GradientDirection,
    pub opacity: f32,
    pub transform: PaintTransform,
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
        }
    }
}

impl Style {
    pub fn background(color: impl Into<Paint>) -> Self {
        Self {
            background: Some(color.into()),
            border: None,
            corner_radius: 0,
            corner_radii: CornerRadius::ZERO,
            gradient: GradientDirection::default(),
            opacity: 1.0,
            transform: PaintTransform::IDENTITY,
        }
    }
}
