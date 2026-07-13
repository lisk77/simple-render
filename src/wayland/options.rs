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
    pub fn new() -> Self {
        Self::default()
    }
    pub fn namespace(mut self, value: impl Into<String>) -> Self {
        self.namespace = value.into();
        self
    }
    pub fn width(mut self, value: u32) -> Self {
        self.width = value;
        self
    }
    pub fn height(mut self, value: u32) -> Self {
        self.height = value;
        self
    }
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    pub fn fullscreen(mut self) -> Self {
        self.width = 0;
        self.height = 0;
        self.anchor = Anchor::Fill;
        self.exclusive_zone = -1;
        self
    }
    pub fn output(mut self, value: OutputTarget) -> Self {
        self.output = Some(value);
        self
    }
    pub fn clear_output(mut self) -> Self {
        self.output = None;
        self
    }
    pub fn layer(mut self, value: Layer) -> Self {
        self.layer = value;
        self
    }
    pub fn anchor(mut self, value: Anchor) -> Self {
        self.anchor = value;
        self
    }
    pub fn margin_all(mut self, value: impl Into<i32>) -> Self {
        let value = value.into();
        self.margins = Margins {
            top: value,
            right: value,
            bottom: value,
            left: value,
        };
        self
    }
    pub fn margin_axis(mut self, horizontal: impl Into<i32>, vertical: impl Into<i32>) -> Self {
        let horizontal = horizontal.into();
        let vertical = vertical.into();
        self.margins = Margins {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        };
        self
    }
    pub fn margin_top(mut self, value: impl Into<i32>) -> Self {
        self.margins.top = value.into();
        self
    }
    pub fn margin_right(mut self, value: impl Into<i32>) -> Self {
        self.margins.right = value.into();
        self
    }
    pub fn margin_bottom(mut self, value: impl Into<i32>) -> Self {
        self.margins.bottom = value.into();
        self
    }
    pub fn margin_left(mut self, value: impl Into<i32>) -> Self {
        self.margins.left = value.into();
        self
    }
    pub fn exclusive_zone(mut self, value: i32) -> Self {
        self.exclusive_zone = value;
        self
    }
    pub fn keyboard_interactivity(mut self, value: KeyboardInteractivity) -> Self {
        self.keyboard_interactivity = value;
        self
    }
    pub fn show<R>(self, renderer: R) -> Result<()>
    where
        R: CanvasRenderer + 'static,
    {
        run_inner(renderer, Some((DEFAULT_SURFACE_ID, self)), None, true)
    }

    pub fn show_with_commands<R>(self, renderer: R, receiver: RenderReceiver) -> Result<()>
    where
        R: CanvasRenderer + 'static,
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
    TopFill,
    TopLeft,
    TopRight,
    Bottom,
    BottomFill,
    BottomLeft,
    BottomRight,
    Left,
    LeftFill,
    Right,
    RightFill,
    Fill,
    Position { x: i32, y: i32 },
}
