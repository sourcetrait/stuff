//! The icons: the header's and each component's PNG, or a default, as the terminal draws them.

use crate::*;

/// An icon's cells: four columns by two rows, near square in most fonts.
pub(crate) const ICON_SIZE: r::Size = r::Size::new(4, 2);
/// The default component icon: three dots in a triangle.
const DEFAULT_ICON: &[u8] = include_bytes!("../../assets/icon/default.png");
/// The default header icon: the sourcetrait logo.
const DEFAULT_LOGO: &[u8] = include_bytes!("../../assets/icon/sourcetrait.png");

/// The icons ready to draw: the header's and each component's, by index.
pub(crate) struct Icons {
    /// The header's icon.
    pub(crate) header: Option<r::Protocol>,
    /// Each component's icon, by index.
    pub(crate) components: Vec<Option<r::Protocol>>,
}

impl Icons {
    /// No icons, for a test that draws none.
    #[cfg(test)]
    pub(crate) fn none(count: usize) -> Self {
        Self { header: None, components: (0..count).map(|_| None).collect() }
    }

    /// Every icon, prepared for the terminal the picker found, on the terminal's background where it is known.
    pub(crate) fn load(catalog: &Catalog, picker: &r::Picker, background: Option<image::Rgba<u8>>) -> Self {
        let mut picker = picker.clone();
        picker.set_background_color(background);
        let header = protocol(&picker, background, catalog.icon.as_deref(), DEFAULT_LOGO);
        let components = catalog
            .components
            .iter()
            .map(|component| protocol(&picker, background, component.icon.as_deref(), DEFAULT_ICON))
            .collect();
        Self { header, components }
    }
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

/// An icon file fitted to its cells and flattened onto the background, the default when it is absent or unreadable.
fn protocol(
    picker: &r::Picker,
    background: Option<image::Rgba<u8>>,
    path: Option<&Path>,
    fallback: &[u8],
) -> Option<r::Protocol> {
    let mut image = path
        .and_then(|path| image::open(path).ok())
        .or_else(|| image::load_from_memory(fallback).ok())?;
    if let Some(background) = background {
        let mut flat = image::DynamicImage::from(image::RgbaImage::from_pixel(image.width(), image.height(), background));
        image::imageops::overlay(&mut flat, &image, 0, 0);
        image = flat;
    }
    picker.new_protocol(image, ICON_SIZE, r::Resize::Fit(None)).ok()
}
