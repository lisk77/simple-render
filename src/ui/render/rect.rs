use super::*;

#[derive(Clone)]
pub struct Rect {
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

impl Default for Rect {
    fn default() -> Self {
        Self {
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
        self.content = layout.content;
        self
    }

    pub fn style(mut self, style: RectStyle) -> Self {
        self.style = style;
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
            fonts: FontCtx::new(),
        })
    }

    pub fn draw_with_commands(self, receiver: wayland::RenderReceiver) -> wayland::Result<()> {
        let options = self.layer_options.clone();
        options.show_with_commands(
            UiRenderer {
                root: self,
                fonts: FontCtx::new(),
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
        let Some(clip) = clip.intersect_bounds(rect) else {
            return;
        };
        let opacity = multiply_opacity(opacity, element.style.opacity);
        let radii = element_corner_radii(element);

        if let Some(paint) = &element.style.background {
            visit(
                PaintCommand::Rect {
                    rect,
                    clip,
                    opacity,
                    paint,
                    gradient: element.style.gradient,
                    radii,
                },
                fonts,
            );
        }
        if let Some(border) = &element.style.border {
            visit(
                PaintCommand::Border {
                    rect,
                    clip,
                    opacity,
                    paint: &border.color,
                    gradient: border.gradient,
                    widths: border_widths(border),
                    radii,
                },
                fonts,
            );
        }
        let content_rect = rect.inset(element.padding);
        if let Some(content) = &element.content {
            match content {
                Content::Text(text) => visit(
                    PaintCommand::Text {
                        rect: content_rect,
                        clip,
                        opacity,
                        text,
                    },
                    fonts,
                ),
                Content::RichText(text) => visit(
                    PaintCommand::RichText {
                        rect: content_rect,
                        clip,
                        opacity,
                        text,
                    },
                    fonts,
                ),
                Content::Image(image) => visit(
                    PaintCommand::Image {
                        rect: content_rect,
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
                let Some(child_clip) =
                    clip.with_rounded_rect(content_rect, content_clip_radii(element))
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
            |child, rect, measured, fonts| {
                Self::visit_layout(
                    child,
                    rect,
                    child_clip,
                    opacity,
                    Some(measured),
                    fonts,
                    visit,
                );
            },
        );
    }
}
