use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bounds {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Bounds {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn union(self, other: Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Self {
            x,
            y,
            width: right.saturating_sub(x),
            height: bottom.saturating_sub(y),
        }
    }

    pub fn translate(self, x: i32, y: i32) -> Self {
        Self {
            x: translate_u32(self.x, x),
            y: translate_u32(self.y, y),
            width: self.width,
            height: self.height,
        }
    }

    pub(in crate::ui) fn inset(self, spacing: Spacing) -> Self {
        let horizontal = spacing.left.saturating_add(spacing.right);
        let vertical = spacing.top.saturating_add(spacing.bottom);
        Self {
            x: self.x.saturating_add(spacing.left),
            y: self.y.saturating_add(spacing.top),
            width: self.width.saturating_sub(horizontal),
            height: self.height.saturating_sub(vertical),
        }
    }

    pub(in crate::ui) fn right(self) -> u32 {
        self.x.saturating_add(self.width)
    }

    pub(in crate::ui) fn bottom(self) -> u32 {
        self.y.saturating_add(self.height)
    }

    pub(in crate::ui) fn intersect(self, other: Self) -> Option<Self> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        let width = right.saturating_sub(x);
        let height = bottom.saturating_sub(y);
        (width > 0 && height > 0).then_some(Self {
            x,
            y,
            width,
            height,
        })
    }
}

fn translate_u32(value: u32, offset: i32) -> u32 {
    if offset.is_negative() {
        value.saturating_sub(offset.unsigned_abs())
    } else {
        value.saturating_add(offset as u32)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CornerRadius {
    pub top_left: u32,
    pub top_right: u32,
    pub bottom_right: u32,
    pub bottom_left: u32,
}

impl CornerRadius {
    pub const ZERO: Self = Self::all(0);

    pub const fn all(radius: u32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    pub(in crate::ui) fn is_zero(self) -> bool {
        self.top_left == 0 && self.top_right == 0 && self.bottom_right == 0 && self.bottom_left == 0
    }

    pub(in crate::ui) fn inset(self, spacing: Spacing) -> Self {
        Self {
            top_left: self.top_left.saturating_sub(spacing.top.max(spacing.left)),
            top_right: self
                .top_right
                .saturating_sub(spacing.top.max(spacing.right)),
            bottom_right: self
                .bottom_right
                .saturating_sub(spacing.bottom.max(spacing.right)),
            bottom_left: self
                .bottom_left
                .saturating_sub(spacing.bottom.max(spacing.left)),
        }
    }

    pub(in crate::ui) fn scaled(self, scale: u32) -> Self {
        Self {
            top_left: self.top_left.saturating_mul(scale),
            top_right: self.top_right.saturating_mul(scale),
            bottom_right: self.bottom_right.saturating_mul(scale),
            bottom_left: self.bottom_left.saturating_mul(scale),
        }
    }

    pub(in crate::ui) fn clamp_to(self, width: u32, height: u32) -> Self {
        let max = width.min(height) / 2;
        Self {
            top_left: self.top_left.min(max),
            top_right: self.top_right.min(max),
            bottom_right: self.bottom_right.min(max),
            bottom_left: self.bottom_left.min(max),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundedClip {
    pub rect: Bounds,
    pub radii: CornerRadius,
}

impl RoundedClip {
    pub(in crate::ui) const EMPTY: Self = Self {
        rect: Bounds {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        radii: CornerRadius::ZERO,
    };
}

const MAX_ROUNDED_CLIPS: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clip {
    pub(in crate::ui) bounds: Bounds,
    pub(in crate::ui) rounded: [RoundedClip; MAX_ROUNDED_CLIPS],
    pub(in crate::ui) rounded_len: u8,
}

impl Clip {
    pub const fn rect(bounds: Bounds) -> Self {
        Self {
            bounds,
            rounded: [RoundedClip::EMPTY; MAX_ROUNDED_CLIPS],
            rounded_len: 0,
        }
    }

    pub fn rounded_rect(rect: Bounds, radius: u32) -> Self {
        Self::rounded_rect_corners(rect, CornerRadius::all(radius))
    }

    pub fn rounded_rect_corners(rect: Bounds, radii: CornerRadius) -> Self {
        Self::rect(rect)
            .with_rounded_rect(rect, radii)
            .unwrap_or_else(|| Self::rect(rect))
    }

    pub fn bounds(self) -> Bounds {
        self.bounds
    }

    pub(in crate::ui) fn intersect_bounds(mut self, bounds: Bounds) -> Option<Self> {
        self.bounds = self.bounds.intersect(bounds)?;
        Some(self)
    }

    pub(in crate::ui) fn with_rounded_rect(
        self,
        rect: Bounds,
        radii: CornerRadius,
    ) -> Option<Self> {
        let mut clip = self.intersect_bounds(rect)?;
        if radii.is_zero() || rect.width == 0 || rect.height == 0 {
            return Some(clip);
        }

        let index = usize::from(clip.rounded_len).min(MAX_ROUNDED_CLIPS - 1);
        clip.rounded[index] = RoundedClip { rect, radii };
        clip.rounded_len = clip
            .rounded_len
            .saturating_add(1)
            .min(MAX_ROUNDED_CLIPS as u8);
        Some(clip)
    }

    pub fn contains(self, x: u32, y: u32) -> bool {
        if x < self.bounds.x
            || y < self.bounds.y
            || x >= self.bounds.right()
            || y >= self.bounds.bottom()
        {
            return false;
        }

        self.rounded[..usize::from(self.rounded_len)]
            .iter()
            .all(|clip| {
                x >= clip.rect.x
                    && y >= clip.rect.y
                    && super::render::rounded_rect_contains(
                        x - clip.rect.x,
                        y - clip.rect.y,
                        clip.rect.width,
                        clip.rect.height,
                        clip.radii,
                    )
            })
    }

    pub(in crate::ui) fn coverage(self, x: u32, y: u32) -> u8 {
        if x < self.bounds.x
            || y < self.bounds.y
            || x >= self.bounds.right()
            || y >= self.bounds.bottom()
        {
            return 0;
        }

        let mut coverage = 255_u16;
        for clip in self.rounded[..usize::from(self.rounded_len)].iter() {
            if x < clip.rect.x
                || y < clip.rect.y
                || x >= clip.rect.right()
                || y >= clip.rect.bottom()
            {
                return 0;
            }

            let rounded_coverage = super::render::rounded_rect_coverage(
                x - clip.rect.x,
                y - clip.rect.y,
                clip.rect.width,
                clip.rect.height,
                clip.radii,
            );
            coverage = coverage * u16::from(rounded_coverage) / 255;
            if coverage == 0 {
                return 0;
            }
        }

        coverage as u8
    }
}
