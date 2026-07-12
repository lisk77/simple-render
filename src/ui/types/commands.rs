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
        anti_alias: bool,
    },
    Border {
        rect: Bounds,
        clip: Clip,
        opacity: f32,
        paint: Paint,
        gradient: GradientDirection,
        widths: BorderWidth,
        radii: CornerRadius,
        anti_alias: bool,
    },
    Text {
        rect: Bounds,
        clip: Clip,
        opacity: f32,
        scale: f32,
        text: Text,
    },
    RichText {
        rect: Bounds,
        clip: Clip,
        opacity: f32,
        scale: f32,
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
        anti_alias: bool,
    },
    Border {
        rect: Bounds,
        clip: Clip,
        opacity: f32,
        paint: &'a Paint,
        gradient: GradientDirection,
        widths: BorderWidth,
        radii: CornerRadius,
        anti_alias: bool,
    },
    Text {
        rect: Bounds,
        clip: Clip,
        opacity: f32,
        scale: f32,
        text: &'a Text,
    },
    RichText {
        rect: Bounds,
        clip: Clip,
        opacity: f32,
        scale: f32,
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
    pub(in crate::ui) fn rect(&self) -> Bounds {
        match self {
            Self::Rect { rect, .. }
            | Self::Border { rect, .. }
            | Self::Text { rect, .. }
            | Self::RichText { rect, .. }
            | Self::Image { rect, .. } => *rect,
        }
    }

    pub(in crate::ui) fn to_owned(&self) -> DrawCommand {
        match self {
            Self::Rect {
                rect,
                clip,
                opacity,
                paint,
                gradient,
                radii,
                anti_alias,
            } => DrawCommand::Rect {
                rect: *rect,
                clip: *clip,
                opacity: *opacity,
                paint: (*paint).clone(),
                gradient: *gradient,
                radii: *radii,
                anti_alias: *anti_alias,
            },
            Self::Border {
                rect,
                clip,
                opacity,
                paint,
                gradient,
                widths,
                radii,
                anti_alias,
            } => DrawCommand::Border {
                rect: *rect,
                clip: *clip,
                opacity: *opacity,
                paint: (*paint).clone(),
                gradient: *gradient,
                widths: *widths,
                radii: *radii,
                anti_alias: *anti_alias,
            },
            Self::Text {
                rect,
                clip,
                opacity,
                scale,
                text,
            } => DrawCommand::Text {
                rect: *rect,
                clip: *clip,
                opacity: *opacity,
                scale: *scale,
                text: (*text).clone(),
            },
            Self::RichText {
                rect,
                clip,
                opacity,
                scale,
                text,
            } => DrawCommand::RichText {
                rect: *rect,
                clip: *clip,
                opacity: *opacity,
                scale: *scale,
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
