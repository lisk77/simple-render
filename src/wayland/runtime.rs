use super::placement::apply_placement;
use super::*;
use crate::animation::{Animation, lerp_i32, lerp_u32};
use crate::ui::Bounds;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

const MAX_REUSABLE_BUFFER_BYTES: usize = 4 * 1024 * 1024;
const MAX_SPARE_FRAMES: usize = 2;
const MAX_DAMAGE_HISTORY: usize = MAX_SPARE_FRAMES + 2;
const FRACTIONAL_SCALE_DENOMINATOR: f32 = 120.0;

pub fn run<R>(renderer: R, options: LayerOptions, receiver: RenderReceiver) -> Result<()>
where
    R: Renderer + 'static,
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
    R: Renderer + 'static,
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
    R: Renderer + 'static,
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
    R: Renderer,
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

    pub(in crate::wayland) fn render_output(
        &self,
        output: &wl_output::WlOutput,
    ) -> Option<RenderOutput> {
        self.output_state.info(output).map(|info| RenderOutput {
            id: info.id,
            name: info.name,
            description: info.description,
            make: info.make,
            model: info.model,
            logical_position: info.logical_position,
            logical_size: info.logical_size,
            scale_factor: info.scale_factor,
        })
    }

    fn render_outputs(&self) -> Vec<RenderOutput> {
        self.output_state
            .outputs()
            .filter_map(|output| self.render_output(&output))
            .collect()
    }

    fn surface_state(&self, id: SurfaceId) -> Option<RenderSurfaceState> {
        self.surfaces.get(&id).map(|surface| surface.state(id))
    }

    fn collect_released_frames(&mut self) {
        let mut released_idle_memory = false;
        for surface in self.surfaces.values_mut() {
            surface.collect_released_frames();
            released_idle_memory |= surface.released_idle_memory();
        }
        let retired_count = self.retired_frames.len();
        self.retired_frames.retain(|frame| !frame.released());
        if self.retired_frames.len() < retired_count || released_idle_memory {
            crate::memory::trim_free_heap_pages();
        }
    }
}

fn resolve_output_target(
    output_state: &OutputState,
    target: Option<&OutputTarget>,
) -> Option<Option<wl_output::WlOutput>> {
    match target {
        None | Some(OutputTarget::Any) => Some(None),
        Some(OutputTarget::Id(target_id)) => output_state
            .outputs()
            .find(|output| {
                output_state
                    .info(output)
                    .is_some_and(|info| info.id == *target_id)
            })
            .map(Some),
        Some(OutputTarget::Name(target_name)) => output_state
            .outputs()
            .find(|output| {
                output_state
                    .info(output)
                    .and_then(|info| info.name)
                    .is_some_and(|name| name == *target_name)
            })
            .map(Some),
    }
}

fn output_initial_scale(output_state: &OutputState, output: Option<&wl_output::WlOutput>) -> u32 {
    output
        .and_then(|output| output_state.info(output))
        .map(|info| info.scale_factor.max(1) as u32)
        .unwrap_or(1)
}

#[derive(Clone)]
pub(in crate::wayland) struct FractionalScaleSurface {
    pub(in crate::wayland) surface: wl_surface::WlSurface,
}

struct SurfaceAnimation {
    animation: Animation,
    started_at: Option<u32>,
    target: SurfaceAnimationTarget,
    destroy_on_complete: bool,
}

