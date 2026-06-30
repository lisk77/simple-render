use std::{error::Error, fmt};

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::{EventLoop, channel as calloop_channel, channel::Event as ChannelEvent},
        calloop_wayland_source::WaylandSource,
        client::{
            Connection, QueueHandle,
            globals::registry_queue_init,
            protocol::{wl_output, wl_shm, wl_surface},
        },
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
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
