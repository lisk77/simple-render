use super::*;
use crate::animation::Animation;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub type RenderSender = calloop_channel::Sender<RenderCommand>;
pub type RenderReceiver = calloop_channel::Channel<RenderCommand>;

pub fn channel() -> (RenderSender, RenderReceiver) {
    calloop_channel::channel()
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceId(pub u64);

pub const DEFAULT_SURFACE_ID: SurfaceId = SurfaceId(0);

#[derive(Debug, Clone)]
pub enum RenderCommand {
    Redraw,
    Resize {
        width: u32,
        height: u32,
    },
    SetMargins(Margins),
    SetAnchor(Anchor),
    CreateSurface {
        id: SurfaceId,
        options: LayerOptions,
    },
    DestroySurface {
        id: SurfaceId,
    },
    RedrawSurface {
        id: SurfaceId,
    },
    ResizeSurface {
        id: SurfaceId,
        width: u32,
        height: u32,
    },
    SetSurfaceMargins {
        id: SurfaceId,
        margins: Margins,
    },
    SetSurfaceAnchor {
        id: SurfaceId,
        anchor: Anchor,
    },
    SetSurfaceLayer {
        id: SurfaceId,
        layer: Layer,
    },
    AnimateSurfaceMargins {
        id: SurfaceId,
        to: Margins,
        animation: Animation,
        destroy_on_complete: bool,
    },
    CancelSurfaceAnimation {
        id: SurfaceId,
    },
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameAction {
    Wait,
    Animate,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderContext {
    /// Surface width in logical pixels.
    pub width: u32,
    /// Surface height in logical pixels.
    pub height: u32,
    /// Integer Wayland buffer scale used for the current frame.
    pub scale: u32,
    /// Backing buffer width in physical pixels.
    pub buffer_width: u32,
    /// Backing buffer height in physical pixels.
    pub buffer_height: u32,
    pub frame_time: Option<u32>,
}

pub trait Renderer: 'static {
    fn draw(&mut self, canvas: &mut Canvas<'_>, context: RenderContext) -> FrameAction;

    fn draw_surface(
        &mut self,
        _: SurfaceId,
        canvas: &mut Canvas<'_>,
        context: RenderContext,
    ) -> FrameAction {
        self.draw(canvas, context)
    }

    fn closed(&mut self) {}

    fn closed_surface(&mut self, _: SurfaceId) {
        self.closed();
    }
}

impl<F> Renderer for F
where
    F: for<'borrow, 'canvas> FnMut(&'borrow mut Canvas<'canvas>, RenderContext) -> FrameAction
        + 'static,
{
    fn draw(&mut self, canvas: &mut Canvas<'_>, context: RenderContext) -> FrameAction {
        self(canvas, context)
    }
}
