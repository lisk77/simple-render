use std::sync::Arc;

use crate::{
    Align, ChangeEvent, ClickEvent, Direction, Element, Inset, Length, Listener, Paint, Position,
    Rect, RectLayout, Style, Text, TextStyle, WidgetId,
};

use super::shared::{hex, rounded_fill};

#[derive(Clone, Debug)]
pub struct CheckboxStyle {
    pub box_normal: crate::Style,
    pub box_hovered: crate::Style,
    pub box_pressed: crate::Style,
    pub box_checked: crate::Style,
    pub text: TextStyle,
    pub mark: Paint,
}

impl Checkbox<()> {
    fn into_element_with(self, listener: Option<Listener<ChangeEvent<bool>>>) -> Element {
        let mut checkbox_box = Rect::new().size_px(20, 20).style(if self.checked {
            self.style.box_checked.clone()
        } else {
            self.style.box_normal.clone()
        });
        if self.checked {
            checkbox_box = checkbox_box.child(check_mark(self.style.mark.clone()));
        }
        let mut root = Rect::new()
            .width(self.width)
            .height(self.height)
            .direction(Direction::Row)
            .align(Align::Center)
            .gap(8)
            .child(checkbox_box)
            .child(
                Rect::new().width(Length::Fit).height_fill().text(
                    Text::new(self.label)
                        .style(self.style.text)
                        .vertical_align_center(),
                ),
            );
        if !self.checked && !self.disabled {
            root = root.interaction_child_styles(
                0,
                self.style.box_hovered.clone(),
                self.style.box_pressed.clone(),
                self.style.box_normal.clone(),
            );
        }
        if let Some(id) = self.id {
            root = root.id(id);
        }
        if !self.disabled
            && let Some(listener) = listener
        {
            let checked = self.checked;
            root = root.on_click(Listener::<ClickEvent>::new(move |_| {
                listener.call(&ChangeEvent { value: !checked });
            }));
        }
        root.into()
    }
}

impl From<Checkbox<()>> for Element {
    fn from(value: Checkbox<()>) -> Self {
        value.into_element_with(None)
    }
}

impl From<Checkbox<Listener<ChangeEvent<bool>>>> for Element {
    fn from(value: Checkbox<Listener<ChangeEvent<bool>>>) -> Self {
        let Checkbox {
            id,
            label,
            checked,
            on_change,
            width,
            height,
            disabled,
            style,
        } = value;
        Checkbox {
            id,
            label,
            checked,
            on_change: (),
            width,
            height,
            disabled,
            style,
        }
        .into_element_with(Some(on_change))
    }
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

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
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
