use crate::{Color, Paint, Style};

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
