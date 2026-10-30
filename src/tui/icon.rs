//! The header's icon and the radio boxes, images for a terminal with a graphics protocol.

use crate::*;

/// The header icon's cells, and a radio box's: four columns by two rows, near square in most fonts.
pub(crate) const ICON_SIZE: r::Size = r::Size::new(4, 2);
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

/// The images ready to draw: the header's icon and the radio boxes; none below the image tier.
pub(crate) struct Icons {
    /// The header's icon.
    pub(crate) header: Option<r::Protocol>,
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
    pub(crate) fn none() -> Self {
        Self { header: None, radios: None }
    }

    /// The header's icon and the radio boxes as images in the image tier, where the picker found a
    /// graphics protocol; none below it.
    pub(crate) fn load(catalog: &Catalog, picker: &r::Picker, background: Option<image::Rgba<u8>>, tier: Tier) -> Self {
        if tier != Tier::Image || !graphics(picker) {
            return Self::none();
        }
        let sixel = matches!(picker.protocol_type(), r::ProtocolType::Sixel);
        let background = background.filter(|_| sixel);
        let mut picker = picker.clone();
        picker.set_background_color(background);
        let images = Images { picker: &picker, background };
        let header = catalog
            .icon
            .as_deref()
            .and_then(|path| image::open(path).ok())
            .or_else(|| image::load_from_memory(DEFAULT_LOGO).ok())
            .and_then(|image| images.protocol(image));
        Self { header, radios: Some(Radios::load(&images)) }
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

/// An image laid over a solid background, its transparency gone.
fn flatten(image: &image::DynamicImage, background: image::Rgba<u8>) -> image::DynamicImage {
    let mut flat = image::DynamicImage::from(image::RgbaImage::from_pixel(image.width(), image.height(), background));
    image::imageops::overlay(&mut flat, image, 0, 0);
    flat
}
