use super::*;

pub(in crate::ui::render) fn measure_text(
    fonts: &mut FontCtx,
    text: TextContent<'_>,
    width: u32,
) -> Size {
    if text.is_empty() || width == 0 {
        return Size::default();
    }

    let FontCtx {
        font_system,
        text_buffer,
        text_runs,
        ..
    } = fonts;
    let font_size = text.default_style().size.max(1) as f32;
    let metrics = Metrics::new(font_size, font_size * 1.3);
    let attrs = attrs_for_style(text.default_style(), None, 0);
    split_text_runs_into(text, text_runs);
    text_buffer.set_metrics(metrics);
    text_buffer.set_size(Some(width as f32), None);
    text_buffer.set_wrap(text.wrap().into_cosmic());
    text_buffer.set_ellipsize(text_ellipsize(text));
    text_buffer.set_rich_text(
        text_runs.iter().enumerate().map(|(index, run)| {
            let family = run
                .is_emoji()
                .then_some(run.emoji_family.as_deref())
                .flatten();
            (run.text(), attrs_for_style(&run.style, family, index + 1))
        }),
        &attrs,
        Shaping::Advanced,
        None,
    );
    text_buffer.shape_until_scroll(font_system, false);

    let mut measured = Size::default();
    for (line_index, run) in text_buffer.layout_runs().enumerate() {
        if text
            .max_lines()
            .is_some_and(|max_lines| line_index as u32 >= max_lines)
        {
            break;
        }
        measured.width = measured.width.max(run.line_w.ceil() as u32);
        measured.height = (run.line_top + run.line_height).ceil() as u32;
    }
    text_runs.clear();

    Size::new(measured.width.min(width), measured.height)
}

pub(in crate::ui::render) fn text_ellipsize(text: TextContent<'_>) -> Ellipsize {
    match (text.overflow(), text.max_lines()) {
        (TextOverflow::Ellipsis, Some(lines)) if lines > 0 => {
            Ellipsize::End(EllipsizeHeightLimit::Lines(lines as usize))
        }
        _ => Ellipsize::None,
    }
}

pub(in crate::ui::render) fn measure_image(
    image: &Image,
    available_width: u32,
    available_height: u32,
) -> Size {
    if image.width == 0 || image.height == 0 || available_width == 0 || available_height == 0 {
        return Size::default();
    }

    match image.fit {
        ImageFit::None => Size::new(
            image.width.min(available_width),
            image.height.min(available_height),
        ),
        ImageFit::Fill => Size::new(available_width, available_height),
        ImageFit::Contain | ImageFit::Cover => {
            let scale_by_width = u64::from(available_width) * u64::from(image.height);
            let scale_by_height = u64::from(available_height) * u64::from(image.width);
            let use_width = match image.fit {
                ImageFit::Contain => scale_by_width <= scale_by_height,
                ImageFit::Cover => scale_by_width >= scale_by_height,
                ImageFit::None | ImageFit::Fill => unreachable!(),
            };
            let (width, height) = if use_width {
                let height = scaled_dimension(image.height, available_width, image.width);
                (available_width, height)
            } else {
                let width = scaled_dimension(image.width, available_height, image.height);
                (width, available_height)
            };
            Size::new(width.min(available_width), height.min(available_height))
        }
    }
}

pub(in crate::ui::render) fn scaled_dimension(value: u32, numerator: u32, denominator: u32) -> u32 {
    if denominator == 0 {
        return 0;
    }
    ((u64::from(value) * u64::from(numerator)) / u64::from(denominator))
        .max(1)
        .min(u64::from(u32::MAX)) as u32
}

pub(in crate::ui::render) fn align_offset(align: Align, available: u32, size: u32) -> u32 {
    match align {
        Align::Start | Align::Stretch => 0,
        Align::Center => available.saturating_sub(size) / 2,
        Align::End => available.saturating_sub(size),
    }
}

pub(in crate::ui::render) fn element_corner_radii(element: &Rect) -> CornerRadius {
    if element.style.corner_radii.is_zero() {
        CornerRadius::all(element.style.corner_radius)
    } else {
        element.style.corner_radii
    }
}

