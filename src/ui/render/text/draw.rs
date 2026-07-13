use super::*;

pub(in crate::ui::render) fn draw_text(
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

pub(in crate::ui::render) fn draw_rich_text(
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

pub(in crate::ui::render) fn attrs_for_style<'a>(
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

pub(in crate::ui::render) fn cosmic_color(color: Color) -> CosmicColor {
    CosmicColor::rgba(color.red, color.green, color.blue, color.alpha)
}

pub(in crate::ui::render) fn split_text_runs_into(
    text: TextContent<'_>,
    result: &mut Vec<TextBufferRun>,
) {
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
