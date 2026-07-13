use super::*;

impl<R: CanvasRenderer> PointerHandler for State<R> {
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

impl<R: CanvasRenderer> KeyboardHandler for State<R> {
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

impl<R: CanvasRenderer> State<R> {
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
                key: event.keysym.raw().into(),
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

impl<R: CanvasRenderer> ProvidesRegistryState for State<R> {
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
