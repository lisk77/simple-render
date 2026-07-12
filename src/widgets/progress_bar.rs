use crate::{GradientDirection, Length, Paint, Rect, RectLayout, Style, lerp_u32};

use super::shared::{hex, rounded_fill};

#[derive(Clone, Debug)]
pub struct ProgressBarStyle {
    pub track: Style,
    pub fill: Style,
}

impl Default for ProgressBarStyle {
    fn default() -> Self {
        Self {
            track: rounded_fill(hex("#272b33"), 5),
            fill: Style {
                background: Some(Paint::gradient([hex("#5e81ac"), hex("#88c0d0")])),
                gradient: GradientDirection::Horizontal,
                corner_radius: 5,
                ..Style::default()
            },
        }
    }
}

impl ProgressBarStyle {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn track(mut self, value: Style) -> Self {
        self.track = value;
        self
    }
    pub fn fill(mut self, value: Style) -> Self {
        self.fill = value;
        self
    }
}

pub struct ProgressBar {
    value: f32,
    width: Length,
    width_px: Option<u32>,
    height: Length,
    style: ProgressBarStyle,
}

impl ProgressBar {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            width: Length::Fill,
            width_px: None,
            height: Length::Px(8),
            style: ProgressBarStyle::default(),
        }
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width_px = match width {
            Length::Px(width) => Some(width),
            _ => None,
        };
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    pub fn style(mut self, style: ProgressBarStyle) -> Self {
        self.style = style;
        self
    }

    pub fn build(self) -> Rect {
        let value = self.value.clamp(0.0, 1.0);
        let fill_width = self
            .width_px
            .map(|width| Length::Px(lerp_u32(0, width, value)))
            .unwrap_or_else(|| Length::Percent(value * 100.0));
        Rect::layout(RectLayout {
            width: self.width,
            height: self.height,
            style: self.style.track,
            ..RectLayout::default()
        })
        .child(Rect::layout(RectLayout {
            width: fill_width,
            height: Length::Fill,
            style: self.style.fill,
            ..RectLayout::default()
        }))
    }
}
