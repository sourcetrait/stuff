//! The icons and the radio boxes, as the terminal can draw them.

use crate::*;

/// An icon's cells, and a radio box's: four columns by two rows, near square in most fonts.
pub(crate) const ICON_SIZE: r::Size = r::Size::new(4, 2);
/// The default component icon as an image: three dots in a triangle.
const DEFAULT_ICON_PNG: &[u8] = include_bytes!("../../assets/icon/default.png");
/// The default component icon for braille, six by six inside its frame.
const DEFAULT_ICON_BMP: &[u8] = include_bytes!("../../assets/icon/default.bmp");
/// The default header icon as an image: the sourcetrait logo.
const DEFAULT_LOGO_PNG: &[u8] = include_bytes!("../../assets/icon/sourcetrait.png");
/// The default header icon for braille, eight by eight.
const DEFAULT_LOGO_BMP: &[u8] = include_bytes!("../../assets/icon/sourcetrait.bmp");

/// An icon as the terminal draws it.
pub(crate) enum Icon {
    /// An image, through the terminal's graphics protocol.
    Image(r::Protocol),
    /// A bitmap in braille.
    Braille(Braille),
}

/// The radio box of each mark, as the terminal draws it.
pub(crate) struct Radios {
    absent: Icon,
    added: Icon,
    to_add: Icon,
    to_remove: Icon,
}

impl Radios {
    /// Each mark's radio box: an image where the terminal speaks a graphics protocol, braille otherwise.
    fn load(picker: Option<&r::Picker>, background: Option<image::Rgba<u8>>) -> Self {
        let radio = |mark: Mark| {
            picker
                .filter(|picker| graphics(picker))
                .and_then(|picker| protocol(picker, background, tui::frame::radio_image(mark)))
                .unwrap_or_else(|| Icon::Braille(Braille::new(&tui::frame::radio_dots(mark), ICON_SIZE)))
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
    /// Each component's icon, framed, by index.
    pub(crate) components: Vec<Option<Icon>>,
    /// The radio box of each mark.
    pub(crate) radios: Radios,
}

impl Icons {
    /// No icons, the radio boxes in braille, for a test.
    #[cfg(test)]
    pub(crate) fn none(count: usize) -> Self {
        Self { header: None, components: (0..count).map(|_| None).collect(), radios: Radios::load(None, None) }
    }

    /// Every icon and radio box, as the terminal the picker found can draw it, on its background
    /// where known.
    pub(crate) fn load(catalog: &Catalog, picker: &r::Picker, background: Option<image::Rgba<u8>>) -> Self {
        let mut picker = picker.clone();
        picker.set_background_color(background);
        let header = icon(&picker, background, &catalog.logo, (DEFAULT_LOGO_PNG, DEFAULT_LOGO_BMP), false);
        let components = catalog
            .components
            .iter()
            .map(|component| icon(&picker, background, &component.logo, (DEFAULT_ICON_PNG, DEFAULT_ICON_BMP), true))
            .collect();
        Self { header, components, radios: Radios::load(Some(&picker), background) }
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

/// An icon by what the terminal draws and what is given: the png where the terminal speaks a
/// graphics protocol, else the bmp in braille, else the default, a png or a bmp, the same way.
fn icon(
    picker: &r::Picker,
    background: Option<image::Rgba<u8>>,
    logo: &Logo,
    (default_png, default_bmp): (&[u8], &[u8]),
    framed: bool,
) -> Option<Icon> {
    let open = |path: Option<&Path>| path.and_then(|path| image::open(path).ok());
    let image = |image: image::DynamicImage| {
        let image = if framed { tui::frame::framed_image(&image) } else { image };
        protocol(picker, background, image)
    };
    let braille = |bitmap: image::DynamicImage| {
        let bitmap = if framed { tui::frame::framed_dots(&bitmap) } else { bitmap };
        Icon::Braille(Braille::new(&bitmap, ICON_SIZE))
    };
    if graphics(picker)
        && let Some(icon) = open(logo.png.as_deref()).and_then(image)
    {
        return Some(icon);
    }
    if let Some(bitmap) = open(logo.bmp.as_deref()) {
        return Some(braille(bitmap));
    }
    if graphics(picker)
        && let Some(icon) = image::load_from_memory(default_png).ok().and_then(image)
    {
        return Some(icon);
    }
    image::load_from_memory(default_bmp).ok().map(braille)
}

/// An image for the graphics protocol, fitted to an icon's cells, flattened onto the background where known.
fn protocol(picker: &r::Picker, background: Option<image::Rgba<u8>>, image: image::DynamicImage) -> Option<Icon> {
    let image = match background {
        Some(background) => flatten(&image, background),
        None => image,
    };
    picker.new_protocol(image, ICON_SIZE, r::Resize::Fit(None)).ok().map(Icon::Image)
}

/// An image laid over a solid background, its transparency gone.
fn flatten(image: &image::DynamicImage, background: image::Rgba<u8>) -> image::DynamicImage {
    let mut flat = image::DynamicImage::from(image::RgbaImage::from_pixel(image.width(), image.height(), background));
    image::imageops::overlay(&mut flat, image, 0, 0);
    flat
}
