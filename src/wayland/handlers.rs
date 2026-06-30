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
        self.renderer.closed_surface(id);
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

    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}

    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}

    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl<R: Renderer> ProvidesRegistryState for State<R> {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState];
}

delegate_compositor!(@<R: Renderer> State<R>);
delegate_output!(@<R: Renderer> State<R>);
delegate_shm!(@<R: Renderer> State<R>);
delegate_layer!(@<R: Renderer> State<R>);
delegate_registry!(@<R: Renderer> State<R>);
