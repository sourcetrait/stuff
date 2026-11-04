//! The stuff.toml and component.toml files: the directory's and each component's.

/// The title the header shows when the directory's stuff.toml names none.
pub(crate) const DEFAULT_TITLE: &str = "Stuff";
/// The most characters a component's description may hold.
pub(crate) const DESCRIPTION_MAX: usize = 80;

/// The directory's stuff.toml as written.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
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

/// A component's action: an export of its mod.nu, named in its component.toml.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    /// Answers whether the component is added.
    Present,
    /// Adds the component.
    Add,
    /// Updates the component, where it can be updated.
    Update,
    /// Removes the component, where it can be removed.
    Remove,
    /// Returns the component's health record, where it has one.
    Health,
}

impl Action {
    /// Every action.
    const ALL: [Self; 5] = [Self::Present, Self::Add, Self::Update, Self::Remove, Self::Health];

    /// The action's name, its export's and its component.toml's.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Add => "add",
            Self::Update => "update",
            Self::Remove => "remove",
            Self::Health => "health",
        }
    }

    /// The action of a name.
    fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.name() == name)
    }
}

/// A set of a component's actions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Actions {
    /// The present action.
    pub(crate) present: bool,
    /// The add action.
    pub(crate) add: bool,
    /// The update action.
    pub(crate) update: bool,
    /// The remove action.
    pub(crate) remove: bool,
    /// The health action.
    pub(crate) health: bool,
}

impl Actions {
    /// Whether the set holds an action.
    pub(crate) fn has(self, action: Action) -> bool {
        match action {
            Action::Present => self.present,
            Action::Add => self.add,
            Action::Update => self.update,
            Action::Remove => self.remove,
            Action::Health => self.health,
        }
    }

    /// The names of the set's actions.
    pub(crate) fn names(self) -> Vec<&'static str> {
        Action::ALL.into_iter().filter(|&action| self.has(action)).map(Action::name).collect()
    }

    /// The set a component.toml's list names; the key names the list for an error.
    fn listed(names: &[String], key: &str) -> std::result::Result<Self, String> {
        let mut actions = Self::default();
        for name in names {
            match Action::named(name) {
                Some(Action::Present) => actions.present = true,
                Some(Action::Add) => actions.add = true,
                Some(Action::Update) => actions.update = true,
                Some(Action::Remove) => actions.remove = true,
                Some(Action::Health) => actions.health = true,
                None => return Err(format!("{key} names {name:?}, which is no action")),
            }
        }
        Ok(actions)
    }
}

/// A component's component.toml as written.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ComponentConfigToml {
    /// The card's title.
    pub(crate) title: Option<String>,
    /// The card's summary, 80 characters at most.
    pub(crate) description: Option<String>,
    /// The components it depends on beside its parent.
    pub(crate) depends: Option<Vec<String>>,
    /// Every action the component offers, the required ones among them.
    pub(crate) actions: Option<Vec<String>>,
    /// The actions where sudo is expected.
    #[serde(rename = "super")]
    pub(crate) superuser: Option<Vec<String>>,
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
    /// Every action it offers.
    pub(crate) actions: Actions,
    /// The actions where sudo is expected.
    pub(crate) supers: Actions,
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
        let actions = Actions::listed(&toml.actions.ok_or("actions is missing")?, "actions")?;
        for required in [Action::Present, Action::Add] {
            if !actions.has(required) {
                return Err(format!("actions doesn't list {}, which is required", required.name()));
            }
        }
        let supers = Actions::listed(&toml.superuser.unwrap_or_default(), "super")?;
        if let Some(unlisted) = supers.names().into_iter().find(|name| !actions.names().contains(name)) {
            return Err(format!("super names {unlisted}, which actions doesn't list"));
        }
        if supers.health {
            return Err("super names health, which runs without sudo".to_string());
        }
        Ok(Self { title, description, depends: toml.depends.unwrap_or_default(), actions, supers })
    }
}
