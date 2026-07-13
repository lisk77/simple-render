use std::sync::Arc;

use super::shared::{hex, rounded_fill};
use crate::{
    Align, ClickEvent, Element, Length, Listener, Paint, Pixels, Rect, Spacing, Text, TextStyle,
    WidgetId,
};

#[derive(Clone, Debug)]
pub struct ButtonStyle {
    pub normal: crate::Style,
    pub hovered: crate::Style,
    pub pressed: crate::Style,
    pub disabled: crate::Style,
    pub text: TextStyle,
    pub padding: Spacing,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            normal: rounded_fill(hex("#2a2f38"), 10),
            hovered: rounded_fill(hex("#33465a"), 10),
            pressed: rounded_fill(hex("#3f5f72"), 10),
            disabled: rounded_fill(hex("#26282d"), 10),
            text: TextStyle {
                color: Paint::solid(hex("#f2f2f2")),
                size: 15,
                bold: true,
                ..TextStyle::default()
            },
            padding: Spacing::axis(14, 8),
        }
    }
}

impl ButtonStyle {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn normal(mut self, value: crate::Style) -> Self {
        self.normal = value;
        self
    }
    pub fn hovered(mut self, value: crate::Style) -> Self {
        self.hovered = value;
        self
    }
    pub fn pressed(mut self, value: crate::Style) -> Self {
        self.pressed = value;
        self
    }
    pub fn disabled(mut self, value: crate::Style) -> Self {
        self.disabled = value;
        self
    }
    pub fn text(mut self, value: TextStyle) -> Self {
        self.text = value;
        self
    }
    pub fn padding_all(mut self, value: impl Into<crate::Pixels>) -> Self {
        self.padding = Spacing::all(value.into().get());
        self
    }
    pub fn padding_axis(
        mut self,
        horizontal: impl Into<crate::Pixels>,
        vertical: impl Into<crate::Pixels>,
    ) -> Self {
        self.padding = Spacing::axis(horizontal.into().get(), vertical.into().get());
        self
    }
    pub fn padding_top(mut self, value: impl Into<crate::Pixels>) -> Self {
        self.padding.top = value.into().get();
        self
    }
    pub fn padding_right(mut self, value: impl Into<crate::Pixels>) -> Self {
        self.padding.right = value.into().get();
        self
    }
    pub fn padding_bottom(mut self, value: impl Into<crate::Pixels>) -> Self {
        self.padding.bottom = value.into().get();
        self
    }
    pub fn padding_left(mut self, value: impl Into<crate::Pixels>) -> Self {
        self.padding.left = value.into().get();
        self
    }
}

pub struct Button<A = ()> {
    id: Option<WidgetId>,
    label: Arc<str>,
    width: Length,
    height: Length,
    disabled: bool,
    style: ButtonStyle,
    text_align: Align,
    text_vertical_align: Align,
    on_click: A,
}

impl Button<()> {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        let label = label.into();
        Self {
            id: None,
            label,
            width: Length::Fit,
            height: Length::Px(36),
            disabled: false,
            style: ButtonStyle::default(),
            text_align: Align::Center,
            text_vertical_align: Align::Center,
            on_click: (),
        }
    }
}

impl<A> Button<A> {
    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = label.into();
        self
    }

    pub fn id(mut self, id: impl Into<WidgetId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.text_align = align;
        self
    }

    pub fn align_center(self) -> Self {
        self.align(Align::Center)
    }

    pub fn vertical_align(mut self, align: Align) -> Self {
        self.text_vertical_align = align;
        self
    }

    pub fn vertical_align_center(self) -> Self {
        self.vertical_align(Align::Center)
    }

    pub fn color(mut self, color: impl Into<Paint>) -> Self {
        self.style.text.color = color.into();
        self
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.style.text.size = size.into().get();
        self
    }

    pub fn bold(mut self) -> Self {
        self.style.text.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.style.text.italic = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn on_click<F>(self, on_click: F) -> Button<F> {
        Button {
            id: self.id,
            label: self.label,
            width: self.width,
            height: self.height,
            disabled: self.disabled,
            style: self.style,
            text_align: self.text_align,
            text_vertical_align: self.text_vertical_align,
            on_click,
        }
    }
}

impl Button<()> {
    fn into_element_with(self, listener: Option<Listener<ClickEvent>>) -> Element {
        let id = self.id;
        let base_style = if self.disabled {
            self.style.disabled.clone()
        } else {
            self.style.normal
        };
        let mut rect = Rect::new()
            .width(self.width)
            .height(self.height)
            .padding(self.style.padding)
            .style(base_style);
        if !self.disabled {
            rect = rect.interaction_styles(
                self.style.hovered,
                self.style.pressed,
                self.style.disabled,
            );
        }
        rect = rect.text(
            Text::new(self.label)
                .style(self.style.text)
                .align(self.text_align)
                .vertical_align(self.text_vertical_align),
        );
        if let Some(id) = id {
            rect = rect.id(id);
        }
        if let Some(listener) = listener {
            rect = rect.on_click(listener);
        }
        rect.into()
    }
}

impl From<Button<()>> for Element {
    fn from(button: Button<()>) -> Self {
        button.into_element_with(None)
    }
}

impl From<Button<Listener<ClickEvent>>> for Element {
    fn from(button: Button<Listener<ClickEvent>>) -> Self {
        let Button {
            id,
            label,
            width,
            height,
            disabled,
            style,
            text_align,
            text_vertical_align,
            on_click,
        } = button;
        Button {
            id,
            label,
            width,
            height,
            disabled,
            style,
            text_align,
            text_vertical_align,
            on_click: (),
        }
        .into_element_with(Some(on_click))
    }
}
