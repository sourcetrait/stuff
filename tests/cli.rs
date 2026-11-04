//! The CLI against a stuff directory of real nu modules, in a home of its own.

use sourcetrait_common::testing::prelude::*;
use std::path::Path;
use std::process::{Command, Output};

static TESTING: testing::Module = testing::module!(Integration, {
    .using_temp_dir()
});

/// What every component's component.toml lists, but for those that say otherwise.
const ACTIONS: &str = "actions = [\"present\", \"add\", \"remove\"]\n";

/// What a run says just before its success.
const FRESH_LOGIN: &str = "\x1b[34m[stuff]\x1b[0m Environment changes require a fresh terminal login\n";

/// add writing the marker.
const ADD: &str = "print $'adding ($ID) from ($env.PWD | path basename)'\n    $ID | save --force (marker)";

/// add that fails.
const FAIL: &str = "print 'about to fail'\n    exit 3";

/// add that succeeds without adding.
const LIE: &str = "print 'claims success'";

/// A module whose state is a marker file in the home, its add the body given.
fn module(id: &str, add: &str) -> String {
    format!(
        "const ID: string = '{id}'

def marker []: nothing -> path {{
    $env.HOME | path join $'($ID | str replace --all \"/\" \"-\").added'
}}

export def present []: nothing -> bool {{
    marker | path exists
}}

export def add []: nothing -> nothing {{
    {add}
}}

export def remove []: nothing -> nothing {{
    print $'removing ($ID)'
    rm --force (marker)
}}
"
    )
}

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
    std::fs::write(path, text).expect("a file");
}

fn component(dir: &Path, id: &str, config: &str, add: &str) {
    let path = dir.join("component").join(id);
    write(&path.join("component.toml"), &format!("{config}{ACTIONS}"));
    write(&path.join("mod.nu"), &module(id, add));
}

/// A stuff directory: base with extra under it, other needing base, and two that fail.
fn fixture(root: &Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let dir = root.join("stuff");
    let home = root.join("home");
    std::fs::create_dir_all(&home).expect("a home");
    write(&dir.join("stuff.toml"), "title = \"Test\"\n");
    component(&dir, "base", "title = \"Base\"\ndescription = \"the base\"\n", ADD);
    component(&dir, "base/extra", "title = \"Extra\"\ndescription = \"an extra\"\n", ADD);
    component(&dir, "other", "title = \"Other\"\ndepends = [\"base\"]\n", ADD);
    component(&dir, "broken", "title = \"Broken\"\n", FAIL);
    component(&dir, "liar", "title = \"Liar\"\n", LIE);
    (dir, home)
}

fn stuff(dir: &Path, home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_stuff"))
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

fn ids(output: &Output) -> Vec<String> {
    stdout(output).lines().map(|line| line.split_whitespace().next().unwrap_or_default().to_string()).collect()
}

#[tested]
fn a_directory_without_stuff_is_unknown_stuff() {
    let t = testing::test!({ .using_temp_dir() });
    let (_, home) = fixture(t.temp_dir());
    let refused = stuff(&home, &home, &["add"]);
    assert!(!refused.status.success());
    assert!(stderr(&refused).contains("what stuff?"), "{}", stderr(&refused));
}

#[tested]
fn a_directory_holding_config_toml_is_unknown_stuff() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    std::fs::rename(dir.join("stuff.toml"), dir.join("config.toml")).expect("renamed");
    let refused = stuff(&dir, &home, &["add"]);
    assert!(!refused.status.success());
    assert!(stderr(&refused).contains("what stuff?"), "{}", stderr(&refused));
}

#[tested]
fn a_subdirectory_without_component_toml_is_no_component() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    write(&dir.join("component/base/data/config.toml"), "title = \"Data\"\n");
    let listed = stuff(&dir, &home, &["add"]);
    assert!(listed.status.success(), "{}", stderr(&listed));
    assert!(!stdout(&listed).contains("base/data"), "{}", stdout(&listed));
}

