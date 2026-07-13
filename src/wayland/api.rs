use super::*;
use crate::animation::Animation;
use crate::input::KeyboardEvent;
use crate::ui::Bounds;
use std::sync::mpsc::{self, Receiver, SendError, Sender};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub type RenderSender = calloop_channel::Sender<RenderCommand>;
pub type RenderReceiver = calloop_channel::Channel<RenderCommand>;
pub type RenderSendError = SendError<RenderCommand>;

pub fn channel() -> (RenderSender, RenderReceiver) {
    calloop_channel::channel()
}

pub fn controller() -> (RenderController, RenderReceiver) {
    let (sender, receiver) = channel();
    (RenderController::new(sender), receiver)
}

#[derive(Clone)]
pub struct RenderController {
    sender: RenderSender,
}

impl RenderController {
    pub fn new(sender: RenderSender) -> Self {
        Self { sender }
    }

    pub fn sender(&self) -> &RenderSender {
        &self.sender
    }

    pub fn into_sender(self) -> RenderSender {
        self.sender
    }

    pub fn send(&self, command: RenderCommand) -> std::result::Result<(), RenderSendError> {
        self.sender.send(command)
    }

    pub fn redraw(&self) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::Redraw)
    }

    pub fn redraw_region(&self, repaint: Bounds) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::RedrawRegion { repaint })
    }

    pub fn resize(&self, width: u32, height: u32) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::Resize { width, height })
    }

    pub fn set_margins(&self, margins: Margins) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::SetMargins(margins))
    }

    pub fn set_anchor(&self, anchor: Anchor) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::SetAnchor(anchor))
    }

    pub fn create_surface(
        &self,
        id: SurfaceId,
        options: LayerOptions,
    ) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::CreateSurface { id, options })
    }

    pub fn destroy_surface(&self, id: SurfaceId) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::DestroySurface { id })
    }

    pub fn redraw_surface(&self, id: SurfaceId) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::RedrawSurface { id })
    }

    pub fn redraw_surface_region(
        &self,
        id: SurfaceId,
        repaint: Bounds,
    ) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::RedrawSurfaceRegion { id, repaint })
    }

    pub fn resize_surface(
        &self,
        id: SurfaceId,
        width: u32,
        height: u32,
    ) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::ResizeSurface { id, width, height })
    }

    pub fn set_surface_margins(
        &self,
        id: SurfaceId,
        margins: Margins,
    ) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::SetSurfaceMargins { id, margins })
    }

    pub fn set_surface_anchor(
        &self,
        id: SurfaceId,
        anchor: Anchor,
    ) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::SetSurfaceAnchor { id, anchor })
    }

    pub fn set_surface_layer(
        &self,
        id: SurfaceId,
        layer: Layer,
    ) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::SetSurfaceLayer { id, layer })
    }

    pub fn animate_surface_margins(
        &self,
        id: SurfaceId,
        to: Margins,
        animation: Animation,
        destroy_on_complete: bool,
    ) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::AnimateSurfaceMargins {
            id,
            to,
            animation,
            destroy_on_complete,
        })
    }

    pub fn animate_surface_size(
        &self,
        id: SurfaceId,
        width: u32,
        height: u32,
        animation: Animation,
        destroy_on_complete: bool,
    ) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::AnimateSurfaceSize {
            id,
            width,
            height,
            animation,
            destroy_on_complete,
        })
    }

    pub fn cancel_surface_animation(
        &self,
        id: SurfaceId,
    ) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::CancelSurfaceAnimation { id })
    }

    pub fn request_outputs(
        &self,
    ) -> std::result::Result<Receiver<Vec<RenderOutput>>, RenderSendError> {
        let (reply, receiver) = mpsc::channel();
        self.send(RenderCommand::RequestOutputs { reply })?;
        Ok(receiver)
    }

    pub fn request_surface_state(
        &self,
        id: SurfaceId,
    ) -> std::result::Result<Receiver<Option<RenderSurfaceState>>, RenderSendError> {
        let (reply, receiver) = mpsc::channel();
        self.send(RenderCommand::RequestSurfaceState { id, reply })?;
        Ok(receiver)
    }

    pub fn exit(&self) -> std::result::Result<(), RenderSendError> {
        self.send(RenderCommand::Exit)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceId(pub u64);

pub const DEFAULT_SURFACE_ID: SurfaceId = SurfaceId(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderOutput {
    pub id: u32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub make: String,
    pub model: String,
    pub logical_position: Option<(i32, i32)>,
    pub logical_size: Option<(i32, i32)>,
    pub scale_factor: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSurfaceState {
    pub id: SurfaceId,
    pub configured: bool,
    pub width: u32,
    pub height: u32,
    pub output: Option<OutputTarget>,
    pub layer: Layer,
    pub anchor: Anchor,
    pub margins: Margins,
    pub scale: u32,
    pub animating: bool,
    pub frame_pending: bool,
}

#[derive(Debug, Clone)]
pub enum RenderCommand {
    Redraw,
    RedrawRegion {
        repaint: Bounds,
    },
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
    RedrawSurfaceRegion {
        id: SurfaceId,
        repaint: Bounds,
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
    AnimateSurfaceSize {
        id: SurfaceId,
        width: u32,
        height: u32,
        animation: Animation,
        destroy_on_complete: bool,
    },
    CancelSurfaceAnimation {
        id: SurfaceId,
    },
    RequestOutputs {
        reply: Sender<Vec<RenderOutput>>,
    },
    RequestSurfaceState {
        id: SurfaceId,
        reply: Sender<Option<RenderSurfaceState>>,
    },
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    Ignore,
    Redraw,
    RedrawRegion(Bounds),
    Animate,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButtonState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerEventKind {
    Enter,
    Leave,
    Motion,
    Button {
        button: u32,
        state: PointerButtonState,
    },
    Axis {
        horizontal: PointerAxis,
        vertical: PointerAxis,
        source: Option<PointerAxisSource>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerAxis {
    pub absolute: f64,
    pub discrete: i32,
    pub stopped: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerAxisSource {
    Wheel,
    Finger,
    Continuous,
    WheelTilt,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerEvent {
    pub surface: SurfaceId,
    pub x: f64,
    pub y: f64,
    pub time: Option<u32>,
    pub serial: Option<u32>,
    pub kind: PointerEventKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameAction {
    Wait,
    Animate,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderContext {
    /// Surface width in logical pixels.
    pub width: u32,
    /// Surface height in logical pixels.
    pub height: u32,
    /// Integer Wayland buffer scale used for the current frame.
    pub scale: u32,
    /// Effective logical-to-buffer scale used for painting.
    pub scale_factor: f32,
    /// Backing buffer width in physical pixels.
    pub buffer_width: u32,
    /// Backing buffer height in physical pixels.
    pub buffer_height: u32,
    pub frame_time: Option<u32>,
    /// Logical surface region that needs repainting, when the runtime can safely
    /// preserve pixels outside this region.
    pub repaint: Option<Bounds>,
}

pub trait CanvasRenderer: 'static {
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

    fn configured_surface(&mut self, _: SurfaceId, _: u32, _: u32) {}

    fn idle(&mut self) {}

    fn idle_surface(&mut self, _: SurfaceId) {
        self.idle();
    }

    fn input_regions(&mut self, _: SurfaceId, _: RenderContext) -> Option<Vec<Bounds>> {
        None
    }

    fn output_added(&mut self, _: RenderOutput) {}

    fn output_updated(&mut self, _: RenderOutput) {}

    fn output_removed(&mut self, _: RenderOutput) {}

    fn pointer_event(&mut self, _: PointerEvent) -> InputAction {
        InputAction::Ignore
    }

    fn keyboard_event(&mut self, _: KeyboardEvent) -> InputAction {
        InputAction::Ignore
    }
}

impl<F> CanvasRenderer for F
where
    F: for<'borrow, 'canvas> FnMut(&'borrow mut Canvas<'canvas>, RenderContext) -> FrameAction
        + 'static,
{
    fn draw(&mut self, canvas: &mut Canvas<'_>, context: RenderContext) -> FrameAction {
        self(canvas, context)
    }
}
