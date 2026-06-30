use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextWrap {
    None,
    Glyph,
    Word,
    #[default]
    WordOrGlyph,
}

impl TextWrap {
    pub(in crate::ui) fn into_cosmic(self) -> Wrap {
        match self {
            Self::None => Wrap::None,
            Self::Glyph => Wrap::Glyph,
            Self::Word => Wrap::Word,
            Self::WordOrGlyph => Wrap::WordOrGlyph,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextOverflow {
    #[default]
    Clip,
    Ellipsis,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    pub color: Paint,
    pub gradient: GradientDirection,
    pub size: u32,
    pub family: Option<String>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub overline: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: Paint::solid(Color::WHITE),
            gradient: GradientDirection::default(),
            size: 14,
            family: None,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            overline: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Text {
    pub content: Arc<str>,
    pub style: TextStyle,
    pub align: Align,
    pub vertical_align: Align,
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
    pub max_lines: Option<u32>,
    pub emoji_family: Option<String>,
}

impl Text {
    pub fn new(content: impl Into<Arc<str>>) -> Self {
        Self {
            content: content.into(),
            ..Self::default()
        }
    }
}

impl Default for Text {
    fn default() -> Self {
        Self {
            content: Arc::from(""),
            style: TextStyle::default(),
            align: Align::Start,
            vertical_align: Align::Start,
            wrap: TextWrap::default(),
            overflow: TextOverflow::default(),
            max_lines: None,
            emoji_family: Some("Noto Color Emoji".into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextRun {
    pub content: Arc<str>,
    pub style: TextStyle,
}

impl TextRun {
    pub fn new(content: impl Into<Arc<str>>, style: TextStyle) -> Self {
        Self {
            content: content.into(),
            style,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RichText {
    pub runs: Arc<[TextRun]>,
    pub align: Align,
    pub vertical_align: Align,
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
    pub max_lines: Option<u32>,
    pub emoji_family: Option<String>,
}

impl RichText {
    pub fn new(runs: impl Into<Arc<[TextRun]>>) -> Self {
        Self {
            runs: runs.into(),
            ..Self::default()
        }
    }
}

impl Default for RichText {
    fn default() -> Self {
        Self {
            runs: Arc::from([]),
            align: Align::Start,
            vertical_align: Align::Start,
            wrap: TextWrap::default(),
            overflow: TextOverflow::default(),
            max_lines: None,
            emoji_family: Some("Noto Color Emoji".into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageFit {
    #[default]
    None,
    Fill,
    Contain,
    Cover,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageFilter {
    #[default]
    Nearest,
    Linear,
}

pub trait RgbaImageSource: fmt::Debug + Send + Sync {
    fn rgba(&self) -> &[u8];
}

#[derive(Clone)]
pub enum ImagePixels {
    Owned(Arc<[u8]>),
    Source(Arc<dyn RgbaImageSource>),
}

impl ImagePixels {
    pub fn as_rgba(&self) -> &[u8] {
        match self {
            Self::Owned(pixels) => pixels,
            Self::Source(source) => source.rgba(),
        }
    }
}

impl fmt::Debug for ImagePixels {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ImagePixels")
            .field("len", &self.as_rgba().len())
            .finish_non_exhaustive()
    }
}

impl PartialEq for ImagePixels {
    fn eq(&self, other: &Self) -> bool {
        self.as_rgba() == other.as_rgba()
    }
}

impl Eq for ImagePixels {}

impl From<Arc<[u8]>> for ImagePixels {
    fn from(pixels: Arc<[u8]>) -> Self {
        Self::Owned(pixels)
    }
}

impl From<Vec<u8>> for ImagePixels {
    fn from(pixels: Vec<u8>) -> Self {
        Self::Owned(pixels.into())
    }
}

impl<const N: usize> From<[u8; N]> for ImagePixels {
    fn from(pixels: [u8; N]) -> Self {
        Self::Owned(Arc::from(pixels))
    }
}

impl RgbaImageSource for Vec<u8> {
    fn rgba(&self) -> &[u8] {
        self
    }
}

impl RgbaImageSource for Box<[u8]> {
    fn rgba(&self) -> &[u8] {
        self
    }
}

impl RgbaImageSource for Arc<[u8]> {
    fn rgba(&self) -> &[u8] {
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: ImagePixels,
    pub stride: u32,
    pub fit: ImageFit,
    pub filter: ImageFilter,
    pub align: Align,
    pub vertical_align: Align,
}

impl Image {
    pub fn new(width: u32, height: u32, rgba: impl Into<ImagePixels>) -> Self {
        Self {
            width,
            height,
            rgba: rgba.into(),
            stride: width.saturating_mul(4),
            fit: ImageFit::default(),
            filter: ImageFilter::default(),
            align: Align::Start,
            vertical_align: Align::Start,
        }
    }

    pub fn with_stride(mut self, stride: u32) -> Self {
        self.stride = stride;
        self
    }

    pub fn from_source(width: u32, height: u32, source: Arc<dyn RgbaImageSource>) -> Self {
        Self::new(width, height, ImagePixels::Source(source))
    }

    pub fn from_source_with_stride(
        width: u32,
        height: u32,
        stride: u32,
        source: Arc<dyn RgbaImageSource>,
    ) -> Self {
        Self::from_source(width, height, source).with_stride(stride)
    }

    pub fn rgba(&self) -> &[u8] {
        self.rgba.as_rgba()
    }
}

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