pub(in crate::ui::render) fn content_clip_radii(element: &Rect) -> CornerRadius {
    element_corner_radii(element).inset(element.padding)
}

pub(in crate::ui::render) fn fill_rounded_rect(
    canvas: &mut Canvas<'_>,
    rect: Bounds,
    clip: Clip,
    opacity: f32,
    radii: CornerRadius,
    anti_alias: bool,
    paint: &Paint,
    gradient: GradientDirection,
    offset: PaintOffset,
) {
    if !opacity_draws(opacity) {
        return;
    }

    let Some(draw) = rect
        .intersect(clip.bounds())
        .and_then(|rect| visible_draw_bounds(canvas, rect, offset))
    else {
        return;
    };

    if radii.is_zero()
        && clip.is_rect()
        && let Paint::Solid(color) = paint
        && let Some(target_x) = target_coord(draw.x, offset.x, canvas.width())
        && let Some(target_y) = target_coord(draw.y, offset.y, canvas.height())
    {
        canvas.blend_rect(
            DamageRect::new(target_x, target_y, draw.width, draw.height),
            color_with_opacity_and_coverage(*color, opacity, 255),
        );
        return;
    }

    for y in draw.y..draw.bottom() {
        for x in draw.x..draw.right() {
            let shape_coverage = rounded_rect_coverage_with_antialias(
                x - rect.x,
                y - rect.y,
                rect.width,
                rect.height,
                radii,
                anti_alias,
            );
            let coverage = multiply_coverage(shape_coverage, clip.coverage(x, y));
            if coverage > 0 {
                let color = paint.at(x - rect.x, y - rect.y, rect.width, rect.height, gradient);
                let Some(target_x) = target_coord(x, offset.x, canvas.width()) else {
                    continue;
                };
                let Some(target_y) = target_coord(y, offset.y, canvas.height()) else {
                    continue;
                };
                canvas.blend_pixel(
                    target_x,
                    target_y,
                    color_with_opacity_and_coverage(color, opacity, coverage),
                );
            }
        }
    }
}

pub(in crate::ui::render) fn stroke_rounded_rect(
    canvas: &mut Canvas<'_>,
    rect: Bounds,
    clip: Clip,
    opacity: f32,
    widths: BorderWidth,
    radii: CornerRadius,
    anti_alias: bool,
    paint: &Paint,
    gradient: GradientDirection,
    offset: PaintOffset,
) {
    if widths.is_zero() || rect.width == 0 || rect.height == 0 || !opacity_draws(opacity) {
        return;
    }

    let Some(draw) = rect
        .intersect(clip.bounds())
        .and_then(|rect| visible_draw_bounds(canvas, rect, offset))
    else {
        return;
    };
    let inset = Spacing {
        top: widths.top,
        right: widths.right,
        bottom: widths.bottom,
        left: widths.left,
    };
    let inner = rect.inset(inset);
    let inner_radii = radii.inset(inset);
    for y in draw.y..draw.bottom() {
        for x in draw.x..draw.right() {
            let outer_coverage = rounded_rect_coverage_with_antialias(
                x - rect.x,
                y - rect.y,
                rect.width,
                rect.height,
                radii,
                anti_alias,
            );
            let inner_coverage =
                rounded_rect_bounds_coverage_with_antialias(inner, x, y, inner_radii, anti_alias);
            let coverage = multiply_coverage(
                outer_coverage.saturating_sub(inner_coverage),
                clip.coverage(x, y),
            );
            if coverage > 0 {
                let color = paint.at(x - rect.x, y - rect.y, rect.width, rect.height, gradient);
                let Some(target_x) = target_coord(x, offset.x, canvas.width()) else {
                    continue;
                };
                let Some(target_y) = target_coord(y, offset.y, canvas.height()) else {
                    continue;
                };
                canvas.blend_pixel(
                    target_x,
                    target_y,
                    color_with_opacity_and_coverage(color, opacity, coverage),
                );
            }
        }
    }
}
