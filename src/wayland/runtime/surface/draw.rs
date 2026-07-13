use super::*;

impl RenderSurface {
    pub(in crate::wayland::runtime) fn draw<R>(
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
        R: CanvasRenderer,
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

    pub(in crate::wayland::runtime) fn set_input_regions_if_changed(
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

    pub(in crate::wayland::runtime) fn draw_frame<R>(
        &mut self,
        shm: &Shm,
        renderer: &mut R,
        id: SurfaceId,
        frame_time: Option<u32>,
        dimensions: BufferDimensions,
        repaint: Option<Bounds>,
    ) -> (Frame, FrameAction, Option<DamageRect>)
    where
        R: CanvasRenderer,
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

    pub(in crate::wayland::runtime) fn draw_transient_frame<R>(
        &mut self,
        shm: &Shm,
        renderer: &mut R,
        id: SurfaceId,
        frame_time: Option<u32>,
        dimensions: BufferDimensions,
        repaint: Option<Bounds>,
    ) -> (Frame, FrameAction, Option<DamageRect>)
    where
        R: CanvasRenderer,
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

    pub(in crate::wayland::runtime) fn buffer_pool(
        &mut self,
        shm: &Shm,
        bytes: usize,
    ) -> Rc<RefCell<SlotPool>> {
        self.pool
            .get_or_insert_with(|| {
                Rc::new(RefCell::new(
                    SlotPool::new(bytes, shm).expect("allocate buffer pool"),
                ))
            })
            .clone()
    }

    pub(in crate::wayland::runtime) fn request_frame<R: CanvasRenderer>(
        &mut self,
        qh: &QueueHandle<State<R>>,
        repaint: Option<Bounds>,
    ) {
        self.pending_repaint = union_optional_bounds(self.pending_repaint, repaint);
        self.retain_animation_buffers();
        if self.frame_pending {
            return;
        }

        self.request_frame_for_commit(qh, None);
        self.layer.commit();
    }

    pub(in crate::wayland::runtime) fn request_frame_for_commit<R: CanvasRenderer>(
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

    pub(in crate::wayland::runtime) fn retain_animation_buffers(&mut self) {
        self.retain_spare_frames = true;
        self.released_idle_memory = false;
    }

    pub(in crate::wayland) fn take_pending_repaint(&mut self) -> Option<Bounds> {
        self.pending_repaint.take()
    }

    pub(in crate::wayland::runtime) fn take_active_frames(&mut self) -> Vec<Frame> {
        let mut frames = Vec::new();
        if let Some(frame) = self.frame.take() {
            frames.push(frame);
        }
        frames.append(&mut self.retired_frames);
        frames
    }

    pub(in crate::wayland::runtime) fn retire_frame(&mut self, frame: Frame) {
        if frame.released() {
            self.keep_spare_frame(frame);
        } else {
            self.retired_frames.push(frame);
        }
    }

    pub(in crate::wayland::runtime) fn collect_released_frames(&mut self) {
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

    pub(in crate::wayland::runtime) fn keep_spare_frame(&mut self, frame: Frame) {
        if !self.retain_spare_frames {
            self.released_idle_memory = true;
            return;
        }
        if self.spare_frames.len() < MAX_SPARE_FRAMES {
            self.spare_frames.push(frame);
        }
    }

    pub(in crate::wayland::runtime) fn take_spare_frame(
        &mut self,
        dimensions: BufferDimensions,
    ) -> Option<Frame> {
        if dimensions.bytes > MAX_REUSABLE_BUFFER_BYTES && !self.retain_spare_frames {
            return None;
        }
        let index = self
            .spare_frames
            .iter()
            .position(|frame| frame.matches(dimensions))?;
        Some(self.spare_frames.swap_remove(index))
    }

    pub(in crate::wayland::runtime) fn take_released_current_frame(
        &mut self,
        dimensions: BufferDimensions,
    ) -> Option<Frame> {
        let frame = self.frame.take()?;
        if frame.released() && frame.matches(dimensions) && frame.canvas_available() {
            return Some(frame);
        }
        self.frame = Some(frame);
        None
    }

    pub(in crate::wayland::runtime) fn buffer_age_repaint(
        &self,
        sequence: u64,
        requested: Option<Bounds>,
    ) -> Option<Bounds> {
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

    pub(in crate::wayland::runtime) fn record_frame_damage(
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

    pub(in crate::wayland::runtime) fn clear_reusable_buffers(&mut self) {
        self.spare_frames.clear();
        self.damage_history.clear();
        self.pool = None;
    }

    pub(in crate::wayland::runtime) fn release_idle_buffers(&mut self) {
        self.retain_spare_frames = false;
        if !self.spare_frames.is_empty() || self.pool.is_some() {
            self.released_idle_memory = true;
        }
        self.clear_reusable_buffers();
    }

    pub(in crate::wayland::runtime) fn released_idle_memory(&mut self) -> bool {
        let released = self.released_idle_memory;
        self.released_idle_memory = false;
        released
    }

    pub(in crate::wayland::runtime) fn state(&self, id: SurfaceId) -> RenderSurfaceState {
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
