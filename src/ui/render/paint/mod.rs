use super::scaling::*;
use super::*;

mod buffers;
mod scaled;

pub(super) fn multiply_opacity(parent: f32, child: f32) -> f32 {
    (parent * child).clamp(0.0, 1.0)
}

pub(super) fn border_widths(border: &Border) -> BorderWidth {
    if border.widths.is_zero() {
        BorderWidth::all(border.width)
    } else {
        border.widths
    }
}

pub(super) fn element_needs_measure(element: &Rect) -> bool {
    matches!(element.width, Length::Fit) || matches!(element.height, Length::Fit)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct PaintOffset {
    pub(super) x: i32,
    pub(super) y: i32,
}

pub(super) fn scale_i32(value: i32, scale: u32) -> i32 {
    let scaled = i64::from(value) * i64::from(scale);
    scaled.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn paint_command_with_offset(
    canvas: &mut Canvas<'_>,
    fonts: &mut FontCtx,
    command: PaintCommand<'_>,
    offset: PaintOffset,
) {
    match command {
        PaintCommand::Rect {
            rect,
            clip,
            opacity,
            paint,
            gradient,
            radii,
            anti_alias,
        } => fill_rounded_rect(
            canvas, rect, clip, opacity, radii, anti_alias, paint, gradient, offset,
        ),
        PaintCommand::Border {
            rect,
            clip,
            opacity,
            paint,
            gradient,
            widths,
            radii,
            anti_alias,
        } => stroke_rounded_rect(
            canvas, rect, clip, opacity, widths, radii, anti_alias, paint, gradient, offset,
        ),
        PaintCommand::Text {
            rect,
            clip,
            opacity,
            scale,
            text,
        } => {
            if scale == 1.0 {
                draw_text(canvas, fonts, rect, clip, opacity, text, offset);
            } else {
                let text = scaled_text_f32(text, scale);
                draw_text(canvas, fonts, rect, clip, opacity, &text, offset);
            }
        }
        PaintCommand::RichText {
            rect,
            clip,
            opacity,
            scale,
            text,
        } => {
            if scale == 1.0 {
                draw_rich_text(canvas, fonts, rect, clip, opacity, text, offset);
            } else {
                let text = scaled_rich_text_f32(text, scale);
                draw_rich_text(canvas, fonts, rect, clip, opacity, &text, offset);
            }
        }
        PaintCommand::Image {
            rect,
            clip,
            opacity,
            image,
        } => draw_image(canvas, rect, clip, opacity, image, offset),
    }
}

fn normalized_opacity(opacity: f32) -> f32 {
    opacity.clamp(0.0, 1.0)
}

pub(super) fn opacity_draws(opacity: f32) -> bool {
    normalized_opacity(opacity) > 0.0
}

pub(super) fn color_with_opacity_and_coverage(color: Color, opacity: f32, coverage: u8) -> [u8; 4] {
    let opacity = normalized_opacity(opacity);
    if opacity >= 1.0 && coverage == 255 {
        return color.into();
    }

    let alpha = (f32::from(color.alpha) * opacity * (f32::from(coverage) / 255.0)).round() as u8;
    Color { alpha, ..color }.into()
}

pub(super) fn pixel_with_opacity_and_coverage(
    mut rgba: [u8; 4],
    opacity: f32,
    coverage: u8,
) -> [u8; 4] {
    let opacity = normalized_opacity(opacity);
    if opacity < 1.0 || coverage != 255 {
        rgba[3] = (f32::from(rgba[3]) * opacity * (f32::from(coverage) / 255.0)).round() as u8;
    }
    rgba
}

fn visible_world_bounds(canvas: &Canvas<'_>, offset: PaintOffset) -> Option<Bounds> {
    let left = i64::from(offset.x).max(0);
    let top = i64::from(offset.y).max(0);
    let right = i64::from(offset.x).saturating_add(i64::from(canvas.width()));
    let bottom = i64::from(offset.y).saturating_add(i64::from(canvas.height()));
    if right <= left || bottom <= top {
        return None;
    }

    Some(Bounds {
        x: left.min(i64::from(u32::MAX)) as u32,
        y: top.min(i64::from(u32::MAX)) as u32,
        width: right.min(i64::from(u32::MAX)).saturating_sub(left).max(0) as u32,
        height: bottom.min(i64::from(u32::MAX)).saturating_sub(top).max(0) as u32,
    })
}

pub(super) fn visible_draw_bounds(
    canvas: &Canvas<'_>,
    bounds: Bounds,
    offset: PaintOffset,
) -> Option<Bounds> {
    bounds.intersect(visible_world_bounds(canvas, offset)?)
}

pub(super) fn target_coord(world: u32, offset: i32, max: u32) -> Option<u32> {
    let target = i64::from(world) - i64::from(offset);
    (target >= 0 && target < i64::from(max)).then_some(target as u32)
}

pub(super) fn paint_scaled_command_with_offset(
    canvas: &mut Canvas<'_>,
    fonts: &mut FontCtx,
    command: PaintCommand<'_>,
    scale: u32,
    offset: PaintOffset,
) {
    if scale == 1 {
        paint_command_with_offset(canvas, fonts, command, offset);
        return;
    }

    match command {
        PaintCommand::Rect {
            rect,
            clip,
            opacity,
            paint,
            gradient,
            radii,
            anti_alias,
        } => fill_rounded_rect(
            canvas,
            scale_bounds(rect, scale),
            scale_clip(clip, scale),
            opacity,
            radii.scaled(scale),
            anti_alias,
            paint,
            gradient,
            offset,
        ),
        PaintCommand::Border {
            rect,
            clip,
            opacity,
            paint,
            gradient,
            widths,
            radii,
            anti_alias,
        } => stroke_rounded_rect(
            canvas,
            scale_bounds(rect, scale),
            scale_clip(clip, scale),
            opacity,
            widths.scaled(scale),
            radii.scaled(scale),
            anti_alias,
            paint,
            gradient,
            offset,
        ),
        PaintCommand::Text {
            rect,
            clip,
            opacity,
            scale: text_scale,
            text,
        } => {
            let text = scaled_text(text, scale);
            let text = if text_scale == 1.0 {
                text
            } else {
                let mut text = text;
                scale_text_style_f32(&mut text.style, text_scale);
                text
            };
            draw_text(
                canvas,
                fonts,
                scale_bounds(rect, scale),
                scale_clip(clip, scale),
                opacity,
                &text,
                offset,
            );
        }
        PaintCommand::RichText {
            rect,
            clip,
            opacity,
            scale: text_scale,
            text,
        } => {
            let text = scaled_rich_text(text, scale);
            let text = if text_scale == 1.0 {
                text
            } else {
                scale_rich_text_styles_f32(text, text_scale)
            };
            draw_rich_text(
                canvas,
                fonts,
                scale_bounds(rect, scale),
                scale_clip(clip, scale),
                opacity,
                &text,
                offset,
            );
        }
        PaintCommand::Image {
            rect,
            clip,
            opacity,
            image,
        } => draw_image(
            canvas,
            scale_bounds(rect, scale),
            scale_clip(clip, scale),
            opacity,
            image,
            offset,
        ),
    }
}

pub(super) fn paint_scaled_f32_command_with_offset(
    canvas: &mut Canvas<'_>,
    fonts: &mut FontCtx,
    command: PaintCommand<'_>,
    scale: f32,
    offset: PaintOffset,
) {
    if scale == 1.0 {
        paint_command_with_offset(canvas, fonts, command, offset);
        return;
    }

    match command {
        PaintCommand::Rect {
            rect,
            clip,
            opacity,
            paint,
            gradient,
            radii,
            anti_alias,
        } => fill_rounded_rect(
            canvas,
            scale_bounds_f32(rect, scale),
            scale_clip_f32(clip, scale),
            opacity,
            scale_corner_radius_f32(radii, scale),
            anti_alias,
            paint,
            gradient,
            offset,
        ),
        PaintCommand::Border {
            rect,
            clip,
            opacity,
            paint,
            gradient,
            widths,
            radii,
            anti_alias,
        } => stroke_rounded_rect(
            canvas,
            scale_bounds_f32(rect, scale),
            scale_clip_f32(clip, scale),
            opacity,
            scale_border_width_f32(widths, scale),
            scale_corner_radius_f32(radii, scale),
            anti_alias,
            paint,
            gradient,
            offset,
        ),
        PaintCommand::Text {
            rect,
            clip,
            opacity,
            scale: text_scale,
            text,
        } => {
            let text = scaled_text_f32(text, scale * text_scale);
            draw_text(
                canvas,
                fonts,
                scale_bounds_f32(rect, scale),
                scale_clip_f32(clip, scale),
                opacity,
                &text,
                offset,
            );
        }
        PaintCommand::RichText {
            rect,
            clip,
            opacity,
            scale: text_scale,
            text,
        } => {
            let text = scaled_rich_text_f32(text, scale * text_scale);
            draw_rich_text(
                canvas,
                fonts,
                scale_bounds_f32(rect, scale),
                scale_clip_f32(clip, scale),
                opacity,
                &text,
                offset,
            );
        }
        PaintCommand::Image {
            rect,
            clip,
            opacity,
            image,
        } => draw_image(
            canvas,
            scale_bounds_f32(rect, scale),
            scale_clip_f32(clip, scale),
            opacity,
            image,
            offset,
        ),
    }
}
