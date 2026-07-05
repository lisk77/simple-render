use crate::{Inset, Length, Rect, RectLayout, Spacing, UiContext, WidgetId};

use super::{
    action::WidgetValueAction,
    shared::{hex, rounded_fill},
};

#[derive(Clone, Debug)]
pub struct ToggleStyle {
    pub track_off: crate::Style,
    pub track_on: crate::Style,
    pub knob: crate::Style,
}

impl Default for ToggleStyle {
    fn default() -> Self {
        Self {
            track_off: rounded_fill(hex("#30343a"), 12),
            track_on: rounded_fill(hex("#5e81ac"), 12),
            knob: rounded_fill(hex("#f2f2f2"), 9),
        }
    }
}

pub struct Toggle<A = ()> {
    id: Option<WidgetId>,
    value: bool,
    on_change: A,
    disabled: bool,
    style: ToggleStyle,
}

impl Toggle<()> {
    pub fn new() -> Self {
        Self {
            id: None,
            value: false,
            on_change: (),
            disabled: false,
            style: ToggleStyle::default(),
        }
    }
}

impl Default for Toggle<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A> Toggle<A> {
    pub fn id(mut self, id: impl Into<WidgetId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn value(mut self, value: bool) -> Self {
        self.value = value;
        self
    }

    pub fn on_change<F>(self, on_change: F) -> Toggle<F> {
        Toggle {
            id: self.id,
            value: self.value,
            on_change,
            disabled: self.disabled,
            style: self.style,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn style(mut self, style: ToggleStyle) -> Self {
        self.style = style;
        self
    }

    pub fn build<S>(self, cx: &mut UiContext<'_, S>) -> Rect
    where
        A: WidgetValueAction<S, bool>,
    {
        let id = self.id.unwrap_or_else(|| WidgetId::from("toggle"));
        let interaction = cx.interaction(id.clone());
        let mut on = self.value;
        if interaction.clicked && !self.disabled && cx.consume_click(&id) {
            on = !on;
            self.on_change.call(cx.state_mut(), on);
            cx.mark_changed();
        }

        Rect::new(RectLayout {
            id: Some(id),
            width: Length::Px(44),
            height: Length::Px(24),
            padding: Spacing::all(3),
            style: if on {
                self.style.track_on
            } else {
                self.style.track_off
            },
            ..RectLayout::default()
        })
        .child(Rect::new(RectLayout {
            width: Length::Px(18),
            height: Length::Px(18),
            position: crate::Position::Absolute,
            inset: if on {
                Inset {
                    top: Some(3),
                    right: Some(3),
                    ..Inset::ZERO
                }
            } else {
                Inset {
                    top: Some(3),
                    left: Some(3),
                    ..Inset::ZERO
                }
            },
            style: self.style.knob,
            ..RectLayout::default()
        }))
    }
}
