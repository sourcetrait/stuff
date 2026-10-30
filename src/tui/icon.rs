//! The icons and the radio boxes, images for a terminal with a graphics protocol.

use crate::*;

/// An icon's cells, and a radio box's: four columns by two rows, near square in most fonts.
pub(crate) const ICON_SIZE: r::Size = r::Size::new(4, 2);
/// The default component icon: three dots in a triangle.
const DEFAULT_ICON: &[u8] = include_bytes!("../../assets/icon/default.png");
/// The default header icon: the sourcetrait logo.
const DEFAULT_LOGO: &[u8] = include_bytes!("../../assets/icon/sourcetrait.png");

/// The radio box of each mark, as an image.
pub(crate) struct Radios {
    absent: Option<r::Protocol>,
    added: Option<r::Protocol>,
    to_add: Option<r::Protocol>,
    to_remove: Option<r::Protocol>,
}

impl Radios {
    /// Each mark's radio box.
    fn load(images: &Images) -> Self {
        let radio = |mark: Mark| images.protocol(tui::frame::radio_image(mark));
        Self {
            absent: radio(Mark::Absent),
            added: radio(Mark::Added),
            to_add: radio(Mark::ToAdd),
            to_remove: radio(Mark::ToRemove),
        }
    }

    /// The radio box of a mark.
    pub(crate) fn get(&self, mark: Mark) -> Option<&r::Protocol> {
        match mark {
            Mark::Absent => self.absent.as_ref(),
            Mark::Added => self.added.as_ref(),
            Mark::ToAdd => self.to_add.as_ref(),
            Mark::ToRemove => self.to_remove.as_ref(),
        }
    }
}

/// The images ready to draw: the header's icon, each component's by index, and the radio boxes;
/// none below the image tier.
pub(crate) struct Icons {
    /// The header's icon.
    pub(crate) header: Option<r::Protocol>,
    /// Each component's icon, framed, by index.
    pub(crate) components: Vec<Option<r::Protocol>>,
    /// The radio box of each mark.
    pub(crate) radios: Option<Radios>,
}

/// What draws the images: the picker, and the background a sixel is flattened onto, where known.
struct Images<'a> {
    picker: &'a r::Picker,
    background: Option<image::Rgba<u8>>,
}

impl Icons {
    /// No images, for a terminal below the image tier and for a test.
    pub(crate) fn none(count: usize) -> Self {
        Self { header: None, components: (0..count).map(|_| None).collect(), radios: None }
    }

    /// Every icon and radio box as an image in the image tier, where the picker found a graphics
    /// protocol; none below it.
    pub(crate) fn load(catalog: &Catalog, picker: &r::Picker, background: Option<image::Rgba<u8>>, tier: Tier) -> Self {
        if tier != Tier::Image || !graphics(picker) {
            return Self::none(catalog.components.len());
        }
        let sixel = matches!(picker.protocol_type(), r::ProtocolType::Sixel);
        let background = background.filter(|_| sixel);
        let mut picker = picker.clone();
        picker.set_background_color(background);
        let images = Images { picker: &picker, background };
        let header = icon(&images, &catalog.logo, DEFAULT_LOGO, false);
        let components = catalog.components.iter().map(|component| icon(&images, &component.logo, DEFAULT_ICON, true)).collect();
        Self { header, components, radios: Some(Radios::load(&images)) }
    }
}

impl Images<'_> {
    /// An image for the graphics protocol, fitted to an icon's cells; a sixel, whose transparency is
    /// all or nothing, is flattened onto the terminal's background where it is known.
    fn protocol(&self, image: image::DynamicImage) -> Option<r::Protocol> {
        let image = match self.background {
            Some(background) => flatten(&image, background),
            None => image,
        };
        self.picker.new_protocol(image, ICON_SIZE, r::Resize::Fit(None)).ok()
    }
}

/// Whether the picker found the terminal speaking a graphics protocol.
pub(crate) fn graphics(picker: &r::Picker) -> bool {
    !matches!(picker.protocol_type(), r::ProtocolType::Halfblocks)
}

/// The terminal's picker, the terminal asked for its background as well.
pub(crate) fn picker() -> r::Picker {
    let options = r::QueryStdioOptions { terminal_background_color_osc: true, ..Default::default() };
    r::Picker::from_query_stdio_with_options(options).unwrap_or_else(|_| r::Picker::halfblocks())
}

/// The terminal's background, where it reported one.
pub(crate) fn background(picker: &r::Picker) -> Option<image::Rgba<u8>> {
    picker.capabilities().iter().find_map(|capability| match capability {
        r::Capability::Background(red, green, blue) => Some(image::Rgba([*red, *green, *blue, 255])),
        _ => None,
    })
}

/// An icon as an image: the png, else the bmp enlarged pixel for pixel, else the default; a
/// component's framed.
fn icon(images: &Images, logo: &Logo, default: &[u8], framed: bool) -> Option<r::Protocol> {
    use image::imageops::FilterType::{Nearest, Triangle};
    let open = |path: Option<&Path>| path.and_then(|path| image::open(path).ok());
    let drawn = |image: image::DynamicImage, filter| {
        let image = if framed {
            tui::frame::framed_image(&image, filter)
        } else if filter == Nearest {
            tui::frame::enlarged(&image)
        } else {
            image
        };
        images.protocol(image)
    };
    open(logo.png.as_deref())
        .and_then(|image| drawn(image, Triangle))
        .or_else(|| open(logo.bmp.as_deref()).and_then(|bitmap| drawn(bitmap, Nearest)))
        .or_else(|| image::load_from_memory(default).ok().and_then(|image| drawn(image, Triangle)))
}

/// An image laid over a solid background, its transparency gone.
fn flatten(image: &image::DynamicImage, background: image::Rgba<u8>) -> image::DynamicImage {
    let mut flat = image::DynamicImage::from(image::RgbaImage::from_pixel(image.width(), image.height(), background));
    image::imageops::overlay(&mut flat, image, 0, 0);
    flat
}
