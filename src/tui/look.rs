//! The look the terminal gets, and the probe that reports what Stuff finds.

use crate::*;

/// The look the terminal gets, by the graphics protocol the startup query finds: fancy where the
/// terminal scales an image to its cells, simple anywhere else.
pub(crate) fn detect(picker: &r::Picker) -> Look {
    match picker.protocol_type() {
        r::ProtocolType::Kitty | r::ProtocolType::Iterm2 => Look::Fancy,
        r::ProtocolType::Sixel | r::ProtocolType::Halfblocks => Look::Simple,
    }
}

/// Prints what Stuff finds the terminal able to draw, a finding a line.
pub(crate) fn probe() -> Result<ExitCode> {
    let picker = tui::icon::picker();
    let font = picker.font_size();
    let window = ratatui::crossterm::terminal::window_size().ok();
    let var = |name: &str| std::env::var(name).unwrap_or_else(|_| "unset".to_string());
    let findings = [
        ("look", format!("{:?}", detect(&picker)).to_lowercase()),
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
