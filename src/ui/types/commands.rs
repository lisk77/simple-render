use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum DrawCommand {
    Rect {
        rect: Bounds,
        clip: Clip,
        opacity: f32,
        paint: Paint,
        gradient: GradientDirection,
        radii: CornerRadius,
    },
    Border {
        rect: Bounds,
        clip: Clip,
        opacity: f32,
        paint: Paint,
        gradient: GradientDirection,
        widths: BorderWidth,
        radii: CornerRadius,
    },
    Text {
        rect: Bounds,
        clip: Clip,
        opacity: f32,
        text: Text,
    },
    RichText {
        rect: Bounds,
        clip: Clip,
        opacity: f32,
        text: RichText,
    },
    Image {
        rect: Bounds,
        clip: Clip,
        opacity: f32,
        image: Image,
    },
}

pub(in crate::ui) enum PaintCommand<'a> {
    Rect {
        rect: Bounds,
        clip: Clip,
        opacity: f32,
        paint: &'a Paint,
        gradient: GradientDirection,
        radii: CornerRadius,
    },
    Border {
        rect: Bounds,
        clip: Clip,
        opacity: f32,
        paint: &'a Paint,
        gradient: GradientDirection,
        widths: BorderWidth,
        radii: CornerRadius,
    },
    Text {
        rect: Bounds,
        clip: Clip,
        opacity: f32,
        text: &'a Text,
    },
    RichText {
        rect: Bounds,
        clip: Clip,
        opacity: f32,
        text: &'a RichText,
    },
    Image {
        rect: Bounds,
        clip: Clip,
        opacity: f32,
        image: &'a Image,
    },
}

impl PaintCommand<'_> {
    pub(in crate::ui) fn to_owned(&self) -> DrawCommand {
        match self {
            Self::Rect {
                rect,
                clip,
                opacity,
                paint,
                gradient,
                radii,
            } => DrawCommand::Rect {
                rect: *rect,
                clip: *clip,
                opacity: *opacity,
                paint: (*paint).clone(),
                gradient: *gradient,
                radii: *radii,
            },
            Self::Border {
                rect,
                clip,
                opacity,
                paint,
                gradient,
                widths,
                radii,
            } => DrawCommand::Border {
                rect: *rect,
                clip: *clip,
                opacity: *opacity,
                paint: (*paint).clone(),
                gradient: *gradient,
                widths: *widths,
                radii: *radii,
            },
            Self::Text {
                rect,
                clip,
                opacity,
                text,
            } => DrawCommand::Text {
                rect: *rect,
                clip: *clip,
                opacity: *opacity,
                text: (*text).clone(),
            },
            Self::RichText {
                rect,
                clip,
                opacity,
                text,
            } => DrawCommand::RichText {
                rect: *rect,
                clip: *clip,
                opacity: *opacity,
                text: (*text).clone(),
            },
            Self::Image {
                rect,
                clip,
                opacity,
                image,
            } => DrawCommand::Image {
                rect: *rect,
                clip: *clip,
                opacity: *opacity,
                image: (*image).clone(),
            },
        }
    }
}
