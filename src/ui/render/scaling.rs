use super::*;
pub(super) fn scale_bounds(bounds: Bounds, scale: u32) -> Bounds {
    Bounds {
        x: scale_value(bounds.x, scale),
        y: scale_value(bounds.y, scale),
        width: scale_value(bounds.width, scale),
        height: scale_value(bounds.height, scale),
    }
}

pub(super) fn scale_value(value: u32, scale: u32) -> u32 {
    value.saturating_mul(scale)
}

pub(super) fn scale_clip(clip: Clip, scale: u32) -> Clip {
    let mut scaled = Clip::rect(scale_bounds(clip.bounds(), scale));
    for rounded in clip.rounded[..usize::from(clip.rounded_len)].iter() {
        scaled = scaled
            .with_rounded_rect(
                scale_bounds(rounded.rect, scale),
                rounded.radii.scaled(scale),
            )
            .expect("scaled rounded clip remains inside scaled bounds");
    }
    scaled
}

pub(super) fn scale_bounds_f32(bounds: Bounds, scale: f32) -> Bounds {
    let x = scale_floor_u32(bounds.x, scale);
    let y = scale_floor_u32(bounds.y, scale);
    let right = scale_ceil_u32(bounds.right(), scale);
    let bottom = scale_ceil_u32(bounds.bottom(), scale);
    Bounds {
        x,
        y,
        width: right.saturating_sub(x),
        height: bottom.saturating_sub(y),
    }
}

pub(super) fn scale_value_f32(value: u32, scale: f32) -> u32 {
    clamp_scaled_u32(f64::from(value) * f64::from(scale), FloatRound::Round)
}

pub(super) fn scale_floor_u32(value: u32, scale: f32) -> u32 {
    clamp_scaled_u32(f64::from(value) * f64::from(scale), FloatRound::Floor)
}

pub(super) fn scale_ceil_u32(value: u32, scale: f32) -> u32 {
    clamp_scaled_u32(f64::from(value) * f64::from(scale), FloatRound::Ceil)
}

#[derive(Clone, Copy)]
enum FloatRound {
    Floor,
    Round,
    Ceil,
}

fn clamp_scaled_u32(value: f64, round: FloatRound) -> u32 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }
    let rounded = match round {
        FloatRound::Floor => value.floor(),
        FloatRound::Round => value.round(),
        FloatRound::Ceil => value.ceil(),
    };
    rounded.min(f64::from(u32::MAX)) as u32
}

pub(super) fn scale_clip_f32(clip: Clip, scale: f32) -> Clip {
    let mut scaled = Clip::rect(scale_bounds_f32(clip.bounds(), scale));
    for rounded in clip.rounded[..usize::from(clip.rounded_len)].iter() {
        if let Some(next) = scaled.with_rounded_rect(
            scale_bounds_f32(rounded.rect, scale),
            scale_corner_radius_f32(rounded.radii, scale),
        ) {
            scaled = next;
        }
    }
    scaled
}

pub(super) fn scale_corner_radius_f32(radii: CornerRadius, scale: f32) -> CornerRadius {
    CornerRadius {
        top_left: scale_value_f32(radii.top_left, scale),
        top_right: scale_value_f32(radii.top_right, scale),
        bottom_right: scale_value_f32(radii.bottom_right, scale),
        bottom_left: scale_value_f32(radii.bottom_left, scale),
    }
}

pub(super) fn scale_border_width_f32(widths: BorderWidth, scale: f32) -> BorderWidth {
    BorderWidth {
        top: scale_value_f32(widths.top, scale),
        right: scale_value_f32(widths.right, scale),
        bottom: scale_value_f32(widths.bottom, scale),
        left: scale_value_f32(widths.left, scale),
    }
}

pub(super) fn scaled_text(text: &Text, scale: u32) -> Text {
    let mut text = text.clone();
    scale_text_style(&mut text.style, scale);
    text
}

pub(super) fn scaled_rich_text(text: &RichText, scale: u32) -> RichText {
    let mut text = text.clone();
    text.runs = text
        .runs
        .iter()
        .cloned()
        .map(|mut run| {
            scale_text_style(&mut run.style, scale);
            run
        })
        .collect();
    text
}

pub(super) fn scale_text_style(style: &mut TextStyle, scale: u32) {
    style.size = scale_value(style.size, scale);
}

pub(super) fn scaled_text_f32(text: &Text, scale: f32) -> Text {
    let mut text = text.clone();
    scale_text_style_f32(&mut text.style, scale);
    text
}

pub(super) fn scaled_rich_text_f32(text: &RichText, scale: f32) -> RichText {
    let mut text = text.clone();
    text.runs = text
        .runs
        .iter()
        .cloned()
        .map(|mut run| {
            scale_text_style_f32(&mut run.style, scale);
            run
        })
        .collect();
    text
}

pub(super) fn scale_rich_text_styles_f32(mut text: RichText, scale: f32) -> RichText {
    for run in Arc::make_mut(&mut text.runs).iter_mut() {
        scale_text_style_f32(&mut run.style, scale);
    }
    text
}

pub(super) fn scale_text_style_f32(style: &mut TextStyle, scale: f32) {
    style.size = scale_value_f32(style.size, scale).max(1);
}
