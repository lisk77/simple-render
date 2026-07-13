use super::*;
use crate::{ClickEvent, KeyboardEvent, Listener};

mod builder;
mod composition;
mod rendering;
mod styling;

#[derive(Clone)]
pub struct Rect {
    pub(super) id: Option<WidgetId>,
    pub(super) layer_options: LayerOptions,
    pub(super) width: Length,
    pub(super) height: Length,
    pub(super) min_width: u32,
    pub(super) min_height: u32,
    pub(super) max_width: Option<u32>,
    pub(super) max_height: Option<u32>,
    pub(super) fill: u32,
    pub(super) direction: Direction,
    pub(super) align: Align,
    pub(super) justify: Align,
    pub(super) overflow: Overflow,
    pub(super) position: Position,
    pub(super) inset: Inset,
    pub(super) padding: Spacing,
    pub(super) gap: u32,
    pub(super) style: Style,
    pub(super) content: Option<Content>,
    pub(super) children: Vec<Rect>,
    pub(super) on_click: Option<Listener<ClickEvent>>,
    pub(super) on_drag: Option<Listener<ClickEvent>>,
    pub(super) on_key: Option<Listener<KeyboardEvent>>,
    pub(super) interaction_styles: Option<(Style, Style, Style)>,
    pub(super) interaction_style_child: Option<usize>,
}

pub type Ui = Rect;

#[derive(Clone, Copy, Debug)]
pub(in crate::ui::render) struct VisualState {
    scale: f32,
    translate_x: f32,
    translate_y: f32,
}

impl VisualState {
    pub(in crate::ui::render) const IDENTITY: Self = Self {
        scale: 1.0,
        translate_x: 0.0,
        translate_y: 0.0,
    };

    fn then_element(self, transform: PaintTransform, origin: Bounds) -> Self {
        let scale = sanitize_scale(transform.scale);
        let origin_x = origin.x as f32;
        let origin_y = origin.y as f32;
        let local_translate_x = origin_x + transform.translate_x as f32 - origin_x * scale;
        let local_translate_y = origin_y + transform.translate_y as f32 - origin_y * scale;

        Self {
            scale: self.scale * scale,
            translate_x: self.translate_x + self.scale * local_translate_x,
            translate_y: self.translate_y + self.scale * local_translate_y,
        }
    }

    fn bounds(self, bounds: Bounds) -> Option<Bounds> {
        if self.scale <= 0.0 {
            return None;
        }

        let left = (bounds.x as f32 * self.scale + self.translate_x).floor();
        let top = (bounds.y as f32 * self.scale + self.translate_y).floor();
        let right = (bounds.right() as f32 * self.scale + self.translate_x).ceil();
        let bottom = (bounds.bottom() as f32 * self.scale + self.translate_y).ceil();
        signed_bounds(left, top, right, bottom)
    }

    fn radii(self, radii: CornerRadius) -> CornerRadius {
        scale_corner_radius(radii, self.scale)
    }

    fn border_widths(self, widths: BorderWidth) -> BorderWidth {
        scale_border_widths(widths, self.scale)
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            id: None,
            layer_options: LayerOptions::default(),
            width: Length::Fit,
            height: Length::Fit,
            min_width: 0,
            min_height: 0,
            max_width: None,
            max_height: None,
            fill: 1,
            direction: Direction::Row,
            align: Align::Start,
            justify: Align::Start,
            overflow: Overflow::Clip,
            position: Position::Flow,
            inset: Inset::ZERO,
            padding: Spacing::ZERO,
            gap: 0,
            style: Style::default(),
            content: None,
            children: Vec::new(),
            on_click: None,
            on_drag: None,
            on_key: None,
            interaction_styles: None,
            interaction_style_child: None,
        }
    }
}

