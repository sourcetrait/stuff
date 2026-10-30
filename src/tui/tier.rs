//! The terminal's tier of capabilities, and the probe that reports what Stuff finds.

use crate::*;

/// The environment variables a terminal that draws its own line characters sets, any one enough.
const LINE_VARS: [&str; 7] = [
    "ALACRITTY_WINDOW_ID",
    "ALACRITTY_SOCKET",
    "VTE_VERSION",
    "KONSOLE_VERSION",
    "KITTY_WINDOW_ID",
    "WEZTERM_EXECUTABLE",
    "WEZTERM_VERSION",
];
/// The names in `TERM` or `TERM_PROGRAM` of terminals that draw their own line characters.
const LINE_NAMES: [&str; 6] = ["alacritty", "foot", "kitty", "ghostty", "wezterm", "iterm"];

/// The tier the terminal falls in: images where it speaks a graphics protocol, lines where it is
/// known to draw its own line characters, plain otherwise.
pub(crate) fn detect(picker: &r::Picker) -> Tier {
    if tui::icon::graphics(picker) {
        Tier::Image
    } else if line_evidence().is_some() {
        Tier::Lines
    } else {
        Tier::Plain
    }
}

/// What shows the terminal draws its own line characters: a variable it sets, or its name.
fn line_evidence() -> Option<String> {
    let set = |name: &str| std::env::var_os(name).is_some_and(|value| !value.is_empty());
    if let Some(name) = LINE_VARS.iter().find(|name| set(name)) {
        return Some((*name).to_string());
    }
    ["TERM", "TERM_PROGRAM"].iter().find_map(|var| {
        let value = std::env::var(var).unwrap_or_default().to_ascii_lowercase();
        LINE_NAMES.iter().find(|name| value.contains(*name)).map(|name| format!("{var} names {name}"))
    })
}

/// Prints what Stuff finds the terminal able to draw, a finding a line.
pub(crate) fn probe() -> Result<ExitCode> {
    let picker = tui::icon::picker();
    let font = picker.font_size();
    let window = ratatui::crossterm::terminal::window_size().ok();
    let var = |name: &str| std::env::var(name).unwrap_or_else(|_| "unset".to_string());
    let findings = [
        ("tier", format!("{:?}", detect(&picker)).to_lowercase()),
        ("protocol", format!("{:?}", picker.protocol_type()).to_lowercase()),
        ("cell", format!("{}x{} px, from the terminal's answer", font.width, font.height)),
        (
            "window",
            window.map_or("unknown".to_string(), |w| {
                format!("{}x{} cells, {}x{} px, from the kernel's window size", w.columns, w.rows, w.width, w.height)
            }),
        ),
        (
            "background",
            tui::icon::background(&picker).map_or("unreported".to_string(), |b| format!("{:?}", b.0)),
        ),
        ("lines", line_evidence().unwrap_or_else(|| "no evidence".to_string())),
        ("capabilities", format!("{:?}", picker.capabilities())),
        ("TERM", var("TERM")),
        ("TERM_PROGRAM", var("TERM_PROGRAM")),
        ("COLORTERM", var("COLORTERM")),
    ];
    for (key, value) in findings {
        println!("{key:13} {value}");
    }
    Ok(ExitCode::SUCCESS)
}
