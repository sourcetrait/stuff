//! The stuff directory loaded: its configuration and its component graph.

use crate::*;

/// The directory under the stuff directory that holds the components.
pub(crate) const COMPONENT_DIR: &str = "component";
/// The configuration file the directory and each component carry.
pub(crate) const CONFIG_FILE: &str = "config.toml";
/// The icon the stuff directory may carry.
pub(crate) const LOGO_FILE: &str = "logo.png";

/// A component as found, its dependencies still names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ComponentSpec {
    /// Its name: its path under `component/`.
    pub(crate) id: String,
    /// Its configuration.
    pub(crate) config: ComponentConfig,
    /// Its parent, by index; a parent precedes its children.
    pub(crate) parent: Option<usize>,
    /// Its directory.
    pub(crate) dir: PathBuf,
}

/// One component: its name, its card, its place in the graph, and its files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Component {
    /// Its name: its path under `component/`.
    pub(crate) id: String,
    /// The card's title.
    pub(crate) title: String,
    /// The card's summary.
    pub(crate) description: String,
    /// Its parent, by index.
    pub(crate) parent: Option<usize>,
    /// Its subcomponents, by index, in name order.
    pub(crate) children: Vec<usize>,
    /// The components it depends on beside its parent, by index.
    pub(crate) depends: Vec<usize>,
    /// Its depth in the tree, a top component's 0.
    pub(crate) depth: usize,
    /// Its directory.
    pub(crate) dir: PathBuf,
}

/// The stuff directory: its configuration and its components in tree order.
#[derive(Debug, Clone)]
pub(crate) struct Catalog {
    /// The stuff directory, absolute.
    pub(crate) dir: PathBuf,
    /// Its configuration.
    pub(crate) config: StuffConfig,
    /// Its icon, when it carries one.
    pub(crate) icon: Option<PathBuf>,
    /// Every component, each parent before its children.
    pub(crate) components: Vec<Component>,
}

impl Catalog {
    /// Loads a stuff directory: its config.toml and every component under it.
    pub(crate) fn load(dir: &Path) -> CatalogResult<Self> {
        let dir = std::fs::canonicalize(dir).context(ReadSnafu { path: display(dir) })?;
        let config: StuffConfigToml = read_toml(&dir.join(CONFIG_FILE))?;
        let icon = Some(dir.join(LOGO_FILE)).filter(|path| path.is_file());
        let mut specs = Vec::new();
        let root = dir.join(COMPONENT_DIR);
        if root.is_dir() {
            collect(&root, None, "", &mut specs)?;
        }
        Self::from_specs(dir, config.into(), icon, specs)
    }

    /// Builds a catalog from its specs, resolving the dependencies and checking them.
    pub(crate) fn from_specs(
        dir: PathBuf,
        config: StuffConfig,
        icon: Option<PathBuf>,
        specs: Vec<ComponentSpec>,
    ) -> CatalogResult<Self> {
        let index: HashMap<&str, usize> =
            specs.iter().enumerate().map(|(i, spec)| (spec.id.as_str(), i)).collect();
        let mut components: Vec<Component> = Vec::with_capacity(specs.len());
        for (i, spec) in specs.iter().enumerate() {
            let depth = match spec.parent {
                Some(parent) if parent < i => components[parent].depth + 1,
                Some(_) => {
                    return GraphSnafu { reason: format!("{} precedes its parent", spec.id) }.fail();
                }
                None => 0,
            };
            let mut depends = Vec::new();
            for name in &spec.config.depends {
                let Some(&target) = index.get(name.as_str()) else {
                    return GraphSnafu {
                        reason: format!("{} depends on {name:?}, which names no component", spec.id),
                    }
                    .fail();
                };
                if !depends.contains(&target) {
                    depends.push(target);
                }
            }
            components.push(Component {
                id: spec.id.clone(),
                title: spec.config.title.clone(),
                description: spec.config.description.clone(),
                parent: spec.parent,
                children: Vec::new(),
                depends,
                depth,
                dir: spec.dir.clone(),
            });
        }
        for i in 0..components.len() {
            if let Some(parent) = components[i].parent {
                components[parent].children.push(i);
            }
        }
        let catalog = Self { dir, config, icon, components };
        catalog.check_acyclic()?;
        Ok(catalog)
    }

    /// What a component needs added first: its parent and its depends.
    pub(crate) fn requirements(&self, i: usize) -> Vec<usize> {
        let component = &self.components[i];
        component.parent.into_iter().chain(component.depends.iter().copied()).collect()
    }

    /// What needs a component: its subcomponents and whatever depends on it.
    pub(crate) fn dependents(&self, i: usize) -> Vec<usize> {
        let mut dependents = self.components[i].children.clone();
        dependents.extend(
            (0..self.components.len()).filter(|&j| self.components[j].depends.contains(&i)),
        );
        dependents
    }