#[tested]
fn a_library_beside_mod_nu_is_its_own() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    write(&dir.join("component/base/words.nu"), "export def note []: nothing -> string { 'from a library' }\n");
    let path = dir.join("component/base/mod.nu");
    let text = std::fs::read_to_string(&path).expect("the module");
    write(&path, &format!("use words.nu\n{}", text.replace("$ID | save", "words note | save")));
    let added = stuff(&dir, &home, &["add", "base"]);
    assert!(added.status.success(), "{}{}", stdout(&added), stderr(&added));
    assert_eq!(std::fs::read_to_string(home.join("base.added")).expect("the marker"), "from a library");
}

#[tested]
fn super_presents_run_after_the_rest_one_at_a_time() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    let log = home.join("present.log");
    let logged = |id: &str| {
        format!(
            "export def present []: nothing -> bool {{
    \"start {id}\\n\" | save --append ($env.HOME | path join present.log)
    sleep 200ms
    \"end {id}\\n\" | save --append ($env.HOME | path join present.log)
    false
}}

export def add []: nothing -> nothing {{ }}

export def remove []: nothing -> nothing {{ }}
"
        )
    };
    for id in ["sudo-one", "sudo-two"] {
        component(&dir, id, &format!("title = \"{id}\"\nsuper = [\"present\"]\n"), ADD);
        write(&dir.join("component").join(id).join("mod.nu"), &logged(id));
    }
    let listed = stuff(&dir, &home, &["add"]);
    assert!(listed.status.success(), "{}", stderr(&listed));
    let order = std::fs::read_to_string(&log).expect("the log");
    assert_eq!(order, "start sudo-one\nend sudo-one\nstart sudo-two\nend sudo-two\n", "{order}");
}

#[tested]
fn add_alone_lists_what_is_not_added() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    let listed = stuff(&dir, &home, &["add"]);
    assert!(listed.status.success(), "{}", stderr(&listed));
    assert_eq!(ids(&listed), ["base", "base/extra", "broken", "liar", "other"]);
}

#[tested]
fn adding_a_subcomponent_adds_its_parent_first() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    let added = stuff(&dir, &home, &["add", "base/extra"]);
    assert!(added.status.success(), "{}{}", stdout(&added), stderr(&added));
    let out = stdout(&added);
    let base = out.find("adding base from base").expect("base added");
    let extra = out.find("adding base/extra from extra").expect("extra added, from its own directory");
    assert!(base < extra, "{out}");
    assert!(home.join("base.added").is_file());
    assert!(home.join("base-extra.added").is_file());
    assert_eq!(ids(&stuff(&dir, &home, &["remove"])), ["base", "base/extra"]);
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
fn a_successful_run_says_a_fresh_login_is_needed_just_before_its_success() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    let added = stuff(&dir, &home, &["add", "base"]);
    assert!(added.status.success(), "{}{}", stdout(&added), stderr(&added));
    let expected = format!("{FRESH_LOGIN}\x1b[32m[stuff]\x1b[0m done\n");
    assert!(stdout(&added).ends_with(&expected), "{:?}", stdout(&added));
}

#[tested]
fn a_failing_action_stops_the_run_with_no_success() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    let failed = stuff(&dir, &home, &["add", "broken"]);
    assert!(!failed.status.success());
    assert!(stdout(&failed).contains("about to fail"), "{}", stdout(&failed));
    assert!(stderr(&failed).contains("broken add: exited 3"), "{}", stderr(&failed));
    assert!(!stdout(&failed).contains("fresh terminal login"), "{}", stdout(&failed));
}

#[tested]
fn nothing_to_do_says_so_with_no_success() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    assert!(stuff(&dir, &home, &["add", "base"]).status.success());
    let again = stuff(&dir, &home, &["add", "base"]);
    assert!(again.status.success(), "{}", stderr(&again));
    assert!(stdout(&again).contains("nothing to add"), "{}", stdout(&again));
    assert!(!stdout(&again).contains("fresh terminal login"), "{}", stdout(&again));
}

