use super::*;

mod draw;
mod frames;
use frames::*;
pub(in crate::wayland::runtime) use frames::{DrawResult, Frame};

pub(super) fn resolve_output_target(
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

pub(super) fn output_initial_scale(
    output_state: &OutputState,
    output: Option<&wl_output::WlOutput>,
) -> u32 {
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
pub(super) enum SurfaceAnimationStart {
    Animate,
    Complete,
    Destroy,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SurfaceAnimationFrame {
    Idle,
    Animate,
    Complete,
    Destroy,
}

impl Margins {
    pub(super) fn lerp(self, to: Self, progress: f32) -> Self {
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
    pub(super) fn new<R: CanvasRenderer>(
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

    pub(super) fn set_size(&mut self, width: u32, height: u32) {
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

    pub(super) fn set_margins(&mut self, margins: Margins) {
        self.margins = margins;
        self.animation = None;
        apply_placement(&self.layer, self.anchor, self.margins);
    }

    pub(super) fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
        apply_placement(&self.layer, self.anchor, self.margins);
    }

    pub(super) fn set_layer(&mut self, layer: Layer) {
        self.layer_kind = layer;
        self.layer.set_layer(layer.into_sctk());
    }

    pub(super) fn update_options(&mut self, options: LayerOptions) {
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

    pub(super) fn start_margins_animation(
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

    pub(super) fn start_size_animation(
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

    pub(super) fn cancel_animation(&mut self) {
        self.animation = None;
    }

    pub(super) fn buffer_scale(&self) -> u32 {
        if self.viewport.is_some() {
            1
        } else {
            self.scale.max(1)
        }
    }

    pub(super) fn scale_factor(&self) -> f32 {
        self.fractional_scale_factor
            .map(|scale| scale as f32 / FRACTIONAL_SCALE_DENOMINATOR)
            .unwrap_or_else(|| self.scale.max(1) as f32)
            .max(1.0)
    }

    pub(super) fn advance_animation(&mut self, time: u32) -> SurfaceAnimationFrame {
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
}
