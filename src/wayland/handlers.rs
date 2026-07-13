use super::runtime::{FractionalScaleSurface, State};
use super::*;
use crate::input::{KeyState, KeyboardEvent, KeyboardEventKind, KeyboardModifiers};

mod input;

impl<R> CompositorHandler for State<R>
where
    R: CanvasRenderer,
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
    R: CanvasRenderer,
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

impl<R: CanvasRenderer> ShmHandler for State<R> {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl<R: CanvasRenderer> Dispatch<WpViewporter, GlobalData> for State<R> {
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

impl<R: CanvasRenderer> Dispatch<WpViewport, GlobalData> for State<R> {
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

impl<R: CanvasRenderer> Dispatch<WpFractionalScaleManagerV1, GlobalData> for State<R> {
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

impl<R: CanvasRenderer> Dispatch<WpFractionalScaleV1, FractionalScaleSurface> for State<R> {
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

impl<R: CanvasRenderer> OutputHandler for State<R> {
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

impl<R: CanvasRenderer> SeatHandler for State<R> {
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

delegate_compositor!(@<R: CanvasRenderer> State<R>);
delegate_output!(@<R: CanvasRenderer> State<R>);
delegate_seat!(@<R: CanvasRenderer> State<R>);
delegate_pointer!(@<R: CanvasRenderer> State<R>);
delegate_keyboard!(@<R: CanvasRenderer> State<R>);
delegate_shm!(@<R: CanvasRenderer> State<R>);
delegate_layer!(@<R: CanvasRenderer> State<R>);
delegate_registry!(@<R: CanvasRenderer> State<R>);
