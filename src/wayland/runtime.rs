use super::placement::apply_placement;
use super::*;

mod queries;
mod surface;
use crate::animation::{Animation, lerp_i32, lerp_u32};
use crate::ui::Bounds;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
pub(in crate::wayland) use surface::*;

const MAX_REUSABLE_BUFFER_BYTES: usize = 4 * 1024 * 1024;
const MAX_SPARE_FRAMES: usize = 2;
const MAX_DAMAGE_HISTORY: usize = MAX_SPARE_FRAMES + 2;
const FRACTIONAL_SCALE_DENOMINATOR: f32 = 120.0;

pub fn run<R>(renderer: R, options: LayerOptions, receiver: RenderReceiver) -> Result<()>
where
    R: CanvasRenderer + 'static,
{
    run_inner(
        renderer,
        Some((DEFAULT_SURFACE_ID, options)),
        Some(receiver),
        true,
    )
}

pub fn run_surfaces<R>(renderer: R, receiver: RenderReceiver) -> Result<()>
where
    R: CanvasRenderer + 'static,
{
    run_inner(renderer, None, Some(receiver), false)
}

pub(in crate::wayland) fn run_inner<R>(
    renderer: R,
    initial_surface: Option<(SurfaceId, LayerOptions)>,
    receiver: Option<RenderReceiver>,
    exit_when_all_surfaces_closed: bool,
) -> Result<()>
where
    R: CanvasRenderer + 'static,
{
    let connection = Connection::connect_to_env()?;
    let (globals, event_queue) = registry_queue_init(&connection)?;
    let qh = event_queue.handle();

    let mut event_loop = EventLoop::<State<R>>::try_new()?;
    let loop_handle = event_loop.handle();
    WaylandSource::new(connection.clone(), event_queue).insert(loop_handle.clone())?;

    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;
    let viewporter = globals.bind(&qh, 1..=1, GlobalData).ok();
    let fractional_scale_manager = globals.bind(&qh, 1..=1, GlobalData).ok();
    let output_state = OutputState::new(&globals, &qh);
    let seat_state = SeatState::new(&globals, &qh);

    let mut surfaces = BTreeMap::new();
    if let Some((id, options)) = initial_surface {
        if let Some(output) = resolve_output_target(&output_state, options.output.as_ref()) {
            let scale = output_initial_scale(&output_state, output.as_ref());
            surfaces.insert(
                id,
                RenderSurface::new(
                    &qh,
                    &compositor,
                    &layer_shell,
                    viewporter.as_ref(),
                    fractional_scale_manager.as_ref(),
                    output.as_ref(),
                    &options,
                    scale,
                ),
            );
        }
    }
    let mut state = State {
        registry_state: RegistryState::new(&globals),
        output_state,
        seat_state,
        compositor,
        layer_shell,
        shm,
        viewporter,
        fractional_scale_manager,
        surfaces,
        pointer: None,
        keyboard: None,
        keyboard_focus: None,
        renderer,
        running: true,
        exit_when_all_surfaces_closed,
        retired_frames: Vec::new(),
    };

    if let Some(receiver) = receiver {
        let commands_qh = qh.clone();
        loop_handle.insert_source(receiver, move |event, _, state| {
            if let ChannelEvent::Msg(command) = event {
                state.handle_command(&commands_qh, command);
            }
        })?;
    }

    while state.running {
        event_loop.dispatch(None, &mut state)?;
        state.collect_released_frames();
    }

    Ok(())
}

pub(in crate::wayland) struct State<R> {
    pub(in crate::wayland) registry_state: RegistryState,
    pub(in crate::wayland) output_state: OutputState,
    pub(in crate::wayland) seat_state: SeatState,
    compositor: CompositorState,
    layer_shell: LayerShell,
    pub(in crate::wayland) shm: Shm,
    pub(in crate::wayland) viewporter: Option<WpViewporter>,
    pub(in crate::wayland) fractional_scale_manager: Option<WpFractionalScaleManagerV1>,
    pub(in crate::wayland) surfaces: BTreeMap<SurfaceId, RenderSurface>,
    pub(in crate::wayland) pointer: Option<wl_pointer::WlPointer>,
    pub(in crate::wayland) keyboard: Option<wl_keyboard::WlKeyboard>,
    pub(in crate::wayland) keyboard_focus: Option<SurfaceId>,
    pub(in crate::wayland) renderer: R,
    pub(in crate::wayland) running: bool,
    exit_when_all_surfaces_closed: bool,
    retired_frames: Vec<Frame>,
}

