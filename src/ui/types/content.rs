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

impl TextStyle {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn color(mut self, value: impl Into<Paint>) -> Self {
        self.color = value.into();
        self
    }
    pub fn gradient(mut self, value: GradientDirection) -> Self {
        self.gradient = value;
        self
    }
    pub fn size(mut self, value: u32) -> Self {
        self.size = value;
        self
    }
    pub fn family(mut self, value: impl Into<String>) -> Self {
        self.family = Some(value.into());
        self
    }

    pub fn clear_family(mut self) -> Self {
        self.family = None;
        self
    }
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }
    pub fn overline(mut self) -> Self {
        self.overline = true;
        self
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

    pub fn content(mut self, content: impl Into<Arc<str>>) -> Self {
        self.content = content.into();
        self
    }

    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    pub fn color(mut self, color: impl Into<Paint>) -> Self {
        self.style.color = color.into();
        self
    }

    pub fn gradient(mut self, gradient: GradientDirection) -> Self {
        self.style.gradient = gradient;
        self
    }

    pub fn size(mut self, size: u32) -> Self {
        self.style.size = size;
        self
    }

    pub fn family(mut self, family: impl Into<String>) -> Self {
        self.style.family = Some(family.into());
        self
    }

    pub fn bold(mut self) -> Self {
        self.style.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.style.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.style.underline = true;
        self
    }

    pub fn strikethrough(mut self) -> Self {
        self.style.strikethrough = true;
        self
    }

    pub fn overline(mut self) -> Self {
        self.style.overline = true;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn align_center(self) -> Self {
        self.align(Align::Center)
    }

    pub fn vertical_align(mut self, align: Align) -> Self {
        self.vertical_align = align;
        self
    }

    pub fn vertical_align_center(self) -> Self {
        self.vertical_align(Align::Center)
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn wrap_word(self) -> Self {
        self.wrap(TextWrap::Word)
    }

    pub fn wrap_glyph(self) -> Self {
        self.wrap(TextWrap::Glyph)
    }

    pub fn wrap_none(self) -> Self {
        self.wrap(TextWrap::None)
    }

    pub fn overflow(mut self, overflow: TextOverflow) -> Self {
        self.overflow = overflow;
        self
    }

    pub fn ellipsis(self) -> Self {
        self.overflow(TextOverflow::Ellipsis)
    }

    pub fn max_lines(mut self, max_lines: u32) -> Self {
        self.max_lines = Some(max_lines);
        self
    }

    pub fn emoji_family(mut self, family: impl Into<String>) -> Self {
        self.emoji_family = Some(family.into());
        self
    }

    pub fn clear_emoji_family(mut self) -> Self {
        self.emoji_family = None;
        self
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

    pub fn content(mut self, content: impl Into<Arc<str>>) -> Self {
        self.content = content.into();
        self
    }
    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
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

    pub fn runs(mut self, runs: impl Into<Arc<[TextRun]>>) -> Self {
        self.runs = runs.into();
        self
    }
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
    pub fn align_center(self) -> Self {
        self.align(Align::Center)
    }
    pub fn vertical_align(mut self, align: Align) -> Self {
        self.vertical_align = align;
        self
    }
    pub fn vertical_align_center(self) -> Self {
        self.vertical_align(Align::Center)
    }
    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = wrap;
        self
    }
    pub fn overflow(mut self, overflow: TextOverflow) -> Self {
        self.overflow = overflow;
        self
    }
    pub fn ellipsis(self) -> Self {
        self.overflow(TextOverflow::Ellipsis)
    }
    pub fn max_lines(mut self, value: u32) -> Self {
        self.max_lines = Some(value);
        self
    }
    pub fn emoji_family(mut self, value: impl Into<String>) -> Self {
        self.emoji_family = Some(value.into());
        self
    }

    pub fn clear_emoji_family(mut self) -> Self {
        self.emoji_family = None;
        self
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

    pub fn stride(self, stride: u32) -> Self {
        self.with_stride(stride)
    }

    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }
    pub fn filter(mut self, filter: ImageFilter) -> Self {
        self.filter = filter;
        self
    }
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
    pub fn align_center(self) -> Self {
        self.align(Align::Center)
    }
    pub fn vertical_align(mut self, align: Align) -> Self {
        self.vertical_align = align;
        self
    }
    pub fn vertical_align_center(self) -> Self {
        self.vertical_align(Align::Center)
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
