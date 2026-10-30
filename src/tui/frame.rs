//! The radio box as an image: a rounded square's border, a moat, and the mark's fill inside.

use crate::*;

/// The side of a radio box's image, in pixels: large enough to be scaled down to any cell size.
const SIDE: u32 = 192;
/// The border's outer half side, in pixels.
const BORDER_HALF: f32 = 88.0;
/// The border's corner radius, in pixels.
const BORDER_RADIUS: f32 = 32.0;
/// The border's width, in pixels.
const STROKE: f32 = 14.0;
/// The fill's half side, in pixels; the moat lies between it and the border.
const FILL_HALF: f32 = 48.0;
/// The fill's corner radius, in pixels.
const FILL_RADIUS: f32 = 16.0;

/// A mark's color, its radio box's border's and its fill's.
fn color(mark: Mark) -> image::Rgba<u8> {
    match mark {
        Mark::Absent | Mark::Added => image::Rgba([150, 150, 150, 255]),
        Mark::ToAdd => image::Rgba([40, 190, 90, 255]),
        Mark::ToRemove => image::Rgba([215, 60, 60, 255]),
    }
}

/// A radio box as an image: its border, its moat, and its fill, antialiased on transparency.
pub(crate) fn radio_image(mark: Mark) -> image::DynamicImage {
    let [red, green, blue, _] = color(mark).0;
    let filled = !matches!(mark, Mark::Absent);
    let mut out = image::RgbaImage::new(SIDE, SIDE);
    for (x, y, pixel) in out.enumerate_pixels_mut() {
        let (px, py) = centered(x, y);
        let fill = if filled { coverage(rounded(px, py, FILL_HALF, FILL_RADIUS)) } else { 0.0 };
        *pixel = image::Rgba([red, green, blue, opacity(ring(px, py) + fill)]);
    }
    out.into()
}

/// A pixel's center, relative to the image's.
fn centered(x: u32, y: u32) -> (f32, f32) {
    let center = SIDE as f32 / 2.0;
    (x as f32 + 0.5 - center, y as f32 + 0.5 - center)
}

/// How much of a pixel the border covers.
fn ring(x: f32, y: f32) -> f32 {
    let outer = rounded(x, y, BORDER_HALF, BORDER_RADIUS);
    coverage(outer) - coverage(outer + STROKE)
}

/// The signed distance from a point to a rounded square centered on the origin, negative inside.
fn rounded(x: f32, y: f32, half: f32, radius: f32) -> f32 {
    let qx = x.abs() - (half - radius);
    let qy = y.abs() - (half - radius);
    qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - radius
}

/// How much of a pixel lies inside a shape, from the shape's signed distance at the pixel's center.
fn coverage(distance: f32) -> f32 {
    (0.5 - distance).clamp(0.0, 1.0)
}

/// A coverage as an alpha byte.
fn opacity(alpha: f32) -> u8 {
    (alpha.clamp(0.0, 1.0) * 255.0).round() as u8
}
