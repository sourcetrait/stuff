//! The config.toml files: the directory's and each component's.

/// The title the header shows when the directory's config names none.
pub(crate) const DEFAULT_TITLE: &str = "Stuff";
/// The most characters a component's description may hold.
pub(crate) const DESCRIPTION_MAX: usize = 80;

/// The directory's config.toml as written.
#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct StuffConfigToml {
    /// The title the header shows.
    pub(crate) title: Option<String>,
}

/// The directory's configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StuffConfig {
    /// The title the header shows.
    pub(crate) title: String,
}

impl From<StuffConfigToml> for StuffConfig {
    fn from(toml: StuffConfigToml) -> Self {
        Self { title: toml.title.unwrap_or_else(|| DEFAULT_TITLE.to_string()) }
    }
}

/// A component's config.toml as written.
#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct ComponentConfigToml {
    /// The card's title.
    pub(crate) title: Option<String>,
    /// The card's summary, 80 characters at most.
    pub(crate) description: Option<String>,
    /// The components it depends on beside its parent.
    pub(crate) depends: Option<Vec<String>>,
}

/// A component's configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ComponentConfig {
    /// The card's title.
    pub(crate) title: String,
    /// The card's summary, 80 characters at most.
    pub(crate) description: String,
    /// The components it depends on beside its parent, by name.
    pub(crate) depends: Vec<String>,
}

impl TryFrom<ComponentConfigToml> for ComponentConfig {
    type Error = String;

    fn try_from(toml: ComponentConfigToml) -> std::result::Result<Self, Self::Error> {
        let title = toml.title.filter(|t| !t.trim().is_empty()).ok_or("title is missing")?;
        let description = toml.description.unwrap_or_default();
        let count = description.chars().count();
        if count > DESCRIPTION_MAX {
            return Err(format!(
                "description is {count} characters, past {DESCRIPTION_MAX}"
            ));
        }
        Ok(Self { title, description, depends: toml.depends.unwrap_or_default() })
    }
}
