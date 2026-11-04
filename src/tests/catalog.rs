use crate::*;

/// A component spec with a title and summary of its own.
pub(super) fn spec(id: &str, parent: Option<usize>, depends: &[&str]) -> ComponentSpec {
    ComponentSpec {
        id: id.to_string(),
        config: ComponentConfig {
            title: id.to_uppercase(),
            description: format!("the {id} component"),
            depends: depends.iter().map(|d| d.to_string()).collect(),
            actions: Actions { present: true, add: true, remove: true, ..Actions::default() },
            supers: Actions::default(),
        },
        parent,
        dir: PathBuf::from("/stuff/component").join(id),
    }
}

/// git, rust with its nightly and riscv, and zed needing nightly and git.
fn specs() -> Vec<ComponentSpec> {
    vec![
        spec("git", None, &[]),
        spec("rust", None, &[]),
        spec("rust/nightly", Some(1), &[]),
        spec("rust/riscv", Some(1), &[]),
        spec("zed", None, &["rust/nightly", "git"]),
    ]
}

/// The fixture catalog.
pub(super) fn catalog() -> Catalog {
    catalog_with_supers("git", Actions::default())
}

/// The fixture catalog, one component's actions marked super.
pub(super) fn catalog_with_supers(id: &str, supers: Actions) -> Catalog {
    catalog_with(id, |spec| spec.config.supers = supers)
}

/// The fixture catalog, one component's spec changed.
pub(super) fn catalog_with(id: &str, change: impl Fn(&mut ComponentSpec)) -> Catalog {
    let specs = specs()
        .into_iter()
        .map(|mut spec| {
            if spec.id == id {
                change(&mut spec);
            }
            spec
        })
        .collect();
    Catalog::from_specs(PathBuf::from("/stuff"), StuffConfig { title: "Test".to_string() }, None, specs)
        .expect("the fixture catalog")
}

#[test]
fn a_subcomponent_sits_under_its_parent() {
    let catalog = catalog();
    assert_eq!(catalog.components[1].children, vec![2, 3]);
    assert_eq!(catalog.components[2].depth, 1);
    assert_eq!(catalog.components[4].depends, vec![2, 0]);
}

#[test]
fn adding_takes_every_requirement_first() {
    assert_eq!(catalog().add_closure(&[4]), vec![1, 2, 0, 4]);
}

#[test]
fn removing_takes_every_dependent_first() {
    assert_eq!(catalog().remove_closure(&[1]), vec![4, 2, 3, 1]);
}

#[test]
fn the_list_puts_the_added_first_among_siblings() {
    let added = [false, true, false, true, false];
    assert_eq!(catalog().display_order(&added), vec![1, 3, 2, 0, 4]);
}

#[test]
fn a_dependency_on_no_component_is_refused() {
    let error = Catalog::from_specs(
        PathBuf::from("/c"),
        StuffConfig { title: "Test".to_string() },
        None,
        vec![spec("a", None, &["missing"])],
    )
    .expect_err("an unknown dependency");
    assert_eq!(error.kind(), CatalogErrorKind::Graph);
}

#[test]
fn a_dependency_cycle_is_refused() {
    let error = Catalog::from_specs(
        PathBuf::from("/c"),
        StuffConfig { title: "Test".to_string() },
        None,
        vec![spec("a", None, &["b"]), spec("b", None, &["a"])],
    )
    .expect_err("a cycle");
    assert_eq!(error.kind(), CatalogErrorKind::Graph);
    assert!(error.to_string().contains("a, b"), "{error}");
}

#[test]
fn an_unknown_name_is_refused_with_the_known_ones() {
    let error = catalog().resolve(&["nope".to_string()]).expect_err("an unknown name");
    assert_eq!(error.kind(), CatalogErrorKind::Unknown);
    assert!(error.to_string().contains("rust/nightly"), "{error}");
}

#[test]
fn a_description_past_eighty_characters_is_refused() {
    let toml = ComponentConfigToml {
        title: Some("Long".to_string()),
        description: Some("x".repeat(81)),
        actions: Some(vec!["present".to_string(), "add".to_string()]),
        ..ComponentConfigToml::default()
    };
    assert!(ComponentConfig::try_from(toml).is_err());
}

/// A component.toml's text as a configuration, or why it is refused.
fn config(text: &str) -> std::result::Result<ComponentConfig, String> {
    let toml: ComponentConfigToml = toml::from_str(text).map_err(|error| error.to_string())?;
    ComponentConfig::try_from(toml)
}

#[test]
fn actions_and_super_are_lists_of_what_a_component_offers() {
    let config = config(
        "title = \"ChatGPT Codex\"\n\
         actions = [\"present\", \"add\", \"update\", \"remove\", \"health\"]\n\
         super = [\"add\", \"remove\"]\n",
    )
    .expect("a config");
    assert_eq!(config.actions, Actions { present: true, add: true, update: true, remove: true, health: true });
    assert_eq!(config.supers, Actions { add: true, remove: true, ..Actions::default() });
}

#[test]
fn an_action_is_plain_unless_super_names_it() {
    let config = config("title = \"Rust\"\nactions = [\"present\", \"add\"]\n").expect("a config");
    assert_eq!(config.supers, Actions::default());
}

#[test]
fn actions_must_be_there_with_the_required_ones_among_them() {
    assert!(config("title = \"None\"\n").expect_err("no actions").contains("actions is missing"));
    let lacking = config("title = \"Lacking\"\nactions = [\"add\"]\n").expect_err("no present");
    assert!(lacking.contains("present"), "{lacking}");
    let lacking = config("title = \"Lacking\"\nactions = [\"present\"]\n").expect_err("no add");
    assert!(lacking.contains("add"), "{lacking}");
}

#[test]
fn remove_update_and_health_are_optional() {
    let config = config("title = \"Least\"\nactions = [\"present\", \"add\"]\n").expect("a config");
    assert!(!config.actions.remove && !config.actions.update && !config.actions.health);
}

#[test]
fn a_name_that_is_no_action_is_refused() {
    let refused = config("title = \"Odd\"\nactions = [\"present\", \"add\", \"fix\"]\n").expect_err("no such action");
    assert!(refused.contains("fix"), "{refused}");
}

#[test]
fn super_may_name_only_what_actions_lists_and_never_health() {
    let refused = config("title = \"Odd\"\nactions = [\"present\", \"add\"]\nsuper = [\"remove\"]\n")
        .expect_err("not listed");
    assert!(refused.contains("remove"), "{refused}");
    let refused = config("title = \"Odd\"\nactions = [\"present\", \"add\", \"health\"]\nsuper = [\"health\"]\n")
        .expect_err("health needs no sudo");
    assert!(refused.contains("without sudo"), "{refused}");
}

#[test]
fn a_key_of_a_component_s_own_is_refused() {
    let refused = config("title = \"Odd\"\nactions = [\"present\", \"add\"]\npackage = \"x\"\n")
        .expect_err("its own key belongs in its module");
    assert!(refused.contains("package"), "{refused}");
}

#[test]
fn a_super_present_makes_the_check_need_sudo() {
    assert!(!catalog().super_present());
    let present = Actions { present: true, ..Actions::default() };
    assert!(catalog_with_supers("rust/riscv", present).super_present());
}
