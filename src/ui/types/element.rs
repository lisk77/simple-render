use super::{Image, RichText, Text};
use crate::{Align, Border, Length, Paint, PaintTransform, Pixels, Rect, Spacing};

#[derive(Clone)]
pub enum Element {
    Rect(Rect),
    Text(Text),
    RichText(RichText),
    Image(Image),
}

impl Element {
    pub(crate) fn into_rect(self) -> Rect {
        match self {
            Self::Rect(rect) => rect,
            Self::Text(text) => Rect::new().text(text),
            Self::RichText(text) => Rect::new().rich_text(text),
            Self::Image(image) => Rect::new().content(image),
        }
    }

    fn map_rect(self, map: impl FnOnce(Rect) -> Rect) -> Self {
        map(self.into_rect()).into()
    }

    pub fn width(self, width: impl Into<Length>) -> Self {
        self.map_rect(|rect| rect.width(width))
    }

    pub fn height(self, height: impl Into<Length>) -> Self {
        self.map_rect(|rect| rect.height(height))
    }

    pub fn size(self, width: impl Into<Length>, height: impl Into<Length>) -> Self {
        self.map_rect(|rect| rect.size(width, height))
    }

    pub fn size_fill(self) -> Self {
        self.map_rect(Rect::size_fill)
    }

    pub fn align(self, align: Align) -> Self {
        self.map_rect(|rect| rect.align(align))
    }

    pub fn align_center(self) -> Self {
        self.align(Align::Center)
    }

    pub fn justify(self, justify: Align) -> Self {
        self.map_rect(|rect| rect.justify(justify))
    }

    pub fn justify_center(self) -> Self {
        self.justify(Align::Center)
    }

    pub fn padding(self, padding: Spacing) -> Self {
        self.map_rect(|rect| rect.padding(padding))
    }

    pub fn padding_all(self, value: impl Into<Pixels>) -> Self {
        self.map_rect(|rect| rect.padding_all(value))
    }

    pub fn gap(self, gap: impl Into<Pixels>) -> Self {
        self.map_rect(|rect| rect.gap(gap))
    }

    pub fn background(self, background: impl Into<Paint>) -> Self {
        self.map_rect(|rect| rect.background(background))
    }

    pub fn border(self, border: Border) -> Self {
        self.map_rect(|rect| rect.border(border))
    }

    pub fn border_color(self, color: impl Into<Paint>) -> Self {
        self.map_rect(|rect| rect.border_color(color))
    }

    pub fn border_width(self, width: impl Into<Pixels>) -> Self {
        self.map_rect(|rect| rect.border_width(width))
    }

    pub fn corner_radius(self, radius: impl Into<Pixels>) -> Self {
        self.map_rect(|rect| rect.corner_radius(radius))
    }

    pub fn opacity(self, opacity: f32) -> Self {
        self.map_rect(|rect| rect.opacity(opacity))
    }

    pub fn anti_alias(self, anti_alias: bool) -> Self {
        self.map_rect(|rect| rect.anti_alias(anti_alias))
    }

    pub fn transform(self, transform: PaintTransform) -> Self {
        self.map_rect(|rect| rect.transform(transform))
    }

    pub fn translate(self, x: i32, y: i32) -> Self {
        self.map_rect(|rect| rect.translate(x, y))
    }

    pub fn scale(self, scale: f32) -> Self {
        self.map_rect(|rect| rect.scale(scale))
    }

    pub fn child(self, child: impl Into<Element>) -> Self {
        self.map_rect(|rect| rect.child(child))
    }
}

pub trait ElementBuilder: Into<Element> + Sized {
    fn element(self) -> Element {
        self.into()
    }

    fn align(self, align: Align) -> Element {
        self.element().align(align)
    }

    fn align_center(self) -> Element {
        self.element().align_center()
    }

    fn justify(self, justify: Align) -> Element {
        self.element().justify(justify)
    }

    fn justify_center(self) -> Element {
        self.element().justify_center()
    }

    fn padding(self, padding: Spacing) -> Element {
        self.element().padding(padding)
    }

    fn padding_all(self, value: impl Into<Pixels>) -> Element {
        self.element().padding_all(value)
    }

    fn gap(self, gap: impl Into<Pixels>) -> Element {
        self.element().gap(gap)
    }

    fn background(self, background: impl Into<Paint>) -> Element {
        self.element().background(background)
    }

    fn border(self, border: Border) -> Element {
        self.element().border(border)
    }

    fn border_color(self, color: impl Into<Paint>) -> Element {
        self.element().border_color(color)
    }

    fn border_width(self, width: impl Into<Pixels>) -> Element {
        self.element().border_width(width)
    }

    fn corner_radius(self, radius: impl Into<Pixels>) -> Element {
        self.element().corner_radius(radius)
    }

    fn opacity(self, opacity: f32) -> Element {
        self.element().opacity(opacity)
    }

    fn anti_alias(self, anti_alias: bool) -> Element {
        self.element().anti_alias(anti_alias)
    }

    fn transform(self, transform: PaintTransform) -> Element {
        self.element().transform(transform)
    }

    fn translate(self, x: i32, y: i32) -> Element {
        self.element().translate(x, y)
    }

    fn scale(self, scale: f32) -> Element {
        self.element().scale(scale)
    }
}

impl<T: Into<Element>> ElementBuilder for T {}

impl From<Rect> for Element {
    fn from(value: Rect) -> Self {
        Self::Rect(value)
    }
}
impl From<Text> for Element {
    fn from(value: Text) -> Self {
        Self::Text(value)
    }
}
impl From<RichText> for Element {
    fn from(value: RichText) -> Self {
        Self::RichText(value)
    }
}
impl From<Image> for Element {
    fn from(value: Image) -> Self {
        Self::Image(value)
    }
}
