//! Stuff: adds and removes components in a home, through a TUI or a CLI.

pub(crate) mod catalog;
pub(crate) mod cli;
pub(crate) mod config;
pub(crate) mod error;
pub(crate) mod plan;
pub(crate) mod run;
pub(crate) mod runner;
pub(crate) mod tui {
    pub(crate) mod app;
    pub(crate) mod frame;
    pub(crate) mod icon;
    pub(crate) mod output;
    pub(crate) mod tier;
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
    cli::{Cli, CliActionKind, CliCommand},
    config::{ComponentConfig, ComponentConfigToml, StuffConfig, StuffConfigToml},
    error::{
        CatalogResult, GraphSnafu, LayoutSnafu, ParseSnafu, ReadSnafu, RunnerResult, ScriptSnafu,
        SpawnSnafu, TerminalSnafu, UnknownSnafu,
    },
    plan::{Plan, Step, StepAction},
    runner::{RunEvent, Runner, Script},
    tui::{
        app::{App, Button, Mark, Screen, Tier},
        icon::Icons,
        output::Output,
    },
};
#[cfg(test)]
pub(crate) use crate::{catalog::ComponentSpec, error::CatalogErrorKind, tui::app::Request};

pub(crate) use ansi_to_tui::IntoText;
pub(crate) use snafu::ResultExt;
pub(crate) use std::{
    collections::HashMap,
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
    mod view;
}
