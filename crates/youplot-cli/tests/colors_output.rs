mod common;
use common::{run, run_fixture};

fn text(b: &[u8]) -> String { String::from_utf8_lossy(b).into_owned() }

#[test]
fn force_color_and_monochrome() {
    let r = run_fixture(&["line", "-C"], "simple.tsv");
    assert_eq!(r.code(), 0);
    assert!(text(r.stderr()).contains("\u{1b}["), "expected ANSI colors");

    let r = run_fixture(&["line", "-M"], "simple.tsv");
    assert_eq!(r.code(), 0);
    assert!(!text(r.stderr()).contains("\u{1b}["), "expected no ANSI colors");

    // Last option wins.
    let r = run_fixture(&["line", "-C", "-M"], "simple.tsv");
    assert!(!text(r.stderr()).contains("\u{1b}["));
    let r = run_fixture(&["line", "-M", "-C"], "simple.tsv");
    assert!(text(r.stderr()).contains("\u{1b}["));
}

#[test]
fn debug_prints_preprocessed_data() {
    let r = run_fixture(&["line", "--debug"], "simple.tsv");
    assert_eq!(r.code(), 0);
    let out = text(r.stdout());
    assert!(out.contains("#<struct YouPlot::Data"), "{out}");
    assert!(out.contains("series=[["), "{out}");
}

#[test]
fn border_and_canvas_errors() {
    let r = run(&["line", "--border", "foo"], b"-1\n2\n");
    assert_ne!(r.code(), 0);
    assert!(text(r.stderr()).contains("unknown border type: foo"));

    let r = run(&["line", "--canvas", "foo"], b"-1\n2\n");
    assert_ne!(r.code(), 0);
    assert!(text(r.stderr()).contains("unknown canvas type: foo"));

    let r = run(&["bar", "--xscale", "bogus"], b"1\n2\n");
    assert_ne!(r.code(), 0);
    assert!(text(r.stderr()).contains("func must be callable"));
}
