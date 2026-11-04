//! Stuff: adds and removes components in a home, through a TUI or a CLI.

pub(crate) mod catalog;
pub(crate) mod cli;
pub(crate) mod config;
pub(crate) mod error;
pub(crate) mod plan;
pub(crate) mod run;
pub(crate) mod runner;
pub(crate) mod source;
pub(crate) mod sudo;
pub(crate) mod tui {
    pub(crate) mod app;
    pub(crate) mod frame;
    pub(crate) mod icon;
    pub(crate) mod look;
    pub(crate) mod output;
    pub(crate) mod view;
}

pub(crate) mod r {
    pub(crate) use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
    pub(crate) use ratatui::layout::{Constraint, Layout, Margin, Rect, Size};
    pub(crate) use ratatui::style::{Color, Modifier, Style};
    pub(crate) use ratatui::text::{Line, Span, Text};
    pub(crate) use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
    pub(crate) use ratatui_image::picker::{Capability, Picker, ProtocolType, cap_parser::QueryStdioOptions};
    pub(crate) use ratatui_image::protocol::Protocol;
    pub(crate) use ratatui_image::{Image, Resize};
}

pub(crate) use crate::{
    catalog::{Catalog, Component},
    cli::{Cli, CliActionKind},
    config::{Action, Actions, ComponentConfig, ComponentConfigToml, StuffConfig, StuffConfigToml},
    error::{
        ActionSnafu, AliasParseSnafu, AliasReadSnafu, CatalogResult, FetchDirSnafu, FetchSnafu,
        FetchStartSnafu, GraphSnafu, LayoutSnafu, NoHomeSnafu, ParseSnafu, ReadSnafu, RunnerResult,
        SourceResult, SpawnSnafu, TerminalSnafu, UnknownSnafu, UnknownStuffSnafu,
    },
    plan::{Plan, Step, StepAction},
    run::{Tone, line},
    runner::{HealthForm, RunEvent, Runner},
    source::Source,
    sudo::{Secret, Validation},
    tui::{
        app::{App, Button, Look, Mark, Screen, SudoDialog},
        icon::Icons,
        output::Output,
    },
};
#[cfg(test)]
pub(crate) use crate::{
    catalog::ComponentSpec,
    error::CatalogErrorKind,
    tui::app::{HealthPage, Request, Then},
};

pub(crate) use ansi_to_tui::IntoText;
pub(crate) use snafu::ResultExt;
pub(crate) use std::{
    collections::HashMap,
    env,
    ffi::{OsStr, OsString},
    io::Read,
    path::{Path, PathBuf},
    process::{Command, ExitCode, ExitStatus, Stdio},
    sync::mpsc,
    thread,
    time::Duration,
};

pub use crate::error::Result;
pub use crate::run::run;

#[cfg(test)]
mod tests {
    mod app;
    mod catalog;
    mod output;
    mod plan;
    mod source;
    mod sudo;
    mod view;
}
