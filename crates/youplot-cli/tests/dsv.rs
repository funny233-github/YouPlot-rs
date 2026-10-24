mod common;
use common::{assert_plot, fixture, run};

#[test]
fn headers_and_comma_delimiter() {
    // The golden file was produced with an explicit title; use it verbatim.
    assert_plot(
        &["line", "-H", "-d,", "-t", "IRIS-LINEPLOT"],
        "iris.csv",
        "iris-lineplot.txt",
    );
}

#[test]
fn transpose_fixture_roundtrip() {
    assert_plot(&["line", "-T"], "simpleT.tsv", "simple-lineplot.txt");
}

#[test]
fn utf16_encoding() {
    assert_plot(
        &["s", "--encoding", "UTF-16", "-H", "-d,", "-t", "IRIS-SCATTER"],
        "iris_utf16.csv",
        "iris-scatter.txt",
    );
}

#[test]
fn invalid_utf8_reports_parse_failure() {
    let raw = fixture("iris_utf16.csv");
    let r = run(&["line"], &raw);
    assert_ne!(r.code(), 0);
    let err = String::from_utf8_lossy(r.stderr());
    assert!(err.contains("Failed to parse the text."), "{err}");
    assert!(err.contains("--encoding"), "{err}");
}

#[test]
fn quoted_csv_fields_and_blank_lines() {
    let input = b"1,\"2,5\"\n\n3,4\n";
    let r = run(&["line", "-d,"], input);
    assert_eq!(r.code(), 0, "{}", String::from_utf8_lossy(r.stderr()));
    assert!(!r.stderr().is_empty());
}

#[test]
fn header_count_mismatch_is_reported() {
    let input = b"a\tb\tc\n1\t2\n";
    let r = run(&["line", "-H"], input);
    assert_ne!(r.code(), 0);
    let err = String::from_utf8_lossy(r.stderr());
    assert!(err.contains("number of headers is greater"), "{err}");
}

#[test]
fn empty_input_produces_nothing() {
    let r = run(&["line"], b"");
    assert_eq!(r.code(), 0);
    assert!(r.stdout().is_empty());
    assert!(r.stderr().is_empty());
}
