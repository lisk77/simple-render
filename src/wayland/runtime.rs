use super::placement::apply_placement;
use super::*;
use crate::animation::{Animation, lerp_i32, lerp_u32};
use std::collections::BTreeMap;

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
    let output_state = OutputState::new(&globals, &qh);
    let seat_state = SeatState::new(&globals, &qh);

    let mut surfaces = BTreeMap::new();
    if let Some((id, options)) = initial_surface {
        if let Some(output) = resolve_output_target(&output_state, options.output.as_ref()) {
            surfaces.insert(
                id,
                RenderSurface::new(&qh, &compositor, &layer_shell, output.as_ref(), &options),
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
            RenderCommand::Redraw => self.draw(qh, DEFAULT_SURFACE_ID, None),
            RenderCommand::Resize { width, height } => {
                self.resize_surface(qh, DEFAULT_SURFACE_ID, width, height)
            }
            RenderCommand::SetMargins(margins) => {
                self.set_surface_margins(DEFAULT_SURFACE_ID, margins)
            }
            RenderCommand::SetAnchor(anchor) => self.set_surface_anchor(DEFAULT_SURFACE_ID, anchor),
            RenderCommand::CreateSurface { id, options } => self.create_surface(qh, id, options),
            RenderCommand::DestroySurface { id } => self.destroy_surface(id),
            RenderCommand::RedrawSurface { id } => self.draw(qh, id, None),
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
                output.as_ref(),
                &options,
            ),
        );
    }

    fn resolve_output_target(
        &self,
        target: Option<&OutputTarget>,
    ) -> Option<Option<wl_output::WlOutput>> {
        resolve_output_target(&self.output_state, target)
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
            self.draw(qh, id, None);
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
                    surface.request_frame(qh);
                }
            }
            SurfaceAnimationStart::Complete => {
                if self
                    .surfaces
                    .get(&id)
                    .is_some_and(|surface| surface.configured)
                {
                    self.draw(qh, id, None);
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
            let draw_result = surface.draw(&self.shm, &mut self.renderer, id, frame_time);
            (animation, draw_result)
        };

        match draw_result {
            DrawResult::Drawn {
                retired_frame,
                next_frame,
            } => {
                if let Some(frame) = retired_frame {
                    self.retired_frames.push(frame);
                }
                if matches!(animation, SurfaceAnimationFrame::Destroy) {
                    self.remove_surface(id);
                    self.maybe_exit_after_surface_removal();
                } else if next_frame || matches!(animation, SurfaceAnimationFrame::Animate) {
                    if let Some(surface) = self.surfaces.get_mut(&id) {
                        surface.request_frame(qh);
                    }
                }
            }
            DrawResult::Exit { retired_frame } => {
                if let Some(frame) = retired_frame {
                    self.retired_frames.push(frame);
                }
                self.remove_surface(id);
                self.maybe_exit_after_surface_removal();
            }
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
                    surface.request_frame(qh);
                }
            }
            InputAction::Animate => {
                if let Some(surface) = self.surfaces.get_mut(&id) {
                    surface.request_frame(qh);
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
        if let Some(frame) = surface.take_frame() {
            self.retired_frames.push(frame);
        }
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
        let retired_count = self.retired_frames.len();
        self.retired_frames.retain(|frame| !frame.released());
        if self.retired_frames.len() < retired_count {
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
    pub(in crate::wayland) configured: bool,
    pub(in crate::wayland) width: u32,
    pub(in crate::wayland) height: u32,
    output: Option<OutputTarget>,
    layer_kind: Layer,
    scale: u32,
    anchor: Anchor,
    margins: Margins,
    animation: Option<SurfaceAnimation>,
    frame: Option<Frame>,
    pub(in crate::wayland) frame_pending: bool,
}

impl RenderSurface {
    fn new<R: Renderer>(
        qh: &QueueHandle<State<R>>,
        compositor: &CompositorState,
        layer_shell: &LayerShell,
        output: Option<&wl_output::WlOutput>,
        options: &LayerOptions,
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

        Self {
            layer,
            configured: false,
            width: options.width,
            height: options.height,
            output: options.output.clone(),
            layer_kind: options.layer,
            scale: 1,
            anchor: options.anchor,
            margins: options.margins,
            animation: None,
            frame: None,
            frame_pending: false,
        }
    }

    fn set_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.layer.set_size(width, height);
    }

    pub(in crate::wayland) fn set_scale(&mut self, scale: u32) -> bool {
        let scale = scale.max(1);
        if self.scale == scale {
            return false;
        }

        self.scale = scale;
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
        shm: &Shm,
        renderer: &mut R,
        id: SurfaceId,
        frame_time: Option<u32>,
    ) -> DrawResult
    where
        R: Renderer,
    {
        let dimensions = BufferDimensions::new(self.width, self.height, self.scale);
        let (frame, action, damage) = self.draw_frame(shm, renderer, id, frame_time, dimensions);

        if let Some(damage) = damage {
            self.layer.wl_surface().damage_buffer(
                damage.x as i32,
                damage.y as i32,
                damage.width as i32,
                damage.height as i32,
            );
        }
        let _ = self.layer.set_buffer_scale(self.scale);
        frame.attach_to(self.layer.wl_surface());
        self.layer.commit();

        let retired_frame = self.frame.replace(frame);

        match action {
            FrameAction::Wait => DrawResult::Drawn {
                retired_frame,
                next_frame: false,
            },
            FrameAction::Animate => DrawResult::Drawn {
                retired_frame,
                next_frame: true,
            },
            FrameAction::Exit => DrawResult::Exit { retired_frame },
        }
    }

    fn draw_frame<R>(
        &mut self,
        shm: &Shm,
        renderer: &mut R,
        id: SurfaceId,
        frame_time: Option<u32>,
        dimensions: BufferDimensions,
    ) -> (Frame, FrameAction, Option<DamageRect>)
    where
        R: Renderer,
    {
        let mut pool = SlotPool::new(dimensions.bytes, shm).expect("allocate buffer pool");
        let (buffer, pixels) = pool
            .create_buffer(
                dimensions.width as i32,
                dimensions.height as i32,
                dimensions.stride as i32,
                wl_shm::Format::Argb8888,
            )
            .expect("allocate buffer");
        let (action, damage) = draw_surface_to_pixels(renderer, id, pixels, dimensions, frame_time);

        (
            Frame {
                buffer,
                _pool: pool,
            },
            action,
            damage,
        )
    }

    fn request_frame<R: Renderer>(&mut self, qh: &QueueHandle<State<R>>) {
        if self.frame_pending {
            return;
        }

        let surface = self.layer.wl_surface();
        surface.frame(qh, surface.clone());
        self.frame_pending = true;
        self.layer.commit();
    }

    fn take_frame(&mut self) -> Option<Frame> {
        self.frame.take()
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
    pub(in crate::wayland) scale: u32,
    pub(in crate::wayland) stride: u32,
    pub(in crate::wayland) bytes: usize,
}

impl BufferDimensions {
    pub(in crate::wayland) fn new(logical_width: u32, logical_height: u32, scale: u32) -> Self {
        let scale = scale.max(1);
        let width = logical_width
            .checked_mul(scale)
            .expect("scaled buffer width overflow");
        let height = logical_height
            .checked_mul(scale)
            .expect("scaled buffer height overflow");
        let stride = width.checked_mul(4).expect("buffer stride overflow");
        let bytes = (stride as usize)
            .checked_mul(height as usize)
            .expect("buffer size overflow");
        Self {
            logical_width,
            logical_height,
            width,
            height,
            scale,
            stride,
            bytes,
        }
    }
}

fn draw_surface_to_pixels<R>(
    renderer: &mut R,
    id: SurfaceId,
    pixels: &mut [u8],
    dimensions: BufferDimensions,
    frame_time: Option<u32>,
) -> (FrameAction, Option<DamageRect>)
where
    R: Renderer,
{
    let mut canvas = Canvas {
        pixels,
        width: dimensions.width,
        height: dimensions.height,
        stride: dimensions.stride,
        scale: dimensions.scale,
        damage: None,
    };
    let action = renderer.draw_surface(
        id,
        &mut canvas,
        RenderContext {
            width: dimensions.logical_width,
            height: dimensions.logical_height,
            scale: dimensions.scale,
            buffer_width: dimensions.width,
            buffer_height: dimensions.height,
            frame_time,
        },
    );
    let damage = canvas.damage();
    (action, damage)
}

enum DrawResult {
    Drawn {
        retired_frame: Option<Frame>,
        next_frame: bool,
    },
    Exit {
        retired_frame: Option<Frame>,
    },
}

struct Frame {
    buffer: Buffer,
    _pool: SlotPool,
}

impl Frame {
    fn attach_to(&self, surface: &wl_surface::WlSurface) {
        self.buffer.attach_to(surface).expect("attach buffer");
    }

    fn released(&self) -> bool {
        !self.buffer.slot().has_active_buffers()
    }
}
