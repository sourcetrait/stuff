use crate::*;

fn lines(output: &Output) -> Vec<String> {
    output.lines().iter().map(|line| String::from_utf8_lossy(line).into_owned()).collect()
}

#[test]
fn a_carriage_return_rewrites_its_line() {
    let mut output = Output::default();
    output.push(b"10%\r20%\r30%\ndone\n");
    assert_eq!(lines(&output), ["30%", "done"]);
}

#[test]
fn a_crlf_ends_a_line_without_erasing_it() {
    let mut output = Output::default();
    output.push(b"one\r\ntwo");
    assert_eq!(lines(&output), ["one", "two"]);
}

#[test]
fn bytes_split_across_pushes_join() {
    let mut output = Output::default();
    output.push(b"hel");
    output.push(b"lo\nwor");
    output.push(b"ld");
    assert_eq!(lines(&output), ["hello", "world"]);
}

#[test]
fn a_note_starts_a_line_of_its_own() {
    let mut output = Output::default();
    output.push(b"partial");
    output.note("[add] Rust");
    output.push(b"next\n");
    assert_eq!(lines(&output), ["partial", "[add] Rust", "next"]);
}

#[test]
fn the_tail_keeps_the_last_rows_and_their_colors() {
    let mut output = Output::default();
    output.push(b"a\nb\n\x1b[32mgreen\x1b[0m\n");
    let text = output.tail(2);
    assert_eq!(text.lines.len(), 2);
    let green = &text.lines[1].spans[0];
    assert_eq!(green.content, "green");
    assert_eq!(green.style.fg, Some(r::Color::Green));
}
