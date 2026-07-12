use crate::{Hit, SurfaceId};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PointerState {
    pub surface: Option<SurfaceId>,
    pub position: Option<(f64, f64)>,
    pub hovered: Option<Hit>,
    pub pressed: Option<Hit>,
    pub pressed_button: Option<u32>,
    pub scroll_delta: (f64, f64),
}