    /// The components and all they need, each after what it needs.
    pub(crate) fn add_closure(&self, roots: &[usize]) -> Vec<usize> {
        let mut seen = vec![false; self.components.len()];
        let mut order = Vec::new();
        for &root in roots {
            self.visit(root, &mut seen, &mut order, &Self::requirements);
        }
        order
    }

    /// The components and all that needs them, each before what it needs.
    pub(crate) fn remove_closure(&self, roots: &[usize]) -> Vec<usize> {
        let mut seen = vec![false; self.components.len()];
        let mut order = Vec::new();
        for &root in roots {
            self.visit(root, &mut seen, &mut order, &Self::dependents);
        }
        order
    }

    /// The list's order: the tree, each set of siblings with the added first.
    pub(crate) fn display_order(&self, added: &[bool]) -> Vec<usize> {
        let roots: Vec<usize> =
            (0..self.components.len()).filter(|&i| self.components[i].parent.is_none()).collect();
        let mut order = Vec::with_capacity(self.components.len());
        self.order_siblings(&roots, added, &mut order);
        order
    }

    /// The components named, by index.
    pub(crate) fn resolve(&self, names: &[String]) -> CatalogResult<Vec<usize>> {
        names
            .iter()
            .map(|name| {
                self.components.iter().position(|c| &c.id == name).ok_or_else(|| {
                    UnknownSnafu { name: name.clone(), known: self.known() }.build()
                })
            })
            .collect()
    }

    /// Every component's name, for an error.
    fn known(&self) -> String {
        self.components.iter().map(|c| c.id.as_str()).collect::<Vec<_>>().join(", ")
    }

    /// Visits a component's edges depth first, pushing each after its edges' ends.
    fn visit(
        &self,
        i: usize,
        seen: &mut [bool],
        order: &mut Vec<usize>,
        edges: &dyn Fn(&Self, usize) -> Vec<usize>,
    ) {
        if seen[i] {
            return;
        }
        seen[i] = true;
        for next in edges(self, i) {
            self.visit(next, seen, order, edges);
        }
        order.push(i);
    }

    /// Orders siblings with the added first, each followed by its own subtree.
    fn order_siblings(&self, siblings: &[usize], added: &[bool], order: &mut Vec<usize>) {
        let mut sorted = siblings.to_vec();
        sorted.sort_by_key(|&i| !added[i]);
        for i in sorted {
            order.push(i);
            self.order_siblings(&self.components[i].children, added, order);
        }
    }

    /// Refuses a dependency cycle, naming the components caught in it.
    fn check_acyclic(&self) -> CatalogResult<()> {
        let count = self.components.len();
        let mut pending: Vec<usize> = (0..count).map(|i| self.requirements(i).len()).collect();
        let mut ready: Vec<usize> = (0..count).filter(|&i| pending[i] == 0).collect();
        let mut done = 0;
        while let Some(i) = ready.pop() {
            done += 1;
            for dependent in self.dependents(i) {
                pending[dependent] -= 1;
                if pending[dependent] == 0 {
                    ready.push(dependent);
                }
            }
        }
        if done == count {
            return Ok(());
        }
        let caught: Vec<&str> = (0..count)
            .filter(|&i| pending[i] > 0)
            .map(|i| self.components[i].id.as_str())
            .collect();
        GraphSnafu { reason: format!("dependency cycle among {}", caught.join(", ")) }.fail()
    }
}

/// Collects the components under a directory, depth first in name order.
fn collect(
    dir: &Path,
    parent: Option<usize>,
    prefix: &str,
    specs: &mut Vec<ComponentSpec>,
) -> CatalogResult<()> {
    let entries = std::fs::read_dir(dir).context(ReadSnafu { path: display(dir) })?;
    let mut paths = entries
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()
        .context(ReadSnafu { path: display(dir) })?;
    paths.sort();
    for path in paths.into_iter().filter(|p| p.is_dir() && p.join(CONFIG_FILE).is_file()) {
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let id = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };
        for script in [Script::Exists, Script::Add, Script::Remove] {
            if !path.join(script.file()).is_file() {
                return LayoutSnafu { path: display(&path), reason: format!("{} is missing", script.file()) }
                    .fail();
            }
        }
        let config_path = path.join(CONFIG_FILE);
        let toml: ComponentConfigToml = read_toml(&config_path)?;
        let config = ComponentConfig::try_from(toml)
            .map_err(|reason| LayoutSnafu { path: display(&config_path), reason }.build())?;
        let index = specs.len();
        specs.push(ComponentSpec { id: id.clone(), config, parent, dir: path.clone() });
        collect(&path, Some(index), &id, specs)?;
    }
    Ok(())
}

/// Reads and parses a TOML file.
fn read_toml<T: serde::de::DeserializeOwned>(path: &Path) -> CatalogResult<T> {
    let text = std::fs::read_to_string(path).context(ReadSnafu { path: display(path) })?;
    toml::from_str(&text).context(ParseSnafu { path: display(path) })
}

/// A path as an error shows it.
fn display(path: &Path) -> String {
    path.display().to_string()
}
