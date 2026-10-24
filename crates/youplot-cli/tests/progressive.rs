mod common;
use common::{run, run_fixture};

fn text(b: &[u8]) -> String { String::from_utf8_lossy(b).into_owned() }

#[test]
fn progressive_mode_restores_cursor() {
    let r = run_fixture(&["line", "-p"], "simple.tsv");
    assert_eq!(r.code(), 0, "{}", text(r.stderr()));
    let out = text(r.stderr());
    assert!(out.starts_with("\u{1b}[?25l"), "hide cursor first");
    assert!(out.ends_with("\u{1b}[0J\u{1b}[?25h"), "cleanup at end: {:?}", &out[out.len().saturating_sub(40)..]);
    assert!(out.contains("\u{1b}[0K"));
    assert!(out.contains("\u{1b}[0J"));
}

#[test]
fn progressive_mode_cleans_up_on_error() {
    let r = run_fixture(&["lineplots", "--fmt", "xyxy", "-H", "-p"], "simple-xyxy-odd.tsv");
    assert_ne!(r.code(), 0);
    let out = text(r.stderr());
    assert!(out.contains("\u{1b}[?25l"));
    assert!(out.contains("\u{1b}[?25h"));
    assert!(out.contains("\u{1b}[0J"));
}

#[test]
fn progressive_mode_rejects_file_output() {
    let r = run(&["line", "-p", "-o", "plot.txt"], b"-1\n2\n");
    assert_ne!(r.code(), 0);
    assert!(text(r.stderr()).contains("In progressive mode, output to a file is not possible."));
}
