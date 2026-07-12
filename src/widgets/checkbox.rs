use std::sync::Arc;

use crate::{
    Align, Content, Direction, Inset, Length, Paint, Position, Rect, RectLayout, Style, Text,
    TextStyle, UiContext, WidgetId,
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
    pub mark: Paint,
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
            mark: Paint::solid(hex("#ffffff")),
        }
    }
}

impl CheckboxStyle {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn box_normal(mut self, value: crate::Style) -> Self {
        self.box_normal = value;
        self
    }
    pub fn box_hovered(mut self, value: crate::Style) -> Self {
        self.box_hovered = value;
        self
    }
    pub fn box_pressed(mut self, value: crate::Style) -> Self {
        self.box_pressed = value;
        self
    }
    pub fn box_checked(mut self, value: crate::Style) -> Self {
        self.box_checked = value;
        self
    }
    pub fn text(mut self, value: TextStyle) -> Self {
        self.text = value;
        self
    }
    pub fn mark(mut self, value: impl Into<Paint>) -> Self {
        self.mark = value.into();
        self
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
    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = label.into();
        self
    }

    pub fn id(mut self, id: impl Into<WidgetId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
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

        let mut checkbox_box = Rect::layout(RectLayout {
            width: Length::Px(20),
            height: Length::Px(20),
            style: box_style,
            ..RectLayout::default()
        });
        if checked {
            checkbox_box = checkbox_box.child(check_mark(self.style.mark.clone()));
        }

        Rect::layout(RectLayout {
            id: Some(id),
            width: self.width,
            height: self.height,
            direction: Direction::Row,
            align: Align::Center,
            gap: 8,
            ..RectLayout::default()
        })
        .child(checkbox_box)
        .child(Rect::layout(RectLayout {
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

fn check_mark(color: Paint) -> Rect {
    Rect::layout(RectLayout {
        width: Length::Fill,
        height: Length::Fill,
        ..RectLayout::default()
    })
    .child(mark_dot(5, 5, color.clone()))
    .child(mark_dot(8, 8, color.clone()))
    .child(mark_dot(11, 11, color.clone()))
    .child(mark_dot(11, 5, color.clone()))
    .child(mark_dot(5, 11, color))
}

fn mark_dot(left: u32, top: u32, color: Paint) -> Rect {
    Rect::layout(RectLayout {
        width: Length::Px(4),
        height: Length::Px(4),
        position: Position::Absolute,
        inset: Inset {
            left: Some(left),
            top: Some(top),
            ..Inset::ZERO
        },
        style: Style {
            background: Some(color),
            corner_radius: 1,
            ..Style::default()
        },
        ..RectLayout::default()
    })
}
