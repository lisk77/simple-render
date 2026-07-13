use std::ops::RangeInclusive;

use crate::{ChangeEvent, ClickEvent, Element, Inset, Length, Listener, Rect, WidgetId, lerp_u32};

use super::shared::{hex, rounded_fill};

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

impl SliderStyle {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn track(mut self, value: crate::Style) -> Self {
        self.track = value;
        self
    }
    pub fn fill(mut self, value: crate::Style) -> Self {
        self.fill = value;
        self
    }
    pub fn knob(mut self, value: crate::Style) -> Self {
        self.knob = value;
        self
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
    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }

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

    pub fn width(mut self, width: impl Into<crate::Pixels>) -> Self {
        self.width = width.into().get();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
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
}

impl Slider<()> {
    fn into_element_with(self, listener: Option<Listener<ChangeEvent<f32>>>) -> Element {
        let min = *self.range.start();
        let max = *self.range.end();
        let fraction = if max > min {
            ((self.value - min) / (max - min)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let knob_radius = KNOB_SIZE / 2;
        let track_width = self.width.saturating_sub(KNOB_SIZE);
        let fill_width = lerp_u32(0, track_width, fraction);
        let knob_left = lerp_u32(0, track_width, fraction);
        let mut root = Rect::new()
            .width_px(self.width)
            .height(self.height)
            .child(
                Rect::new()
                    .width_fill()
                    .height_px(8)
                    .absolute(Inset::new().top(8).left(knob_radius).right(knob_radius))
                    .style(self.style.track),
            )
            .child(
                Rect::new()
                    .width_px(fill_width)
                    .height_px(8)
                    .absolute(Inset::new().top(8).left(knob_radius))
                    .style(self.style.fill),
            )
            .child(
                Rect::new()
                    .size_px(KNOB_SIZE, KNOB_SIZE)
                    .absolute(Inset::new().top(4).left(knob_left))
                    .style(self.style.knob),
            );
        if let Some(id) = self.id {
            root = root.id(id);
        }
        if !self.disabled
            && max > min
            && let Some(listener) = listener
        {
            root =
                root.child(Rect::new().size_fill().absolute_all(0).on_drag(
                    Listener::<ClickEvent>::new(move |event| {
                        let left = f64::from(event.bounds.x.saturating_add(knob_radius));
                        let fraction =
                            ((event.x - left) / f64::from(track_width.max(1))).clamp(0.0, 1.0);
                        listener.call(&ChangeEvent {
                            value: min + (max - min) * fraction as f32,
                        });
                    }),
                ));
        }
        root.into()
    }
}

impl From<Slider<()>> for Element {
    fn from(value: Slider<()>) -> Self {
        value.into_element_with(None)
    }
}
impl From<Slider<Listener<ChangeEvent<f32>>>> for Element {
    fn from(value: Slider<Listener<ChangeEvent<f32>>>) -> Self {
        let Slider {
            id,
            range,
            value,
            on_change,
            width,
            height,
            disabled,
            style,
        } = value;
        Slider {
            id,
            range,
            value,
            on_change: (),
            width,
            height,
            disabled,
            style,
        }
        .into_element_with(Some(on_change))
    }
}
