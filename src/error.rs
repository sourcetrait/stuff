//! The crate's errors: each module's wrapped in the crate's own.

/// The crate's result.
pub type Result<T, E = Error> = std::result::Result<T, E>;
/// The catalog's result.
pub(crate) type CatalogResult<T> = Result<T, CatalogError>;
/// The runner's result.
pub(crate) type RunnerResult<T> = Result<T, RunnerError>;

/// A failure loading the components, running their actions, or drawing.
#[derive(Debug, snafu::Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    /// The stuff directory failed to load.
    #[snafu(transparent)]
    Catalog { source: CatalogError },
    /// An action failed to run or reported the wrong state.
    #[snafu(transparent)]
    Runner { source: RunnerError },
    /// The terminal failed to draw or read a key.
    #[snafu(display("terminal: {source}"))]
    Terminal { source: std::io::Error },
    /// The source named is no stuff Stuff knows: not a stuff directory.
    #[snafu(display("what stuff? {name:?}"))]
    UnknownStuff { name: String },
}

/// The kind of an [`Error`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Catalog,
    Runner,
    Terminal,
    UnknownStuff,
}

impl Error {
    /// The error's kind.
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Catalog { .. } => ErrorKind::Catalog,
            Self::Runner { .. } => ErrorKind::Runner,
            Self::Terminal { .. } => ErrorKind::Terminal,
            Self::UnknownStuff { .. } => ErrorKind::UnknownStuff,
        }
    }
}

/// A failure loading the stuff directory.
#[derive(Debug, snafu::Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum CatalogError {
    /// A file or directory failed to read.
    #[snafu(display("reading {path}: {source}"))]
    Read { path: String, source: std::io::Error },
    /// A stuff.toml or a component.toml failed to parse.
    #[snafu(display("parsing {path}: {source}"))]
    Parse { path: String, source: toml::de::Error },
    /// A component's files or fields break the layout.
    #[snafu(display("{path}: {reason}"))]
    Layout { path: String, reason: String },
    /// A dependency names no component, or the dependencies form a cycle.
    #[snafu(display("{reason}"))]
    Graph { reason: String },
    /// A name given names no component.
    #[snafu(display("unknown component {name:?}; known: {known}"))]
    Unknown { name: String, known: String },
}

/// The kind of a [`CatalogError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogErrorKind {
    Read,
    Parse,
    Layout,
    Graph,
    Unknown,
}

impl CatalogError {
    /// The error's kind.
    pub fn kind(&self) -> CatalogErrorKind {
        match self {
            Self::Read { .. } => CatalogErrorKind::Read,
            Self::Parse { .. } => CatalogErrorKind::Parse,
            Self::Layout { .. } => CatalogErrorKind::Layout,
            Self::Graph { .. } => CatalogErrorKind::Graph,
            Self::Unknown { .. } => CatalogErrorKind::Unknown,
        }
    }
}

/// A failure running a component's action.
#[derive(Debug, snafu::Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum RunnerError {
    /// nu failed to start.
    #[snafu(display("{component} {action}: starting nu: {source}"))]
    Spawn { component: String, action: String, source: std::io::Error },
    /// An action failed, or present disagreed with what ran, or a module breaks its contract.
    #[snafu(display("{component} {action}: {reason}"))]
    Action { component: String, action: String, reason: String },
}

/// The kind of a [`RunnerError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerErrorKind {
    Spawn,
    Action,
}

impl RunnerError {
    /// The error's kind.
    pub fn kind(&self) -> RunnerErrorKind {
        match self {
            Self::Spawn { .. } => RunnerErrorKind::Spawn,
            Self::Action { .. } => RunnerErrorKind::Action,
        }
    }
}
