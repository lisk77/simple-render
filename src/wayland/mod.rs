use std::{error::Error, fmt};

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, Region},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_shm,
    globals::GlobalData,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::{EventLoop, channel as calloop_channel, channel::Event as ChannelEvent},
        calloop_wayland_source::WaylandSource,
        client::{
            Connection, Dispatch, Proxy, QueueHandle,
            globals::registry_queue_init,
            protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
        },
        protocols::wp::{
            fractional_scale::v1::client::{
                wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
                wp_fractional_scale_v1::{Event as WpFractionalScaleEvent, WpFractionalScaleV1},
            },
            viewporter::client::{
                wp_viewport::{self, WpViewport},
                wp_viewporter::WpViewporter,
            },
        },
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        Capability, SeatHandler, SeatState,
        keyboard::{KeyEvent as SctkKeyEvent, KeyboardHandler, Keysym, Modifiers as SctkModifiers},
        pointer::{
            PointerEvent as SctkPointerEvent, PointerEventKind as SctkPointerEventKind,
            PointerHandler,
        },
    },
    shell::{
        WaylandSurface,
        wlr_layer::{
            Anchor as SctkAnchor, KeyboardInteractivity as SctkKeyboardInteractivity,
            Layer as SctkLayer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
        },
    },
    shm::{
        Shm, ShmHandler,
        slot::{Buffer, SlotPool},
    },
};

mod api;
mod canvas;
mod handlers;
mod options;
mod placement;
mod runtime;

pub use api::*;
pub use canvas::{Canvas, DamageRect};
pub use options::*;
pub use runtime::{run, run_surfaces};
