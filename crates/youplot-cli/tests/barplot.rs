mod common;
use common::{assert_error, assert_plot};

#[test]
fn iris_barplot() {
    assert_plot(
        &["barplot", "-H", "-d,", "-t", "IRIS-BARPLOT"],
        "iris.csv",
        "iris-barplot.txt",
    );
    assert_plot(
        &["bar", "-H", "-d,", "-t", "IRIS-BARPLOT"],
        "iris.csv",
        "iris-barplot.txt",
    );
}

#[test]
fn integer_labels_are_not_floats() {
    assert_plot(&["bar"], "integer.tsv", "integer-barplot.txt");
    assert_plot(
        &["bar"],
        "integer-like-float.tsv",
        "integer-like-float-barplot.txt",
    );
}

#[test]
fn negative_values_are_rejected() {
    assert_error(
        &["bar"],
        "simple.tsv",
        "All values have to be positive. Negative bars are not supported.",
    );
}
