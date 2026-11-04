//! A source told apart by its form.

use crate::*;
use crate::source::fetched_name;

fn form(text: &str) -> Source {
    Source::parse(Path::new(text))
}

#[test]
fn a_scheme_or_git_form_is_a_uri() {
    for uri in [
        "http://host/stuff",
        "https://host/stuff.git",
        "git://host/stuff",
        "ssh://git@host:22/stuff",
        "git@host:owner/stuff.git",
        "user@host:stuff",
    ] {
        assert_eq!(form(uri), Source::Uri(uri.to_string()), "{uri}");
    }
}

#[test]
fn what_looks_like_a_path_is_a_path() {
    for path in ["./rust", "../rust", ".", "~/stuff", "~", "/usr/share/stuff", "dir/stuff", "user@host/stuff", "", ".hidden"] {
        assert_eq!(form(path), Source::Path(PathBuf::from(path)), "{path}");
    }
}

#[test]
fn a_plain_word_is_an_alias() {
    for word in ["rust", "my-stuff", "a_b", "user@host", "host:path", "ftp:"] {
        assert_eq!(form(word), Source::Alias(word.to_string()), "{word}");
    }
}

#[test]
fn a_fetched_name_keeps_apart_two_that_read_alike() {
    let one = fetched_name("https://host/a/b");
    let two = fetched_name("https://host/a_b");
    assert_ne!(one, two);
    assert!(one.starts_with("https___host_a_b-"), "{one}");
    assert!(!one.contains('/'), "{one}");
}
