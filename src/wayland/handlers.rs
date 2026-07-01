use super::runtime::State;
use super::*;

impl<R> CompositorHandler for State<R>
where
    R: Renderer,
{
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        new_factor: i32,
    ) {
        let Some(id) = self.surface_id_for_wl_surface(surface) else {
            return;
        };
        let Some(surface) = self.surfaces.get_mut(&id) else {
            return;
        };
        if surface.set_scale(new_factor.max(1) as u32) {
            self.draw(qh, id, None);
        }
    }

    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        wl_surface: &wl_surface::WlSurface,
        time: u32,
    ) {
        let Some(id) = self.surface_id_for_wl_surface(wl_surface) else {
            return;
        };
        let Some(surface) = self.surfaces.get_mut(&id) else {
            return;
        };
        surface.frame_pending = false;
        self.draw(qh, id, Some(time));
    }

    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
}

impl<R> LayerShellHandler for State<R>
where
    R: Renderer,
{
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, layer: &LayerSurface) {
        let Some(id) = self.surface_id_for_layer(layer) else {
            return;
        };
        self.remove_surface(id);
        self.maybe_exit_after_surface_removal();
    }

    fn configure(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        let Some(id) = self.surface_id_for_layer(layer) else {
            return;
        };
        let Some(surface) = self.surfaces.get_mut(&id) else {
            return;
        };

        if configure.new_size.0 != 0 {
            surface.width = configure.new_size.0;
        }
        if configure.new_size.1 != 0 {
            surface.height = configure.new_size.1;
        }
        surface.configured = true;
        self.renderer
            .configured_surface(id, surface.width, surface.height);
        self.draw(qh, id, None);
    }
}

impl<R: Renderer> ShmHandler for State<R> {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl<R: Renderer> OutputHandler for State<R> {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, output: wl_output::WlOutput) {
        if let Some(output) = self.render_output(&output) {
            self.renderer.output_added(output);
        }
    }

    fn update_output(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        if let Some(output) = self.render_output(&output) {
            self.renderer.output_updated(output);
        }
    }

    fn output_destroyed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        if let Some(output) = self.render_output(&output) {
            self.renderer.output_removed(output);
        }
    }
}

impl<R: Renderer> SeatHandler for State<R> {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.pointer.is_none() {
            self.pointer = self.seat_state.get_pointer(qh, &seat).ok();
        }
    }

    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer {
            self.pointer.take();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl<R: Renderer> PointerHandler for State<R> {
    fn pointer_frame(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &wl_pointer::WlPointer,
        events: &[SctkPointerEvent],
    ) {
        for event in events {
            let Some(id) = self.surface_id_for_wl_surface(&event.surface) else {
                continue;
            };
            let event = pointer_event_from_sctk(id, event);
            let action = self.renderer.pointer_event(event);
            self.handle_input_action(qh, id, action);
        }
    }
}

impl<R: Renderer> ProvidesRegistryState for State<R> {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState, SeatState];
}

fn pointer_event_from_sctk(id: SurfaceId, event: &SctkPointerEvent) -> PointerEvent {
    let (x, y) = event.position;
    let (time, serial, kind) = match &event.kind {
        SctkPointerEventKind::Enter { serial } => (None, Some(*serial), PointerEventKind::Enter),
        SctkPointerEventKind::Leave { serial } => (None, Some(*serial), PointerEventKind::Leave),
        SctkPointerEventKind::Motion { time } => (Some(*time), None, PointerEventKind::Motion),
        SctkPointerEventKind::Press {
            time,
            button,
            serial,
        } => (
            Some(*time),
            Some(*serial),
            PointerEventKind::Button {
                button: *button,
                state: PointerButtonState::Pressed,
            },
        ),
        SctkPointerEventKind::Release {
            time,
            button,
            serial,
        } => (
            Some(*time),
            Some(*serial),
            PointerEventKind::Button {
                button: *button,
                state: PointerButtonState::Released,
            },
        ),
        SctkPointerEventKind::Axis {
            time,
            horizontal,
            vertical,
            source,
        } => (
            Some(*time),
            None,
            PointerEventKind::Axis {
                horizontal: PointerAxis {
                    absolute: horizontal.absolute,
                    discrete: horizontal.discrete,
                    stopped: horizontal.stop,
                },
                vertical: PointerAxis {
                    absolute: vertical.absolute,
                    discrete: vertical.discrete,
                    stopped: vertical.stop,
                },
                source: source.map(pointer_axis_source_from_sctk),
            },
        ),
    };

    PointerEvent {
        surface: id,
        x,
        y,
        time,
        serial,
        kind,
    }
}

fn pointer_axis_source_from_sctk(source: wl_pointer::AxisSource) -> PointerAxisSource {
    match source {
        wl_pointer::AxisSource::Wheel => PointerAxisSource::Wheel,
        wl_pointer::AxisSource::Finger => PointerAxisSource::Finger,
        wl_pointer::AxisSource::Continuous => PointerAxisSource::Continuous,
        wl_pointer::AxisSource::WheelTilt => PointerAxisSource::WheelTilt,
        _ => PointerAxisSource::Unknown,
    }
}

delegate_compositor!(@<R: Renderer> State<R>);
delegate_output!(@<R: Renderer> State<R>);
delegate_seat!(@<R: Renderer> State<R>);
delegate_pointer!(@<R: Renderer> State<R>);
delegate_shm!(@<R: Renderer> State<R>);
delegate_layer!(@<R: Renderer> State<R>);
delegate_registry!(@<R: Renderer> State<R>);
