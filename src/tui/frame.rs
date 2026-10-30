//! Rounded squares: the radio box, and the frame around a component's icon, as images and in braille.

use crate::*;

/// The side of a square's image, in pixels.
const SIDE: u32 = 96;
/// A border's outer half side, in pixels.
const BORDER_HALF: f32 = 44.0;
/// A border's corner radius, in pixels.
const BORDER_RADIUS: f32 = 16.0;
/// A border's width, in pixels.
const STROKE: f32 = 7.0;
/// A radio box's fill's half side, in pixels; the moat lies between it and the border.
const FILL_HALF: f32 = 24.0;
/// A radio box's fill's corner radius, in pixels.
const FILL_RADIUS: f32 = 8.0;
/// The color of an icon's frame.
const FRAME: image::Rgba<u8> = image::Rgba([150, 150, 150, 255]);
/// A transparent pixel.
const CLEAR: image::Rgba<u8> = image::Rgba([0, 0, 0, 0]);

/// A mark's color, its radio box's border's and its fill's.
fn color(mark: Mark) -> image::Rgba<u8> {
    match mark {
        Mark::Absent | Mark::Added => image::Rgba([150, 150, 150, 255]),
        Mark::ToAdd => image::Rgba([40, 190, 90, 255]),
        Mark::ToRemove => image::Rgba([215, 60, 60, 255]),
    }
}

/// Whether a mark fills its radio box.
fn filled(mark: Mark) -> bool {
    !matches!(mark, Mark::Absent)
}

/// A radio box as an image: its border, its moat, and its fill, antialiased on transparency.
pub(crate) fn radio_image(mark: Mark) -> image::DynamicImage {
    let [red, green, blue, _] = color(mark).0;
    let mut out = image::RgbaImage::new(SIDE, SIDE);
    for (x, y, pixel) in out.enumerate_pixels_mut() {
        let (px, py) = centered(x, y);
        let fill = if filled(mark) { coverage(rounded(px, py, FILL_HALF, FILL_RADIUS)) } else { 0.0 };
        *pixel = image::Rgba([red, green, blue, opacity(ring(px, py) + fill)]);
    }
    out.into()
}

/// An icon framed as an image: a rounded square's border, and the icon inside it, its edges touching
/// the border and its corners clipped to the border's curve.
pub(crate) fn framed_image(icon: &image::DynamicImage) -> image::DynamicImage {
    let inside = BORDER_HALF - STROKE;
    let side = (2.0 * inside).round() as u32;
    let fitted = icon.resize(side, side, image::imageops::FilterType::Triangle).to_rgba8();
    let mut out = image::RgbaImage::new(SIDE, SIDE);
    let (left, top) = ((SIDE - fitted.width()) / 2, (SIDE - fitted.height()) / 2);
    image::imageops::overlay(&mut out, &fitted, i64::from(left), i64::from(top));
    let [red, green, blue, _] = FRAME.0;
    for (x, y, pixel) in out.enumerate_pixels_mut() {
        let (px, py) = centered(x, y);
        let [icon_red, icon_green, icon_blue, icon_alpha] = pixel.0;
        let border = ring(px, py);
        let below = f32::from(icon_alpha) / 255.0 * coverage(rounded(px, py, inside, BORDER_RADIUS - STROKE));
        let alpha = border + below * (1.0 - border);
        let mix = |over: u8, under: u8| {
            let total = f32::from(over) * border + f32::from(under) * below * (1.0 - border);
            if alpha > 0.0 { (total / alpha).round() as u8 } else { 0 }
        };
        *pixel = image::Rgba([mix(red, icon_red), mix(green, icon_green), mix(blue, icon_blue), opacity(alpha)]);
    }
    out.into()
}

/// A radio box as braille dots, eight by eight: a ring with its corners rounded off, a moat one dot
/// wide, and a fill four dots square.
pub(crate) fn radio_dots(mark: Mark) -> image::DynamicImage {
    let color = color(mark);
    let inside = |v: u32| (2..6).contains(&v);
    let mut out = ring_dots(color);
    if filled(mark) {
        for (x, y, pixel) in out.enumerate_pixels_mut() {
            if inside(x) && inside(y) {
                *pixel = color;
            }
        }
    }
    out.into()
}

/// An icon framed in braille dots, eight by eight: the ring, and the icon's bitmap fitted to the six
/// by six inside it.
pub(crate) fn framed_dots(bitmap: &image::DynamicImage) -> image::DynamicImage {
    let inside = image::imageops::resize(&bitmap.to_rgba8(), 6, 6, image::imageops::FilterType::Nearest);
    let mut out = ring_dots(FRAME);
    image::imageops::overlay(&mut out, &inside, 1, 1);
    out.into()
}

/// A ring of braille dots, eight by eight, its corners rounded off.
fn ring_dots(color: image::Rgba<u8>) -> image::RgbaImage {
    let edge = |v: u32| v == 0 || v == 7;
    image::RgbaImage::from_fn(8, 8, |x, y| if (edge(x) || edge(y)) && !(edge(x) && edge(y)) { color } else { CLEAR })
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
