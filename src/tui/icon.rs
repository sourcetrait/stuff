//! The icons and the radio boxes, drawn as the terminal's tier allows.

use crate::*;

/// An icon's cells, and a radio box's: four columns by two rows, near square in most fonts.
pub(crate) const ICON_SIZE: r::Size = r::Size::new(4, 2);
/// The default component icon as an image: three dots in a triangle.
const DEFAULT_ICON_PNG: &[u8] = include_bytes!("../../assets/icon/default.png");
/// The default component icon as a bitmap, four pixels square.
const DEFAULT_ICON_BMP: &[u8] = include_bytes!("../../assets/icon/default.bmp");
/// The default header icon as an image: the sourcetrait logo.
const DEFAULT_LOGO_PNG: &[u8] = include_bytes!("../../assets/icon/sourcetrait.png");
/// The default header icon as a bitmap, four pixels square.
const DEFAULT_LOGO_BMP: &[u8] = include_bytes!("../../assets/icon/sourcetrait.bmp");

/// An icon or a radio box as the terminal draws it.
pub(crate) enum Icon {
    /// An image, through the terminal's graphics protocol.
    Image(r::Protocol),
    /// A bitmap in half-block pixels.
    Pixels(Pixels),
    /// A box filled with a color.
    Fill(r::Color),
}

/// What draws the images: the picker, and the background a sixel is flattened onto, where known.
struct Images<'a> {
    picker: &'a r::Picker,
    background: Option<image::Rgba<u8>>,
}

/// The radio box of each mark, as the terminal draws it.
pub(crate) struct Radios {
    absent: Icon,
    added: Icon,
    to_add: Icon,
    to_remove: Icon,
}

impl Radios {
    /// Each mark's radio box: an image when images are drawn, a box filled in the mark's color otherwise.
    fn load(images: Option<&Images>) -> Self {
        let radio = |mark: Mark| {
            let fill = match mark {
                Mark::Absent => r::Color::DarkGray,
                Mark::Added => r::Color::Gray,
                Mark::ToAdd => r::Color::Green,
                Mark::ToRemove => r::Color::Red,
            };
            images.and_then(|images| images.protocol(tui::frame::radio_image(mark))).unwrap_or(Icon::Fill(fill))
        };
        Self {
            absent: radio(Mark::Absent),
            added: radio(Mark::Added),
            to_add: radio(Mark::ToAdd),
            to_remove: radio(Mark::ToRemove),
        }
    }

    /// The radio box of a mark.
    pub(crate) fn get(&self, mark: Mark) -> &Icon {
        match mark {
            Mark::Absent => &self.absent,
            Mark::Added => &self.added,
            Mark::ToAdd => &self.to_add,
            Mark::ToRemove => &self.to_remove,
        }
    }
}

/// The icons ready to draw: the header's, each component's by index, and the radio boxes.
pub(crate) struct Icons {
    /// The header's icon.
    pub(crate) header: Option<Icon>,
    /// Each component's icon, by index.
    pub(crate) components: Vec<Option<Icon>>,
    /// The radio box of each mark.
    pub(crate) radios: Radios,
}

impl Icons {
    /// No icons, and filled radio boxes, for a test.
    #[cfg(test)]
    pub(crate) fn none(count: usize) -> Self {
        Self { header: None, components: (0..count).map(|_| None).collect(), radios: Radios::load(None) }
    }

    /// Every icon and radio box: images in the image tier, where the picker found a graphics protocol;
    /// half-block pixels and filled boxes otherwise.
    pub(crate) fn load(catalog: &Catalog, picker: &r::Picker, background: Option<image::Rgba<u8>>, tier: Tier) -> Self {
        let sixel = matches!(picker.protocol_type(), r::ProtocolType::Sixel);
        let mut picker = picker.clone();
        picker.set_background_color(background.filter(|_| sixel));
        let images = (tier == Tier::Image && graphics(&picker))
            .then_some(Images { picker: &picker, background: background.filter(|_| sixel) });
        let header = icon(images.as_ref(), &catalog.logo, (DEFAULT_LOGO_PNG, DEFAULT_LOGO_BMP), false);
        let components = catalog
            .components
            .iter()
            .map(|component| icon(images.as_ref(), &component.logo, (DEFAULT_ICON_PNG, DEFAULT_ICON_BMP), true))
            .collect();
        Self { header, components, radios: Radios::load(images.as_ref()) }
    }
}

impl Images<'_> {
    /// An image for the graphics protocol, fitted to an icon's cells; a sixel, whose transparency is
    /// all or nothing, is flattened onto the terminal's background where it is known.
    fn protocol(&self, image: image::DynamicImage) -> Option<Icon> {
        let image = match self.background {
            Some(background) => flatten(&image, background),
            None => image,
        };
        self.picker.new_protocol(image, ICON_SIZE, r::Resize::Fit(None)).ok().map(Icon::Image)
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

/// An icon by what is drawn and what is given. With images: the png, else the bmp enlarged, else the
/// default png. Without: the bmp in half-block pixels, else the default bmp. A component's image is
/// framed.
fn icon(images: Option<&Images>, logo: &Logo, (default_png, default_bmp): (&[u8], &[u8]), framed: bool) -> Option<Icon> {
    use image::imageops::FilterType::{Nearest, Triangle};
    let open = |path: Option<&Path>| path.and_then(|path| image::open(path).ok());
    let bitmap = || open(logo.bmp.as_deref());
    let Some(images) = images else {
        let bitmap = bitmap().or_else(|| image::load_from_memory(default_bmp).ok())?;
        return Some(Icon::Pixels(Pixels::new(&bitmap, ICON_SIZE)));
    };
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
        .or_else(|| bitmap().and_then(|bitmap| drawn(bitmap, Nearest)))
        .or_else(|| image::load_from_memory(default_png).ok().and_then(|image| drawn(image, Triangle)))
}

/// An image laid over a solid background, its transparency gone.
fn flatten(image: &image::DynamicImage, background: image::Rgba<u8>) -> image::DynamicImage {
    let mut flat = image::DynamicImage::from(image::RgbaImage::from_pixel(image.width(), image.height(), background));
    image::imageops::overlay(&mut flat, image, 0, 0);
    flat
}
