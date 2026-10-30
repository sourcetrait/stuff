//! The CLI against a stuff directory of real nu scripts, in a home of its own.

use sourcetrait_common::testing::prelude::*;
use std::path::Path;
use std::process::{Command, Output};

static TESTING: testing::Module = testing::module!(Integration, {
    .using_temp_dir()
});

/// present.nu for a component whose state is a marker file in the home.
const PRESENT: &str = "def main [] {
    let marker = ($env.HOME | path join $'($env.COMPONENT_ID | str replace --all \"/\" \"-\").added')
    if ($marker | path exists) { exit 0 } else { exit 1 }
}
";

/// add.nu writing the marker, and the marker's text from its own config.toml.
const ADD: &str = "def main [] {
    let marker = ($env.HOME | path join $'($env.COMPONENT_ID | str replace --all \"/\" \"-\").added')
    let note = (open ($env.COMPONENT_DIR | path join config.toml) | get note? | default 'none')
    print $'adding ($env.COMPONENT_ID) from ($env.STUFF_DIR | path basename)'
    $note | save --force $marker
}
";

/// remove.nu deleting the marker.
const REMOVE: &str = "def main [] {
    let marker = ($env.HOME | path join $'($env.COMPONENT_ID | str replace --all \"/\" \"-\").added')
    print $'removing ($env.COMPONENT_ID)'
    rm --force $marker
}
";

/// add.nu that fails.
const FAIL: &str = "def main [] { print 'about to fail'; exit 3 }\n";

/// add.nu that succeeds without adding.
const LIE: &str = "def main [] { print 'claims success' }\n";

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
    std::fs::write(path, text).expect("a file");
}

fn component(dir: &Path, id: &str, config: &str, add: &str) {
    let path = dir.join("component").join(id);
    write(&path.join("config.toml"), config);
    write(&path.join("present.nu"), PRESENT);
    write(&path.join("add.nu"), add);
    write(&path.join("remove.nu"), REMOVE);
}

/// A stuff directory: base with extra under it, other needing base, and two that fail.
fn fixture(root: &Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let dir = root.join("stuff");
    let home = root.join("home");
    std::fs::create_dir_all(&home).expect("a home");
    write(&dir.join("config.toml"), "title = \"Test\"\n");
    component(&dir, "base", "title = \"Base\"\ndescription = \"the base\"\nnote = \"from base\"\n", ADD);
    component(&dir, "base/extra", "title = \"Extra\"\ndescription = \"an extra\"\n", ADD);
    component(&dir, "other", "title = \"Other\"\ndepends = [\"base\"]\n", ADD);
    component(&dir, "broken", "title = \"Broken\"\n", FAIL);
    component(&dir, "liar", "title = \"Liar\"\n", LIE);
    (dir, home)
}

fn stuff(dir: &Path, home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_stuff"))
        .arg("in")
        .arg(dir)
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .output()
        .expect("the stuff binary runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[tested]
fn a_source_comes_before_the_directory() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    let refused = Command::new(env!("CARGO_BIN_EXE_stuff"))
        .arg(&dir)
        .arg("add")
        .env("HOME", &home)
        .output()
        .expect("the stuff binary runs");
    assert!(!refused.status.success());
    assert!(stderr(&refused).contains("unrecognized subcommand"), "{}", stderr(&refused));
}

#[tested]
fn add_alone_lists_what_is_not_added() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    let listed = stuff(&dir, &home, &["add"]);
    assert!(listed.status.success(), "{}", stderr(&listed));
    let ids: Vec<String> = stdout(&listed)
        .lines()
        .map(|line| line.split_whitespace().next().unwrap_or_default().to_string())
        .collect();
    assert_eq!(ids, ["base", "base/extra", "broken", "liar", "other"]);
}

#[tested]
fn adding_a_subcomponent_adds_its_parent_first() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    let added = stuff(&dir, &home, &["add", "base/extra"]);
    assert!(added.status.success(), "{}{}", stdout(&added), stderr(&added));
    let out = stdout(&added);
    let base = out.find("adding base from stuff").expect("base added");
    let extra = out.find("adding base/extra").expect("extra added");
    assert!(base < extra, "{out}");
    assert_eq!(std::fs::read_to_string(home.join("base.added")).expect("base's marker"), "from base");
    assert!(home.join("base-extra.added").is_file());
    let listed = stuff(&dir, &home, &["remove"]);
    let ids: Vec<String> = stdout(&listed)
        .lines()
        .map(|line| line.split_whitespace().next().unwrap_or_default().to_string())
        .collect();
    assert_eq!(ids, ["base", "base/extra"], "{}", stdout(&listed));
}

#[tested]
fn removing_a_parent_removes_what_needs_it_first() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    assert!(stuff(&dir, &home, &["add", "base/extra", "other"]).status.success());
    let removed = stuff(&dir, &home, &["remove", "base"]);
    assert!(removed.status.success(), "{}{}", stdout(&removed), stderr(&removed));
    let out = stdout(&removed);
    let base = out.find("removing base\n").expect("base removed");
    assert!(out.find("removing base/extra").expect("extra removed") < base, "{out}");
    assert!(out.find("removing other").expect("other removed") < base, "{out}");
    assert!(!home.join("base.added").exists());
    assert!(!home.join("other.added").exists());
}

#[tested]
fn a_failing_script_stops_the_run() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    let failed = stuff(&dir, &home, &["add", "broken"]);
    assert!(!failed.status.success());
    assert!(stdout(&failed).contains("about to fail"), "{}", stdout(&failed));
    assert!(stderr(&failed).contains("broken add.nu: exited 3"), "{}", stderr(&failed));
}

#[tested]
fn present_must_agree_with_what_ran() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    let lied = stuff(&dir, &home, &["add", "liar"]);
    assert!(!lied.status.success());
    assert!(stderr(&lied).contains("liar present.nu: reports not added after add"), "{}", stderr(&lied));
}

#[tested]
fn an_unknown_component_is_refused() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    let refused = stuff(&dir, &home, &["add", "nope"]);
    assert!(!refused.status.success());
    assert!(stderr(&refused).contains("unknown component \"nope\""), "{}", stderr(&refused));
}

#[tested]
fn a_component_missing_a_script_is_refused() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    std::fs::remove_file(dir.join("component/other/remove.nu")).expect("removed");
    let refused = stuff(&dir, &home, &["add"]);
    assert!(!refused.status.success());
    assert!(stderr(&refused).contains("remove.nu is missing"), "{}", stderr(&refused));
}

#[tested]
fn the_user_config_reaches_each_script() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    write(&home.join(".config/nushell/env.nu"), "$env.FROM_ENV_NU = 'env'\n");
    write(&home.join(".config/nushell/config.nu"), "$env.FROM_CONFIG_NU = 'config'\n");
    component(
        &dir,
        "configured",
        "title = \"Configured\"\n",
        "def main [] {
    let marker = ($env.HOME | path join 'configured.added')
    $'($env.FROM_ENV_NU)+($env.FROM_CONFIG_NU)' | save --force $marker
}
",
    );
    let added = stuff(&dir, &home, &["add", "configured"]);
    assert!(added.status.success(), "{}{}", stdout(&added), stderr(&added));
    assert_eq!(std::fs::read_to_string(home.join("configured.added")).expect("a marker"), "env+config");
}
