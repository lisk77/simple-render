use super::*;

impl<R: CanvasRenderer> State<R> {
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

    pub(super) fn render_outputs(&self) -> Vec<RenderOutput> {
        self.output_state
            .outputs()
            .filter_map(|output| self.render_output(&output))
            .collect()
    }

    pub(super) fn surface_state(&self, id: SurfaceId) -> Option<RenderSurfaceState> {
        self.surfaces.get(&id).map(|surface| surface.state(id))
    }

    pub(super) fn collect_released_frames(&mut self) {
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
