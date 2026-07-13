use super::*;

mod context;
mod geometry;
mod image;
mod layout;
mod listener;
mod managed;
mod scaling;
mod text;

pub use context::Context;
pub use listener::{ChangeEvent, ClickEvent, Listener};

pub(crate) use managed::ManagedRenderer;
pub use managed::Render;

pub(super) use geometry::*;
use image::*;
use layout::*;
use text::*;

const RETAINED_TEXT_RUN_CAPACITY: usize = 256;
const RETAINED_GLYPH_CAPACITY: usize = 4096;

type GlyphPaint = (PhysicalGlyph, usize, CosmicColor);

#[derive(Clone, Copy)]
struct ByteRun {
    start: usize,
    end: usize,
    is_emoji: bool,
}

#[derive(Clone)]
struct TextBufferRun {
    source: Arc<str>,
    byte_run: ByteRun,
    style: TextStyle,
    emoji_family: Option<String>,
}

impl TextBufferRun {
    fn text(&self) -> &str {
        &self.source[self.byte_run.start..self.byte_run.end]
    }

    fn is_emoji(&self) -> bool {
        self.byte_run.is_emoji
    }
}

pub struct FontCtx {
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_buffer: Buffer,
    text_runs: Vec<TextBufferRun>,
    glyphs: Vec<GlyphPaint>,
}

#[derive(Clone, Debug, Default)]
pub struct FontCtxOptions {
    sources: Option<Vec<FontSource>>,
}

impl FontCtxOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn font_sources(mut self, fonts: impl IntoIterator<Item = FontSource>) -> Self {
        self.sources = Some(fonts.into_iter().collect());
        self
    }
    pub fn system() -> Self {
        Self { sources: None }
    }

    pub fn from_font_sources(fonts: impl IntoIterator<Item = FontSource>) -> Self {
        Self {
            sources: Some(fonts.into_iter().collect()),
        }
    }

    fn create(&self) -> FontCtx {
        match &self.sources {
            Some(sources) => FontCtx::new_with_font_sources(sources.iter().cloned()),
            None => FontCtx::new(),
        }
    }
}

pub struct LazyFontCtx {
    options: FontCtxOptions,
    fonts: Option<FontCtx>,
}

impl LazyFontCtx {
    pub fn new() -> Self {
        Self::with_options(FontCtxOptions::system())
    }

    pub fn with_options(options: FontCtxOptions) -> Self {
        Self {
            options,
            fonts: None,
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.fonts.is_some()
    }

    pub fn get(&mut self) -> &mut FontCtx {
        if self.fonts.is_none() {
            self.fonts = Some(self.options.create());
        }
        self.fonts.as_mut().expect("font context was just created")
    }

    pub fn clear_raster_cache(&mut self) {
        if let Some(fonts) = &mut self.fonts {
            fonts.clear_raster_cache();
        }
    }

    pub fn trim_scratch(&mut self) {
        if let Some(fonts) = &mut self.fonts {
            fonts.trim_scratch();
        }
    }

    pub fn trim_frame_memory(&mut self) {
        self.clear_raster_cache();
        self.trim_scratch();
    }

    pub fn release(&mut self) {
        self.fonts = None;
        crate::memory::trim_free_heap_pages();
    }
}

impl Default for LazyFontCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl FontCtx {
    pub fn new() -> Self {
        Self::from_font_system(FontSystem::new())
    }

    pub fn new_with_font_sources(fonts: impl IntoIterator<Item = FontSource>) -> Self {
        let mut db = fontdb::Database::new();
        for source in fonts {
            db.load_font_source(source);
        }
        let default_family = db
            .faces()
            .find_map(|face| face.families.first().map(|(family, _)| family.clone()));
        if let Some(family) = default_family {
            db.set_sans_serif_family(family.clone());
            db.set_serif_family(family.clone());
            db.set_monospace_family(family);
        }

        Self::from_font_system(FontSystem::new_with_locale_and_db("en-US".into(), db))
    }

    fn from_font_system(font_system: FontSystem) -> Self {
        Self {
            font_system,
            swash_cache: SwashCache::new(),
            text_buffer: Buffer::new_empty(Metrics::new(1.0, 1.3)),
            text_runs: Vec::new(),
            glyphs: Vec::new(),
        }
    }

    pub fn clear_raster_cache(&mut self) {
        self.swash_cache = SwashCache::new();
    }

    pub fn trim_scratch(&mut self) {
        self.text_buffer = Buffer::new_empty(Metrics::new(1.0, 1.3));
        trim_vec_capacity(&mut self.text_runs, RETAINED_TEXT_RUN_CAPACITY);
        trim_vec_capacity(&mut self.glyphs, RETAINED_GLYPH_CAPACITY);
    }

    pub fn trim_frame_memory(&mut self) {
        self.clear_raster_cache();
        self.trim_scratch();
    }
}

impl Default for FontCtx {
    fn default() -> Self {
        Self::new()
    }
}

fn trim_vec_capacity<T>(values: &mut Vec<T>, retained_capacity: usize) {
    if values.capacity() > retained_capacity {
        values.clear();
        values.shrink_to(retained_capacity);
    }
}

mod paint;
mod rect;

use paint::*;
pub use rect::{Rect, Ui};

struct UiRenderer {
    root: Rect,
    fonts: LazyFontCtx,
}

impl CanvasRenderer for UiRenderer {
    fn draw(&mut self, canvas: &mut Canvas<'_>, context: RenderContext) -> FrameAction {
        let fonts = self.fonts.get();
        if let Some(repaint) = context
            .repaint
            .and_then(|repaint| repaint.intersect(Bounds::new(0, 0, context.width, context.height)))
        {
            canvas.clear_rect(
                scale_repaint_bounds(repaint, context.scale_factor),
                Color::TRANSPARENT.into(),
            );
            self.root.paint_clipped_scaled_f32_with_fonts(
                canvas,
                fonts,
                context.width,
                context.height,
                context.scale_factor,
                repaint,
            );
        } else {
            canvas.clear(Color::TRANSPARENT.into());
            self.root.paint_scaled_f32_with_fonts(
                canvas,
                fonts,
                context.width,
                context.height,
                context.scale_factor,
            );
        }
        FrameAction::Wait
    }

    fn idle_surface(&mut self, _: SurfaceId) {
        self.fonts.trim_frame_memory();
    }

    fn closed_surface(&mut self, _: SurfaceId) {
        self.fonts.release();
    }
}

fn scale_repaint_bounds(bounds: Bounds, scale: f32) -> DamageRect {
    let scale = if scale.is_finite() {
        scale.max(1.0)
    } else {
        1.0
    };
    let x = (bounds.x as f32 * scale).floor() as u32;
    let y = (bounds.y as f32 * scale).floor() as u32;
    let right = (bounds.x.saturating_add(bounds.width) as f32 * scale).ceil() as u32;
    let bottom = (bounds.y.saturating_add(bounds.height) as f32 * scale).ceil() as u32;
    DamageRect::new(x, y, right.saturating_sub(x), bottom.saturating_sub(y))
}
