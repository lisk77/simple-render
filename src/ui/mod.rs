use std::{error::Error, fmt, str::FromStr, sync::Arc};

use cosmic_text::{
    Attrs, Buffer, Color as CosmicColor, DecorationSpan, Ellipsize, EllipsizeHeightLimit, Family,
    FontSystem, LayoutRun, Metrics, PhysicalGlyph, Shaping, Style as FontStyle, SwashCache,
    UnderlineStyle, Weight, Wrap, fontdb,
};

use crate::input::WidgetId;
use crate::wayland::{
    self, Anchor, Canvas, CanvasRenderer, DamageRect, FrameAction, KeyboardInteractivity, Layer,
    LayerOptions, Margins, OutputTarget, RenderContext, SurfaceId,
};

mod render;
mod types;

pub use cosmic_text::fontdb::Source as FontSource;
pub use render::{
    ChangeEvent, ClickEvent, Context, FontCtx, FontCtxOptions, LazyFontCtx, Listener, Rect, Render,
    Ui,
};
pub use types::*;
