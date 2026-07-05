use super::*;

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
}

pub type Ui = Rect;

#[derive(Clone, Copy, Debug)]
struct VisualState {
    scale: f32,
    translate_x: f32,
    translate_y: f32,
}

impl VisualState {
    const IDENTITY: Self = Self {
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
        }
    }
}

impl Rect {
    pub fn new(layout: RectLayout) -> Self {
        Self::default().with_layout(layout)
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn layout(layout: RectLayout) -> Self {
        Self::new(layout)
    }

    pub fn styled(style: RectStyle) -> Self {
        Self::empty().style(style)
    }

    pub fn with_layout(mut self, layout: RectLayout) -> Self {
        self.id = layout.id;
        if let Some(surface) = layout.surface {
            self.layer_options = surface.into();
        }
        self.width = layout.width;
        self.height = layout.height;
        self.min_width = layout.min_width;
        self.min_height = layout.min_height;
        self.max_width = layout.max_width;
        self.max_height = layout.max_height;
        self.fill = layout.fill;
        self.direction = layout.direction;
        self.align = layout.align;
        self.justify = layout.justify;
        self.overflow = layout.overflow;
        self.position = layout.position;
        self.inset = layout.inset;
        self.padding = layout.padding;
        self.gap = layout.gap;
        self.style = layout.style;
        if !layout.transform.is_identity() {
            self.style.transform = layout.transform;
        }
        self.content = layout.content;
        self
    }

    pub fn style(mut self, style: RectStyle) -> Self {
        self.style = style;
        self
    }

    pub fn id(mut self, id: impl Into<WidgetId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    pub fn size(mut self, width: Length, height: Length) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn min_width(mut self, min_width: u32) -> Self {
        self.min_width = min_width;
        self
    }

    pub fn min_height(mut self, min_height: u32) -> Self {
        self.min_height = min_height;
        self
    }

    pub fn max_width(mut self, max_width: impl Into<Option<u32>>) -> Self {
        self.max_width = max_width.into();
        self
    }

    pub fn max_height(mut self, max_height: impl Into<Option<u32>>) -> Self {
        self.max_height = max_height.into();
        self
    }

    pub fn fill(mut self, fill: u32) -> Self {
        self.fill = fill;
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn justify(mut self, justify: Align) -> Self {
        self.justify = justify;
        self
    }

    pub fn overflow(mut self, overflow: Overflow) -> Self {
        self.overflow = overflow;
        self
    }

    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    pub fn inset(mut self, inset: Inset) -> Self {
        self.inset = inset;
        self
    }

    pub fn absolute(mut self, inset: Inset) -> Self {
        self.position = Position::Absolute;
        self.inset = inset;
        self
    }

    pub fn padding(mut self, padding: Spacing) -> Self {
        self.padding = padding;
        self
    }

    pub fn gap(mut self, gap: u32) -> Self {
        self.gap = gap;
        self
    }

    pub fn background(mut self, background: impl Into<Paint>) -> Self {
        self.style.background = Some(background.into());
        self
    }

    pub fn border(mut self, border: Border) -> Self {
        self.style.border = Some(border);
        self
    }

    pub fn corner_radius(mut self, radius: u32) -> Self {
        self.style.corner_radius = radius;
        self
    }

    pub fn corner_radii(mut self, radii: CornerRadius) -> Self {
        self.style.corner_radii = radii;
        self
    }

    pub fn gradient(mut self, gradient: GradientDirection) -> Self {
        self.style.gradient = gradient;
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.style.opacity = opacity;
        self
    }

    pub fn anti_alias(mut self, anti_alias: AntiAlias) -> Self {
        self.style.anti_alias = anti_alias;
        self
    }

    pub fn transform(mut self, transform: PaintTransform) -> Self {
        self.style.transform = transform;
        self
    }

    pub fn translate(mut self, x: i32, y: i32) -> Self {
        self.style.transform.translate_x = x;
        self.style.transform.translate_y = y;
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.style.transform.scale = scale;
        self
    }

    pub fn content(mut self, content: impl Into<Content>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn with_surface(mut self, surface: Surface) -> Self {
        self.layer_options = surface.into();
        self
    }

    pub fn begin(_: Bounds) -> Self {
        Self::default()
    }

    pub fn build(bounds: Bounds, content: impl FnOnce(&mut Self)) -> Self {
        let mut ui = Self::begin(bounds);
        content(&mut ui);
        ui
    }

    pub fn child(mut self, child: Rect) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = Rect>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn draw(self) -> wayland::Result<()> {
        let options = self.layer_options.clone();
        options.show(UiRenderer {
            root: self,
            fonts: LazyFontCtx::new(),
        })
    }

    pub fn draw_with_commands(self, receiver: wayland::RenderReceiver) -> wayland::Result<()> {
        let options = self.layer_options.clone();
        options.show_with_commands(
            UiRenderer {
                root: self,
                fonts: LazyFontCtx::new(),
            },
            receiver,
        )
    }

    pub fn commands(&self, bounds: Bounds) -> Vec<DrawCommand> {
        let mut fonts = FontCtx::new();
        self.commands_with_fonts(bounds, &mut fonts)
    }

    pub fn commands_with_fonts(&self, bounds: Bounds, fonts: &mut FontCtx) -> Vec<DrawCommand> {
        let mut commands = Vec::new();
        Self::visit_layout(
            self,
            bounds,
            Clip::rect(bounds),
            VisualState::IDENTITY,
            1.0,
            None,
            fonts,
            &mut |command, _| {
                commands.push(command.to_owned());
            },
        );
        fonts.trim_scratch();
        commands
    }

    pub fn visual_bounds(&self, bounds: Bounds) -> Option<Bounds> {
        let mut fonts = FontCtx::new();
        self.visual_bounds_with_fonts(bounds, &mut fonts)
    }

    pub fn visual_bounds_with_fonts(&self, bounds: Bounds, fonts: &mut FontCtx) -> Option<Bounds> {
        let mut visual_bounds: Option<Bounds> = None;
        Self::visit_layout(
            self,
            bounds,
            Clip::rect(bounds),
            VisualState::IDENTITY,
            1.0,
            None,
            fonts,
            &mut |command, _| {
                let bounds = command.rect();
                visual_bounds = Some(match visual_bounds {
                    Some(current) => current.union(bounds),
                    None => bounds,
                });
            },
        );
        fonts.trim_scratch();
        visual_bounds
    }

    pub fn measure(&self, available_width: u32, available_height: u32) -> MeasuredSize {
        let mut fonts = FontCtx::new();
        self.measure_with_fonts(available_width, available_height, &mut fonts)
    }

    pub fn measure_with_fonts(
        &self,
        available_width: u32,
        available_height: u32,
        fonts: &mut FontCtx,
    ) -> MeasuredSize {
        let measured = measure_element(self, fonts, available_width, available_height);
        fonts.trim_scratch();
        measured.into()
    }

    pub fn hit_test(&self, bounds: Bounds, x: f64, y: f64) -> Option<Hit> {
        let mut fonts = FontCtx::new();
        self.hit_test_with_fonts(bounds, x, y, &mut fonts)
    }

    pub fn hit_test_with_fonts(
        &self,
        bounds: Bounds,
        x: f64,
        y: f64,
        fonts: &mut FontCtx,
    ) -> Option<Hit> {
        let mut path = Vec::new();
        self.hit_test_detailed_with_fonts(bounds, x, y, fonts, &mut path)
    }

    pub fn hit_test_path(
        &self,
        bounds: Bounds,
        x: f64,
        y: f64,
        path: &mut Vec<usize>,
    ) -> Option<Bounds> {
        let mut fonts = FontCtx::new();
        self.hit_test_path_with_fonts(bounds, x, y, &mut fonts, path)
    }

    pub fn hit_test_path_with_fonts(
        &self,
        bounds: Bounds,
        x: f64,
        y: f64,
        fonts: &mut FontCtx,
        path: &mut Vec<usize>,
    ) -> Option<Bounds> {
        self.hit_test_detailed_with_fonts(bounds, x, y, fonts, path)
            .map(|hit| hit.bounds)
    }

    fn hit_test_detailed_with_fonts(
        &self,
        bounds: Bounds,
        x: f64,
        y: f64,
        fonts: &mut FontCtx,
        path: &mut Vec<usize>,
    ) -> Option<Hit> {
        path.clear();
        let Some((x, y)) = hit_point(x, y) else {
            fonts.trim_scratch();
            return None;
        };

        let mut current_path = Vec::new();
        let mut hit_id = None;
        let bounds = Self::hit_test_layout(
            self,
            bounds,
            Clip::rect(bounds),
            VisualState::IDENTITY,
            None,
            None,
            None,
            fonts,
            x,
            y,
            &mut current_path,
            path,
            &mut hit_id,
        );
        fonts.trim_scratch();
        bounds.map(|bounds| Hit {
            path: path.clone(),
            id: hit_id,
            bounds,
        })
    }

    pub fn paint(&mut self, canvas: &mut Canvas<'_>) {
        let mut fonts = FontCtx::new();
        self.paint_with_fonts(canvas, &mut fonts);
    }

    pub fn paint_with_fonts(&mut self, canvas: &mut Canvas<'_>, fonts: &mut FontCtx) {
        self.paint_scaled_with_fonts(canvas, fonts, canvas.width(), canvas.height(), 1);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint_bgra_with_fonts(
        &mut self,
        pixels: &mut [u8],
        buffer_width: u32,
        buffer_height: u32,
        stride: u32,
        logical_width: u32,
        logical_height: u32,
        scale: u32,
        fonts: &mut FontCtx,
    ) -> Option<DamageRect> {
        self.paint_bgra_viewport_with_fonts(
            pixels,
            buffer_width,
            buffer_height,
            stride,
            logical_width,
            logical_height,
            0,
            0,
            scale,
            fonts,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint_bgra_viewport_with_fonts(
        &mut self,
        pixels: &mut [u8],
        buffer_width: u32,
        buffer_height: u32,
        stride: u32,
        logical_width: u32,
        logical_height: u32,
        viewport_x: i32,
        viewport_y: i32,
        scale: u32,
        fonts: &mut FontCtx,
    ) -> Option<DamageRect> {
        let mut canvas =
            Canvas::from_bgra_pixels(pixels, buffer_width, buffer_height, stride, scale)?;
        self.paint_viewport_with_fonts(
            &mut canvas,
            fonts,
            logical_width,
            logical_height,
            viewport_x,
            viewport_y,
            scale,
        );
        canvas.damage()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint_bgra_transformed_with_fonts(
        &mut self,
        pixels: &mut [u8],
        buffer_width: u32,
        buffer_height: u32,
        stride: u32,
        logical_width: u32,
        logical_height: u32,
        scale: u32,
        transform: PaintTransform,
        fonts: &mut FontCtx,
    ) -> Option<DamageRect> {
        self.paint_bgra_transformed_viewport_with_fonts(
            pixels,
            buffer_width,
            buffer_height,
            stride,
            logical_width,
            logical_height,
            0,
            0,
            scale,
            transform,
            fonts,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint_bgra_transformed_viewport_with_fonts(
        &mut self,
        pixels: &mut [u8],
        buffer_width: u32,
        buffer_height: u32,
        stride: u32,
        logical_width: u32,
        logical_height: u32,
        viewport_x: i32,
        viewport_y: i32,
        scale: u32,
        transform: PaintTransform,
        fonts: &mut FontCtx,
    ) -> Option<DamageRect> {
        let mut canvas =
            Canvas::from_bgra_pixels(pixels, buffer_width, buffer_height, stride, scale)?;
        self.paint_transformed_viewport_with_fonts(
            &mut canvas,
            fonts,
            logical_width,
            logical_height,
            viewport_x,
            viewport_y,
            scale,
            transform,
        );
        canvas.damage()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint_bgra(
        &mut self,
        pixels: &mut [u8],
        buffer_width: u32,
        buffer_height: u32,
        stride: u32,
        logical_width: u32,
        logical_height: u32,
        scale: u32,
    ) -> Option<DamageRect> {
        let mut fonts = FontCtx::new();
        self.paint_bgra_with_fonts(
            pixels,
            buffer_width,
            buffer_height,
            stride,
            logical_width,
            logical_height,
            scale,
            &mut fonts,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint_bgra_transformed(
        &mut self,
        pixels: &mut [u8],
        buffer_width: u32,
        buffer_height: u32,
        stride: u32,
        logical_width: u32,
        logical_height: u32,
        scale: u32,
        transform: PaintTransform,
    ) -> Option<DamageRect> {
        let mut fonts = FontCtx::new();
        self.paint_bgra_transformed_with_fonts(
            pixels,
            buffer_width,
            buffer_height,
            stride,
            logical_width,
            logical_height,
            scale,
            transform,
            &mut fonts,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint_bgra_transformed_viewport(
        &mut self,
        pixels: &mut [u8],
        buffer_width: u32,
        buffer_height: u32,
        stride: u32,
        logical_width: u32,
        logical_height: u32,
        viewport_x: i32,
        viewport_y: i32,
        scale: u32,
        transform: PaintTransform,
    ) -> Option<DamageRect> {
        let mut fonts = FontCtx::new();
        self.paint_bgra_transformed_viewport_with_fonts(
            pixels,
            buffer_width,
            buffer_height,
            stride,
            logical_width,
            logical_height,
            viewport_x,
            viewport_y,
            scale,
            transform,
            &mut fonts,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint_bgra_viewport(
        &mut self,
        pixels: &mut [u8],
        buffer_width: u32,
        buffer_height: u32,
        stride: u32,
        logical_width: u32,
        logical_height: u32,
        viewport_x: i32,
        viewport_y: i32,
        scale: u32,
    ) -> Option<DamageRect> {
        let mut fonts = FontCtx::new();
        self.paint_bgra_viewport_with_fonts(
            pixels,
            buffer_width,
            buffer_height,
            stride,
            logical_width,
            logical_height,
            viewport_x,
            viewport_y,
            scale,
            &mut fonts,
        )
    }

    pub fn paint_scaled_with_fonts(
        &mut self,
        canvas: &mut Canvas<'_>,
        fonts: &mut FontCtx,
        width: u32,
        height: u32,
        scale: u32,
    ) {
        self.paint_viewport_with_fonts(canvas, fonts, width, height, 0, 0, scale);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint_transformed_with_fonts(
        &mut self,
        canvas: &mut Canvas<'_>,
        fonts: &mut FontCtx,
        width: u32,
        height: u32,
        scale: u32,
        transform: PaintTransform,
    ) {
        self.paint_transformed_viewport_with_fonts(
            canvas, fonts, width, height, 0, 0, scale, transform,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint_viewport_with_fonts(
        &mut self,
        canvas: &mut Canvas<'_>,
        fonts: &mut FontCtx,
        width: u32,
        height: u32,
        viewport_x: i32,
        viewport_y: i32,
        scale: u32,
    ) {
        let scale = scale.max(1);
        let offset = PaintOffset {
            x: scale_i32(viewport_x, scale),
            y: scale_i32(viewport_y, scale),
        };
        let bounds = Bounds::new(0, 0, width, height);
        Self::visit_layout(
            self,
            bounds,
            Clip::rect(bounds),
            VisualState::IDENTITY,
            1.0,
            None,
            fonts,
            &mut |command, fonts| {
                paint_scaled_command_with_offset(canvas, fonts, command, scale, offset);
            },
        );
        fonts.trim_scratch();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint_transformed_viewport_with_fonts(
        &mut self,
        canvas: &mut Canvas<'_>,
        fonts: &mut FontCtx,
        width: u32,
        height: u32,
        viewport_x: i32,
        viewport_y: i32,
        scale: u32,
        transform: PaintTransform,
    ) {
        let scale = scale.max(1);
        let visual_scale = if transform.scale.is_finite() {
            transform.scale.max(0.0)
        } else {
            0.0
        };
        if visual_scale <= 0.0 {
            fonts.trim_scratch();
            return;
        }
        if transform.is_identity() {
            self.paint_viewport_with_fonts(
                canvas, fonts, width, height, viewport_x, viewport_y, scale,
            );
            return;
        }

        let total_scale = scale as f32 * visual_scale;
        let offset = PaintOffset {
            x: scale_i32_f32(viewport_x, total_scale).saturating_sub(transform.translate_x),
            y: scale_i32_f32(viewport_y, total_scale).saturating_sub(transform.translate_y),
        };
        let bounds = Bounds::new(0, 0, width, height);
        Self::visit_layout(
            self,
            bounds,
            Clip::rect(bounds),
            VisualState::IDENTITY,
            1.0,
            None,
            fonts,
            &mut |command, fonts| {
                paint_scaled_f32_command_with_offset(canvas, fonts, command, total_scale, offset);
            },
        );
        fonts.trim_scratch();
    }

    pub fn render(&mut self, canvas: &mut Canvas<'_>) {
        self.paint(canvas);
    }

    fn visit_layout(
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
