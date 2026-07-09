use super::runtime::{FractionalScaleSurface, State};
use super::*;
use crate::input::{KeyState, KeyboardEvent, KeyboardEventKind, KeyboardModifiers};

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
            self.draw(qh, id, None, None);
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
        let repaint = surface.take_pending_repaint();
        self.draw(qh, id, Some(time), repaint);
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
        self.draw(qh, id, None, None);
    }
}

impl<R: Renderer> ShmHandler for State<R> {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl<R: Renderer> Dispatch<WpViewporter, GlobalData> for State<R> {
    fn event(
        _: &mut Self,
        _: &WpViewporter,
        _: <WpViewporter as Proxy>::Event,
        _: &GlobalData,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl<R: Renderer> Dispatch<WpViewport, GlobalData> for State<R> {
    fn event(
        _: &mut Self,
        _: &WpViewport,
        _: wp_viewport::Event,
        _: &GlobalData,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl<R: Renderer> Dispatch<WpFractionalScaleManagerV1, GlobalData> for State<R> {
    fn event(
        _: &mut Self,
        _: &WpFractionalScaleManagerV1,
        _: <WpFractionalScaleManagerV1 as Proxy>::Event,
        _: &GlobalData,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl<R: Renderer> Dispatch<WpFractionalScaleV1, FractionalScaleSurface> for State<R> {
    fn event(
        state: &mut Self,
        _: &WpFractionalScaleV1,
        event: <WpFractionalScaleV1 as Proxy>::Event,
        data: &FractionalScaleSurface,
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        let WpFractionalScaleEvent::PreferredScale { scale } = event else {
            return;
        };
        let Some(id) = state.surface_id_for_wl_surface(&data.surface) else {
            return;
        };
        let Some(surface) = state.surfaces.get_mut(&id) else {
            return;
        };
        if surface.set_fractional_scale(scale) && surface.configured {
            state.draw(qh, id, None, None);
        }
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
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        if let Some(output) = self.render_output(&output) {
            let output_id = output.id;
            let output_name = output.name.clone();
            let output_scale = output.scale_factor.max(1) as u32;
            let redraw_surfaces: Vec<_> = self
                .surfaces
                .iter_mut()
                .filter_map(|(id, surface)| {
                    let matches_output = match &surface.output {
                        Some(OutputTarget::Id(target_id)) => *target_id == output_id,
                        Some(OutputTarget::Name(target_name)) => {
                            output_name.as_ref() == Some(target_name)
                        }
                        None | Some(OutputTarget::Any) => false,
                    };
                    (matches_output && surface.set_scale(output_scale) && surface.configured)
                        .then_some(*id)
                })
                .collect();
            self.renderer.output_updated(output);
            for id in redraw_surfaces {
                self.draw(qh, id, None, None);
            }
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
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            self.keyboard = self.seat_state.get_keyboard(qh, &seat, None).ok();
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
        if capability == Capability::Keyboard {
            self.keyboard.take().map(|keyboard| keyboard.release());
            self.keyboard_focus = None;
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

impl<R: Renderer> KeyboardHandler for State<R> {
    fn enter(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        serial: u32,
        raw: &[u32],
        keysyms: &[Keysym],
    ) {
        let id = self.surface_id_for_wl_surface(surface);
        self.keyboard_focus = id;
        let event = KeyboardEvent {
            surface: id,
            time: None,
            serial: Some(serial),
            kind: KeyboardEventKind::Enter {
                pressed_keycodes: raw.to_vec(),
                pressed_keysyms: keysyms.iter().map(|keysym| keysym.raw()).collect(),
            },
        };
        let action = self.renderer.keyboard_event(event);
        self.handle_keyboard_input_action(qh, id, action);
    }

    fn leave(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        serial: u32,
    ) {
        let id = self.surface_id_for_wl_surface(surface);
        if self.keyboard_focus == id {
            self.keyboard_focus = None;
        }
        let event = KeyboardEvent {
            surface: id,
            time: None,
            serial: Some(serial),
            kind: KeyboardEventKind::Leave,
        };
        let action = self.renderer.keyboard_event(event);
        self.handle_keyboard_input_action(qh, id, action);
    }

    fn press_key(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        serial: u32,
        event: SctkKeyEvent,
    ) {
        self.handle_key_event(qh, serial, KeyState::Pressed, event);
    }

    fn release_key(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        serial: u32,
        event: SctkKeyEvent,
    ) {
        self.handle_key_event(qh, serial, KeyState::Released, event);
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        serial: u32,
        modifiers: SctkModifiers,
        layout: u32,
    ) {
        let id = self.keyboard_focus;
        let event = KeyboardEvent {
            surface: id,
            time: None,
            serial: Some(serial),
            kind: KeyboardEventKind::Modifiers {
                modifiers: keyboard_modifiers_from_sctk(modifiers),
                layout,
            },
        };
        let action = self.renderer.keyboard_event(event);
        self.handle_keyboard_input_action(qh, id, action);
    }
}

impl<R: Renderer> State<R> {
    fn handle_key_event(
        &mut self,
        qh: &QueueHandle<Self>,
        serial: u32,
        state: KeyState,
        event: SctkKeyEvent,
    ) {
        let id = self.keyboard_focus;
        let event = KeyboardEvent {
            surface: id,
            time: Some(event.time),
            serial: Some(serial),
            kind: KeyboardEventKind::Key {
                state,
                raw_code: event.raw_code,
                keysym: event.keysym.raw(),
                utf8: event.utf8,
            },
        };
        let action = self.renderer.keyboard_event(event);
        self.handle_keyboard_input_action(qh, id, action);
    }

    fn handle_keyboard_input_action(
        &mut self,
        qh: &QueueHandle<Self>,
        id: Option<SurfaceId>,
        action: InputAction,
    ) {
        if let Some(id) = id {
            self.handle_input_action(qh, id, action);
            return;
        }

        if action == InputAction::Exit {
            self.running = false;
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

fn keyboard_modifiers_from_sctk(modifiers: SctkModifiers) -> KeyboardModifiers {
    KeyboardModifiers {
        ctrl: modifiers.ctrl,
        alt: modifiers.alt,
        shift: modifiers.shift,
        caps_lock: modifiers.caps_lock,
        logo: modifiers.logo,
        num_lock: modifiers.num_lock,
    }
}

delegate_compositor!(@<R: Renderer> State<R>);
delegate_output!(@<R: Renderer> State<R>);
delegate_seat!(@<R: Renderer> State<R>);
delegate_pointer!(@<R: Renderer> State<R>);
delegate_keyboard!(@<R: Renderer> State<R>);
delegate_shm!(@<R: Renderer> State<R>);
delegate_layer!(@<R: Renderer> State<R>);
delegate_registry!(@<R: Renderer> State<R>);
