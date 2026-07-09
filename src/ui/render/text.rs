use super::geometry::{
    multiply_coverage, rounded_rect_bounds_coverage_with_antialias,
    rounded_rect_coverage_with_antialias,
};
use super::*;

#[derive(Clone, Copy)]
pub(super) enum TextContent<'a> {
    Plain(&'a Text),
    Rich(&'a RichText),
}

impl TextContent<'_> {
    fn is_empty(self) -> bool {
        match self {
            Self::Plain(text) => text.content.is_empty(),
            Self::Rich(text) => text.runs.iter().all(|run| run.content.is_empty()),
        }
    }

    fn align(self) -> Align {
        match self {
            Self::Plain(text) => text.align,
            Self::Rich(text) => text.align,
        }
    }

    fn vertical_align(self) -> Align {
        match self {
            Self::Plain(text) => text.vertical_align,
            Self::Rich(text) => text.vertical_align,
        }
    }

    fn wrap(self) -> TextWrap {
        match self {
            Self::Plain(text) => text.wrap,
            Self::Rich(text) => text.wrap,
        }
    }

    fn overflow(self) -> TextOverflow {
        match self {
            Self::Plain(text) => text.overflow,
            Self::Rich(text) => text.overflow,
        }
    }

    fn max_lines(self) -> Option<u32> {
        match self {
            Self::Plain(text) => text.max_lines,
            Self::Rich(text) => text.max_lines,
        }
    }

    fn default_style(&self) -> &TextStyle {
        match self {
            Self::Plain(text) => &text.style,
            Self::Rich(text) => text
                .runs
                .first()
                .map(|run| &run.style)
                .expect("rich text has at least one non-empty run"),
        }
    }
}

