use super::*;

impl Rect {
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
}
