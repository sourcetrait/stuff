//! A picture in braille: each pixel of a small bitmap a dot, for a terminal with no graphics protocol.

use crate::*;

/// Each dot's bit in a braille character, by column, then by row.
const BITS: [[u32; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];
/// The first braille character, its dots all empty.
const BLANK: u32 = 0x2800;

/// One cell of braille: its character, and its dots' color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BrailleCell {
    /// A braille character, or a space where the cell has no dots.
    pub(crate) symbol: char,
    /// The dots' color.
    pub(crate) color: Option<r::Color>,
}

/// A bitmap in braille.
#[derive(Debug, Clone)]
pub(crate) struct Braille {
    /// Its size in cells.
    pub(crate) size: r::Size,
    /// Its cells, row by row.
    pub(crate) cells: Vec<BrailleCell>,
}

impl Braille {
    /// A bitmap fitted to its cells' dots, two across and four down a cell, each pixel neither black nor
    /// transparent a dot, each cell's dots in their pixels' average color.
    pub(crate) fn new(image: &image::DynamicImage, size: r::Size) -> Self {
        let width = u32::from(size.width) * 2;
        let height = u32::from(size.height) * 4;
        let pixels = image::imageops::resize(&image.to_rgba8(), width, height, image::imageops::FilterType::Nearest);
        let mut cells = Vec::with_capacity(usize::from(size.width) * usize::from(size.height));
        for row in 0..u32::from(size.height) {
            for column in 0..u32::from(size.width) {
                let mut bits = 0;
                let mut sum = [0u32; 3];
                let mut dots = 0u32;
                for (column_bits, dx) in BITS.iter().zip(0u32..) {
                    for (bit, dy) in column_bits.iter().zip(0u32..) {
                        let [red, green, blue, alpha] = pixels.get_pixel(column * 2 + dx, row * 4 + dy).0;
                        if alpha >= 128 && red.max(green).max(blue) >= 64 {
                            bits |= bit;
                            sum = [sum[0] + u32::from(red), sum[1] + u32::from(green), sum[2] + u32::from(blue)];
                            dots += 1;
                        }
                    }
                }
                let symbol = if bits == 0 { ' ' } else { char::from_u32(BLANK + bits).unwrap_or(' ') };
                let channel = |total: u32| u8::try_from(total / dots.max(1)).unwrap_or(u8::MAX);
                let color = (dots > 0).then(|| r::Color::Rgb(channel(sum[0]), channel(sum[1]), channel(sum[2])));
                cells.push(BrailleCell { symbol, color });
            }
        }
        Self { size, cells }
    }
}

impl r::Widget for &Braille {
    fn render(self, area: r::Rect, buffer: &mut r::Buffer) {
        let rows = self.cells.chunks(usize::from(self.size.width.max(1)));
        for (row, y) in rows.zip(area.top()..area.bottom()) {
            for (cell, x) in row.iter().zip(area.left()..area.right()) {
                buffer[(x, y)].set_char(cell.symbol).set_fg(cell.color.unwrap_or(r::Color::Reset));
            }
        }
    }
}
