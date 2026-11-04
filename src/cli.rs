//! The command line: the source, then an action and its components; or the probe.

use crate::*;

/// Adds and removes components in your home, through a TUI or this CLI.
#[derive(Debug, clap::Parser)]
#[command(name = "stuff", version, about)]
pub(crate) struct Cli {
    /// How the TUI looks: fancy or simple; by default fancy where the terminal scales images to its cells.
    #[arg(long, global = true, value_enum)]
    pub(crate) look: Option<Look>,
    /// Reports what the terminal can draw: its look, graphics protocol, and cell size.
    #[arg(long, exclusive = true)]
    pub(crate) probe: bool,
    /// The stuff: a stuff directory.
    #[arg(required_unless_present = "probe")]
    pub(crate) source: Option<PathBuf>,
    /// The action; none opens the TUI.
    #[command(subcommand)]
    pub(crate) action: Option<CliAction>,
}

/// An action on components.
#[derive(Debug, clap::Subcommand)]
pub(crate) enum CliAction {
    /// Adds the components named, or lists those not added.
    Add {
        /// The components to add, with what they depend on.
        components: Vec<String>,
    },
    /// Updates the components named, or lists those added that can be updated.
    Update {
        /// The components to update.
        components: Vec<String>,
    },
    /// Removes the components named, or lists those added.
    Remove {
        /// The components to remove, with what depends on them.
        components: Vec<String>,
    },
    /// Reports the health of the components named, as NUON.
    Health {
        /// The components to check: a record for one, a record of records for several.
        #[arg(required = true)]
        components: Vec<String>,
    },
}

/// The kind of a [`CliAction`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CliActionKind {
    Add,
    Update,
    Remove,
    Health,
}

impl CliAction {
    /// The action's kind.
    pub(crate) fn kind(&self) -> CliActionKind {
        match self {
            Self::Add { .. } => CliActionKind::Add,
            Self::Update { .. } => CliActionKind::Update,
            Self::Remove { .. } => CliActionKind::Remove,
            Self::Health { .. } => CliActionKind::Health,
        }
    }

    /// The components named.
    pub(crate) fn components(&self) -> &[String] {
        match self {
            Self::Add { components }
            | Self::Update { components }
            | Self::Remove { components }
            | Self::Health { components } => components,
        }
    }
}
