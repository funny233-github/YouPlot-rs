mod common;
use common::assert_plot;

#[test]
fn default_histogram_and_aliases() {
    assert_plot(&["histogram"], "simple.tsv", "simple-histogram.txt");
    assert_plot(&["hist"], "simple.tsv", "simple-histogram.txt");
}

#[test]
fn custom_symbol() {
    assert_plot(
        &["hist", "--symbol", "@"],
        "simple.tsv",
        "simple-histogram-symbol-@.txt",
    );
}

#[test]
fn iris_histogram() {
    assert_plot(
        &["hist", "-H", "-d,", "-t", "IRIS-HISTOGRAM"],
        "iris.csv",
        "iris-histogram.txt",
    );
}

#[test]
fn nbins_controls_bin_count() {
    let r = common::run_fixture(&["hist", "-n", "2"], "simple.tsv");
    assert_eq!(r.code(), 0, "{}", String::from_utf8_lossy(r.stderr()));
    let out = String::from_utf8_lossy(r.stderr());
    let bins = out.lines().filter(|l| l.contains('[') && l.contains(')')).count();
    // `nbins` is only approximate: 2 requested, 3 bins produced here.
    assert_eq!(bins, 3, "{out}");
}

#[test]
fn closed_option_is_rejected_like_ruby() {
    // YouPlot passes the string through to enumerable-statistics, which only
    // accepts :left/:right symbols, so any --closed value raises.
    common::assert_error(
        &["hist", "--closed", "left"],
        "simple.tsv",
        "invalid value for :closed keyword",
    );
}
