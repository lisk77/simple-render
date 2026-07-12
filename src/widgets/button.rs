use std::sync::Arc;

use crate::{
    Align, Content, Length, Paint, Rect, RectLayout, Spacing, Text, TextStyle, UiContext, WidgetId,
};

use super::{
    action::WidgetAction,
    shared::{hex, interaction_style, rounded_fill},
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
    pub fn padding_all(mut self, value: u32) -> Self {
        self.padding = Spacing::all(value);
        self
    }
    pub fn padding_axis(mut self, horizontal: u32, vertical: u32) -> Self {
        self.padding = Spacing::axis(horizontal, vertical);
        self
    }
    pub fn padding_top(mut self, value: u32) -> Self {
        self.padding.top = value;
        self
    }
    pub fn padding_right(mut self, value: u32) -> Self {
        self.padding.right = value;
        self
    }
    pub fn padding_bottom(mut self, value: u32) -> Self {
        self.padding.bottom = value;
        self
    }
    pub fn padding_left(mut self, value: u32) -> Self {
        self.padding.left = value;
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

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
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
            on_click,
        }
    }

    pub fn build<S>(self, cx: &mut UiContext<'_, S>) -> Rect
    where
        A: WidgetAction<S>,
    {
        let id = self.id.unwrap_or_else(|| WidgetId::new(self.label.clone()));
        let interaction = cx.interaction(id.clone());
        if interaction.clicked && !self.disabled && cx.consume_click(&id) {
            self.on_click.call(cx.state_mut());
            cx.mark_changed();
        }

        let style = interaction_style(
            interaction,
            self.disabled,
            &self.style.normal,
            &self.style.hovered,
            &self.style.pressed,
            &self.style.disabled,
        );

        Rect::layout(RectLayout {
            id: Some(id),
            width: self.width,
            height: self.height,
            padding: self.style.padding,
            style,
            content: Some(Content::Text(Text {
                content: self.label,
                style: self.style.text,
                align: Align::Center,
                vertical_align: Align::Center,
                ..Text::default()
            })),
            ..RectLayout::default()
        })
    }
}
