use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    #[default]
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Overflow {
    #[default]
    Clip,
    Visible,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Position {
    #[default]
    Flow,
    Absolute,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Length {
    #[default]
    Fit,
    Fill,
    Px(u32),
    Percent(f32),
}

impl Length {
    pub const fn px(value: u32) -> Self {
        Self::Px(value)
    }

    pub const fn percent(value: f32) -> Self {
        Self::Percent(value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Spacing {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl Spacing {
    pub const ZERO: Self = Self::all(0);

    pub const fn all(value: u32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn axis(horizontal: u32, vertical: u32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Inset {
    pub top: Option<u32>,
    pub right: Option<u32>,
    pub bottom: Option<u32>,
    pub left: Option<u32>,
}

impl Inset {
    pub const ZERO: Self = Self {
        top: None,
        right: None,
        bottom: None,
        left: None,
    };

    pub const fn all(value: u32) -> Self {
        Self {
            top: Some(value),
            right: Some(value),
            bottom: Some(value),
            left: Some(value),
        }
    }

    pub const fn axis(horizontal: u32, vertical: u32) -> Self {
        Self {
            top: Some(vertical),
            right: Some(horizontal),
            bottom: Some(vertical),
            left: Some(horizontal),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::ui) struct Size {
    pub(in crate::ui) width: u32,
    pub(in crate::ui) height: u32,
}

impl Size {
    pub(in crate::ui) fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub(in crate::ui) fn axis(self, direction: Direction) -> u32 {
        match direction {
            Direction::Row => self.width,
            Direction::Column => self.height,
        }
    }

    pub(in crate::ui) fn cross(self, direction: Direction) -> u32 {
        match direction {
            Direction::Row => self.height,
            Direction::Column => self.width,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MeasuredSize {
    pub width: u32,
    pub height: u32,
}

impl From<Size> for MeasuredSize {
    fn from(size: Size) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Hit {
    /// Child-index path from the root to the hit element. The root element is an empty path.
    pub path: Vec<usize>,
    pub bounds: Bounds,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RectLayout {
    pub surface: Option<Surface>,
    pub width: Length,
    pub height: Length,
    pub min_width: u32,
    pub min_height: u32,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub fill: u32,
    pub direction: Direction,
    pub align: Align,
    pub justify: Align,
    pub overflow: Overflow,
    pub position: Position,
    pub inset: Inset,
    pub padding: Spacing,
    pub gap: u32,
    pub style: Style,
    pub transform: PaintTransform,
    pub content: Option<Content>,
}

impl Default for RectLayout {
    fn default() -> Self {
        Self {
            surface: None,
            width: Length::Fit,
            height: Length::Fit,
            min_width: 0,
            min_height: 0,
            max_width: None,
            max_height: None,
            fill: 1,
            direction: Direction::Row,
            align: Align::Start,
            justify: Align::Start,
            overflow: Overflow::Clip,
            position: Position::Flow,
            inset: Inset::ZERO,
            padding: Spacing::ZERO,
            gap: 0,
            style: Style::default(),
            transform: PaintTransform::IDENTITY,
            content: None,
        }
    }
}

pub type RectStyle = Style;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Surface {
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

impl Default for Surface {
    fn default() -> Self {
        let options = LayerOptions::default();
        Self {
            namespace: options.namespace,
            width: options.width,
            height: options.height,
            output: options.output,
            layer: options.layer,
            anchor: options.anchor,
            margins: options.margins,
            exclusive_zone: options.exclusive_zone,
            keyboard_interactivity: options.keyboard_interactivity,
        }
    }
}

impl From<Surface> for LayerOptions {
    fn from(surface: Surface) -> Self {
        Self {
            namespace: surface.namespace,
            width: surface.width,
            height: surface.height,
            output: surface.output,
            layer: surface.layer,
            anchor: surface.anchor,
            margins: surface.margins,
            exclusive_zone: surface.exclusive_zone,
            keyboard_interactivity: surface.keyboard_interactivity,
        }
    }
}
