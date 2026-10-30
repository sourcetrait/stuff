//! The run's output as a terminal shows it: lines, a carriage return rewriting its line.

use crate::*;

/// The spaces a tab becomes.
const TAB: &[u8] = b"    ";

/// The bytes the scripts wrote, split into lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Output {
    /// The lines, the last one still open.
    lines: Vec<Vec<u8>>,
    /// Whether a carriage return waits to rewrite the open line.
    rewrite: bool,
}

impl Output {
    /// Appends bytes as a terminal would show them.
    pub(crate) fn push(&mut self, bytes: &[u8]) {
        if self.lines.is_empty() {
            self.lines.push(Vec::new());
        }
        for &byte in bytes {
            match byte {
                b'\n' => {
                    self.lines.push(Vec::new());
                    self.rewrite = false;
                }
                b'\r' => self.rewrite = true,
                _ => {
                    let line = self.lines.last_mut().expect("an open line");
                    if self.rewrite {
                        line.clear();
                        self.rewrite = false;
                    }
                    if byte == b'\t' {
                        line.extend_from_slice(TAB);
                    } else {
                        line.push(byte);
                    }
                }
            }
        }
    }

    /// Appends a line of its own, after the open line if it holds anything.
    pub(crate) fn note(&mut self, text: &str) {
        if self.lines.last().is_some_and(|line| !line.is_empty()) {
            self.push(b"\n");
        }
        self.push(text.as_bytes());
        self.push(b"\n");
    }

    /// The finished lines and the open one, if it holds anything.
    pub(crate) fn lines(&self) -> &[Vec<u8>] {
        match self.lines.last() {
            Some(last) if last.is_empty() => &self.lines[..self.lines.len() - 1],
            _ => &self.lines,
        }
    }

    /// The last `rows` lines, their ANSI colors kept.
    pub(crate) fn tail(&self, rows: usize) -> r::Text<'static> {
        let lines = self.lines();
        let start = lines.len().saturating_sub(rows);
        let styled: Vec<r::Line<'static>> = lines[start..]
            .iter()
            .map(|bytes| match bytes.into_text() {
                Ok(text) => text.lines.into_iter().next().unwrap_or_default(),
                Err(_) => r::Line::raw(String::from_utf8_lossy(bytes).into_owned()),
            })
            .collect();
        r::Text::from(styled)
    }
}
