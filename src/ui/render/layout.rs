use super::*;

pub(super) fn measure_element(
    element: &Rect,
    fonts: &mut FontCtx,
    available_width: u32,
    available_height: u32,
) -> Size {
    let content_available =
        Bounds::new(0, 0, available_width, available_height).inset(element.padding);
    let content_size = match &element.content {
        Some(Content::Text(text)) => {
            measure_text(fonts, TextContent::Plain(text), content_available.width)
        }
        Some(Content::RichText(text)) => {
            measure_text(fonts, TextContent::Rich(text), content_available.width)
        }
        Some(Content::Image(image)) => {
            measure_image(image, content_available.width, content_available.height)
        }
        None => Size::default(),
    };
    let child_size = measure_children(
        element,
        fonts,
        content_available.width,
        content_available.height,
    );
    let measured = Size::new(
        content_size.width.max(child_size.width),
        content_size.height.max(child_size.height),
    );
    Size::new(
        constrain_dimension(
            measured
                .width
                .saturating_add(element.padding.left)
                .saturating_add(element.padding.right)
                .min(available_width),
            element.min_width,
            element.max_width,
        ),
        constrain_dimension(
            measured
                .height
                .saturating_add(element.padding.top)
                .saturating_add(element.padding.bottom)
                .min(available_height),
            element.min_height,
            element.max_height,
        ),
    )
}

pub(super) fn measure_children(
    parent: &Rect,
    fonts: &mut FontCtx,
    available_width: u32,
    available_height: u32,
) -> Size {
    if parent
        .children
        .iter()
        .all(|child| child.position != Position::Flow)
    {
        return Size::default();
    }

    let direction = parent.direction;
    let axis_available = match direction {
        Direction::Row => available_width,
        Direction::Column => available_height,
    };
    let cross_available = match direction {
        Direction::Row => available_height,
        Direction::Column => available_width,
    };
    let flow_count = parent
        .children
        .iter()
        .filter(|child| child.position == Position::Flow)
        .count();
    let gap_total = parent
        .gap
        .saturating_mul(flow_count.saturating_sub(1) as u32);
    let axis_available_without_gaps = axis_available.saturating_sub(gap_total);
    let mut axis_used = 0_u32;
    let mut cross_used = 0_u32;

    for child in parent
        .children
        .iter()
        .filter(|child| child.position == Position::Flow)
    {
        let measured = measure_element(child, fonts, available_width, available_height);
        let axis = match axis_length(child, direction) {
            Length::Fill => constrain_axis(measured.axis(direction), child, direction),
            _ => resolve_axis_length(
                child,
                direction,
                axis_available_without_gaps,
                measured.axis(direction),
            ),
        };
        let cross = match cross_length(child, direction) {
            Length::Fill => constrain_cross(measured.cross(direction), child, direction),
            _ => resolve_cross_length(child, direction, cross_available, measured.cross(direction)),
        };
        axis_used = axis_used.saturating_add(axis);
        cross_used = cross_used.max(cross);
    }

    axis_used = axis_used.saturating_add(gap_total).min(axis_available);
    match direction {
        Direction::Row => Size::new(axis_used, cross_used.min(cross_available)),
        Direction::Column => Size::new(cross_used.min(cross_available), axis_used),
    }
}

