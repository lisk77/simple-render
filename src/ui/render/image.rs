use super::text::{align_offset, scaled_dimension};
use super::*;

pub(super) fn draw_image(
    canvas: &mut Canvas<'_>,
    rect: Bounds,
    clip: Clip,
    opacity: f32,
    image: &Image,
    offset: PaintOffset,
) {
    if image.width == 0
        || image.height == 0
        || rect.width == 0
        || rect.height == 0
        || !opacity_draws(opacity)
    {
        return;
    }

    let required = image_required_len(image);
    let pixels = image.rgba();
    if required.is_none_or(|required| pixels.len() < required) {
        return;
    }

    let placement = image_placement(image, rect);
    let Some(draw) = placement
        .destination
        .intersect(clip.bounds())
        .and_then(|rect| visible_draw_bounds(canvas, rect, offset))
    else {
        return;
    };
    for target_y in draw.y..draw.bottom() {
        for target_x in draw.x..draw.right() {
            let coverage = clip.coverage(target_x, target_y);
            if coverage == 0 {
                continue;
            }
            let local_x = target_x - placement.destination.x;
            let local_y = target_y - placement.destination.y;
            let pixel = match image.filter {
                ImageFilter::Nearest => sample_image_nearest(image, placement, local_x, local_y),
                ImageFilter::Linear => sample_image_linear(image, placement, local_x, local_y),
            };
            let Some(canvas_x) = target_coord(target_x, offset.x, canvas.width()) else {
                continue;
            };
            let Some(canvas_y) = target_coord(target_y, offset.y, canvas.height()) else {
                continue;
            };
            canvas.blend_pixel(
                canvas_x,
                canvas_y,
                pixel_with_opacity_and_coverage(pixel, opacity, coverage),
            );
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ImagePlacement {
    pub(super) destination: Bounds,
    pub(super) source_x: u32,
    pub(super) source_y: u32,
    pub(super) source_width: u32,
    pub(super) source_height: u32,
}

impl ImagePlacement {
    fn source_x(self, destination_x: u32) -> u32 {
        map_scaled_coordinate(destination_x, self.destination.width, self.source_width)
            .saturating_add(self.source_x)
    }

    fn source_y(self, destination_y: u32) -> u32 {
        map_scaled_coordinate(destination_y, self.destination.height, self.source_height)
            .saturating_add(self.source_y)
    }
}

pub(super) fn sample_image_nearest(
    image: &Image,
    placement: ImagePlacement,
    destination_x: u32,
    destination_y: u32,
) -> [u8; 4] {
    let source_x = placement.source_x(destination_x);
    let source_y = placement.source_y(destination_y);
    image_pixel(image, source_x, source_y)
}

pub(super) fn sample_image_linear(
    image: &Image,
    placement: ImagePlacement,
    destination_x: u32,
    destination_y: u32,
) -> [u8; 4] {
    let (source_x, x_fraction) = map_linear_coordinate(
        destination_x,
        placement.destination.width,
        placement.source_width,
    );
    let (source_y, y_fraction) = map_linear_coordinate(
        destination_y,
        placement.destination.height,
        placement.source_height,
    );
    let source_x = source_x.saturating_add(placement.source_x);
    let source_y = source_y.saturating_add(placement.source_y);
    let next_x = (source_x + 1).min(placement.source_x + placement.source_width - 1);
    let next_y = (source_y + 1).min(placement.source_y + placement.source_height - 1);
    let top_left = image_pixel(image, source_x, source_y);
    let top_right = image_pixel(image, next_x, source_y);
    let bottom_left = image_pixel(image, source_x, next_y);
    let bottom_right = image_pixel(image, next_x, next_y);
    let top = interpolate_pixel(top_left, top_right, x_fraction);
    let bottom = interpolate_pixel(bottom_left, bottom_right, x_fraction);
    interpolate_pixel(top, bottom, y_fraction)
}

pub(super) fn image_pixel(image: &Image, x: u32, y: u32) -> [u8; 4] {
    let index = (y * image.stride + x * 4) as usize;
    let pixels = image.rgba();
    [
        pixels[index],
        pixels[index + 1],
        pixels[index + 2],
        pixels[index + 3],
    ]
}

pub(super) fn image_required_len(image: &Image) -> Option<usize> {
    let packed_row = image.width.checked_mul(4)?;
    if image.stride < packed_row {
        return None;
    }
    if image.height == 0 {
        return Some(0);
    }

    image
        .stride
        .checked_mul(image.height.saturating_sub(1))?
        .checked_add(packed_row)
        .map(|bytes| bytes as usize)
}

pub(super) fn interpolate_pixel(from: [u8; 4], to: [u8; 4], fraction: u32) -> [u8; 4] {
    let alpha = interpolate_u8_fixed(from[3], to[3], fraction);
    if alpha == 0 {
        return [0, 0, 0, 0];
    }

    [
        interpolate_premultiplied_component(from[0], from[3], to[0], to[3], alpha, fraction),
        interpolate_premultiplied_component(from[1], from[3], to[1], to[3], alpha, fraction),
        interpolate_premultiplied_component(from[2], from[3], to[2], to[3], alpha, fraction),
        alpha,
    ]
}

fn interpolate_premultiplied_component(
    from: u8,
    from_alpha: u8,
    to: u8,
    to_alpha: u8,
    alpha: u8,
    fraction: u32,
) -> u8 {
    let from = u32::from(from) * u32::from(from_alpha);
    let to = u32::from(to) * u32::from(to_alpha);
    let premultiplied = interpolate_u32_fixed(from, to, fraction);
    ((premultiplied + u32::from(alpha) / 2) / u32::from(alpha)).min(255) as u8
}

fn interpolate_u8_fixed(from: u8, to: u8, fraction: u32) -> u8 {
    interpolate_u32_fixed(u32::from(from), u32::from(to), fraction) as u8
}

fn interpolate_u32_fixed(from: u32, to: u32, fraction: u32) -> u32 {
    let inverse = 256_u32.saturating_sub(fraction);
    (from * inverse + to * fraction) / 256
}

pub(super) fn image_placement(image: &Image, rect: Bounds) -> ImagePlacement {
    match image.fit {
        ImageFit::None => {
            let width = rect.width.min(image.width);
            let height = rect.height.min(image.height);
            ImagePlacement {
                destination: align_rect(
                    rect,
                    Size::new(width, height),
                    image.align,
                    image.vertical_align,
                ),
                source_x: 0,
                source_y: 0,
                source_width: width,
                source_height: height,
            }
        }
        ImageFit::Fill => ImagePlacement {
            destination: rect,
            source_x: 0,
            source_y: 0,
            source_width: image.width,
            source_height: image.height,
        },
        ImageFit::Contain => {
            let size = contain_size(image.width, image.height, rect.width, rect.height);
            ImagePlacement {
                destination: align_rect(rect, size, image.align, image.vertical_align),
                source_x: 0,
                source_y: 0,
                source_width: image.width,
                source_height: image.height,
            }
        }
        ImageFit::Cover => {
            let source = cover_source_rect(image.width, image.height, rect.width, rect.height);
            ImagePlacement {
                destination: rect,
                source_x: source.x,
                source_y: source.y,
                source_width: source.width,
                source_height: source.height,
            }
        }
    }
}

fn align_rect(rect: Bounds, size: Size, align: Align, vertical_align: Align) -> Bounds {
    Bounds {
        x: rect
            .x
            .saturating_add(align_offset(align, rect.width, size.width)),
        y: rect
            .y
            .saturating_add(align_offset(vertical_align, rect.height, size.height)),
        width: size.width.min(rect.width),
        height: size.height.min(rect.height),
    }
}

fn contain_size(source_width: u32, source_height: u32, width: u32, height: u32) -> Size {
    if source_width == 0 || source_height == 0 || width == 0 || height == 0 {
        return Size::default();
    }
    if u64::from(width) * u64::from(source_height) <= u64::from(height) * u64::from(source_width) {
        Size::new(
            width,
            scaled_dimension(source_height, width, source_width).min(height),
        )
    } else {
        Size::new(
            scaled_dimension(source_width, height, source_height).min(width),
            height,
        )
    }
}

fn cover_source_rect(source_width: u32, source_height: u32, width: u32, height: u32) -> Bounds {
    if source_width == 0 || source_height == 0 || width == 0 || height == 0 {
        return Bounds::default();
    }

    if u64::from(width) * u64::from(source_height) >= u64::from(height) * u64::from(source_width) {
        let crop_height = scaled_dimension(source_width, height, width).min(source_height);
        Bounds::new(
            0,
            centered_crop_offset(crop_height, source_height),
            source_width,
            crop_height,
        )
    } else {
        let crop_width = scaled_dimension(source_height, width, height).min(source_width);
        Bounds::new(
            centered_crop_offset(crop_width, source_width),
            0,
            crop_width,
            source_height,
        )
    }
}

fn centered_crop_offset(crop: u32, source: u32) -> u32 {
    source.saturating_sub(crop) / 2
}

fn map_scaled_coordinate(destination: u32, destination_size: u32, source_size: u32) -> u32 {
    if destination_size == 0 || source_size == 0 {
        return 0;
    }
    ((u64::from(destination) * u64::from(source_size)) / u64::from(destination_size))
        .min(u64::from(source_size.saturating_sub(1))) as u32
}

fn map_linear_coordinate(destination: u32, destination_size: u32, source_size: u32) -> (u32, u32) {
    if destination_size <= 1 || source_size <= 1 {
        return (0, 0);
    }

    let scaled =
        u64::from(destination) * u64::from(source_size - 1) * 256 / u64::from(destination_size - 1);
    let index = (scaled / 256).min(u64::from(source_size - 1)) as u32;
    let fraction = (scaled % 256) as u32;
    (index, fraction)
}
