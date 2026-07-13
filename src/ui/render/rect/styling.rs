use super::*;

impl Rect {
    pub fn background(mut self, background: impl Into<Paint>) -> Self {
        self.style.background = Some(background.into());
        self
    }

    pub fn border(mut self, border: Border) -> Self {
        self.style.border = Some(border);
        self
    }

    pub fn border_color(mut self, color: impl Into<Paint>) -> Self {
        self.style.border.get_or_insert_with(Border::new).color = color.into();
        self
    }

    pub fn border_width(mut self, width: impl Into<Pixels>) -> Self {
        self.style.border.get_or_insert_with(Border::new).width = width.into().get();
        self
    }

    pub fn border_widths(mut self, widths: BorderWidth) -> Self {
        self.style.border.get_or_insert_with(Border::new).widths = widths;
        self
    }

    pub fn border_top_width(mut self, width: impl Into<Pixels>) -> Self {
        self.style.border.get_or_insert_with(Border::new).widths.top = width.into().get();
        self
    }

    pub fn border_right_width(mut self, width: impl Into<Pixels>) -> Self {
        self.style
            .border
            .get_or_insert_with(Border::new)
            .widths
            .right = width.into().get();
        self
    }

    pub fn border_bottom_width(mut self, width: impl Into<Pixels>) -> Self {
        self.style
            .border
            .get_or_insert_with(Border::new)
            .widths
            .bottom = width.into().get();
        self
    }

    pub fn border_left_width(mut self, width: impl Into<Pixels>) -> Self {
        self.style
            .border
            .get_or_insert_with(Border::new)
            .widths
            .left = width.into().get();
        self
    }

    pub fn border_gradient(mut self, gradient: GradientDirection) -> Self {
        self.style.border.get_or_insert_with(Border::new).gradient = gradient;
        self
    }

    pub fn corner_radius(mut self, radius: impl Into<Pixels>) -> Self {
        self.style.corner_radius = radius.into().get();
        self
    }

    pub fn corner_radii(mut self, radii: CornerRadius) -> Self {
        self.style.corner_radii = radii;
        self
    }

    pub fn corner_top_left(mut self, radius: impl Into<Pixels>) -> Self {
        self.style.corner_radii.top_left = radius.into().get();
        self
    }

    pub fn corner_top_right(mut self, radius: impl Into<Pixels>) -> Self {
        self.style.corner_radii.top_right = radius.into().get();
        self
    }

    pub fn corner_bottom_right(mut self, radius: impl Into<Pixels>) -> Self {
        self.style.corner_radii.bottom_right = radius.into().get();
        self
    }

    pub fn corner_bottom_left(mut self, radius: impl Into<Pixels>) -> Self {
        self.style.corner_radii.bottom_left = radius.into().get();
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

    pub fn anti_alias(mut self, anti_alias: bool) -> Self {
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
}
