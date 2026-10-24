mod common;
use common::{fixture, run, run_fixture};

#[test]
fn plot_to_stdout_with_optional_argument() {
    let r = run_fixture(&["line", "-o", "-"], "simple.tsv");
    assert_eq!(r.code(), 0);
    assert_eq!(r.stdout(), fixture("simple-lineplot.txt").as_slice());
    assert!(r.stderr().is_empty());
    let r = run_fixture(&["line", "-o"], "simple.tsv");
    assert_eq!(r.code(), 0);
    assert_eq!(r.stdout(), fixture("simple-lineplot.txt").as_slice());
    assert!(r.stderr().is_empty());
}

#[test]
fn plot_to_file() {
    let r = run_fixture(&["line", "-o", "plot.txt"], "simple.tsv");
    assert_eq!(r.code(), 0);
    assert!(r.stderr().is_empty());
    assert_eq!(r.file("plot.txt"), fixture("simple-lineplot.txt"));
}

#[test]
fn pass_input_to_stdout() {
    let r = run_fixture(&["box", "-O"], "simple.tsv");
    assert_eq!(r.code(), 0);
    assert_eq!(r.stdout(), fixture("simple.tsv").as_slice());
    assert_eq!(r.stderr(), fixture("simple-boxplot.txt").as_slice());
}

#[test]
fn pass_input_to_stdout_with_hyphen() {
    let r = run_fixture(&["box", "-O", "-", "-t", "BOX"], "simple.tsv");
    assert_eq!(r.code(), 0);
    assert_eq!(r.stdout(), fixture("simple.tsv").as_slice());
    assert!(String::from_utf8_lossy(r.stderr()).contains("BOX"));
}

#[test]
fn pass_input_to_file() {
    let r = run_fixture(&["line", "-O", "pass.tsv"], "simple.tsv");
    assert_eq!(r.code(), 0);
    assert_eq!(r.file("pass.tsv"), fixture("simple.tsv"));
    assert_eq!(r.stderr(), fixture("simple-lineplot.txt").as_slice());
}

#[test]
fn explicit_stdout_output_does_not_create_dash_file() {
    let r = run(&["line", "-o", "-"], &fixture("simple.tsv"));
    assert_eq!(r.code(), 0);
    assert!(!std::path::Path::new(&r.dir.join("-")).exists());
}
