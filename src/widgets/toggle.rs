use crate::{ChangeEvent, ClickEvent, Element, Inset, Listener, Rect, WidgetId};

use super::shared::{hex, rounded_fill};

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

impl ToggleStyle {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn track_off(mut self, value: crate::Style) -> Self {
        self.track_off = value;
        self
    }
    pub fn track_on(mut self, value: crate::Style) -> Self {
        self.track_on = value;
        self
    }
    pub fn knob(mut self, value: crate::Style) -> Self {
        self.knob = value;
        self
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
}

impl Toggle<()> {
    fn into_element_with(self, listener: Option<Listener<ChangeEvent<bool>>>) -> Element {
        let mut root = Rect::new()
            .size_px(44, 24)
            .padding_all(3)
            .style(if self.value {
                self.style.track_on
            } else {
                self.style.track_off
            })
            .child(
                Rect::new()
                    .size_px(18, 18)
                    .position(crate::Position::Absolute)
                    .inset(if self.value {
                        Inset::new().top(3).right(3)
                    } else {
                        Inset::new().top(3).left(3)
                    })
                    .style(self.style.knob),
            );
        if let Some(id) = self.id {
            root = root.id(id);
        }
        if !self.disabled
            && let Some(listener) = listener
        {
            let value = self.value;
            root = root.on_click(Listener::<ClickEvent>::new(move |_| {
                listener.call(&ChangeEvent { value: !value })
            }));
        }
        root.into()
    }
}

impl From<Toggle<()>> for Element {
    fn from(value: Toggle<()>) -> Self {
        value.into_element_with(None)
    }
}
impl From<Toggle<Listener<ChangeEvent<bool>>>> for Element {
    fn from(value: Toggle<Listener<ChangeEvent<bool>>>) -> Self {
        let Toggle {
            id,
            value: current,
            on_change,
            disabled,
            style,
        } = value;
        Toggle {
            id,
            value: current,
            on_change: (),
            disabled,
            style,
        }
        .into_element_with(Some(on_change))
    }
}
