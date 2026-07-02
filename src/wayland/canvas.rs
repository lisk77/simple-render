use super::*;
use crate::ui::Bounds;

pub struct Canvas<'a> {
    pub(in crate::wayland) pixels: &'a mut [u8],
    pub(in crate::wayland) width: u32,
    pub(in crate::wayland) height: u32,
    pub(in crate::wayland) stride: u32,
    pub(in crate::wayland) scale: u32,
    pub(in crate::wayland) damage: Option<DamageRect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamageRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl<'a> Canvas<'a> {
    pub fn from_bgra_pixels(
        pixels: &'a mut [u8],
        width: u32,
        height: u32,
        stride: u32,
        scale: u32,
    ) -> Option<Self> {
        if stride < width.checked_mul(4)? {
            return None;
        }
        let required = if height == 0 {
            0
        } else {
            stride
                .checked_mul(height.saturating_sub(1))?
                .checked_add(width.checked_mul(4)?)?
        };
        if pixels.len() < required as usize {
            return None;
        }

        Some(Self {
            pixels,
            width,
            height,
            stride,
            scale: scale.max(1),
            damage: None,
        })
    }

    pub fn pixels(&self) -> &[u8] {
        self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [u8] {
        self.damage_all();
        self.pixels
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn stride(&self) -> u32 {
        self.stride
    }

    pub fn scale(&self) -> u32 {
        self.scale
    }

    pub fn clear(&mut self, rgba: [u8; 4]) {
        let bgra = rgba_to_bgra(rgba);
        for pixel in self.pixels.chunks_exact_mut(4) {
            pixel.copy_from_slice(&bgra);
        }
        self.damage_all();
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }

        let index = ((y * self.stride) + (x * 4)) as usize;
        self.pixels[index..index + 4].copy_from_slice(&rgba_to_bgra(rgba));
        self.damage_rect(DamageRect::new(x, y, 1, 1));
    }

    pub fn blend_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        if x >= self.width || y >= self.height || rgba[3] == 0 {
            return;
        }

        let index = ((y * self.stride) + (x * 4)) as usize;
        let background = bgra_to_rgba([
            self.pixels[index],
            self.pixels[index + 1],
            self.pixels[index + 2],
            self.pixels[index + 3],
        ]);
        self.pixels[index..index + 4].copy_from_slice(&rgba_to_bgra(blend_color(background, rgba)));
        self.damage_rect(DamageRect::new(x, y, 1, 1));
    }

    pub fn damage(&self) -> Option<DamageRect> {
        self.damage
    }

    pub fn damage_rect(&mut self, rect: DamageRect) {
        let Some(rect) = rect.clamp(self.width, self.height) else {
            return;
        };
        self.damage = Some(match self.damage {
            Some(damage) => damage.union(rect),
            None => rect,
        });
    }

    pub fn damage_all(&mut self) {
        self.damage_rect(DamageRect::new(0, 0, self.width, self.height));
    }
}

impl fmt::Debug for Canvas<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Canvas")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("stride", &self.stride)
            .finish_non_exhaustive()
    }
}

impl DamageRect {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_bounds(bounds: Bounds) -> Self {
        Self::new(bounds.x, bounds.y, bounds.width, bounds.height)
    }

    pub fn from_bounds_pair(previous: Bounds, current: Bounds) -> Self {
        Self::from_bounds(previous.union(current))
    }

    pub fn include_bounds(self, bounds: Bounds) -> Self {
        self.union(Self::from_bounds(bounds))
    }

    pub fn right(self) -> u32 {
        self.x.saturating_add(self.width)
    }

    pub fn bottom(self) -> u32 {
        self.y.saturating_add(self.height)
    }

    pub fn union(self, other: Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Self {
            x,
            y,
            width: right.saturating_sub(x),
            height: bottom.saturating_sub(y),
        }
    }

    pub fn clamp(self, width: u32, height: u32) -> Option<Self> {
        let x = self.x.min(width);
        let y = self.y.min(height);
        let right = self.right().min(width);
        let bottom = self.bottom().min(height);
        let width = right.saturating_sub(x);
        let height = bottom.saturating_sub(y);
        (width > 0 && height > 0).then_some(Self {
            x,
            y,
            width,
            height,
        })
    }
}

fn rgba_to_bgra([red, green, blue, alpha]: [u8; 4]) -> [u8; 4] {
    [blue, green, red, alpha]
}

fn bgra_to_rgba([blue, green, red, alpha]: [u8; 4]) -> [u8; 4] {
    [red, green, blue, alpha]
}

pub(in crate::wayland) fn blend_color(background: [u8; 4], foreground: [u8; 4]) -> [u8; 4] {
    let foreground_alpha = u32::from(foreground[3]);
    if foreground_alpha == 0 {
        return background;
    }
    if foreground_alpha == 255 {
        return foreground;
    }

    let background_alpha = u32::from(background[3]);
    let inverse_foreground_alpha = 255 - foreground_alpha;
    let output_alpha = foreground_alpha + background_alpha * inverse_foreground_alpha / 255;
    if output_alpha == 0 {
        return [0, 0, 0, 0];
    }

    let blend_component = |foreground: u8, background: u8| {
        let foreground = u32::from(foreground) * foreground_alpha;
        let background = u32::from(background) * background_alpha * inverse_foreground_alpha / 255;
        ((foreground + background) / output_alpha).min(255) as u8
    };

    [
        blend_component(foreground[0], background[0]),
        blend_component(foreground[1], background[1]),
        blend_component(foreground[2], background[2]),
        output_alpha.min(255) as u8,
    ]
}
