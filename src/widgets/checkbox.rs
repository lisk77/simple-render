use std::sync::Arc;

use crate::{
    Align, Content, Direction, Length, Paint, Rect, RectLayout, Text, TextStyle, UiContext,
    WidgetId,
};

use super::{
    action::WidgetValueAction,
    shared::{hex, interaction_style, rounded_fill},
};

#[derive(Clone, Debug)]
pub struct CheckboxStyle {
    pub box_normal: crate::Style,
    pub box_hovered: crate::Style,
    pub box_pressed: crate::Style,
    pub box_checked: crate::Style,
    pub text: TextStyle,
    pub mark: TextStyle,
}

impl Default for CheckboxStyle {
    fn default() -> Self {
        Self {
            box_normal: rounded_fill(hex("#272b33"), 5),
            box_hovered: rounded_fill(hex("#33465a"), 5),
            box_pressed: rounded_fill(hex("#3f5f72"), 5),
            box_checked: rounded_fill(hex("#5e81ac"), 5),
            text: TextStyle {
                color: Paint::solid(hex("#e5e9f0")),
                size: 14,
                ..TextStyle::default()
            },
            mark: TextStyle {
                color: Paint::solid(hex("#ffffff")),
                size: 14,
                bold: true,
                ..TextStyle::default()
            },
        }
    }
}

pub struct Checkbox<A = ()> {
    id: Option<WidgetId>,
    label: Arc<str>,
    checked: bool,
    on_change: A,
    width: Length,
    height: Length,
    disabled: bool,
    style: CheckboxStyle,
}

impl Checkbox<()> {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            id: None,
            label: label.into(),
            checked: false,
            on_change: (),
            width: Length::Fit,
            height: Length::Px(28),
            disabled: false,
            style: CheckboxStyle::default(),
        }
    }
}

impl<A> Checkbox<A> {
    pub fn id(mut self, id: impl Into<WidgetId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn on_change<F>(self, on_change: F) -> Checkbox<F> {
        Checkbox {
            id: self.id,
            label: self.label,
            checked: self.checked,
            on_change,
            width: self.width,
            height: self.height,
            disabled: self.disabled,
            style: self.style,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn style(mut self, style: CheckboxStyle) -> Self {
        self.style = style;
        self
    }

    pub fn build<S>(self, cx: &mut UiContext<'_, S>) -> Rect
    where
        A: WidgetValueAction<S, bool>,
    {
        let id = self.id.unwrap_or_else(|| WidgetId::new(self.label.clone()));
        let interaction = cx.interaction(id.clone());
        let mut checked = self.checked;
        if interaction.clicked && !self.disabled && cx.consume_click(&id) {
            checked = !checked;
            self.on_change.call(cx.state_mut(), checked);
            cx.mark_changed();
        }

        let box_style = if checked {
            self.style.box_checked.clone()
        } else {
            interaction_style(
                interaction,
                self.disabled,
                &self.style.box_normal,
                &self.style.box_hovered,
                &self.style.box_pressed,
                &self.style.box_normal,
            )
        };

        Rect::new(RectLayout {
            id: Some(id),
            width: self.width,
            height: self.height,
            direction: Direction::Row,
            align: Align::Center,
            gap: 8,
            ..RectLayout::default()
        })
        .child(Rect::new(RectLayout {
            width: Length::Px(20),
            height: Length::Px(20),
            style: box_style,
            content: checked.then(|| {
                Content::Text(Text {
                    content: Arc::from("x"),
                    style: self.style.mark,
                    align: Align::Center,
                    vertical_align: Align::Center,
                    ..Text::default()
                })
            }),
            ..RectLayout::default()
        }))
        .child(Rect::new(RectLayout {
            width: Length::Fit,
            height: Length::Fill,
            content: Some(Content::Text(Text {
                content: self.label,
                style: self.style.text,
                vertical_align: Align::Center,
                ..Text::default()
            })),
            ..RectLayout::default()
        }))
    }
}