#[tested]
fn present_must_agree_with_what_ran() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    let lied = stuff(&dir, &home, &["add", "liar"]);
    assert!(!lied.status.success());
    assert!(stderr(&lied).contains("liar present: reports not added after add"), "{}", stderr(&lied));
}

#[tested]
fn a_present_that_fails_is_an_error_and_never_absent() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    let path = dir.join("component/other/mod.nu");
    let text = std::fs::read_to_string(&path).expect("the module");
    write(&path, &text.replace("marker | path exists", "['a broken present'] | each {|m| error make --unspanned {msg: $m} } | is-empty"));
    let refused = stuff(&dir, &home, &["add"]);
    assert!(!refused.status.success(), "{}", stdout(&refused));
    assert!(stderr(&refused).contains("other present: exited 1"), "{}", stderr(&refused));
    assert!(stderr(&refused).contains("a broken present"), "{}", stderr(&refused));
}

#[tested]
fn a_present_answering_no_bool_is_an_error() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    let path = dir.join("component/other/mod.nu");
    let text = std::fs::read_to_string(&path).expect("the module");
    write(&path, &text.replace("marker | path exists", "['yes'] | first"));
    let refused = stuff(&dir, &home, &["add"]);
    assert!(!refused.status.success(), "{}", stdout(&refused));
    assert!(stderr(&refused).contains("not a bool"), "{}", stderr(&refused));
}

#[tested]
fn a_broken_user_config_is_an_error_and_never_absent() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    write(&home.join(".config/nushell/config.nu"), "$env.BROKEN = ($env.NOT_SET | path join 'x')\n");
    let refused = stuff(&dir, &home, &["add"]);
    assert!(!refused.status.success(), "{}", stdout(&refused));
    assert!(stderr(&refused).contains("exited"), "{}", stderr(&refused));
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
fn a_component_without_its_module_is_refused() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    std::fs::remove_file(dir.join("component/other/mod.nu")).expect("removed");
    let refused = stuff(&dir, &home, &["add"]);
    assert!(!refused.status.success());
    assert!(stderr(&refused).contains("mod.nu is missing"), "{}", stderr(&refused));
}

#[tested]
fn a_module_that_fails_to_parse_is_refused() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    write(&dir.join("component/other/mod.nu"), "export def add [ {\n");
    let refused = stuff(&dir, &home, &["add"]);
    assert!(!refused.status.success());
    assert!(stderr(&refused).contains("other mod.nu: exited"), "{}", stderr(&refused));
}

#[tested]
fn an_action_listed_and_not_exported_is_refused() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    write(
        &dir.join("component/other/component.toml"),
        "title = \"Other\"\nactions = [\"present\", \"add\", \"remove\", \"update\"]\n",
    );
    let refused = stuff(&dir, &home, &["add"]);
    assert!(!refused.status.success());
    assert!(stderr(&refused).contains("other mod.nu: doesn't export update"), "{}", stderr(&refused));
}

#[tested]
fn an_export_not_listed_is_refused() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    write(&dir.join("component/other/component.toml"), "title = \"Other\"\nactions = [\"present\", \"add\"]\n");
    let refused = stuff(&dir, &home, &["add"]);
    assert!(!refused.status.success());
    assert!(stderr(&refused).contains("other mod.nu: exports remove"), "{}", stderr(&refused));
}

#[tested]
fn a_private_helper_is_no_export() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    assert!(std::fs::read_to_string(dir.join("component/other/mod.nu")).expect("the module").contains("def marker"));
    let listed = stuff(&dir, &home, &["add"]);
    assert!(listed.status.success(), "{}", stderr(&listed));
}

#[tested]
fn a_component_toml_of_the_old_form_is_refused() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    write(
        &dir.join("component/other/component.toml"),
        "title = \"Other\"\nactions = [\"present\", \"add\", \"remove\"]\n\n[add]\nsuper = true\n",
    );
    let refused = stuff(&dir, &home, &["add"]);
    assert!(!refused.status.success());
    assert!(stderr(&refused).contains("component.toml"), "{}", stderr(&refused));
}

