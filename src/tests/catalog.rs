use crate::*;

/// A component spec with a title and summary of its own.
pub(super) fn spec(id: &str, parent: Option<usize>, depends: &[&str]) -> ComponentSpec {
    ComponentSpec {
        id: id.to_string(),
        config: ComponentConfig {
            title: id.to_uppercase(),
            description: format!("the {id} component"),
            depends: depends.iter().map(|d| d.to_string()).collect(),
            supers: Supers::default(),
        },
        parent,
        dir: PathBuf::from("/stuff/component").join(id),
        removable: true,
        health: false,
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
    catalog_with_supers("git", Supers::default())
}

/// The fixture catalog, one component's scripts marked super.
pub(super) fn catalog_with_supers(id: &str, supers: Supers) -> Catalog {
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
        ..ComponentConfigToml::default()
    };
    assert!(ComponentConfig::try_from(toml).is_err());
}

#[test]
fn an_interface_block_marks_its_script_super() {
    let toml: ComponentConfigToml =
        toml::from_str("title = \"Claude Code\"\n\n[add]\nsuper = true\n\n[remove]\nsuper = true\n")
            .expect("a component.toml");
    let config = ComponentConfig::try_from(toml).expect("a config");
    assert_eq!(config.supers, Supers { present: false, add: true, remove: true });
}

#[test]
fn a_script_is_plain_unless_its_block_says_super() {
    let toml: ComponentConfigToml =
        toml::from_str("title = \"Rust\"\n\n[present]\n\n[add]\nsuper = false\n").expect("a component.toml");
    assert_eq!(ComponentConfig::try_from(toml).expect("a config").supers, Supers::default());
}

#[test]
fn a_super_present_script_makes_the_check_need_sudo() {
    assert!(!catalog().super_present());
    let present = Supers { present: true, ..Supers::default() };
    assert!(catalog_with_supers("rust/riscv", present).super_present());
}
