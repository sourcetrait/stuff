//! The stuff.toml and component.toml files: the directory's and each component's.

use crate::*;

/// The title the header shows when the directory's stuff.toml names none.
pub(crate) const DEFAULT_TITLE: &str = "Stuff";
/// The most characters a component's description may hold.
pub(crate) const DESCRIPTION_MAX: usize = 80;

/// The directory's stuff.toml as written.
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

/// A script's block in a component.toml as written: `[present]`, `[add]`, or `[remove]`.
#[derive(Debug, Default, Clone, Copy, serde::Deserialize)]
pub(crate) struct InterfaceToml {
    /// Whether the script needs sudo.
    #[serde(rename = "super")]
    pub(crate) superuser: Option<bool>,
}

impl InterfaceToml {
    /// Whether a block, if written, marks its script super.
    fn marks(block: Option<Self>) -> bool {
        block.and_then(|block| block.superuser).unwrap_or(false)
    }
}

/// A component's component.toml as written.
#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct ComponentConfigToml {
    /// The card's title.
    pub(crate) title: Option<String>,
    /// The card's summary, 80 characters at most.
    pub(crate) description: Option<String>,
    /// The components it depends on beside its parent.
    pub(crate) depends: Option<Vec<String>>,
    /// The present script's block.
    pub(crate) present: Option<InterfaceToml>,
    /// The add script's block.
    pub(crate) add: Option<InterfaceToml>,
    /// The remove script's block.
    pub(crate) remove: Option<InterfaceToml>,
}

/// Which of a component's scripts are super: each calls sudo itself, so it needs the credential.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Supers {
    /// The present script.
    pub(crate) present: bool,
    /// The add script.
    pub(crate) add: bool,
    /// The remove script.
    pub(crate) remove: bool,
}

impl Supers {
    /// Whether a script is super.
    pub(crate) fn of(self, script: Script) -> bool {
        match script {
            Script::Present => self.present,
            Script::Add => self.add,
            Script::Remove => self.remove,
        }
    }
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
    /// Which of its scripts are super.
    pub(crate) supers: Supers,
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
        let supers = Supers {
            present: InterfaceToml::marks(toml.present),
            add: InterfaceToml::marks(toml.add),
            remove: InterfaceToml::marks(toml.remove),
        };
        Ok(Self { title, description, depends: toml.depends.unwrap_or_default(), supers })
    }
}
