use super::runtime::run_inner;
use super::*;

#[derive(Debug, Clone)]
pub struct LayerOptions {
    pub namespace: String,
    pub width: u32,
    pub height: u32,
    pub output: Option<OutputTarget>,
    pub layer: Layer,
    pub anchor: Anchor,
    pub margins: Margins,
    pub exclusive_zone: i32,
    pub keyboard_interactivity: KeyboardInteractivity,
}

impl Default for LayerOptions {
    fn default() -> Self {
        Self {
            namespace: "simple-render".to_owned(),
            width: 256,
            height: 256,
            output: None,
            layer: Layer::Top,
            anchor: Anchor::Top,
            margins: Margins::default(),
            exclusive_zone: 0,
            keyboard_interactivity: KeyboardInteractivity::None,
        }
    }
}

impl LayerOptions {
    pub fn show<R>(self, renderer: R) -> Result<()>
    where
        R: Renderer + 'static,
    {
        run_inner(renderer, Some((DEFAULT_SURFACE_ID, self)), None, true)
    }

    pub fn show_with_commands<R>(self, renderer: R, receiver: RenderReceiver) -> Result<()>
    where
        R: Renderer + 'static,
    {
        run_inner(
            renderer,
            Some((DEFAULT_SURFACE_ID, self)),
            Some(receiver),
            true,
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Margins {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputTarget {
    Any,
    Id(u32),
    Name(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardInteractivity {
    None,
    Exclusive,
    OnDemand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Center,
    Top,
    TopLeft,
    TopRight,
    Bottom,
    BottomLeft,
    BottomRight,
    Left,
    Right,
    Fill,
    Position { x: i32, y: i32 },
}
