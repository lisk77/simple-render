use super::*;

pub(in crate::ui::render) fn draw_text_decorations(
    canvas: &mut Canvas<'_>,
    run: &LayoutRun<'_>,
    rect: Bounds,
    clip: Clip,
    opacity: f32,
    align_x: i32,
    align_y: i32,
    x_offset: i32,
    y_offset: i32,
    runs: &[TextBufferRun],
    default_style: &TextStyle,
    offset: PaintOffset,
) {
    for span in run.decorations {
        draw_text_decoration_span(
            canvas,
            run,
            span,
            rect,
            clip,
            opacity,
            align_x,
            align_y,
            x_offset,
            y_offset,
            runs,
            default_style,
            offset,
        );
    }
}

fn draw_text_decoration_span(
    canvas: &mut Canvas<'_>,
    run: &LayoutRun<'_>,
    span: &DecorationSpan,
    rect: Bounds,
    clip: Clip,
    opacity: f32,
    align_x: i32,
    align_y: i32,
    x_offset: i32,
    y_offset: i32,
    runs: &[TextBufferRun],
    default_style: &TextStyle,
    offset: PaintOffset,
) {
    let glyphs = &run.glyphs[span.glyph_range.clone()];
    if glyphs.is_empty() {
        return;
    }

    let mut x_min = f32::INFINITY;
    let mut x_max = f32::NEG_INFINITY;
    for glyph in glyphs {
        x_min = x_min.min(glyph.x);
        x_max = x_max.max(glyph.x + glyph.w);
    }
    let width = (x_max - x_min).max(0.0) as u32;
    if width == 0 {
        return;
    }

    let style = glyphs
        .first()
        .map(|glyph| style_for_metadata(glyph.metadata, runs, default_style))
        .unwrap_or(default_style);
    let default_color = cosmic_color(style.color.first());
    let x = x_min as i32 + align_x + x_offset;
    let deco = &span.data;
    let text_decoration = &deco.text_decoration;
    let font_size = span.font_size;

    match text_decoration.underline {
        UnderlineStyle::None => {}
        UnderlineStyle::Single => {
            let color = text_decoration
                .underline_color_opt
                .or(span.color_opt)
                .unwrap_or(default_color);
            let thickness = (deco.underline_metrics.thickness * font_size)
                .max(1.0)
                .ceil() as u32;
            let y = (run.line_y - deco.underline_metrics.offset * font_size) as i32
                + align_y
                + y_offset;
            draw_text_decoration_rect(
                canvas, rect, clip, opacity, x, y, width, thickness, color, style, offset,
            );
        }
        UnderlineStyle::Double => {
            let color = text_decoration
                .underline_color_opt
                .or(span.color_opt)
                .unwrap_or(default_color);
            let thickness = (deco.underline_metrics.thickness * font_size)
                .max(1.0)
                .ceil() as u32;
            let gap = thickness as i32;
            let y = (run.line_y - deco.underline_metrics.offset * font_size) as i32
                + align_y
                + y_offset;
            draw_text_decoration_rect(
                canvas, rect, clip, opacity, x, y, width, thickness, color, style, offset,
            );
            draw_text_decoration_rect(
                canvas,
                rect,
                clip,
                opacity,
                x,
                y + thickness as i32 + gap,
                width,
                thickness,
                color,
                style,
                offset,
            );
        }
    }

    if text_decoration.strikethrough {
        let color = text_decoration
            .strikethrough_color_opt
            .or(span.color_opt)
            .unwrap_or(default_color);
        let thickness = (deco.strikethrough_metrics.thickness * font_size)
            .max(1.0)
            .ceil() as u32;
        let y = (run.line_y - deco.strikethrough_metrics.offset * font_size) as i32
            + align_y
            + y_offset;
        draw_text_decoration_rect(
            canvas, rect, clip, opacity, x, y, width, thickness, color, style, offset,
        );
    }

    if text_decoration.overline {
        let color = text_decoration
            .overline_color_opt
            .or(span.color_opt)
            .unwrap_or(default_color);
        let thickness = (deco.underline_metrics.thickness * font_size)
            .max(1.0)
            .ceil() as u32;
        let y =
            (run.line_y - deco.ascent * font_size).max(run.line_top) as i32 + align_y + y_offset;
        draw_text_decoration_rect(
            canvas, rect, clip, opacity, x, y, width, thickness, color, style, offset,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_text_decoration_rect(
    canvas: &mut Canvas<'_>,
    rect: Bounds,
    clip: Clip,
    opacity: f32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    color: CosmicColor,
    style: &TextStyle,
    offset: PaintOffset,
) {
    if width == 0 || height == 0 {
        return;
    }

    for dy in 0..height {
        for dx in 0..width {
            let Some(target_x) = x.checked_add(dx as i32) else {
                continue;
            };
            let Some(target_y) = y.checked_add(dy as i32) else {
                continue;
            };
            if target_x < 0 || target_y < 0 {
                continue;
            }
            let world_x = target_x as u32;
            let world_y = target_y as u32;
            let Some(target_x) = target_coord(world_x, offset.x, canvas.width()) else {
                continue;
            };
            let Some(target_y) = target_coord(world_y, offset.y, canvas.height()) else {
                continue;
            };
            let coverage = clip.coverage(world_x, world_y);
            if coverage == 0 {
                continue;
            }
            let Some(local_x) = world_x.checked_sub(rect.x) else {
                continue;
            };
            let Some(local_y) = world_y.checked_sub(rect.y) else {
                continue;
            };
            let color = text_pixel_color(color, style, local_x, local_y, rect.width, rect.height);
            canvas.blend_pixel(
                target_x,
                target_y,
                color_with_opacity_and_coverage(color, opacity, coverage),
            );
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::ui::render) struct TextPixelBounds {
    pub(in crate::ui::render) min_x: i32,
    pub(in crate::ui::render) max_x: i32,
}

impl TextPixelBounds {
    fn new(x: i32) -> Self {
        Self { min_x: x, max_x: x }
    }

    fn include(&mut self, x: i32) {
        self.min_x = self.min_x.min(x);
        self.max_x = self.max_x.max(x);
    }

    fn width(self) -> i32 {
        self.max_x.saturating_sub(self.min_x).saturating_add(1)
    }
}

pub(in crate::ui::render) fn text_pixel_bounds(
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
    glyphs: &[GlyphPaint],
    runs: &[TextBufferRun],
    text: TextContent<'_>,
) -> Option<TextPixelBounds> {
    if matches!(text.align(), Align::Start | Align::Stretch) {
        return None;
    }

    let mut bounds: Option<TextPixelBounds> = None;
    for (glyph, metadata, base_color) in glyphs {
        let _style = style_for_metadata(*metadata, runs, text.default_style());
        let color = *base_color;
        swash_cache.with_pixels(font_system, glyph.cache_key, color, |x, _, _| {
            let local_x = glyph.x.saturating_add(x);
            match &mut bounds {
                Some(bounds) => bounds.include(local_x),
                None => bounds = Some(TextPixelBounds::new(local_x)),
            }
        });
    }

    bounds
}

pub(in crate::ui::render) fn style_for_metadata<'a>(
    metadata: usize,
    runs: &'a [TextBufferRun],
    default_style: &'a TextStyle,
) -> &'a TextStyle {
    metadata
        .checked_sub(1)
        .and_then(|index| runs.get(index))
        .map(|run| &run.style)
        .unwrap_or(default_style)
}

pub(in crate::ui::render) fn text_horizontal_align_offset(
    align: Align,
    available: u32,
    bounds: Option<TextPixelBounds>,
) -> i32 {
    let Some(bounds) = bounds else {
        return 0;
    };

    let available = i32::try_from(available).unwrap_or(i32::MAX);
    match align {
        Align::Start | Align::Stretch => 0,
        Align::Center => available.saturating_sub(bounds.width()) / 2 - bounds.min_x,
        Align::End => available.saturating_sub(bounds.width()) - bounds.min_x,
    }
}

pub(in crate::ui::render) fn text_line_offset(
    align: Align,
    available: u32,
    line_height: i32,
) -> i32 {
    let available = i32::try_from(available).unwrap_or(i32::MAX);
    match align {
        Align::Start | Align::Stretch => 0,
        Align::Center => available.saturating_sub(line_height) / 2,
        Align::End => available.saturating_sub(line_height),
    }
}

pub(in crate::ui::render) fn text_pixel_color(
    color: CosmicColor,
    style: &TextStyle,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Color {
    let first = style.color.first();
    if color.r() != first.red || color.g() != first.green || color.b() != first.blue {
        return Color::rgba(color.r(), color.g(), color.b(), color.a());
    }

    let sampled = style.color.at(x, y, width, height, style.gradient);
    let alpha = ((u32::from(color.a()) * u32::from(sampled.alpha)) / 255).min(255) as u8;
    Color::rgba(sampled.red, sampled.green, sampled.blue, alpha)
}
