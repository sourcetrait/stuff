//! sudo's credential, asked for and refreshed through `sudo -S -v`; no script runs under sudo.

use crate::*;

use std::io::Write;

/// The program that holds the credential.
pub(crate) const SUDO: &str = "sudo";

/// A password being typed: kept out of Debug and zeroed when dropped.
#[derive(Clone, Default)]
pub(crate) struct Secret(zeroize::Zeroizing<String>);

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(..)")
    }
}

impl From<&str> for Secret {
    fn from(text: &str) -> Self {
        Self(zeroize::Zeroizing::new(text.to_string()))
    }
}

impl Secret {
    /// Appends a character.
    pub(crate) fn push(&mut self, c: char) {
        self.0.push(c);
    }

    /// Removes the last character.
    pub(crate) fn pop(&mut self) {
        self.0.pop();
    }

    /// Takes the password, leaving this one empty.
    pub(crate) fn take(&mut self) -> Self {
        std::mem::take(self)
    }

    /// The password's text.
    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

/// A `sudo -S -v` under way: its prompt and messages sent to the TUI as they arrive, then its exit;
/// a password line written to its stdin. Dropping it closes the stdin, which ends sudo.
#[derive(Debug)]
pub(crate) struct Validation {
    /// sudo's stdin.
    stdin: Option<std::process::ChildStdin>,
}

impl Validation {
    /// Starts sudo validating the credential, reading a password from its stdin when it asks.
    pub(crate) fn start(program: &OsStr, events: mpsc::Sender<RunEvent>) -> Self {
        let spawned = Command::new(program)
            .args(["-S", "-v"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn();
        match spawned {
            Ok(mut child) => {
                let stdin = child.stdin.take();
                let stderr = child.stderr.take();
                thread::spawn(move || {
                    if let Some(stderr) = stderr {
                        runner::forward_as(Box::new(stderr), &events, |bytes| RunEvent::SudoOutput { bytes });
                    }
                    let ok = child.wait().is_ok_and(|status| status.success());
                    let _ = events.send(RunEvent::SudoExit { ok });
                });
                Self { stdin }
            }
            Err(error) => {
                let message = format!("starting {}: {error}\n", program.to_string_lossy());
                let _ = events.send(RunEvent::SudoOutput { bytes: message.into_bytes() });
                let _ = events.send(RunEvent::SudoExit { ok: false });
                Self { stdin: None }
            }
        }
    }

    /// Writes a password to sudo, a line.
    pub(crate) fn send(&mut self, password: &Secret) {
        if let Some(stdin) = self.stdin.as_mut() {
            let _ = stdin
                .write_all(password.expose().as_bytes())
                .and_then(|()| stdin.write_all(b"\n"))
                .and_then(|()| stdin.flush());
        }
    }
}
