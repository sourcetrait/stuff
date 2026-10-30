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

    /// Every icon, prepared for the terminal the picker found.
    pub(crate) fn load(catalog: &Catalog, picker: &r::Picker) -> Self {
        let header = protocol(picker, catalog.icon.as_deref(), DEFAULT_LOGO);
        let components = catalog
            .components
            .iter()
            .map(|component| protocol(picker, component.icon.as_deref(), DEFAULT_ICON))
            .collect();
        Self { header, components }
    }
}

/// An icon file fitted to its cells, the default when it is absent or unreadable.
fn protocol(picker: &r::Picker, path: Option<&Path>, fallback: &[u8]) -> Option<r::Protocol> {
    let image = path
        .and_then(|path| image::open(path).ok())
        .or_else(|| image::load_from_memory(fallback).ok())?;
    picker.new_protocol(image, ICON_SIZE, r::Resize::Fit(None)).ok()
}