#[tested]
fn a_component_without_remove_stays_with_a_warning() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    let keeper = "const ID: string = 'keeper'

def marker []: nothing -> path { $env.HOME | path join 'keeper.added' }

export def present []: nothing -> bool { marker | path exists }

export def add []: nothing -> nothing { $ID | save --force (marker) }
";
    let path = dir.join("component/keeper");
    write(&path.join("component.toml"), "title = \"Keeper\"\nactions = [\"present\", \"add\"]\n");
    write(&path.join("mod.nu"), keeper);
    assert!(stuff(&dir, &home, &["add", "keeper", "base"]).status.success());
    let listed = stuff(&dir, &home, &["remove"]);
    assert!(stdout(&listed).contains("keeper"), "{}", stdout(&listed));
    let removed = stuff(&dir, &home, &["remove", "keeper", "base"]);
    assert!(removed.status.success(), "{}{}", stdout(&removed), stderr(&removed));
    assert!(stderr(&removed).contains("\x1b[33m[stuff]\x1b[0m remove not supported: keeper"), "{}", stderr(&removed));
    assert!(home.join("keeper.added").is_file(), "keeper stays");
    assert!(!home.join("base.added").exists(), "the rest goes");
}

/// A component offering update, which writes its marker again.
fn updatable(dir: &Path, id: &str) {
    let path = dir.join("component").join(id);
    write(
        &path.join("component.toml"),
        &format!("title = \"{id}\"\nactions = [\"present\", \"add\", \"update\", \"remove\"]\n"),
    );
    let text = format!(
        "{}\nexport def update []: nothing -> nothing {{\n    print $'updating ($ID)'\n    'updated' | save --force (marker)\n}}\n",
        module(id, ADD)
    );
    write(&path.join("mod.nu"), &text);
}

#[tested]
fn update_alone_lists_what_is_added_and_offers_it() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    updatable(&dir, "fresh");
    updatable(&dir, "stale");
    assert!(stuff(&dir, &home, &["add", "base", "stale"]).status.success());
    assert_eq!(ids(&stuff(&dir, &home, &["update"])), ["stale"]);
}

#[tested]
fn update_runs_on_what_is_added_and_says_so() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    updatable(&dir, "stale");
    assert!(stuff(&dir, &home, &["add", "stale"]).status.success());
    let updated = stuff(&dir, &home, &["update", "stale"]);
    assert!(updated.status.success(), "{}{}", stdout(&updated), stderr(&updated));
    assert!(stdout(&updated).contains("updating stale"), "{}", stdout(&updated));
    assert!(stdout(&updated).contains("fresh terminal login"), "{}", stdout(&updated));
    assert_eq!(std::fs::read_to_string(home.join("stale.added")).expect("the marker"), "updated");
}

#[tested]
fn update_passes_over_what_is_not_added() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    updatable(&dir, "stale");
    let updated = stuff(&dir, &home, &["update", "stale"]);
    assert!(updated.status.success(), "{}{}", stdout(&updated), stderr(&updated));
    assert!(stdout(&updated).contains("nothing to update"), "{}", stdout(&updated));
    assert!(!home.join("stale.added").exists());
}

#[tested]
fn update_warns_of_what_does_not_offer_it() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    updatable(&dir, "stale");
    assert!(stuff(&dir, &home, &["add", "base", "stale"]).status.success());
    let updated = stuff(&dir, &home, &["update", "base", "stale"]);
    assert!(updated.status.success(), "{}{}", stdout(&updated), stderr(&updated));
    assert!(stderr(&updated).contains("\x1b[33m[stuff]\x1b[0m update not supported: base"), "{}", stderr(&updated));
    assert!(stdout(&updated).contains("updating stale"), "{}", stdout(&updated));
}