#[derive(Clone, Copy)]
enum SurfaceAnimationTarget {
    Margins {
        from: Margins,
        to: Margins,
    },
    Size {
        from_width: u32,
        from_height: u32,
        to_width: u32,
        to_height: u32,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SurfaceAnimationStart {
    Animate,
    Complete,
    Destroy,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SurfaceAnimationFrame {
    Idle,
    Animate,
    Complete,
    Destroy,
}

impl Margins {
    fn lerp(self, to: Self, progress: f32) -> Self {
        Self {
            top: lerp_i32(self.top, to.top, progress),
            right: lerp_i32(self.right, to.right, progress),
            bottom: lerp_i32(self.bottom, to.bottom, progress),
            left: lerp_i32(self.left, to.left, progress),
        }
    }
}

pub(in crate::wayland) struct RenderSurface {
    pub(in crate::wayland) layer: LayerSurface,
    viewport: Option<WpViewport>,
    _fractional_scale: Option<WpFractionalScaleV1>,
    pub(in crate::wayland) configured: bool,
    pub(in crate::wayland) width: u32,
    pub(in crate::wayland) height: u32,
    pub(in crate::wayland) output: Option<OutputTarget>,
    layer_kind: Layer,
    scale: u32,
    fractional_scale_factor: Option<u32>,
    anchor: Anchor,
    margins: Margins,
    animation: Option<SurfaceAnimation>,
    pool: Option<Rc<RefCell<SlotPool>>>,
    input_regions: Option<Vec<Bounds>>,
    input_regions_set: bool,
    frame: Option<Frame>,
    retired_frames: Vec<Frame>,
    spare_frames: Vec<Frame>,
    damage_history: Vec<(u64, Bounds)>,
    frame_sequence: u64,
    pending_repaint: Option<Bounds>,
    retain_spare_frames: bool,
    released_idle_memory: bool,
    pub(in crate::wayland) frame_pending: bool,
}

impl RenderSurface {
    fn new<R: Renderer>(
        qh: &QueueHandle<State<R>>,
        compositor: &CompositorState,
        layer_shell: &LayerShell,
        viewporter: Option<&WpViewporter>,
        fractional_scale_manager: Option<&WpFractionalScaleManagerV1>,
        output: Option<&wl_output::WlOutput>,
        options: &LayerOptions,
        scale: u32,
    ) -> Self {
        let wl_surface = compositor.create_surface(qh);
        let layer = layer_shell.create_layer_surface(
            qh,
            wl_surface,
            options.layer.into_sctk(),
            Some(options.namespace.as_str()),
            output,
        );
        layer.set_keyboard_interactivity(options.keyboard_interactivity.into_sctk());
        layer.set_exclusive_zone(options.exclusive_zone);
        layer.set_size(options.width, options.height);
        apply_placement(&layer, options.anchor, options.margins);
        layer.commit();
        let viewport = viewporter
            .map(|viewporter| viewporter.get_viewport(layer.wl_surface(), qh, GlobalData));
        let fractional_scale = fractional_scale_manager.and_then(|manager| {
            viewport.as_ref().map(|_| {
                manager.get_fractional_scale(
                    layer.wl_surface(),
                    qh,
                    FractionalScaleSurface {
                        surface: layer.wl_surface().clone(),
                    },
                )
            })
        });

        Self {
            layer,
            viewport,
            _fractional_scale: fractional_scale,
            configured: false,
            width: options.width,
            height: options.height,
            output: options.output.clone(),
            layer_kind: options.layer,
            scale: scale.max(1),
            fractional_scale_factor: None,
            anchor: options.anchor,
            margins: options.margins,
            animation: None,
            pool: None,
            input_regions: None,
            input_regions_set: false,
            frame: None,
            retired_frames: Vec::new(),
            spare_frames: Vec::new(),
            damage_history: Vec::new(),
            frame_sequence: 0,
            pending_repaint: None,
            retain_spare_frames: false,
            released_idle_memory: false,
            frame_pending: false,
        }
    }

    fn set_size(&mut self, width: u32, height: u32) {
        if self.width != width || self.height != height {
            self.clear_reusable_buffers();
            self.pending_repaint = None;
        }
        self.width = width;
        self.height = height;
        self.layer.set_size(width, height);
    }

    pub(in crate::wayland) fn set_scale(&mut self, scale: u32) -> bool {
        let scale = scale.max(1);
        let previous = self.scale_factor();
        if self.scale == scale {
            return false;
        }

        self.clear_reusable_buffers();
        self.pending_repaint = None;
        self.scale = scale;
        if (self.scale_factor() - previous).abs() < f32::EPSILON {
            return false;
        }
        true
    }

    pub(in crate::wayland) fn set_fractional_scale(&mut self, scale: u32) -> bool {
        let scale = scale.max(1);
        if self.fractional_scale_factor == Some(scale) {
            return false;
        }

        self.clear_reusable_buffers();
        self.pending_repaint = None;
        self.fractional_scale_factor = Some(scale);
        true
    }

    fn set_margins(&mut self, margins: Margins) {
        self.margins = margins;
        self.animation = None;
        apply_placement(&self.layer, self.anchor, self.margins);
    }

    fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
        apply_placement(&self.layer, self.anchor, self.margins);
    }

    fn set_layer(&mut self, layer: Layer) {
        self.layer_kind = layer;
        self.layer.set_layer(layer.into_sctk());
    }

    fn update_options(&mut self, options: LayerOptions) {
        self.layer
            .set_keyboard_interactivity(options.keyboard_interactivity.into_sctk());
        self.layer.set_exclusive_zone(options.exclusive_zone);
        self.set_layer(options.layer);
        self.set_size(options.width, options.height);
        self.anchor = options.anchor;
        self.margins = options.margins;
        self.animation = None;
        apply_placement(&self.layer, self.anchor, self.margins);
    }

    fn start_margins_animation(
        &mut self,
        to: Margins,
        animation: Animation,
        destroy_on_complete: bool,
    ) -> SurfaceAnimationStart {
        if animation.duration_ms == 0 {
            self.margins = to;
            self.animation = None;
            apply_placement(&self.layer, self.anchor, self.margins);
            return if destroy_on_complete {
                SurfaceAnimationStart::Destroy
            } else {
                SurfaceAnimationStart::Complete
            };
        }

        self.animation = Some(SurfaceAnimation {
            animation,
            started_at: None,
            target: SurfaceAnimationTarget::Margins {
                from: self.margins,
                to,
            },
            destroy_on_complete,
        });
        SurfaceAnimationStart::Animate
    }

    fn start_size_animation(
        &mut self,
        width: u32,
        height: u32,
        animation: Animation,
        destroy_on_complete: bool,
    ) -> SurfaceAnimationStart {
        if animation.duration_ms == 0 {
            self.set_size(width, height);
            self.animation = None;
            return if destroy_on_complete {
                SurfaceAnimationStart::Destroy
            } else {
                SurfaceAnimationStart::Complete
            };
        }

        self.animation = Some(SurfaceAnimation {
            animation,
            started_at: None,
            target: SurfaceAnimationTarget::Size {
                from_width: self.width,
                from_height: self.height,
                to_width: width,
                to_height: height,
            },
            destroy_on_complete,
        });
        SurfaceAnimationStart::Animate
    }

    fn cancel_animation(&mut self) {
        self.animation = None;
    }

    fn buffer_scale(&self) -> u32 {
        if self.viewport.is_some() {
            1
        } else {
            self.scale.max(1)
        }
    }

    fn scale_factor(&self) -> f32 {
        self.fractional_scale_factor
            .map(|scale| scale as f32 / FRACTIONAL_SCALE_DENOMINATOR)
            .unwrap_or_else(|| self.scale.max(1) as f32)
            .max(1.0)
    }

    fn advance_animation(&mut self, time: u32) -> SurfaceAnimationFrame {
        let Some(mut animation) = self.animation.take() else {
            return SurfaceAnimationFrame::Idle;
        };

        let started_at = *animation.started_at.get_or_insert(time);
        let frame = animation.animation.frame(time.saturating_sub(started_at));
        match animation.target {
            SurfaceAnimationTarget::Margins { from, to } => {
                self.margins = from.lerp(to, frame.progress);
                apply_placement(&self.layer, self.anchor, self.margins);
            }
            SurfaceAnimationTarget::Size {
                from_width,
                from_height,
                to_width,
                to_height,
            } => {
                self.set_size(
                    lerp_u32(from_width, to_width, frame.progress),
                    lerp_u32(from_height, to_height, frame.progress),
                );
            }
        }

        if frame.complete {
            if animation.destroy_on_complete {
                SurfaceAnimationFrame::Destroy
            } else {
                SurfaceAnimationFrame::Complete
            }
        } else {
            self.animation = Some(animation);
            SurfaceAnimationFrame::Animate
        }
    }

    fn draw<R>(
        &mut self,
        qh: &QueueHandle<State<R>>,
        shm: &Shm,
        compositor: &CompositorState,
        renderer: &mut R,
        id: SurfaceId,
        frame_time: Option<u32>,
        repaint: Option<Bounds>,
        animation_next_frame: bool,
    ) -> DrawResult
    where
        R: Renderer,
    {
        if repaint.is_none() {
            self.pending_repaint = None;
        }
        let dimensions = BufferDimensions::new(
            self.width,
            self.height,
            self.buffer_scale(),
            self.scale_factor(),
        );
        self.set_input_regions_if_changed(
            compositor,
            renderer.input_regions(id, dimensions.context(frame_time, None)),
        );
        let (frame, action, damage) =
            self.draw_frame(shm, renderer, id, frame_time, dimensions, repaint);

        let next_frame = matches!(action, FrameAction::Animate) || animation_next_frame;
        if next_frame {
            self.pending_repaint = union_optional_bounds(
                self.pending_repaint,
                damage.map(|damage| dimensions.logical_damage(damage)),
            );
        }
        if let Some(damage) = damage {
            self.layer.wl_surface().damage_buffer(
                damage.x as i32,
                damage.y as i32,
                damage.width as i32,
                damage.height as i32,
            );
        }
        if let Some(viewport) = &self.viewport {
            let _ = self.layer.set_buffer_scale(1);
            viewport.set_destination(self.width as i32, self.height as i32);
        } else {
            let _ = self.layer.set_buffer_scale(self.scale);
        }
        if next_frame {
            self.request_frame_for_commit(qh, None);
        }
        let frame = self.record_frame_damage(frame, dimensions, damage);
        frame.attach_to(self.layer.wl_surface());
        self.layer.commit();

        if matches!(action, FrameAction::Animate) {
            self.retain_animation_buffers();
        }
        if let Some(retired_frame) = self.frame.replace(frame) {
            self.retire_frame(retired_frame);
        }

        match action {
            FrameAction::Wait => DrawResult::Drawn { next_frame },
            FrameAction::Animate => DrawResult::Drawn { next_frame },
            FrameAction::Exit => DrawResult::Exit,
        }
    }

    fn set_input_regions_if_changed(
        &mut self,
        compositor: &CompositorState,
        regions: Option<Vec<Bounds>>,
    ) {
        if self.input_regions_set && self.input_regions == regions {
            return;
        }

        self.input_regions = regions.clone();
        self.input_regions_set = true;

        let Some(regions) = regions else {
            self.layer.wl_surface().set_input_region(None);
            return;
        };

        let Ok(region) = Region::new(compositor) else {
            return;
        };
        for bounds in regions {
            if bounds.width == 0 || bounds.height == 0 {
                continue;
            }
            region.add(
                bounds.x.min(i32::MAX as u32) as i32,
                bounds.y.min(i32::MAX as u32) as i32,
                bounds.width.min(i32::MAX as u32) as i32,
                bounds.height.min(i32::MAX as u32) as i32,
            );
        }
        self.layer
            .wl_surface()
            .set_input_region(Some(region.wl_region()));
    }

    fn draw_frame<R>(
        &mut self,
        shm: &Shm,
        renderer: &mut R,
        id: SurfaceId,
        frame_time: Option<u32>,
        dimensions: BufferDimensions,
        repaint: Option<Bounds>,
    ) -> (Frame, FrameAction, Option<DamageRect>)
    where
        R: Renderer,
    {
        self.collect_released_frames();
        if let Some(repaint) = repaint {
            if let Some(frame) = self.take_released_current_frame(dimensions) {
                if let Some(frame) =
                    draw_reusable_frame(renderer, id, frame_time, dimensions, frame, Some(repaint))
                {
                    return frame;
                }
            }
        }
        if self.retain_spare_frames
            && let Some(frame) = self.take_released_current_frame(dimensions)
        {
            if let Some(frame) =
                draw_reusable_frame(renderer, id, frame_time, dimensions, frame, None)
            {
                return frame;
            }
        }

        if let Some(frame) = self.take_spare_frame(dimensions) {
            let aged_repaint = self.buffer_age_repaint(frame.sequence, repaint);
            if let Some(frame) =
                draw_reusable_frame(renderer, id, frame_time, dimensions, frame, aged_repaint)
            {
                return frame;
            }
        }

        if dimensions.bytes > MAX_REUSABLE_BUFFER_BYTES && !self.retain_spare_frames {
            self.clear_reusable_buffers();
            return self.draw_transient_frame(shm, renderer, id, frame_time, dimensions, None);
        }

        let pool = self.buffer_pool(shm, dimensions.bytes);
        let (buffer, action, damage) = {
            let mut pool_ref = pool.borrow_mut();
            let (new_buffer, _) = pool_ref
                .create_buffer(
                    dimensions.width as i32,
                    dimensions.height as i32,
                    dimensions.stride as i32,
                    wl_shm::Format::Argb8888,
                )
                .expect("allocate buffer");
            let buffer = FrameBuffer::new(new_buffer, dimensions);
            let pixels = buffer
                .buffer
                .canvas(&mut pool_ref)
                .expect("buffer should be reusable");
            let (action, damage) =
                draw_surface_to_pixels(renderer, id, pixels, dimensions, frame_time, None);
            (buffer, action, damage)
        };

        (
            Frame {
                buffer,
                pool,
                sequence: 0,
            },
            action,
            damage,
        )
    }

    fn draw_transient_frame<R>(
        &mut self,
        shm: &Shm,
        renderer: &mut R,
        id: SurfaceId,
        frame_time: Option<u32>,
        dimensions: BufferDimensions,
        repaint: Option<Bounds>,
    ) -> (Frame, FrameAction, Option<DamageRect>)
    where
        R: Renderer,
    {
        let pool = Rc::new(RefCell::new(
            SlotPool::new(dimensions.bytes, shm).expect("allocate buffer pool"),
        ));
        let (buffer, action, damage) = {
            let mut pool_ref = pool.borrow_mut();
            let (buffer, pixels) = pool_ref
                .create_buffer(
                    dimensions.width as i32,
                    dimensions.height as i32,
                    dimensions.stride as i32,
                    wl_shm::Format::Argb8888,
                )
                .expect("allocate buffer");
            let (action, damage) =
                draw_surface_to_pixels(renderer, id, pixels, dimensions, frame_time, repaint);
            (FrameBuffer::new(buffer, dimensions), action, damage)
        };

        (
            Frame {
                buffer,
                pool,
                sequence: 0,
            },
            action,
            damage,
        )
    }

    fn buffer_pool(&mut self, shm: &Shm, bytes: usize) -> Rc<RefCell<SlotPool>> {
        self.pool
            .get_or_insert_with(|| {
                Rc::new(RefCell::new(
                    SlotPool::new(bytes, shm).expect("allocate buffer pool"),
                ))
            })
            .clone()
    }

    fn request_frame<R: Renderer>(&mut self, qh: &QueueHandle<State<R>>, repaint: Option<Bounds>) {
        self.pending_repaint = union_optional_bounds(self.pending_repaint, repaint);
        self.retain_animation_buffers();
        if self.frame_pending {
            return;
        }

        self.request_frame_for_commit(qh, None);
        self.layer.commit();
    }

    fn request_frame_for_commit<R: Renderer>(
        &mut self,
        qh: &QueueHandle<State<R>>,
        repaint: Option<Bounds>,
    ) {
        self.pending_repaint = union_optional_bounds(self.pending_repaint, repaint);
        self.retain_animation_buffers();
        if self.frame_pending {
            return;
        }

        let surface = self.layer.wl_surface();
        surface.frame(qh, surface.clone());
        self.frame_pending = true;
    }

    fn retain_animation_buffers(&mut self) {
        self.retain_spare_frames = true;
        self.released_idle_memory = false;
    }

    pub(in crate::wayland) fn take_pending_repaint(&mut self) -> Option<Bounds> {
        self.pending_repaint.take()
    }

    fn take_active_frames(&mut self) -> Vec<Frame> {
        let mut frames = Vec::new();
        if let Some(frame) = self.frame.take() {
            frames.push(frame);
        }
        frames.append(&mut self.retired_frames);
        frames
    }

    fn retire_frame(&mut self, frame: Frame) {
        if frame.released() {
            self.keep_spare_frame(frame);
        } else {
            self.retired_frames.push(frame);
        }
    }

    fn collect_released_frames(&mut self) {
        let mut index = 0;
        while index < self.retired_frames.len() {
            if self.retired_frames[index].released() {
                let frame = self.retired_frames.swap_remove(index);
                self.keep_spare_frame(frame);
            } else {
                index += 1;
            }
        }
    }

    fn keep_spare_frame(&mut self, frame: Frame) {
        if !self.retain_spare_frames {
            self.released_idle_memory = true;
            return;
        }
        if self.spare_frames.len() < MAX_SPARE_FRAMES {
            self.spare_frames.push(frame);
        }
    }

    fn take_spare_frame(&mut self, dimensions: BufferDimensions) -> Option<Frame> {
        if dimensions.bytes > MAX_REUSABLE_BUFFER_BYTES && !self.retain_spare_frames {
            return None;
        }
        let index = self
            .spare_frames
            .iter()
            .position(|frame| frame.matches(dimensions))?;
        Some(self.spare_frames.swap_remove(index))
    }

    fn take_released_current_frame(&mut self, dimensions: BufferDimensions) -> Option<Frame> {
        let frame = self.frame.take()?;
        if frame.released() && frame.matches(dimensions) && frame.canvas_available() {
            return Some(frame);
        }
        self.frame = Some(frame);
        None
    }

    fn buffer_age_repaint(&self, sequence: u64, requested: Option<Bounds>) -> Option<Bounds> {
        if self
            .damage_history
            .first()
            .is_some_and(|(oldest_sequence, _)| sequence < *oldest_sequence)
        {
            return None;
        }

        self.damage_history
            .iter()
            .filter(|(damage_sequence, _)| *damage_sequence > sequence)
            .fold(requested, |repaint, (_, damage)| {
                union_optional_bounds(repaint, Some(*damage))
            })
    }

    fn record_frame_damage(
        &mut self,
        mut frame: Frame,
        dimensions: BufferDimensions,
        damage: Option<DamageRect>,
    ) -> Frame {
        self.frame_sequence = self.frame_sequence.saturating_add(1);
        frame.sequence = self.frame_sequence;

        if let Some(damage) = damage {
            self.damage_history
                .push((self.frame_sequence, dimensions.logical_damage(damage)));
            if self.damage_history.len() > MAX_DAMAGE_HISTORY {
                self.damage_history
                    .drain(0..self.damage_history.len() - MAX_DAMAGE_HISTORY);
            }
        }

        frame
    }

    fn clear_reusable_buffers(&mut self) {
        self.spare_frames.clear();
        self.damage_history.clear();
        self.pool = None;
    }

    fn release_idle_buffers(&mut self) {
        self.retain_spare_frames = false;
        if !self.spare_frames.is_empty() || self.pool.is_some() {
            self.released_idle_memory = true;
        }
        self.clear_reusable_buffers();
    }

    fn released_idle_memory(&mut self) -> bool {
        let released = self.released_idle_memory;
        self.released_idle_memory = false;
        released
    }

    fn state(&self, id: SurfaceId) -> RenderSurfaceState {
        RenderSurfaceState {
            id,
            configured: self.configured,
            width: self.width,
            height: self.height,
            output: self.output.clone(),
            layer: self.layer_kind,
            anchor: self.anchor,
            margins: self.margins,
            scale: self.scale,
            animating: self.animation.is_some(),
            frame_pending: self.frame_pending,
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::wayland) struct BufferDimensions {
    pub(in crate::wayland) logical_width: u32,
    pub(in crate::wayland) logical_height: u32,
    pub(in crate::wayland) width: u32,
    pub(in crate::wayland) height: u32,
    pub(in crate::wayland) buffer_scale: u32,
    pub(in crate::wayland) scale_factor: f32,
    pub(in crate::wayland) stride: u32,
    pub(in crate::wayland) bytes: usize,
}

impl BufferDimensions {
    pub(in crate::wayland) fn new(
        logical_width: u32,
        logical_height: u32,
        buffer_scale: u32,
        scale_factor: f32,
    ) -> Self {
        let buffer_scale = buffer_scale.max(1);
        let scale_factor = if scale_factor.is_finite() {
            scale_factor.max(1.0)
        } else {
            buffer_scale as f32
        };
        let width = scaled_buffer_extent(logical_width, scale_factor);
        let height = scaled_buffer_extent(logical_height, scale_factor);
        let stride = width.checked_mul(4).expect("buffer stride overflow");
        let bytes = (stride as usize)
            .checked_mul(height as usize)
            .expect("buffer size overflow");
        Self {
            logical_width,
            logical_height,
            width,
            height,
            buffer_scale,
            scale_factor,
            stride,
            bytes,
        }
    }

    pub(in crate::wayland) fn context(
        self,
        frame_time: Option<u32>,
        repaint: Option<Bounds>,
    ) -> RenderContext {
        RenderContext {
            width: self.logical_width,
            height: self.logical_height,
            scale: self.buffer_scale,
            scale_factor: self.scale_factor,
            buffer_width: self.width,
            buffer_height: self.height,
            frame_time,
            repaint,
        }
    }

    pub(in crate::wayland) fn logical_damage(self, damage: DamageRect) -> Bounds {
        let scale = self.scale_factor.max(1.0);
        let x = (damage.x as f32 / scale).floor() as u32;
        let y = (damage.y as f32 / scale).floor() as u32;
        let right = (damage.right() as f32 / scale).ceil() as u32;
        let bottom = (damage.bottom() as f32 / scale).ceil() as u32;
        Bounds::new(
            x.min(self.logical_width),
            y.min(self.logical_height),
            right.min(self.logical_width).saturating_sub(x),
            bottom.min(self.logical_height).saturating_sub(y),
        )
    }
}

fn scaled_buffer_extent(logical: u32, scale: f32) -> u32 {
    let scaled = (logical as f32 * scale).round();
    assert!(
        scaled.is_finite() && scaled >= 0.0 && scaled <= u32::MAX as f32,
        "scaled buffer extent overflow"
    );
    scaled as u32
}

fn draw_surface_to_pixels<R>(
    renderer: &mut R,
    id: SurfaceId,
    pixels: &mut [u8],
    dimensions: BufferDimensions,
    frame_time: Option<u32>,
    repaint: Option<Bounds>,
) -> (FrameAction, Option<DamageRect>)
where
    R: Renderer,
{
    let mut canvas = Canvas {
        pixels,
        width: dimensions.width,
        height: dimensions.height,
        stride: dimensions.stride,
        scale: dimensions.buffer_scale,
        damage: None,
    };
    let action = renderer.draw_surface(id, &mut canvas, dimensions.context(frame_time, repaint));
    let damage = canvas.damage();
    (action, damage)
}

fn draw_reusable_frame<R>(
    renderer: &mut R,
    id: SurfaceId,
    frame_time: Option<u32>,
    dimensions: BufferDimensions,
    frame: Frame,
    repaint: Option<Bounds>,
) -> Option<(Frame, FrameAction, Option<DamageRect>)>
where
    R: Renderer,
{
    let pool = frame.pool.clone();
    let buffer = frame.buffer;
    let sequence = frame.sequence;
    let (buffer, action, damage) = {
        let mut pool_ref = pool.borrow_mut();
        let pixels = buffer.buffer.canvas(&mut pool_ref)?;
        let (action, damage) =
            draw_surface_to_pixels(renderer, id, pixels, dimensions, frame_time, repaint);
        (buffer, action, damage)
    };

    Some((
        Frame {
            buffer,
            pool,
            sequence,
        },
        action,
        damage,
    ))
}

fn union_optional_bounds(current: Option<Bounds>, next: Option<Bounds>) -> Option<Bounds> {
    match (current, next) {
        (Some(current), Some(next)) => Some(current.union(next)),
        (Some(current), None) => Some(current),
        (None, Some(next)) => Some(next),
        (None, None) => None,
    }
}

enum DrawResult {
    Drawn { next_frame: bool },
    Exit,
}

struct Frame {
    buffer: FrameBuffer,
    pool: Rc<RefCell<SlotPool>>,
    sequence: u64,
}

impl Frame {
    fn attach_to(&self, surface: &wl_surface::WlSurface) {
        self.buffer
            .buffer
            .attach_to(surface)
            .expect("attach buffer");
    }

    fn released(&self) -> bool {
        !self.buffer.buffer.slot().has_active_buffers()
    }

    fn matches(&self, dimensions: BufferDimensions) -> bool {
        self.buffer.matches(dimensions)
    }

    fn canvas_available(&self) -> bool {
        self.buffer
            .buffer
            .canvas(&mut self.pool.borrow_mut())
            .is_some()
    }
}

struct FrameBuffer {
    buffer: Buffer,
    width: i32,
    height: i32,
    stride: i32,
}

impl FrameBuffer {
    fn new(buffer: Buffer, dimensions: BufferDimensions) -> Self {
        Self {
            buffer,
            width: dimensions.width as i32,
            height: dimensions.height as i32,
            stride: dimensions.stride as i32,
        }
    }

    fn matches(&self, dimensions: BufferDimensions) -> bool {
        self.width == dimensions.width as i32
            && self.height == dimensions.height as i32
            && self.stride == dimensions.stride as i32
    }
}
