use super::*;
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
