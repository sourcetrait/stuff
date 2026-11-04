//! The crate's errors: each module's wrapped in the crate's own.

/// The crate's result.
pub type Result<T, E = Error> = std::result::Result<T, E>;
/// The catalog's result.
pub(crate) type CatalogResult<T> = Result<T, CatalogError>;
/// The runner's result.
pub(crate) type RunnerResult<T> = Result<T, RunnerError>;
/// The source's result.
pub(crate) type SourceResult<T> = Result<T, SourceError>;

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
    /// The source failed to reach any stuff.
    #[snafu(transparent)]
    Source { source: SourceError },
}

/// The kind of an [`Error`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Catalog,
    Runner,
    Terminal,
    Source,
}

impl Error {
    /// The error's kind.
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Catalog { .. } => ErrorKind::Catalog,
            Self::Runner { .. } => ErrorKind::Runner,
            Self::Terminal { .. } => ErrorKind::Terminal,
            Self::Source { .. } => ErrorKind::Source,
        }
    }
}

/// A failure reaching the stuff a source names.
#[derive(Debug, snafu::Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum SourceError {
    /// The source named is no stuff Stuff knows.
    #[snafu(display("what stuff? {name:?}: {tried}"))]
    UnknownStuff { name: String, tried: String },
    /// The aliases' file failed to read.
    #[snafu(display("reading {path}: {source}"))]
    AliasRead { path: String, source: std::io::Error },
    /// The aliases' file failed to parse.
    #[snafu(display("parsing {path}: {source}"))]
    AliasParse { path: String, source: toml::de::Error },
    /// Neither an XDG home nor a home is set.
    #[snafu(display("neither {variable} nor HOME is set"))]
    NoHome { variable: String },
    /// The directory kept for a URI's stuff failed.
    #[snafu(display("fetching {uri}: {source}"))]
    FetchDir { uri: String, source: std::io::Error },
    /// git failed to start.
    #[snafu(display("fetching {uri}: starting git: {source}"))]
    FetchStart { uri: String, source: std::io::Error },
    /// git failed.
    #[snafu(display("fetching {uri}: git: {status}"))]
    Fetch { uri: String, status: String },
}

/// The kind of a [`SourceError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceErrorKind {
    UnknownStuff,
    AliasRead,
    AliasParse,
    NoHome,
    FetchDir,
    FetchStart,
    Fetch,
}

impl SourceError {
    /// The error's kind.
    pub fn kind(&self) -> SourceErrorKind {
        match self {
            Self::UnknownStuff { .. } => SourceErrorKind::UnknownStuff,
            Self::AliasRead { .. } => SourceErrorKind::AliasRead,
            Self::AliasParse { .. } => SourceErrorKind::AliasParse,
            Self::NoHome { .. } => SourceErrorKind::NoHome,
            Self::FetchDir { .. } => SourceErrorKind::FetchDir,
            Self::FetchStart { .. } => SourceErrorKind::FetchStart,
            Self::Fetch { .. } => SourceErrorKind::Fetch,
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
