use std::ops::RangeInclusive;

use crate::{Inset, Length, Rect, RectLayout, Spacing, UiContext, WidgetId, lerp_u32};

use super::{
    action::WidgetValueAction,
    shared::{hex, rounded_fill},
};

const KNOB_SIZE: u32 = 16;

#[derive(Clone, Debug)]
pub struct SliderStyle {
    pub track: crate::Style,
    pub fill: crate::Style,
    pub knob: crate::Style,
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            track: rounded_fill(hex("#272b33"), 5),
            fill: rounded_fill(hex("#5e81ac"), 5),
            knob: rounded_fill(hex("#f2f2f2"), 8),
        }
    }
}

pub struct Slider<A = ()> {
    id: Option<WidgetId>,
    range: RangeInclusive<f32>,
    value: f32,
    on_change: A,
    width: u32,
    height: Length,
    disabled: bool,
    style: SliderStyle,
}

impl Slider<()> {
    pub fn new(range: RangeInclusive<f32>) -> Self {
        Self {
            id: None,
            range,
            value: 0.0,
            on_change: (),
            width: 200,
            height: Length::Px(24),
            disabled: false,
            style: SliderStyle::default(),
        }
    }
}

impl<A> Slider<A> {
    pub fn id(mut self, id: impl Into<WidgetId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    pub fn on_change<F>(self, on_change: F) -> Slider<F> {
        Slider {
            id: self.id,
            range: self.range,
            value: self.value,
            on_change,
            width: self.width,
            height: self.height,
            disabled: self.disabled,
            style: self.style,
        }
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn style(mut self, style: SliderStyle) -> Self {
        self.style = style;
        self
    }

    pub fn build<S>(self, cx: &mut UiContext<'_, S>) -> Rect
    where
        A: WidgetValueAction<S, f32>,
    {
        self.build_with_value(cx).0
    }

    pub fn build_with_value<S>(self, cx: &mut UiContext<'_, S>) -> (Rect, f32)
    where
        A: WidgetValueAction<S, f32>,
    {
        let id = self.id.unwrap_or_else(|| WidgetId::from("slider"));
        let interaction = cx.interaction(id.clone());
        let mut value = self.value;
        let mut changed = false;
        let min = *self.range.start();
        let max = *self.range.end();
        let knob_radius = KNOB_SIZE / 2;
        let track_width = self.width.saturating_sub(KNOB_SIZE);
        if cx.actions_enabled()
            && interaction.pressed
            && !self.disabled
            && max > min
            && let (Some((x, _)), Some(bounds)) = (
                cx.input().pointer().position,
                cx.input().pointer().pressed.as_ref().map(|hit| hit.bounds),
            )
        {
            let track_left = f64::from(bounds.x.saturating_add(knob_radius));
            let fraction = ((x - track_left) / f64::from(track_width.max(1))).clamp(0.0, 1.0);
            let next = min + (max - min) * fraction as f32;
            if (value - next).abs() > f32::EPSILON {
                value = next;
                changed = true;
            }
        }
        if changed {
            self.on_change.call(cx.state_mut(), value);
            cx.mark_changed();
        }

        let value_fraction = if max > min {
            ((value - min) / (max - min)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let fill_width = lerp_u32(0, track_width, value_fraction);
        let knob_left = lerp_u32(0, track_width, value_fraction);

        (
            Rect::new(RectLayout {
                id: Some(id),
                width: Length::Px(self.width),
                height: self.height,
                padding: Spacing::ZERO,
                ..RectLayout::default()
            })
            .child(Rect::new(RectLayout {
                width: Length::Fill,
                height: Length::Px(8),
                position: crate::Position::Absolute,
                inset: Inset {
                    top: Some(8),
                    left: Some(knob_radius),
                    right: Some(knob_radius),
                    ..Inset::ZERO
                },
                style: self.style.track,
                ..RectLayout::default()
            }))
            .child(Rect::new(RectLayout {
                width: Length::Px(fill_width),
                height: Length::Px(8),
                position: crate::Position::Absolute,
                inset: Inset {
                    top: Some(8),
                    left: Some(knob_radius),
                    ..Inset::ZERO
                },
                style: self.style.fill,
                ..RectLayout::default()
            }))
            .child(Rect::new(RectLayout {
                width: Length::Px(KNOB_SIZE),
                height: Length::Px(KNOB_SIZE),
                position: crate::Position::Absolute,
                inset: Inset {
                    top: Some(4),
                    left: Some(knob_left),
                    ..Inset::ZERO
                },
                style: self.style.knob,
                ..RectLayout::default()
            })),
            value,
        )
    }
}
