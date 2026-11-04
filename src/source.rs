//! The source on the command line: a URI, a stuff directory, or an alias.

use crate::*;

/// The schemes a URI takes.
const SCHEMES: [&str; 4] = ["http://", "https://", "git://", "ssh://"];
/// The aliases' file, under the configuration home.
const ALIASES_FILE: &str = "stuff/alias.toml";
/// Where a URI's stuff is kept, under the cache home.
const FETCHED_DIR: &str = "stuff/source";

/// A source by its form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Source {
    /// A git remote, by its scheme or by git's `user@host:path`.
    Uri(String),
    /// A path, holding a `/` or starting with `.` or `~`.
    Path(PathBuf),
    /// A plain word.
    Alias(String),
}

impl Source {
    /// The form a source takes as written.
    pub(crate) fn parse(source: &Path) -> Self {
        let text = source.to_string_lossy();
        if SCHEMES.iter().any(|scheme| text.starts_with(scheme)) || is_scp_like(&text) {
            Self::Uri(text.into_owned())
        } else if text.contains('/') || text.starts_with(['.', '~']) || text.is_empty() {
            Self::Path(source.to_path_buf())
        } else {
            Self::Alias(text.into_owned())
        }
    }

    /// The stuff directory the source reaches, a URI's fetched first.
    pub(crate) fn directory(&self) -> SourceResult<PathBuf> {
        match self {
            Self::Uri(uri) => {
                let dir = fetch(uri)?;
                stuff_at(dir, uri, "what it fetched holds no stuff.toml")
            }
            Self::Path(path) => stuff_at(expand_home(path), &path.display().to_string(), "not a stuff directory"),
            Self::Alias(word) => {
                let file = config_home()?.join(ALIASES_FILE);
                let Some(target) = aliases(&file)?.remove(word) else {
                    let tried = format!("no alias by that name in {}", file.display());
                    return UnknownStuffSnafu { name: word, tried }.fail();
                };
                match Self::parse(Path::new(&target)) {
                    Self::Alias(other) => {
                        let tried = format!("its alias names {other:?}, neither a URI nor a path");
                        UnknownStuffSnafu { name: word, tried }.fail()
                    }
                    reached => reached.directory(),
                }
            }
        }
    }
}

/// Whether a source reads as git's `user@host:path`.
fn is_scp_like(text: &str) -> bool {
    let Some((user, rest)) = text.split_once('@') else {
        return false;
    };
    let Some((host, path)) = rest.split_once(':') else {
        return false;
    };
    let plain = |part: &str| !part.is_empty() && !part.contains(['/', ':']);
    plain(user) && plain(host) && !path.is_empty()
}

/// A path with its leading `~` taken as the home, as a shell would for an alias's value.
fn expand_home(path: &Path) -> PathBuf {
    match (path.strip_prefix("~"), env::var_os("HOME")) {
        (Ok(rest), Some(home)) => PathBuf::from(home).join(rest),
        _ => path.to_path_buf(),
    }
}

/// The directory where it holds stuff.
fn stuff_at(dir: PathBuf, name: &str, tried: &str) -> SourceResult<PathBuf> {
    if crate::catalog::holds_stuff(&dir) {
        Ok(dir)
    } else {
        UnknownStuffSnafu { name, tried }.fail()
    }
}

/// The aliases a file names, none where there is no file.
fn aliases(file: &Path) -> SourceResult<HashMap<String, String>> {
    let path = file.display().to_string();
    match std::fs::read_to_string(file) {
        Ok(text) => toml::from_str(&text).context(AliasParseSnafu { path }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(HashMap::new()),
        Err(source) => Err(source).context(AliasReadSnafu { path }),
    }
}

/// An XDG home: its variable, else its place under the home.
fn xdg_home(variable: &str, default: &str) -> SourceResult<PathBuf> {
    if let Some(home) = env::var_os(variable).filter(|home| !home.is_empty()) {
        return Ok(PathBuf::from(home));
    }
    match env::var_os("HOME") {
        Some(home) => Ok(PathBuf::from(home).join(default)),
        None => NoHomeSnafu { variable }.fail(),
    }
}

/// The configuration home.
fn config_home() -> SourceResult<PathBuf> {
    xdg_home("XDG_CONFIG_HOME", ".config")
}

/// The name a URI's stuff is kept under: its own words, and a hash keeping apart two that read alike.
pub(crate) fn fetched_name(uri: &str) -> String {
    let words: String =
        uri.chars().map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' { c } else { '_' }).collect();
    let mut hasher = std::hash::DefaultHasher::new();
    std::hash::Hash::hash(uri, &mut hasher);
    format!("{words}-{:016x}", std::hash::Hasher::finish(&hasher))
}

/// A URI's stuff brought up to its remote's head, in the directory kept for it.
fn fetch(uri: &str) -> SourceResult<PathBuf> {
    let root = xdg_home("XDG_CACHE_HOME", ".cache")?.join(FETCHED_DIR);
    std::fs::create_dir_all(&root).context(FetchDirSnafu { uri })?;
    let dir = root.join(fetched_name(uri));
    eprintln!("{}", line(Tone::Info, &format!("fetching {uri}")));
    if dir.join(".git").is_dir() {
        for args in [
            &["fetch", "-q", "--depth", "1", "origin", "HEAD"][..],
            &["reset", "-q", "--hard", "FETCH_HEAD"],
            &["clean", "-q", "-ffdx"],
        ] {
            git(uri, Some(&dir), args.iter().map(OsStr::new))?;
        }
        return Ok(dir);
    }
    // A clone that failed part of the way leaves nothing a later run takes for stuff.
    let partial = root.join(format!("{}.partial", fetched_name(uri)));
    if partial.exists() {
        std::fs::remove_dir_all(&partial).context(FetchDirSnafu { uri })?;
    }
    let clone = ["clone", "-q", "--depth", "1"].map(OsStr::new);
    git(uri, None, clone.into_iter().chain([OsStr::new(uri), partial.as_os_str()]))?;
    std::fs::rename(&partial, &dir).context(FetchDirSnafu { uri })?;
    Ok(dir)
}

/// One git command, in a directory where given, its output passed through.
fn git<'a>(uri: &str, dir: Option<&Path>, args: impl IntoIterator<Item = &'a OsStr>) -> SourceResult<()> {
    let mut command = Command::new("git");
    if let Some(dir) = dir {
        command.arg("-C").arg(dir);
    }
    let status = command.args(args).status().context(FetchStartSnafu { uri })?;
    if status.success() {
        Ok(())
    } else {
        FetchSnafu { uri, status: status.to_string() }.fail()
    }
}
