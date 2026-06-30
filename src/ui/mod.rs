use std::{error::Error, fmt, str::FromStr, sync::Arc};

use cosmic_text::{
    Attrs, Buffer, Color as CosmicColor, DecorationSpan, Ellipsize, EllipsizeHeightLimit, Family,
    FontSystem, LayoutRun, Metrics, PhysicalGlyph, Shaping, Style as FontStyle, SwashCache,
    UnderlineStyle, Weight, Wrap,
};

use crate::wayland::{
    self, Anchor, Canvas, DamageRect, FrameAction, KeyboardInteractivity, Layer, LayerOptions,
    Margins, OutputTarget, RenderContext, Renderer,
};

mod render;
mod types;

pub use render::{FontCtx, PaintTransform, Rect, Ui};
pub use types::*;
