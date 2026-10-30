//! The command line: the source, then an action and its components.

use crate::*;

/// Adds and removes components in your home, through a TUI or this CLI.
#[derive(Debug, clap::Parser)]
#[command(name = "stuff", version, about)]
pub(crate) struct Cli {
    /// How the TUI draws; pretty by default where the terminal draws images, ugly elsewhere.
    #[arg(long, global = true, value_enum)]
    pub(crate) look: Option<Look>,
    /// Where the components come from.
    #[command(subcommand)]
    pub(crate) source: CliSource,
}

/// Where the components come from.
#[derive(Debug, clap::Subcommand)]
pub(crate) enum CliSource {
    /// The components in a stuff directory.
    In {
        /// The stuff directory.
        dir: PathBuf,
        /// The action; none opens the TUI.
        #[command(subcommand)]
        action: Option<CliAction>,
    },
}

/// An action on components.
#[derive(Debug, clap::Subcommand)]
pub(crate) enum CliAction {
    /// Adds the components named, or lists those not added.
    Add {
        /// The components to add, with what they depend on.
        components: Vec<String>,
    },
    /// Removes the components named, or lists those added.
    Remove {
        /// The components to remove, with what depends on them.
        components: Vec<String>,
    },
}

/// The kind of a [`CliAction`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CliActionKind {
    Add,
    Remove,
}

impl CliAction {
    /// The action's kind.
    pub(crate) fn kind(&self) -> CliActionKind {
        match self {
            Self::Add { .. } => CliActionKind::Add,
            Self::Remove { .. } => CliActionKind::Remove,
        }
    }

    /// The components named.
    pub(crate) fn components(&self) -> &[String] {
        match self {
            Self::Add { components } | Self::Remove { components } => components,
        }
    }
}