pub(super) fn layout_children(
    parent: &Rect,
    content: Bounds,
    fonts: &mut FontCtx,
    mut layout_child: impl FnMut(usize, &Rect, Bounds, Size, &mut FontCtx),
) {
    let count = parent
        .children
        .iter()
        .filter(|child| child.position == Position::Flow)
        .count();
    if count == 0 {
        layout_absolute_children(parent, content, fonts, layout_child);
        return;
    }

    let gap_total = parent.gap.saturating_mul(count.saturating_sub(1) as u32);
    let axis_total = match parent.direction {
        Direction::Row => content.width,
        Direction::Column => content.height,
    };
    let axis_available = axis_total.saturating_sub(gap_total);
    let cross_available = match parent.direction {
        Direction::Row => content.height,
        Direction::Column => content.width,
    };

    let mut fixed_axis = 0_u32;
    let mut fill_weight = 0_u32;
    for child in parent
        .children
        .iter()
        .filter(|child| child.position == Position::Flow)
    {
        match axis_length(child, parent.direction) {
            Length::Fill => {
                fill_weight = fill_weight.saturating_add(child.fill.max(1));
            }
            _ => {
                let measured = measure_element(child, fonts, content.width, content.height);
                let axis = resolve_axis_length(
                    child,
                    parent.direction,
                    axis_available,
                    measured.axis(parent.direction),
                );
                fixed_axis = fixed_axis.saturating_add(axis);
            }
        }
    }

    let remaining = axis_available.saturating_sub(fixed_axis);
    let mut distributed = 0_u32;
    let mut remaining_weight = fill_weight;
    let mut used_axis = gap_total;
    for child in parent
        .children
        .iter()
        .filter(|child| child.position == Position::Flow)
    {
        let axis = match axis_length(child, parent.direction) {
            Length::Fill => {
                let weight = child.fill.max(1);
                let share = if remaining_weight <= weight {
                    remaining.saturating_sub(distributed)
                } else {
                    remaining.saturating_sub(distributed) * weight / remaining_weight
                };
                remaining_weight = remaining_weight.saturating_sub(weight);
                distributed = distributed.saturating_add(share);
                constrain_axis(share, child, parent.direction)
            }
            _ => {
                let measured = measure_element(child, fonts, content.width, content.height);
                resolve_axis_length(
                    child,
                    parent.direction,
                    axis_available,
                    measured.axis(parent.direction),
                )
            }
        };
        used_axis = used_axis.saturating_add(axis);
    }

    let start_offset = align_offset(parent.justify, axis_total, used_axis);
    let mut cursor = match parent.direction {
        Direction::Row => content.x.saturating_add(start_offset),
        Direction::Column => content.y.saturating_add(start_offset),
    };

    let mut distributed = 0_u32;
    let mut remaining_weight = fill_weight;
    for (index, child) in parent
        .children
        .iter()
        .enumerate()
        .filter(|(_, child)| child.position == Position::Flow)
    {
        let measured = measure_element(child, fonts, content.width, content.height);
        let axis = match axis_length(child, parent.direction) {
            Length::Fill => {
                let weight = child.fill.max(1);
                let share = if remaining_weight <= weight {
                    remaining.saturating_sub(distributed)
                } else {
                    remaining.saturating_sub(distributed) * weight / remaining_weight
                };
                remaining_weight = remaining_weight.saturating_sub(weight);
                distributed = distributed.saturating_add(share);
                constrain_axis(share, child, parent.direction)
            }
            _ => resolve_axis_length(
                child,
                parent.direction,
                axis_available,
                measured.axis(parent.direction),
            ),
        };
        if axis == 0 {
            continue;
        };
        let cross = match cross_length(child, parent.direction) {
            Length::Fill => cross_available,
            Length::Fit if parent.align == Align::Stretch => cross_available,
            _ => resolve_cross_length(
                child,
                parent.direction,
                cross_available,
                measured.cross(parent.direction),
            ),
        };
        let cross_offset = align_offset(parent.align, cross_available, cross);
        let rect = match parent.direction {
            Direction::Row => Bounds {
                x: cursor,
                y: content.y.saturating_add(cross_offset),
                width: axis,
                height: cross,
            },
            Direction::Column => Bounds {
                x: content.x.saturating_add(cross_offset),
                y: cursor,
                width: cross,
                height: axis,
            },
        };
        cursor = cursor.saturating_add(axis).saturating_add(parent.gap);
        layout_child(index, child, rect, measured, fonts);
    }

    layout_absolute_children(parent, content, fonts, layout_child);
}

pub(super) fn layout_absolute_children(
    parent: &Rect,
    content: Bounds,
    fonts: &mut FontCtx,
    mut layout_child: impl FnMut(usize, &Rect, Bounds, Size, &mut FontCtx),
) {
    for (index, child) in parent
        .children
        .iter()
        .enumerate()
        .filter(|(_, child)| child.position == Position::Absolute)
    {
        let (rect, measured) = absolute_child_rect(child, content, fonts);
        if rect.width == 0 || rect.height == 0 {
            continue;
        }
        layout_child(index, child, rect, measured, fonts);
    }
}

