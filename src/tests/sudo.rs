use crate::*;

use std::os::unix::fs::PermissionsExt;

/// A stand-in for `sudo -S -v`: its prompt on stderr, then a line from stdin, the right password
/// or "Sorry, try again.".
const STAND_IN: &str = "#!/bin/sh
[ \"$*\" = \"-S -v\" ] || exit 9
printf '[sudo] password for test: ' >&2
read -r line
[ \"$line\" = right ] && exit 0
echo 'Sorry, try again.' >&2
exit 1
";

/// How long a test waits on the stand-in.
const WAIT: Duration = Duration::from_secs(10);

/// The stand-in, written into a directory of the test's own.
fn stand_in(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stuff-sudo-{}-{test}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a directory");
    let path = dir.join("sudo");
    std::fs::write(&path, STAND_IN).expect("the stand-in");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("executable");
    path
}

/// The bytes sudo writes until it asks.
fn prompt(events: &mpsc::Receiver<RunEvent>) -> Vec<u8> {
    match events.recv_timeout(WAIT).expect("sudo writes") {
        RunEvent::SudoOutput { bytes } => bytes,
        other => panic!("expected sudo's output, got {other:?}"),
    }
}

/// What sudo writes after the prompt, and whether it ended with the credential.
fn finish(events: &mpsc::Receiver<RunEvent>) -> (String, bool) {
    let mut written = Vec::new();
    loop {
        match events.recv_timeout(WAIT).expect("sudo ends") {
            RunEvent::SudoOutput { bytes } => written.extend(bytes),
            RunEvent::SudoExit { ok } => return (String::from_utf8_lossy(&written).into_owned(), ok),
            other => panic!("expected sudo's events, got {other:?}"),
        }
    }
}

#[test]
fn the_right_password_gets_the_credential() {
    let program = stand_in("right");
    let (sender, events) = mpsc::channel();
    let mut validation = Validation::start(program.as_os_str(), sender);
    assert_eq!(prompt(&events), b"[sudo] password for test: ");
    validation.send(&Secret::from("right"));
    assert_eq!(finish(&events), (String::new(), true));
}

#[test]
fn a_wrong_password_ends_without_the_credential() {
    let program = stand_in("wrong");
    let (sender, events) = mpsc::channel();
    let mut validation = Validation::start(program.as_os_str(), sender);
    prompt(&events);
    validation.send(&Secret::from("wrong"));
    assert_eq!(finish(&events), ("Sorry, try again.\n".to_string(), false));
}

#[test]
fn dropping_the_validation_ends_sudo() {
    let program = stand_in("drop");
    let (sender, events) = mpsc::channel();
    let validation = Validation::start(program.as_os_str(), sender);
    prompt(&events);
    drop(validation);
    assert!(!finish(&events).1);
}

#[test]
fn a_missing_sudo_says_so_and_ends_without_the_credential() {
    let (sender, events) = mpsc::channel();
    let _validation = Validation::start(OsStr::new("stuff-no-such-sudo"), sender);
    let (written, ok) = finish(&events);
    assert!(written.starts_with("starting stuff-no-such-sudo: "), "{written}");
    assert!(!ok);
}

#[test]
fn a_secret_never_debugs_its_text() {
    let secret = Secret::from("hunter2");
    assert_eq!(format!("{secret:?}"), "Secret(..)");
}
