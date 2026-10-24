mod common;
use common::{assert_plot, fixture, run, run_fixture};

#[test]
fn colors_lists_every_color() {
    // colors.txt in the fixtures is the exact output of `uplot colors`.
    assert_plot(&["colors"], "simple.tsv", "colors.txt");
}

#[test]
fn colors_names_only() {
    let full = fixture("colors.txt");
    let full = String::from_utf8(full).unwrap();
    let r = run_fixture(&["colors", "--names"], "simple.tsv");
    assert_eq!(r.code(), 0);
    let names = String::from_utf8(r.stderr().to_vec()).unwrap();
    // Names-only omits the "  ●" swatch but keeps every escape sequence.
    assert!(!names.contains("  ●"));
    assert!(names.contains("black"));
    assert!(names.contains("255"));
    assert_eq!(names.matches("\u{1b}[0m").count(), full.matches("\u{1b}[0m").count());
}

#[test]
fn colors_can_be_redirected_to_stdout() {
    let r = run(&["colors", "-o", "-"], b"");
    assert_eq!(r.code(), 0);
    assert_eq!(r.stdout(), fixture("colors.txt").as_slice());
    assert!(r.stderr().is_empty());
}