impl Rect {
    pub(in crate::ui::render) fn visit_layout(
        element: &Rect,
        bounds: Bounds,
        clip: Clip,
        state: VisualState,
        opacity: f32,
        premeasured: Option<Size>,
        fonts: &mut FontCtx,
        visit: &mut dyn FnMut(PaintCommand<'_>, &mut FontCtx),
    ) {
        let measured = if element_needs_measure(element) {
            premeasured
                .unwrap_or_else(|| measure_element(element, fonts, bounds.width, bounds.height))
        } else {
            Size::default()
        };
        let width = resolve_length(
            element.width,
            bounds.width,
            measured.width,
            element.min_width,
            element.max_width,
        );
        let height = resolve_length(
            element.height,
            bounds.height,
            measured.height,
            element.min_height,
            element.max_height,
        );
        let rect = Bounds {
            x: bounds.x,
            y: bounds.y,
            width,
            height,
        };
        let state = state.then_element(element.style.transform, rect);
        let visual_rect = state.bounds(rect);
        let own_clip = visual_rect.and_then(|rect| clip.intersect_bounds(rect));
        let opacity = multiply_opacity(opacity, element.style.opacity);
        let radii = state.radii(element_corner_radii(element));

        if let (Some(rect), Some(clip), Some(paint)) =
            (visual_rect, own_clip, element.style.background.as_ref())
        {
            visit(
                PaintCommand::Rect {
                    rect,
                    clip,
                    opacity,
                    paint,
                    gradient: element.style.gradient,
                    radii,
                    anti_alias: element.style.anti_alias,
                },
                fonts,
            );
        }
        if let (Some(rect), Some(clip), Some(border)) =
            (visual_rect, own_clip, element.style.border.as_ref())
        {
            visit(
                PaintCommand::Border {
                    rect,
                    clip,
                    opacity,
                    paint: &border.color,
                    gradient: border.gradient,
                    widths: state.border_widths(border_widths(border)),
                    radii,
                    anti_alias: element.style.anti_alias,
                },
                fonts,
            );
        }
        let content_rect = rect.inset(element.padding);
        let visual_content_rect = state.bounds(content_rect);
        if let (Some(rect), Some(clip), Some(content)) =
            (visual_content_rect, own_clip, element.content.as_ref())
        {
            match content {
                Content::Text(text) => visit(
                    PaintCommand::Text {
                        rect,
                        clip,
                        opacity,
                        scale: state.scale,
                        text,
                    },
                    fonts,
                ),
                Content::RichText(text) => visit(
                    PaintCommand::RichText {
                        rect,
                        clip,
                        opacity,
                        scale: state.scale,
                        text,
                    },
                    fonts,
                ),
                Content::Image(image) => visit(
                    PaintCommand::Image {
                        rect,
                        clip,
                        opacity,
                        image,
                    },
                    fonts,
                ),
            }
        }

        if element.children.is_empty() {
            return;
        }

        let child_clip = match element.overflow {
            Overflow::Clip => {
                let (Some(clip), Some(content_rect)) = (own_clip, visual_content_rect) else {
                    return;
                };
                let Some(child_clip) =
                    clip.with_rounded_rect(content_rect, state.radii(content_clip_radii(element)))
                else {
                    return;
                };
                child_clip
            }
            Overflow::Visible => clip,
        };
        layout_children(
            element,
            content_rect,
            fonts,
            |_, child, rect, measured, fonts| {
                Self::visit_layout(
                    child,
                    rect,
                    child_clip,
                    state,
                    opacity,
                    Some(measured),
                    fonts,
                    visit,
                );
            },
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn hit_test_layout(
        element: &Rect,
        bounds: Bounds,
        clip: Clip,
        state: VisualState,
        premeasured: Option<Size>,
        inherited_id: Option<WidgetId>,
        inherited_id_bounds: Option<Bounds>,
        fonts: &mut FontCtx,
        x: u32,
        y: u32,
        current_path: &mut Vec<usize>,
        hit_path: &mut Vec<usize>,
        hit_id: &mut Option<WidgetId>,
    ) -> Option<Bounds> {
        let measured = if element_needs_measure(element) {
            premeasured
                .unwrap_or_else(|| measure_element(element, fonts, bounds.width, bounds.height))
        } else {
            Size::default()
        };
        let width = resolve_length(
            element.width,
            bounds.width,
            measured.width,
            element.min_width,
            element.max_width,
        );
        let height = resolve_length(
            element.height,
            bounds.height,
            measured.height,
            element.min_height,
            element.max_height,
        );
        let rect = Bounds {
            x: bounds.x,
            y: bounds.y,
            width,
            height,
        };
        let state = state.then_element(element.style.transform, rect);
        let visual_rect = state.bounds(rect)?;
        let clip = clip.intersect_bounds(visual_rect)?;
        let content_rect = rect.inset(element.padding);
        let visual_content_rect = state.bounds(content_rect)?;
        let has_own_id = element.id.is_some();
        let current_id = element.id.clone().or(inherited_id);
        let current_id_bounds = if has_own_id {
            Some(visual_rect)
        } else {
            inherited_id_bounds
        };
        let child_clip = match element.overflow {
            Overflow::Clip => clip.with_rounded_rect(
                visual_content_rect,
                state.radii(content_clip_radii(element)),
            )?,
            Overflow::Visible => clip,
        };

        let mut hit = None;
        layout_children(
            element,
            content_rect,
            fonts,
            |index, child, rect, measured, fonts| {
                current_path.push(index);
                if let Some(bounds) = Self::hit_test_layout(
                    child,
                    rect,
                    child_clip,
                    state,
                    Some(measured),
                    current_id.clone(),
                    current_id_bounds,
                    fonts,
                    x,
                    y,
                    current_path,
                    hit_path,
                    hit_id,
                ) {
                    hit = Some(bounds);
                }
                current_path.pop();
            },
        );

        if let Some(bounds) = hit {
            return Some(bounds);
        }

        let hit_clip =
            clip.with_rounded_rect(visual_rect, state.radii(element_corner_radii(element)))?;
        if hit_clip.contains(x, y) {
            hit_path.clear();
            hit_path.extend_from_slice(current_path);
            *hit_id = current_id;
            Some(current_id_bounds.unwrap_or(visual_rect))
        } else {
            None
        }
    }
}

fn signed_bounds(left: f32, top: f32, right: f32, bottom: f32) -> Option<Bounds> {
    if !left.is_finite()
        || !top.is_finite()
        || !right.is_finite()
        || !bottom.is_finite()
        || right <= left
        || bottom <= top
    {
        return None;
    }

    let left = left.max(0.0).min(u32::MAX as f32) as u32;
    let top = top.max(0.0).min(u32::MAX as f32) as u32;
    let right = right.max(0.0).min(u32::MAX as f32) as u32;
    let bottom = bottom.max(0.0).min(u32::MAX as f32) as u32;
    let width = right.saturating_sub(left);
    let height = bottom.saturating_sub(top);
    (width > 0 && height > 0).then_some(Bounds {
        x: left,
        y: top,
        width,
        height,
    })
}

fn scale_corner_radius(radii: CornerRadius, scale: f32) -> CornerRadius {
    CornerRadius {
        top_left: scale_u32_f32(radii.top_left, scale),
        top_right: scale_u32_f32(radii.top_right, scale),
        bottom_right: scale_u32_f32(radii.bottom_right, scale),
        bottom_left: scale_u32_f32(radii.bottom_left, scale),
    }
}

fn scale_border_widths(widths: BorderWidth, scale: f32) -> BorderWidth {
    BorderWidth {
        top: scale_u32_f32(widths.top, scale),
        right: scale_u32_f32(widths.right, scale),
        bottom: scale_u32_f32(widths.bottom, scale),
        left: scale_u32_f32(widths.left, scale),
    }
}

fn scale_u32_f32(value: u32, scale: f32) -> u32 {
    let scaled = f64::from(value) * f64::from(scale);
    if !scaled.is_finite() || scaled <= 0.0 {
        0
    } else {
        scaled.round().min(f64::from(u32::MAX)) as u32
    }
}

fn hit_point(x: f64, y: f64) -> Option<(u32, u32)> {
    if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
        return None;
    }
    let x = x.floor();
    let y = y.floor();
    if x > u32::MAX as f64 || y > u32::MAX as f64 {
        return None;
    }
    Some((x as u32, y as u32))
}
