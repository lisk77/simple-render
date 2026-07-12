use super::*;

pub(super) fn multiply_coverage(a: u8, b: u8) -> u8 {
    ((u16::from(a) * u16::from(b)) / 255) as u8
}

pub(super) fn rounded_rect_bounds_coverage_with_antialias(
    bounds: Bounds,
    x: u32,
    y: u32,
    radii: CornerRadius,
    anti_alias: bool,
) -> u8 {
    if x < bounds.x || y < bounds.y || x >= bounds.right() || y >= bounds.bottom() {
        return 0;
    }

    rounded_rect_coverage_with_antialias(
        x - bounds.x,
        y - bounds.y,
        bounds.width,
        bounds.height,
        radii,
        anti_alias,
    )
}

pub(in crate::ui) fn rounded_rect_coverage(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radii: CornerRadius,
) -> u8 {
    if x >= width || y >= height {
        return 0;
    }

    let radii = radii.clamp_to(width, height);
    if radii.is_zero() || inside_cornerless_center(x, y, width, height, radii) {
        return 255;
    }

    if x < radii.top_left && y < radii.top_left {
        return corner_coverage(x, y, radii.top_left, radii.top_left, radii.top_left);
    }
    if x >= width.saturating_sub(radii.top_right) && y < radii.top_right {
        return corner_coverage(
            x,
            y,
            width.saturating_sub(radii.top_right),
            radii.top_right,
            radii.top_right,
        );
    }
    if x >= width.saturating_sub(radii.bottom_right)
        && y >= height.saturating_sub(radii.bottom_right)
    {
        return corner_coverage(
            x,
            y,
            width.saturating_sub(radii.bottom_right),
            height.saturating_sub(radii.bottom_right),
            radii.bottom_right,
        );
    }
    if x < radii.bottom_left && y >= height.saturating_sub(radii.bottom_left) {
        return corner_coverage(
            x,
            y,
            radii.bottom_left,
            height.saturating_sub(radii.bottom_left),
            radii.bottom_left,
        );
    }

    255
}

pub(in crate::ui) fn rounded_rect_contains(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radii: CornerRadius,
) -> bool {
    if x >= width || y >= height {
        return false;
    }

    let radii = radii.clamp_to(width, height);
    if radii.is_zero() || inside_cornerless_center(x, y, width, height, radii) {
        return true;
    }

    if x < radii.top_left && y < radii.top_left {
        return corner_contains(x, y, radii.top_left, radii.top_left, radii.top_left);
    }
    if x >= width.saturating_sub(radii.top_right) && y < radii.top_right {
        return corner_contains(
            x,
            y,
            width.saturating_sub(radii.top_right),
            radii.top_right,
            radii.top_right,
        );
    }
    if x >= width.saturating_sub(radii.bottom_right)
        && y >= height.saturating_sub(radii.bottom_right)
    {
        return corner_contains(
            x,
            y,
            width.saturating_sub(radii.bottom_right),
            height.saturating_sub(radii.bottom_right),
            radii.bottom_right,
        );
    }
    if x < radii.bottom_left && y >= height.saturating_sub(radii.bottom_left) {
        return corner_contains(
            x,
            y,
            radii.bottom_left,
            height.saturating_sub(radii.bottom_left),
            radii.bottom_left,
        );
    }

    true
}

fn inside_cornerless_center(x: u32, y: u32, width: u32, height: u32, radii: CornerRadius) -> bool {
    let left = radii.top_left.max(radii.bottom_left);
    let right = width.saturating_sub(radii.top_right.max(radii.bottom_right));
    let top = radii.top_left.max(radii.top_right);
    let bottom = height.saturating_sub(radii.bottom_left.max(radii.bottom_right));
    (x >= left && x < right) || (y >= top && y < bottom)
}

fn corner_contains(x: u32, y: u32, center_x: u32, center_y: u32, radius: u32) -> bool {
    if radius == 0 {
        return true;
    }

    let center_x = u64::from(center_x) * 2;
    let center_y = u64::from(center_y) * 2;
    let pixel_x = u64::from(x) * 2 + 1;
    let pixel_y = u64::from(y) * 2 + 1;
    let radius = u64::from(radius) * 2;
    center_x.abs_diff(pixel_x).pow(2) + center_y.abs_diff(pixel_y).pow(2) <= radius.pow(2)
}

fn corner_coverage(x: u32, y: u32, center_x: u32, center_y: u32, radius: u32) -> u8 {
    if radius == 0 {
        return 255;
    }

    const SCALE: i64 = 8;
    const SAMPLES: [i64; 4] = [1, 3, 5, 7];

    let center_x = i64::from(center_x) * SCALE;
    let center_y = i64::from(center_y) * SCALE;
    let radius = i64::from(radius) * SCALE;
    let radius_squared = radius * radius;
    let mut covered = 0_u32;

    for sample_y in SAMPLES {
        for sample_x in SAMPLES {
            let pixel_x = i64::from(x) * SCALE + sample_x;
            let pixel_y = i64::from(y) * SCALE + sample_y;
            let dx = pixel_x - center_x;
            let dy = pixel_y - center_y;
            if dx * dx + dy * dy <= radius_squared {
                covered += 1;
            }
        }
    }

    (covered * 255 / 16) as u8
}

pub(super) fn rounded_rect_coverage_with_antialias(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radii: CornerRadius,
    anti_alias: bool,
) -> u8 {
    if anti_alias {
        rounded_rect_coverage(x, y, width, height, radii)
    } else if rounded_rect_contains(x, y, width, height, radii) {
        255
    } else {
        0
    }
}
