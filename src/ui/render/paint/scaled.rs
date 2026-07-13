use super::*;
use crate::ui::render::rect::VisualState;

impl Rect {
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

    pub fn paint_scaled_f32_with_fonts(
        &mut self,
        canvas: &mut Canvas<'_>,
        fonts: &mut FontCtx,
        width: u32,
        height: u32,
        scale: f32,
    ) {
        self.paint_f32_viewport_with_fonts(canvas, fonts, width, height, 0, 0, scale);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint_clipped_scaled_with_fonts(
        &mut self,
        canvas: &mut Canvas<'_>,
        fonts: &mut FontCtx,
        width: u32,
        height: u32,
        scale: u32,
        clip_bounds: Bounds,
    ) {
        self.paint_clipped_viewport_with_fonts(
            canvas,
            fonts,
            width,
            height,
            0,
            0,
            scale,
            clip_bounds,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint_clipped_scaled_f32_with_fonts(
        &mut self,
        canvas: &mut Canvas<'_>,
        fonts: &mut FontCtx,
        width: u32,
        height: u32,
        scale: f32,
        clip_bounds: Bounds,
    ) {
        self.paint_clipped_f32_viewport_with_fonts(
            canvas,
            fonts,
            width,
            height,
            0,
            0,
            scale,
            clip_bounds,
        );
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
    pub fn paint_f32_viewport_with_fonts(
        &mut self,
        canvas: &mut Canvas<'_>,
        fonts: &mut FontCtx,
        width: u32,
        height: u32,
        viewport_x: i32,
        viewport_y: i32,
        scale: f32,
    ) {
        let scale = if scale.is_finite() {
            scale.max(1.0)
        } else {
            1.0
        };
        let offset = PaintOffset {
            x: scale_i32_f32(viewport_x, scale),
            y: scale_i32_f32(viewport_y, scale),
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
                paint_scaled_f32_command_with_offset(canvas, fonts, command, scale, offset);
            },
        );
        fonts.trim_scratch();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint_clipped_viewport_with_fonts(
        &mut self,
        canvas: &mut Canvas<'_>,
        fonts: &mut FontCtx,
        width: u32,
        height: u32,
        viewport_x: i32,
        viewport_y: i32,
        scale: u32,
        clip_bounds: Bounds,
    ) {
        let scale = scale.max(1);
        let offset = PaintOffset {
            x: scale_i32(viewport_x, scale),
            y: scale_i32(viewport_y, scale),
        };
        let bounds = Bounds::new(0, 0, width, height);
        let Some(clip_bounds) = bounds.intersect(clip_bounds) else {
            fonts.trim_scratch();
            return;
        };

        Self::visit_layout(
            self,
            bounds,
            Clip::rect(clip_bounds),
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
    pub fn paint_clipped_f32_viewport_with_fonts(
        &mut self,
        canvas: &mut Canvas<'_>,
        fonts: &mut FontCtx,
        width: u32,
        height: u32,
        viewport_x: i32,
        viewport_y: i32,
        scale: f32,
        clip_bounds: Bounds,
    ) {
        let scale = if scale.is_finite() {
            scale.max(1.0)
        } else {
            1.0
        };
        let offset = PaintOffset {
            x: scale_i32_f32(viewport_x, scale),
            y: scale_i32_f32(viewport_y, scale),
        };
        let bounds = Bounds::new(0, 0, width, height);
        let Some(clip_bounds) = bounds.intersect(clip_bounds) else {
            fonts.trim_scratch();
            return;
        };

        Self::visit_layout(
            self,
            bounds,
            Clip::rect(clip_bounds),
            VisualState::IDENTITY,
            1.0,
            None,
            fonts,
            &mut |command, fonts| {
                paint_scaled_f32_command_with_offset(canvas, fonts, command, scale, offset);
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
}
