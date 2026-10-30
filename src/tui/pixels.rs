//! A small bitmap as half-block pixels, two to a cell, one over the other, for a terminal without images.

use crate::*;

/// One cell of pixels: its top pixel's color and its bottom's, `None` where the bitmap is clear.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PixelCell {
    top: Option<r::Color>,
    bottom: Option<r::Color>,
}

/// A bitmap in half-block pixels.
#[derive(Debug, Clone)]
pub(crate) struct Pixels {
    /// Its size in cells.
    size: r::Size,
    /// Its cells, row by row.
    cells: Vec<PixelCell>,
}

impl Pixels {
    /// A bitmap fitted to its cells' pixels, one across and two down a cell, its black and transparent
    /// pixels clear.
    pub(crate) fn new(bitmap: &image::DynamicImage, size: r::Size) -> Self {
        let width = u32::from(size.width);
        let height = u32::from(size.height) * 2;
        let pixels = image::imageops::resize(&bitmap.to_rgba8(), width, height, image::imageops::FilterType::Nearest);
        let color = |x: u32, y: u32| {
            let [red, green, blue, alpha] = pixels.get_pixel(x, y).0;
            (alpha >= 128 && red.max(green).max(blue) >= 64).then_some(r::Color::Rgb(red, green, blue))
        };
        let cells = (0..u32::from(size.height))
            .flat_map(|row| (0..width).map(move |column| (column, row)))
            .map(|(column, row)| PixelCell { top: color(column, row * 2), bottom: color(column, row * 2 + 1) })
            .collect();
        Self { size, cells }
    }
}

impl r::Widget for &Pixels {
    fn render(self, area: r::Rect, buffer: &mut r::Buffer) {
        let rows = self.cells.chunks(usize::from(self.size.width.max(1)));
        for (row, y) in rows.zip(area.top()..area.bottom()) {
            for (cell, x) in row.iter().zip(area.left()..area.right()) {
                let target = &mut buffer[(x, y)];
                match (cell.top, cell.bottom) {
                    (Some(top), Some(bottom)) if top == bottom => {
                        target.set_char(' ').set_bg(top);
                    }
                    (Some(top), Some(bottom)) => {
                        target.set_char('▀').set_fg(top).set_bg(bottom);
                    }
                    (Some(top), None) => {
                        target.set_char('▀').set_fg(top);
                    }
                    (None, Some(bottom)) => {
                        target.set_char('▄').set_fg(bottom);
                    }
                    (None, None) => {}
                }
            }
        }
    }
}