pub(super) fn measure_text(fonts: &mut FontCtx, text: TextContent<'_>, width: u32) -> Size {
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

pub(super) fn text_ellipsize(text: TextContent<'_>) -> Ellipsize {
    match (text.overflow(), text.max_lines()) {
        (TextOverflow::Ellipsis, Some(lines)) if lines > 0 => {
            Ellipsize::End(EllipsizeHeightLimit::Lines(lines as usize))
        }
        _ => Ellipsize::None,
    }
}

pub(super) fn measure_image(image: &Image, available_width: u32, available_height: u32) -> Size {
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

pub(super) fn scaled_dimension(value: u32, numerator: u32, denominator: u32) -> u32 {
    if denominator == 0 {
        return 0;
    }
    ((u64::from(value) * u64::from(numerator)) / u64::from(denominator))
        .max(1)
        .min(u64::from(u32::MAX)) as u32
}

pub(super) fn align_offset(align: Align, available: u32, size: u32) -> u32 {
    match align {
        Align::Start | Align::Stretch => 0,
        Align::Center => available.saturating_sub(size) / 2,
        Align::End => available.saturating_sub(size),
    }
}

pub(super) fn element_corner_radii(element: &Rect) -> CornerRadius {
    if element.style.corner_radii.is_zero() {
        CornerRadius::all(element.style.corner_radius)
    } else {
        element.style.corner_radii
    }
}

pub(super) fn content_clip_radii(element: &Rect) -> CornerRadius {
    element_corner_radii(element).inset(element.padding)
}

pub(super) fn fill_rounded_rect(
    canvas: &mut Canvas<'_>,
    rect: Bounds,
    clip: Clip,
    opacity: f32,
    radii: CornerRadius,
    anti_alias: AntiAlias,
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

pub(super) fn stroke_rounded_rect(
    canvas: &mut Canvas<'_>,
    rect: Bounds,
    clip: Clip,
    opacity: f32,
    widths: BorderWidth,
    radii: CornerRadius,
    anti_alias: AntiAlias,
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

pub(super) fn draw_text(
    canvas: &mut Canvas<'_>,
    fonts: &mut FontCtx,
    rect: Bounds,
    clip: Clip,
    opacity: f32,
    text: &Text,
    offset: PaintOffset,
) {
    draw_text_content(
        canvas,
        fonts,
        rect,
        clip,
        opacity,
        TextContent::Plain(text),
        offset,
    );
}

pub(super) fn draw_rich_text(
    canvas: &mut Canvas<'_>,
    fonts: &mut FontCtx,
    rect: Bounds,
    clip: Clip,
    opacity: f32,
    text: &RichText,
    offset: PaintOffset,
) {
    draw_text_content(
        canvas,
        fonts,
        rect,
        clip,
        opacity,
        TextContent::Rich(text),
        offset,
    );
}

fn draw_text_content(
    canvas: &mut Canvas<'_>,
    fonts: &mut FontCtx,
    rect: Bounds,
    clip: Clip,
    opacity: f32,
    text: TextContent<'_>,
    offset: PaintOffset,
) {
    if text.is_empty()
        || rect.width == 0
        || rect.height == 0
        || rect.intersect(clip.bounds()).is_none()
        || !opacity_draws(opacity)
    {
        return;
    }

    let FontCtx {
        font_system,
        swash_cache,
        text_buffer,
        text_runs,
        glyphs,
    } = fonts;
    let font_size = text.default_style().size.max(1) as f32;
    let metrics = Metrics::new(font_size, font_size * 1.3);
    let attrs = attrs_for_style(text.default_style(), None, 0);
    split_text_runs_into(text, text_runs);
    text_buffer.set_metrics(metrics);
    text_buffer.set_size(Some(rect.width as f32), Some(rect.height as f32));
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
    draw_text_buffer(
        canvas,
        font_system,
        swash_cache,
        glyphs,
        text_buffer,
        rect,
        clip,
        opacity,
        text,
        text_runs,
        offset,
    );
    text_runs.clear();
}

pub(super) fn attrs_for_style<'a>(
    style: &'a TextStyle,
    family_override: Option<&'a str>,
    metadata: usize,
) -> Attrs<'a> {
    let uses_override = family_override.is_some();
    let mut attrs = Attrs::new()
        .color(cosmic_color(style.color.first()))
        .metadata(metadata)
        .metrics(Metrics::new(
            style.size.max(1) as f32,
            style.size.max(1) as f32 * 1.3,
        ))
        .weight(if style.bold && !uses_override {
            Weight::BOLD
        } else {
            Weight::NORMAL
        })
        .style(if style.italic && !uses_override {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        });

    if let Some(family) = family_override.or(style.family.as_deref()) {
        attrs = attrs.family(Family::Name(family));
    }
    if style.underline {
        attrs = attrs.underline(UnderlineStyle::Single);
    }
    if style.strikethrough {
        attrs = attrs.strikethrough();
    }
    if style.overline {
        attrs = attrs.overline();
    }

    attrs
}

fn cosmic_color(color: Color) -> CosmicColor {
    CosmicColor::rgba(color.red, color.green, color.blue, color.alpha)
}

fn split_text_runs_into(text: TextContent<'_>, result: &mut Vec<TextBufferRun>) {
    result.clear();
    match text {
        TextContent::Plain(text) => {
            append_split_text_runs(&text.content, &text.style, &text.emoji_family, result)
        }
        TextContent::Rich(text) => {
            for run in text.runs.iter() {
                append_split_text_runs(&run.content, &run.style, &text.emoji_family, result);
            }
        }
    }
}

fn append_split_text_runs(
    text: &Arc<str>,
    style: &TextStyle,
    emoji_family: &Option<String>,
    result: &mut Vec<TextBufferRun>,
) {
    let mut start = 0;
    let mut is_emoji = false;

    for (index, character) in text.char_indices() {
        let next_is_emoji =
            is_emoji_character(character) || (is_emoji && is_emoji_sequence_character(character));
        if index != start && next_is_emoji != is_emoji {
            push_text_buffer_run(result, text, start, index, is_emoji, style, emoji_family);
            start = index;
        }
        is_emoji = next_is_emoji;
    }

    if start != text.len() {
        push_text_buffer_run(
            result,
            text,
            start,
            text.len(),
            is_emoji,
            style,
            emoji_family,
        );
    }
}

fn push_text_buffer_run(
    result: &mut Vec<TextBufferRun>,
    source: &Arc<str>,
    start: usize,
    end: usize,
    is_emoji: bool,
    style: &TextStyle,
    emoji_family: &Option<String>,
) {
    result.push(TextBufferRun {
        source: Arc::clone(source),
        byte_run: ByteRun {
            start,
            end,
            is_emoji,
        },
        style: style.clone(),
        emoji_family: emoji_family.clone(),
    });
}

fn is_emoji_character(character: char) -> bool {
    matches!(
        character as u32,
        0x00A9
            | 0x00AE
            | 0x203C
            | 0x2049
            | 0x2122
            | 0x2139
            | 0x2194..=0x21FF
            | 0x2300..=0x23FF
            | 0x2600..=0x27BF
            | 0x2934..=0x2935
            | 0x2B05..=0x2B55
            | 0x3030
            | 0x303D
            | 0x3297
            | 0x3299
            | 0x1F000..=0x1FAFF
    )
}

fn is_emoji_sequence_character(character: char) -> bool {
    matches!(character, '\u{200D}' | '\u{20E3}' | '\u{FE0E}' | '\u{FE0F}')
}

fn draw_text_buffer(
    canvas: &mut Canvas<'_>,
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
    glyphs: &mut Vec<GlyphPaint>,
    buffer: &mut Buffer,
    rect: Bounds,
    clip: Clip,
    opacity: f32,
    text: TextContent<'_>,
    runs: &[TextBufferRun],
    offset: PaintOffset,
) {
    glyphs.clear();
    buffer.shape_until_scroll(font_system, false);
    let mut text_height = 0_i32;
    for (line_index, run) in buffer.layout_runs().enumerate() {
        if text
            .max_lines()
            .is_some_and(|max_lines| line_index as u32 >= max_lines)
        {
            break;
        }
        text_height = text_height.max((run.line_top + run.line_height).ceil() as i32);
        for glyph in run.glyphs {
            let physical = glyph.physical((0.0, run.line_y), 1.0);
            let style = style_for_metadata(glyph.metadata, runs, text.default_style());
            let color = glyph
                .color_opt
                .unwrap_or_else(|| cosmic_color(style.color.first()));
            glyphs.push((physical, glyph.metadata, color));
        }
    }

    let align_x = text_horizontal_align_offset(
        text.align(),
        rect.width,
        text_pixel_bounds(font_system, swash_cache, glyphs, runs, text),
    );
    let align_y = text_line_offset(text.vertical_align(), rect.height, text_height);
    let x_offset = i32::try_from(rect.x).unwrap_or(i32::MAX);
    let y_offset = i32::try_from(rect.y).unwrap_or(i32::MAX);
    let Some(draw) = rect
        .intersect(clip.bounds())
        .and_then(|rect| visible_draw_bounds(canvas, rect, offset))
    else {
        return;
    };
    let min_x = i32::try_from(draw.x).unwrap_or(i32::MAX);
    let min_y = i32::try_from(draw.y).unwrap_or(i32::MAX);
    let max_x = i32::try_from(draw.right()).unwrap_or(i32::MAX);
    let max_y = i32::try_from(draw.bottom()).unwrap_or(i32::MAX);

    for (glyph, metadata, base_color) in glyphs.iter() {
        let style = style_for_metadata(*metadata, runs, text.default_style());
        let color = *base_color;
        swash_cache.with_pixels(font_system, glyph.cache_key, color, |x, y, color| {
            let local_x = glyph.x.saturating_add(x).saturating_add(align_x);
            let local_y = glyph.y.saturating_add(y).saturating_add(align_y);
            let Some(target_x) = local_x.checked_add(x_offset) else {
                return;
            };
            let Some(target_y) = local_y.checked_add(y_offset) else {
                return;
            };
            let world_x = target_x;
            let world_y = target_y;
            if world_x < min_x || world_y < min_y || world_x >= max_x || world_y >= max_y {
                return;
            }
            if world_x < 0 || world_y < 0 {
                return;
            }
            let world_x = world_x as u32;
            let world_y = world_y as u32;
            let coverage = clip.coverage(world_x, world_y);
            if coverage == 0 {
                return;
            }

            let Some(local_x) = world_x.checked_sub(rect.x) else {
                return;
            };
            let Some(local_y) = world_y.checked_sub(rect.y) else {
                return;
            };
            let color = text_pixel_color(color, style, local_x, local_y, rect.width, rect.height);
            let Some(target_x) = target_coord(world_x, offset.x, canvas.width()) else {
                return;
            };
            let Some(target_y) = target_coord(world_y, offset.y, canvas.height()) else {
                return;
            };
            canvas.blend_pixel(
                target_x,
                target_y,
                color_with_opacity_and_coverage(color, opacity, coverage),
            );
        });
    }

    for (line_index, run) in buffer.layout_runs().enumerate() {
        if text
            .max_lines()
            .is_some_and(|max_lines| line_index as u32 >= max_lines)
        {
            break;
        }
        draw_text_decorations(
            canvas,
            &run,
            rect,
            clip,
            opacity,
            align_x,
            align_y,
            x_offset,
            y_offset,
            runs,
            text.default_style(),
            offset,
        );
    }
    glyphs.clear();
}

fn draw_text_decorations(
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
pub(super) struct TextPixelBounds {
    pub(super) min_x: i32,
    pub(super) max_x: i32,
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

pub(super) fn text_pixel_bounds(
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

fn style_for_metadata<'a>(
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

pub(super) fn text_horizontal_align_offset(
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

pub(super) fn text_line_offset(align: Align, available: u32, line_height: i32) -> i32 {
    let available = i32::try_from(available).unwrap_or(i32::MAX);
    match align {
        Align::Start | Align::Stretch => 0,
        Align::Center => available.saturating_sub(line_height) / 2,
        Align::End => available.saturating_sub(line_height),
    }
}

fn text_pixel_color(
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