#[tested]
fn an_update_leaving_it_absent_is_an_error() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    updatable(&dir, "stale");
    assert!(stuff(&dir, &home, &["add", "stale"]).status.success());
    let path = dir.join("component/stale/mod.nu");
    let text = std::fs::read_to_string(&path).expect("the module");
    write(&path, &text.replace("'updated' | save --force (marker)", "rm --force (marker)"));
    let updated = stuff(&dir, &home, &["update", "stale"]);
    assert!(!updated.status.success());
    assert!(stderr(&updated).contains("stale present: reports not added after update"), "{}", stderr(&updated));
}

/// A component with a health action, its record reporting its own present.
fn with_health(dir: &Path, id: &str, health: &str) {
    let path = dir.join("component").join(id);
    let toml = std::fs::read_to_string(path.join("component.toml")).expect("its toml").replace(
        "\"remove\"]",
        "\"remove\", \"health\"]",
    );
    write(&path.join("component.toml"), &toml);
    let text = std::fs::read_to_string(path.join("mod.nu")).expect("its module");
    write(&path.join("mod.nu"), &format!("{text}\n{health}"));
    write(&path.join("health.nutype"), HEALTH_TYPE);
}

/// health reporting its component's own present, and a note when present.
const HEALTH: &str = "export def health []: nothing -> record<component: string, present: oneof<bool, record<note: string>>> {
    if not (present) { return {component: $ID, present: false} }
    {component: $ID, present: {note: 'well'}}
}
";

/// health.nutype of that record.
const HEALTH_TYPE: &str = "record<component: path, present: oneof<bool, record<note: string>>>\n";

#[tested]
fn health_of_one_component_is_its_record() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    with_health(&dir, "base", HEALTH);
    assert!(stuff(&dir, &home, &["add", "base"]).status.success());
    let health = stuff(&dir, &home, &["health", "base"]);
    assert!(health.status.success(), "{}", stderr(&health));
    assert_eq!(stdout(&health).trim(), "{component: base, present: {note: well}}");
}

#[tested]
fn health_of_several_is_a_record_of_records() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    with_health(&dir, "base", HEALTH);
    with_health(&dir, "other", HEALTH);
    assert!(stuff(&dir, &home, &["add", "base"]).status.success());
    let health = stuff(&dir, &home, &["health", "base", "other", "liar"]);
    assert!(health.status.success(), "{}", stderr(&health));
    assert_eq!(
        stdout(&health).trim(),
        "{\"base\": {component: base, present: {note: well}}, \"other\": {component: other, present: false}, \
         \"liar\": {component: \"liar\", present: false}}"
    );
}

#[tested]
fn a_health_action_without_its_nutype_is_refused() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    with_health(&dir, "base", HEALTH);
    std::fs::remove_file(dir.join("component/base/health.nutype")).expect("removed");
    let refused = stuff(&dir, &home, &["add"]);
    assert!(!refused.status.success());
    assert!(stderr(&refused).contains("health.nutype is missing"), "{}", stderr(&refused));
}

#[tested]
fn a_health_record_of_the_wrong_shape_is_refused() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    with_health(&dir, "base", "export def health [] { 'fine' }\n");
    let refused = stuff(&dir, &home, &["health", "base"]);
    assert!(!refused.status.success());
    assert!(stderr(&refused).contains("not record<component: path"), "{}", stderr(&refused));
}

#[tested]
fn the_user_config_reaches_each_action() {
    let t = testing::test!({ .using_temp_dir() });
    let (dir, home) = fixture(t.temp_dir());
    write(&home.join(".config/nushell/env.nu"), "$env.FROM_ENV_NU = 'env'\n");
    write(&home.join(".config/nushell/config.nu"), "$env.FROM_CONFIG_NU = 'config'\n");
    component(
        &dir,
        "configured",
        "title = \"Configured\"\n",
        "$'($env.FROM_ENV_NU)+($env.FROM_CONFIG_NU)' | save --force (marker)",
    );
    let added = stuff(&dir, &home, &["add", "configured"]);
    assert!(added.status.success(), "{}{}", stdout(&added), stderr(&added));
    assert_eq!(std::fs::read_to_string(home.join("configured.added")).expect("a marker"), "env+config");
}
