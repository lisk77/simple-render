use super::*;

impl Rect {
    pub fn width_fill(self) -> Self {
        self.width(Length::Fill)
    }

    pub fn height_fill(self) -> Self {
        self.height(Length::Fill)
    }

    pub fn size_fill(self) -> Self {
        self.size(Length::Fill, Length::Fill)
    }

    pub fn width_px(self, width: impl Into<Pixels>) -> Self {
        self.width(width.into())
    }

    pub fn height_px(self, height: impl Into<Pixels>) -> Self {
        self.height(height.into())
    }

    pub fn size_px(self, width: impl Into<Pixels>, height: impl Into<Pixels>) -> Self {
        self.size(width.into(), height.into())
    }

    pub fn padding_all(self, value: impl Into<Pixels>) -> Self {
        self.padding(Spacing::all(value.into().get()))
    }

    pub fn padding_axis(self, horizontal: impl Into<Pixels>, vertical: impl Into<Pixels>) -> Self {
        self.padding(Spacing::axis(
            horizontal.into().get(),
            vertical.into().get(),
        ))
    }

    pub fn text(mut self, text: Text) -> Self {
        self.content = Some(Content::Text(text));
        self
    }

    pub fn rich_text(mut self, text: RichText) -> Self {
        self.content = Some(Content::RichText(text));
        self
    }

    pub fn width_value(&self) -> Length {
        self.width
    }

    pub fn height_value(&self) -> Length {
        self.height
    }

    pub fn direction_value(&self) -> Direction {
        self.direction
    }

    pub fn padding_value(&self) -> Spacing {
        self.padding
    }

    pub fn gap_value(&self) -> u32 {
        self.gap
    }

    pub fn style_ref(&self) -> &Style {
        &self.style
    }

    pub fn corner_radius_value(&self) -> u32 {
        self.style.corner_radius
    }

    pub fn begin(_: Bounds) -> Self {
        Self::default()
    }

    pub fn build(bounds: Bounds, content: impl FnOnce(&mut Self)) -> Self {
        let mut ui = Self::begin(bounds);
        content(&mut ui);
        ui
    }

    pub fn child(mut self, child: impl Into<Element>) -> Self {
        self.children.push(child.into().into_rect());
        self
    }

    pub fn children<I, T>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Element>,
    {
        self.children
            .extend(children.into_iter().map(|child| child.into().into_rect()));
        self
    }
}
