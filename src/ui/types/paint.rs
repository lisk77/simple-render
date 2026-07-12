use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);

    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::rgba(red, green, blue, 255)
    }

    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub const fn to_rgba(self) -> [u8; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }

    pub fn from_hex(value: &str) -> Result<Self, ColorParseError> {
        value.parse()
    }
}

impl From<[u8; 4]> for Color {
    fn from([red, green, blue, alpha]: [u8; 4]) -> Self {
        Self::rgba(red, green, blue, alpha)
    }
}

impl From<[u8; 3]> for Color {
    fn from([red, green, blue]: [u8; 3]) -> Self {
        Self::rgb(red, green, blue)
    }
}

impl From<Color> for [u8; 4] {
    fn from(color: Color) -> Self {
        color.to_rgba()
    }
}

impl FromStr for Color {
    type Err = ColorParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let hex = value.strip_prefix('#').unwrap_or(value);
        if !hex.is_ascii() || !matches!(hex.len(), 6 | 8) {
            return Err(ColorParseError);
        }

        let parse =
            |offset| u8::from_str_radix(&hex[offset..offset + 2], 16).map_err(|_| ColorParseError);
        let red = parse(0)?;
        let green = parse(2)?;
        let blue = parse(4)?;
        let alpha = if hex.len() == 8 { parse(6)? } else { 255 };

        Ok(Self::rgba(red, green, blue, alpha))
    }
}

impl TryFrom<&str> for Color {
    type Error = ColorParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorParseError;

impl fmt::Display for ColorParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("expected #RRGGBB or #RRGGBBAA color")
    }
}

impl Error for ColorParseError {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GradientDirection {
    #[default]
    Horizontal,
    Vertical,
    Diagonal,
    DiagonalReverse,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintTransform {
    pub scale: f32,
    pub translate_x: i32,
    pub translate_y: i32,
}

impl PaintTransform {
    pub const IDENTITY: Self = Self {
        scale: 1.0,
        translate_x: 0,
        translate_y: 0,
    };

    pub const fn identity() -> Self {
        Self::IDENTITY
    }

    pub const fn new(scale: f32, translate_x: i32, translate_y: i32) -> Self {
        Self {
            scale,
            translate_x,
            translate_y,
        }
    }

    pub const fn scale(scale: f32) -> Self {
        Self {
            scale,
            ..Self::IDENTITY
        }
    }

    pub const fn translate(translate_x: i32, translate_y: i32) -> Self {
        Self {
            translate_x,
            translate_y,
            ..Self::IDENTITY
        }
    }

    pub fn compose(self, next: Self) -> Self {
        let scale = sanitize_scale(self.scale) * sanitize_scale(next.scale);
        Self {
            scale,
            translate_x: scale_i32_f32(next.translate_x, sanitize_scale(self.scale))
                .saturating_add(self.translate_x),
            translate_y: scale_i32_f32(next.translate_y, sanitize_scale(self.scale))
                .saturating_add(self.translate_y),
        }
    }

    pub fn is_identity(self) -> bool {
        self.scale == 1.0 && self.translate_x == 0 && self.translate_y == 0
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_translate(mut self, translate_x: i32, translate_y: i32) -> Self {
        self.translate_x = translate_x;
        self.translate_y = translate_y;
        self
    }
}

impl Default for PaintTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

pub(in crate::ui) fn sanitize_scale(scale: f32) -> f32 {
    if scale.is_finite() {
        scale.max(0.0)
    } else {
        0.0
    }
}

pub(in crate::ui) fn scale_i32_f32(value: i32, scale: f32) -> i32 {
    let scaled = f64::from(value) * f64::from(scale);
    if !scaled.is_finite() {
        return if scaled.is_sign_negative() {
            i32::MIN
        } else {
            i32::MAX
        };
    }
    scaled
        .round()
        .clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32
}

pub(in crate::ui::types) fn interpolate_color(
    from: Color,
    to: Color,
    position: u64,
    max_position: u64,
) -> Color {
    Color::rgba(
        interpolate_component(from.red, to.red, position, max_position),
        interpolate_component(from.green, to.green, position, max_position),
        interpolate_component(from.blue, to.blue, position, max_position),
        interpolate_component(from.alpha, to.alpha, position, max_position),
    )
}

fn interpolate_component(from: u8, to: u8, position: u64, max_position: u64) -> u8 {
    let from = u64::from(from);
    let to = u64::from(to);
    if to >= from {
        (from + (to - from) * position / max_position) as u8
    } else {
        (from - (from - to) * position / max_position) as u8
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Paint {
    Solid(Color),
    Gradient(Arc<[Color]>),
}

impl Paint {
    pub fn solid(color: impl Into<Color>) -> Self {
        Self::Solid(color.into())
    }

    pub fn gradient(colors: impl Into<Arc<[Color]>>) -> Self {
        let colors = colors.into();
        if colors.is_empty() {
            Self::Solid(Color::TRANSPARENT)
        } else {
            Self::Gradient(colors)
        }
    }

    pub fn first(&self) -> Color {
        match self {
            Self::Solid(color) => *color,
            Self::Gradient(colors) => colors[0],
        }
    }

    pub fn at(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        direction: GradientDirection,
    ) -> Color {
        let colors = match self {
            Self::Solid(color) => return *color,
            Self::Gradient(colors) if colors.len() == 1 => return colors[0],
            Self::Gradient(colors) => colors,
        };

        let (position, max_position) = match direction {
            GradientDirection::Horizontal => (x, width.saturating_sub(1)),
            GradientDirection::Vertical => (y, height.saturating_sub(1)),
            GradientDirection::Diagonal => (
                x.saturating_add(y),
                width
                    .saturating_sub(1)
                    .saturating_add(height.saturating_sub(1)),
            ),
            GradientDirection::DiagonalReverse => (
                width.saturating_sub(1).saturating_sub(x).saturating_add(y),
                width
                    .saturating_sub(1)
                    .saturating_add(height.saturating_sub(1)),
            ),
        };
        if max_position == 0 {
            return colors[0];
        }

        let stop_count = colors.len() - 1;
        let scaled = u64::from(position.min(max_position)) * stop_count as u64;
        let max_position = u64::from(max_position);
        let index = (scaled / max_position) as usize;
        if index >= stop_count {
            return colors[stop_count];
        }

        let remainder = scaled % max_position;
        interpolate_color(colors[index], colors[index + 1], remainder, max_position)
    }
}

impl Default for Paint {
    fn default() -> Self {
        Self::Solid(Color::TRANSPARENT)
    }
}

impl From<Color> for Paint {
    fn from(color: Color) -> Self {
        Self::Solid(color)
    }
}

impl From<[u8; 4]> for Paint {
    fn from(color: [u8; 4]) -> Self {
        Self::Solid(color.into())
    }
}

impl From<[u8; 3]> for Paint {
    fn from(color: [u8; 3]) -> Self {
        Self::Solid(color.into())
    }
}
