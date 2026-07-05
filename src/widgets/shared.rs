use crate::{Color, Paint, Style, WidgetInteraction};

pub(crate) fn interaction_style(
    interaction: WidgetInteraction,
    disabled: bool,
    normal: &Style,
    hovered: &Style,
    pressed: &Style,
    disabled_style: &Style,
) -> Style {
    if disabled {
        disabled_style.clone()
    } else if interaction.pressed {
        pressed.clone()
    } else if interaction.hovered {
        hovered.clone()
    } else {
        normal.clone()
    }
}

pub(crate) fn rounded_fill(color: Color, radius: u32) -> Style {
    Style {
        background: Some(Paint::solid(color)),
        corner_radius: radius,
        ..Style::default()
    }
}

pub(crate) fn hex(value: &str) -> Color {
    Color::from_hex(value).expect("valid widget color")
}
