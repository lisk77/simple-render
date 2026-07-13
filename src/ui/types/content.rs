use super::*;

mod image;
mod text;

pub use image::*;
pub use text::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    Text(Text),
    RichText(RichText),
    Image(Image),
}

impl From<Text> for Content {
    fn from(text: Text) -> Self {
        Self::Text(text)
    }
}

impl From<RichText> for Content {
    fn from(text: RichText) -> Self {
        Self::RichText(text)
    }
}

impl From<Image> for Content {
    fn from(image: Image) -> Self {
        Self::Image(image)
    }
}
