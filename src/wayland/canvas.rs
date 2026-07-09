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
        let bgra = rgba_to_premultiplied_bgra(rgba);
        for pixel in self.pixels.chunks_exact_mut(4) {
            pixel.copy_from_slice(&bgra);
        }
        self.damage_all();
    }

    pub fn clear_rect(&mut self, rect: DamageRect, rgba: [u8; 4]) {
        let Some(rect) = rect.clamp(self.width, self.height) else {
            return;
        };
        let bgra = rgba_to_premultiplied_bgra(rgba);
        let left = (rect.x * 4) as usize;
        let row_len = (rect.width * 4) as usize;
        for y in rect.y..rect.bottom() {
            let start = (y * self.stride) as usize + left;
            let end = start + row_len;
            for pixel in self.pixels[start..end].chunks_exact_mut(4) {
                pixel.copy_from_slice(&bgra);
            }
        }
        self.damage_rect(rect);
    }

    pub fn blend_rect(&mut self, rect: DamageRect, rgba: [u8; 4]) {
        if rgba[3] == 0 {
            return;
        }
        let Some(rect) = rect.clamp(self.width, self.height) else {
            return;
        };

        let bgra = rgba_to_premultiplied_bgra(rgba);
        let left = (rect.x * 4) as usize;
        let row_len = (rect.width * 4) as usize;
        for y in rect.y..rect.bottom() {
            let start = (y * self.stride) as usize + left;
            let end = start + row_len;
            if rgba[3] == 255 {
                for pixel in self.pixels[start..end].chunks_exact_mut(4) {
                    pixel.copy_from_slice(&bgra);
                }
            } else {
                for pixel in self.pixels[start..end].chunks_exact_mut(4) {
                    blend_bgra_pixel(pixel, bgra);
                }
            }
        }
        self.damage_rect(rect);
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }

        let index = ((y * self.stride) + (x * 4)) as usize;
        self.pixels[index..index + 4].copy_from_slice(&rgba_to_premultiplied_bgra(rgba));
        self.damage_rect(DamageRect::new(x, y, 1, 1));
    }

    pub fn blend_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        if x >= self.width || y >= self.height || rgba[3] == 0 {
            return;
        }

        let index = ((y * self.stride) + (x * 4)) as usize;
        let bgra = rgba_to_premultiplied_bgra(rgba);
        if rgba[3] == 255 {
            self.pixels[index..index + 4].copy_from_slice(&bgra);
        } else {
            blend_bgra_pixel(&mut self.pixels[index..index + 4], bgra);
        }
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

fn rgba_to_premultiplied_bgra([red, green, blue, alpha]: [u8; 4]) -> [u8; 4] {
    [
        premultiply(blue, alpha),
        premultiply(green, alpha),
        premultiply(red, alpha),
        alpha,
    ]
}

fn premultiply(component: u8, alpha: u8) -> u8 {
    ((u16::from(component) * u16::from(alpha) + 127) / 255) as u8
}

fn blend_bgra_pixel(background: &mut [u8], foreground: [u8; 4]) {
    let foreground_alpha = u32::from(foreground[3]);
    if foreground_alpha == 0 {
        return;
    }
    if foreground_alpha == 255 {
        background.copy_from_slice(&foreground);
        return;
    }

    let background_alpha = u32::from(background[3]);
    let inverse_foreground_alpha = 255 - foreground_alpha;
    let output_alpha = foreground_alpha + mul_div_255(background_alpha, inverse_foreground_alpha);
    if output_alpha == 0 {
        background.copy_from_slice(&[0, 0, 0, 0]);
        return;
    }

    let blend_component = |foreground: u8, background: u8| {
        (u32::from(foreground) + mul_div_255(u32::from(background), inverse_foreground_alpha))
            .min(255) as u8
    };

    background[0] = blend_component(foreground[0], background[0]);
    background[1] = blend_component(foreground[1], background[1]);
    background[2] = blend_component(foreground[2], background[2]);
    background[3] = output_alpha.min(255) as u8;
}

fn mul_div_255(value: u32, alpha: u32) -> u32 {
    (value * alpha + 127) / 255
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_premultiplies_alpha() {
        let mut pixels = [0_u8; 4];
        let mut canvas = Canvas::from_bgra_pixels(&mut pixels, 1, 1, 4, 1).expect("canvas");
        canvas.clear([255, 255, 255, 128]);

        assert_eq!(canvas.pixels(), &[128, 128, 128, 128]);
    }

    #[test]
    fn blend_pixel_stores_premultiplied_color() {
        let mut pixels = [0_u8; 4];
        let mut canvas = Canvas::from_bgra_pixels(&mut pixels, 1, 1, 4, 1).expect("canvas");
        canvas.blend_pixel(0, 0, [255, 255, 255, 128]);

        assert_eq!(canvas.pixels(), &[128, 128, 128, 128]);
    }

    #[test]
    fn blend_pixel_uses_premultiplied_over_operator() {
        let mut pixels = [0_u8; 4];
        let mut canvas = Canvas::from_bgra_pixels(&mut pixels, 1, 1, 4, 1).expect("canvas");
        canvas.put_pixel(0, 0, [0, 0, 255, 128]);
        canvas.blend_pixel(0, 0, [255, 0, 0, 128]);

        assert_eq!(canvas.pixels(), &[64, 0, 128, 192]);
    }
}