pub(super) fn absolute_child_rect(
    child: &Rect,
    content: Bounds,
    fonts: &mut FontCtx,
) -> (Bounds, Size) {
    let left = child.inset.left.unwrap_or(0);
    let right = child.inset.right.unwrap_or(0);
    let top = child.inset.top.unwrap_or(0);
    let bottom = child.inset.bottom.unwrap_or(0);
    let available_width = content.width.saturating_sub(left).saturating_sub(right);
    let available_height = content.height.saturating_sub(top).saturating_sub(bottom);
    let measured = measure_element(child, fonts, available_width, available_height);
    let width = if child.inset.left.is_some()
        && child.inset.right.is_some()
        && matches!(child.width, Length::Fill)
    {
        constrain_dimension(available_width, child.min_width, child.max_width)
    } else {
        resolve_length(
            child.width,
            available_width,
            measured.width,
            child.min_width,
            child.max_width,
        )
    };
    let height = if child.inset.top.is_some()
        && child.inset.bottom.is_some()
        && matches!(child.height, Length::Fill)
    {
        constrain_dimension(available_height, child.min_height, child.max_height)
    } else {
        resolve_length(
            child.height,
            available_height,
            measured.height,
            child.min_height,
            child.max_height,
        )
    };
    let x = match (child.inset.left, child.inset.right) {
        (Some(left), _) => content.x.saturating_add(left),
        (None, Some(right)) => content.right().saturating_sub(right).saturating_sub(width),
        (None, None) => content.x,
    };
    let y = match (child.inset.top, child.inset.bottom) {
        (Some(top), _) => content.y.saturating_add(top),
        (None, Some(bottom)) => content
            .bottom()
            .saturating_sub(bottom)
            .saturating_sub(height),
        (None, None) => content.y,
    };

    (
        Bounds {
            x,
            y,
            width,
            height,
        },
        measured,
    )
}

pub(super) fn axis_length(element: &Rect, direction: Direction) -> Length {
    match direction {
        Direction::Row => element.width,
        Direction::Column => element.height,
    }
}

pub(super) fn cross_length(element: &Rect, direction: Direction) -> Length {
    match direction {
        Direction::Row => element.height,
        Direction::Column => element.width,
    }
}

pub(super) fn resolve_axis_length(
    element: &Rect,
    direction: Direction,
    available: u32,
    fit: u32,
) -> u32 {
    let (min, max) = match direction {
        Direction::Row => (element.min_width, element.max_width),
        Direction::Column => (element.min_height, element.max_height),
    };
    resolve_length(axis_length(element, direction), available, fit, min, max)
}

pub(super) fn resolve_cross_length(
    element: &Rect,
    direction: Direction,
    available: u32,
    fit: u32,
) -> u32 {
    let (min, max) = match direction {
        Direction::Row => (element.min_height, element.max_height),
        Direction::Column => (element.min_width, element.max_width),
    };
    resolve_length(cross_length(element, direction), available, fit, min, max)
}

pub(super) fn constrain_axis(size: u32, element: &Rect, direction: Direction) -> u32 {
    let (min, max) = match direction {
        Direction::Row => (element.min_width, element.max_width),
        Direction::Column => (element.min_height, element.max_height),
    };
    constrain_dimension(size, min, max)
}

pub(super) fn constrain_cross(size: u32, element: &Rect, direction: Direction) -> u32 {
    let (min, max) = match direction {
        Direction::Row => (element.min_height, element.max_height),
        Direction::Column => (element.min_width, element.max_width),
    };
    constrain_dimension(size, min, max)
}

pub(super) fn resolve_length(
    length: Length,
    available: u32,
    fit: u32,
    min: u32,
    max: Option<u32>,
) -> u32 {
    let resolved = match length {
        Length::Fit => fit,
        Length::Fill => available,
        Length::Px(value) => value.min(available),
        Length::Percent(percent) => {
            ((available as f32 * percent.clamp(0.0, 100.0) / 100.0).round() as u32).min(available)
        }
    };
    constrain_dimension(resolved, min, max)
}

pub(super) fn constrain_dimension(size: u32, min: u32, max: Option<u32>) -> u32 {
    let max = max.unwrap_or(u32::MAX).max(min);
    size.max(min).min(max)
}
