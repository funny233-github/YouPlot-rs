mod common;
use common::{run, run_fixture};

fn text(b: &[u8]) -> String { String::from_utf8_lossy(b).into_owned() }

#[test]
fn version() {
    let r = run_fixture(&["--version"], "simple.tsv");
    assert_eq!(r.code(), 0);
    assert_eq!(text(r.stdout()), "YouPlot 0.5.0\n");
    let r = run_fixture(&["-v"], "simple.tsv");
    assert_eq!(r.code(), 0);
    assert_eq!(text(r.stdout()), "YouPlot 0.5.0\n");
    let r = run_fixture(&["line", "--version"], "simple.tsv");
    assert_eq!(text(r.stdout()), "YouPlot 0.5.0\n");
}

#[test]
fn no_command_prints_main_help_to_stderr() {
    let r = run(&[], b"");
    assert_eq!(r.code(), 0);
    let err = text(r.stderr());
    assert!(err.starts_with('\n'), "{err:?}");
    assert!(err.contains("Program: YouPlot (Tools for plotting on the terminal)"));
    assert!(err.contains("Commands:"));
    assert!(r.stdout().is_empty());
}

#[test]
fn help_goes_to_stdout() {
    for args in [vec!["--help"], vec!["help"]] {
        let r = run(&[], b"");
        let _ = r; // keep helper usage uniform
        let r = run(&args, b"");
        assert_eq!(r.code(), 0);
        let out = text(r.stdout());
        assert!(out.starts_with('\n'));
        assert!(out.contains("Usage:   uplot <command> [options] <in.tsv>"));
    }
}

#[test]
fn subcommand_help() {
    let r = run(&["line", "--help"], b"");
    assert_eq!(r.code(), 0);
    let out = text(r.stdout());
    assert!(out.contains("Usage: YouPlot line [options] <in.tsv>"));
    assert!(out.contains("--xlim FLOAT,FLOAT"));
    assert!(out.contains("Common options:"));
    assert!(out.ends_with("--debug             print preprocessed data\n\n"));
}

#[test]
fn unknown_command() {
    let r = run(&["foo"], b"");
    assert_eq!(r.code(), 1);
    assert_eq!(text(r.stderr()), "YouPlot: unrecognized command 'foo'\n");
}

#[test]
fn invalid_option() {
    let r = run(&["line", "--bogus"], b"");
    assert_eq!(r.code(), 1);
    assert!(text(r.stderr()).starts_with("YouPlot: invalid option: --bogus"));
}

#[test]
fn missing_and_invalid_arguments() {
    let r = run(&["line", "-w"], b"");
    assert_eq!(r.code(), 1);
    assert_eq!(text(r.stderr()), "YouPlot: missing argument: -w\n");

    let r = run(&["line", "--width", "x"], b"");
    assert_eq!(r.code(), 1);
    assert_eq!(text(r.stderr()), "YouPlot: invalid argument: --width x\n");
}
