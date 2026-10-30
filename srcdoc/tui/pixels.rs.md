# pixels.rs

## struct Pixels

A cell holds two pixels, the upper half block's color over the cell's
background. A cell whose two pixels match is drawn as a background alone,
which fills the cell in any terminal. A clear pixel leaves what is behind it,
so a tinted row shows through.