impl<R> State<R>
where
    R: CanvasRenderer,
{
    fn handle_command(&mut self, qh: &QueueHandle<Self>, command: RenderCommand) {
        match command {
            RenderCommand::Redraw => self.draw(qh, DEFAULT_SURFACE_ID, None, None),
            RenderCommand::RedrawRegion { repaint } => {
                self.draw(qh, DEFAULT_SURFACE_ID, None, Some(repaint))
            }
            RenderCommand::Resize { width, height } => {
                self.resize_surface(qh, DEFAULT_SURFACE_ID, width, height)
            }
            RenderCommand::SetMargins(margins) => {
                self.set_surface_margins(DEFAULT_SURFACE_ID, margins)
            }
            RenderCommand::SetAnchor(anchor) => self.set_surface_anchor(DEFAULT_SURFACE_ID, anchor),
            RenderCommand::CreateSurface { id, options } => self.create_surface(qh, id, options),
            RenderCommand::DestroySurface { id } => self.destroy_surface(id),
            RenderCommand::RedrawSurface { id } => self.draw(qh, id, None, None),
            RenderCommand::RedrawSurfaceRegion { id, repaint } => {
                self.draw(qh, id, None, Some(repaint))
            }
            RenderCommand::ResizeSurface { id, width, height } => {
                self.resize_surface(qh, id, width, height)
            }
            RenderCommand::SetSurfaceMargins { id, margins } => {
                self.set_surface_margins(id, margins)
            }
            RenderCommand::SetSurfaceAnchor { id, anchor } => self.set_surface_anchor(id, anchor),
            RenderCommand::SetSurfaceLayer { id, layer } => self.set_surface_layer(id, layer),
            RenderCommand::AnimateSurfaceMargins {
                id,
                to,
                animation,
                destroy_on_complete,
            } => self.animate_surface_margins(qh, id, to, animation, destroy_on_complete),
            RenderCommand::AnimateSurfaceSize {
                id,
                width,
                height,
                animation,
                destroy_on_complete,
            } => self.animate_surface_size(qh, id, width, height, animation, destroy_on_complete),
            RenderCommand::CancelSurfaceAnimation { id } => self.cancel_surface_animation(id),
            RenderCommand::RequestOutputs { reply } => {
                let _ = reply.send(self.render_outputs());
            }
            RenderCommand::RequestSurfaceState { id, reply } => {
                let _ = reply.send(self.surface_state(id));
            }
            RenderCommand::Exit => self.running = false,
        }
    }

    fn create_surface(&mut self, qh: &QueueHandle<Self>, id: SurfaceId, options: LayerOptions) {
        let Some(output) = self.resolve_output_target(options.output.as_ref()) else {
            return;
        };
        let scale = self.output_initial_scale(output.as_ref());

        if let Some(surface) = self.surfaces.get_mut(&id) {
            if surface.output == options.output {
                surface.update_options(options);
                surface.layer.commit();
                return;
            }
            self.remove_surface(id);
        }

        self.surfaces.insert(
            id,
            RenderSurface::new(
                qh,
                &self.compositor,
                &self.layer_shell,
                self.viewporter.as_ref(),
                self.fractional_scale_manager.as_ref(),
                output.as_ref(),
                &options,
                scale,
            ),
        );
    }

    fn resolve_output_target(
        &self,
        target: Option<&OutputTarget>,
    ) -> Option<Option<wl_output::WlOutput>> {
        resolve_output_target(&self.output_state, target)
    }

    fn output_initial_scale(&self, output: Option<&wl_output::WlOutput>) -> u32 {
        output_initial_scale(&self.output_state, output)
    }

    fn destroy_surface(&mut self, id: SurfaceId) {
        self.remove_surface(id);
        self.maybe_exit_after_surface_removal();
    }

    fn resize_surface(&mut self, qh: &QueueHandle<Self>, id: SurfaceId, width: u32, height: u32) {
        let Some(surface) = self.surfaces.get_mut(&id) else {
            return;
        };
        surface.cancel_animation();
        surface.set_size(width, height);
        if surface.configured {
            self.draw(qh, id, None, None);
        } else {
            surface.layer.commit();
        }
    }

    fn set_surface_margins(&mut self, id: SurfaceId, margins: Margins) {
        let Some(surface) = self.surfaces.get_mut(&id) else {
            return;
        };
        surface.set_margins(margins);
        surface.layer.commit();
    }

    fn set_surface_anchor(&mut self, id: SurfaceId, anchor: Anchor) {
        let Some(surface) = self.surfaces.get_mut(&id) else {
            return;
        };
        surface.set_anchor(anchor);
        surface.layer.commit();
    }

    fn set_surface_layer(&mut self, id: SurfaceId, layer: Layer) {
        let Some(surface) = self.surfaces.get_mut(&id) else {
            return;
        };
        surface.set_layer(layer);
        surface.layer.commit();
    }

    fn animate_surface_margins(
        &mut self,
        qh: &QueueHandle<Self>,
        id: SurfaceId,
        to: Margins,
        animation: Animation,
        destroy_on_complete: bool,
    ) {
        let result = {
            let Some(surface) = self.surfaces.get_mut(&id) else {
                return;
            };
            let result = surface.start_margins_animation(to, animation, destroy_on_complete);
            surface.layer.commit();
            result
        };

        self.handle_surface_animation_start(qh, id, result);
    }

    fn animate_surface_size(
        &mut self,
        qh: &QueueHandle<Self>,
        id: SurfaceId,
        width: u32,
        height: u32,
        animation: Animation,
        destroy_on_complete: bool,
    ) {
        let result = {
            let Some(surface) = self.surfaces.get_mut(&id) else {
                return;
            };
            let result =
                surface.start_size_animation(width, height, animation, destroy_on_complete);
            surface.layer.commit();
            result
        };

        self.handle_surface_animation_start(qh, id, result);
    }

    fn handle_surface_animation_start(
        &mut self,
        qh: &QueueHandle<Self>,
        id: SurfaceId,
        result: SurfaceAnimationStart,
    ) {
        match result {
            SurfaceAnimationStart::Animate => {
                if let Some(surface) = self.surfaces.get_mut(&id) {
                    surface.request_frame(qh, None);
                }
            }
            SurfaceAnimationStart::Complete => {
                if self
                    .surfaces
                    .get(&id)
                    .is_some_and(|surface| surface.configured)
                {
                    self.draw(qh, id, None, None);
                }
            }
            SurfaceAnimationStart::Destroy => {
                self.remove_surface(id);
                self.maybe_exit_after_surface_removal();
            }
        }
    }

    fn cancel_surface_animation(&mut self, id: SurfaceId) {
        let Some(surface) = self.surfaces.get_mut(&id) else {
            return;
        };
        surface.cancel_animation();
    }

    pub(in crate::wayland) fn draw(
        &mut self,
        qh: &QueueHandle<Self>,
        id: SurfaceId,
        frame_time: Option<u32>,
        repaint: Option<Bounds>,
    ) {
        let (animation, draw_result) = {
            let Some(surface) = self.surfaces.get_mut(&id) else {
                return;
            };
            if !surface.configured || surface.width == 0 || surface.height == 0 {
                return;
            }

            let animation = frame_time
                .map(|time| surface.advance_animation(time))
                .unwrap_or(SurfaceAnimationFrame::Idle);
            let animation_next_frame = matches!(animation, SurfaceAnimationFrame::Animate);
            let draw_result = surface.draw(
                qh,
                &self.shm,
                &self.compositor,
                &mut self.renderer,
                id,
                frame_time,
                repaint,
                animation_next_frame,
            );
            (animation, draw_result)
        };

        let mut idle_surface = false;
        let mut trim_idle_memory = false;
        match draw_result {
            DrawResult::Drawn { next_frame } => {
                if matches!(animation, SurfaceAnimationFrame::Destroy) {
                    self.remove_surface(id);
                    self.maybe_exit_after_surface_removal();
                } else if let Some(surface) = self.surfaces.get_mut(&id) {
                    if next_frame || matches!(animation, SurfaceAnimationFrame::Animate) {
                        surface.retain_animation_buffers();
                    } else {
                        surface.release_idle_buffers();
                        trim_idle_memory = surface.released_idle_memory();
                        idle_surface = true;
                    }
                }
            }
            DrawResult::Exit => {
                self.remove_surface(id);
                self.maybe_exit_after_surface_removal();
            }
        }
        if idle_surface {
            self.renderer.idle_surface(id);
            crate::memory::trim_free_heap_pages();
        } else if trim_idle_memory {
            crate::memory::trim_free_heap_pages();
        }
    }

    pub(in crate::wayland) fn handle_input_action(
        &mut self,
        qh: &QueueHandle<Self>,
        id: SurfaceId,
        action: InputAction,
    ) {
        match action {
            InputAction::Ignore => {}
            InputAction::Redraw => {
                if let Some(surface) = self.surfaces.get_mut(&id) {
                    surface.request_frame(qh, None);
                }
            }
            InputAction::RedrawRegion(repaint) => {
                if let Some(surface) = self.surfaces.get_mut(&id) {
                    surface.request_frame(qh, Some(repaint));
                }
            }
            InputAction::Animate => {
                if let Some(surface) = self.surfaces.get_mut(&id) {
                    surface.request_frame(qh, None);
                }
            }
            InputAction::Exit => {
                self.remove_surface(id);
                self.maybe_exit_after_surface_removal();
            }
        }
    }

    pub(in crate::wayland) fn surface_id_for_wl_surface(
        &self,
        wl_surface: &wl_surface::WlSurface,
    ) -> Option<SurfaceId> {
        self.surfaces
            .iter()
            .find_map(|(id, surface)| (surface.layer.wl_surface() == wl_surface).then_some(*id))
    }

    pub(in crate::wayland) fn surface_id_for_layer(
        &self,
        layer: &LayerSurface,
    ) -> Option<SurfaceId> {
        self.surfaces
            .iter()
            .find_map(|(id, surface)| (&surface.layer == layer).then_some(*id))
    }

    pub(in crate::wayland) fn maybe_exit_after_surface_removal(&mut self) {
        if self.exit_when_all_surfaces_closed && self.surfaces.is_empty() {
            self.running = false;
        }
    }

    pub(in crate::wayland) fn remove_surface(&mut self, id: SurfaceId) {
        let Some(mut surface) = self.surfaces.remove(&id) else {
            return;
        };
        self.retired_frames.extend(surface.take_active_frames());
        self.renderer.closed_surface(id);
    }
}
