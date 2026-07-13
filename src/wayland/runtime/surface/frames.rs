use super::*;

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

pub(super) fn scaled_buffer_extent(logical: u32, scale: f32) -> u32 {
    let scaled = (logical as f32 * scale).round();
    assert!(
        scaled.is_finite() && scaled >= 0.0 && scaled <= u32::MAX as f32,
        "scaled buffer extent overflow"
    );
    scaled as u32
}

pub(super) fn draw_surface_to_pixels<R>(
    renderer: &mut R,
    id: SurfaceId,
    pixels: &mut [u8],
    dimensions: BufferDimensions,
    frame_time: Option<u32>,
    repaint: Option<Bounds>,
) -> (FrameAction, Option<DamageRect>)
where
    R: CanvasRenderer,
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

pub(super) fn draw_reusable_frame<R>(
    renderer: &mut R,
    id: SurfaceId,
    frame_time: Option<u32>,
    dimensions: BufferDimensions,
    frame: Frame,
    repaint: Option<Bounds>,
) -> Option<(Frame, FrameAction, Option<DamageRect>)>
where
    R: CanvasRenderer,
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

pub(super) fn union_optional_bounds(
    current: Option<Bounds>,
    next: Option<Bounds>,
) -> Option<Bounds> {
    match (current, next) {
        (Some(current), Some(next)) => Some(current.union(next)),
        (Some(current), None) => Some(current),
        (None, Some(next)) => Some(next),
        (None, None) => None,
    }
}

pub(in crate::wayland::runtime) enum DrawResult {
    Drawn { next_frame: bool },
    Exit,
}

pub(in crate::wayland::runtime) struct Frame {
    pub(super) buffer: FrameBuffer,
    pub(super) pool: Rc<RefCell<SlotPool>>,
    pub(super) sequence: u64,
}

impl Frame {
    pub(in crate::wayland::runtime) fn attach_to(&self, surface: &wl_surface::WlSurface) {
        self.buffer
            .buffer
            .attach_to(surface)
            .expect("attach buffer");
    }

    pub(in crate::wayland::runtime) fn released(&self) -> bool {
        !self.buffer.buffer.slot().has_active_buffers()
    }

    pub(in crate::wayland::runtime) fn matches(&self, dimensions: BufferDimensions) -> bool {
        self.buffer.matches(dimensions)
    }

    pub(in crate::wayland::runtime) fn canvas_available(&self) -> bool {
        self.buffer
            .buffer
            .canvas(&mut self.pool.borrow_mut())
            .is_some()
    }
}

pub(in crate::wayland::runtime) struct FrameBuffer {
    pub(super) buffer: Buffer,
    width: i32,
    height: i32,
    stride: i32,
}

impl FrameBuffer {
    pub(in crate::wayland::runtime) fn new(buffer: Buffer, dimensions: BufferDimensions) -> Self {
        Self {
            buffer,
            width: dimensions.width as i32,
            height: dimensions.height as i32,
            stride: dimensions.stride as i32,
        }
    }

    pub(in crate::wayland::runtime) fn matches(&self, dimensions: BufferDimensions) -> bool {
        self.width == dimensions.width as i32
            && self.height == dimensions.height as i32
            && self.stride == dimensions.stride as i32
    }
}
