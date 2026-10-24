mod common;
use common::assert_plot;

#[test]
fn default_boxplot_and_aliases() {
    assert_plot(&["boxplot"], "simple.tsv", "simple-boxplot.txt");
    assert_plot(&["box"], "simple.tsv", "simple-boxplot.txt");
}

#[test]
fn iris_boxplot() {
    assert_plot(
        &["box", "-H", "-d,", "-t", "IRIS-BOXPLOT"],
        "iris.csv",
        "iris-boxplot.txt",
    );
}
