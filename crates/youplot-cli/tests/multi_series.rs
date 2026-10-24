mod common;
use common::{assert_error, assert_plot};

#[test]
fn lineplots_xyy_three_series() {
    assert_plot(
        &["lineplots", "-H", "-d,", "-t", "IRIS-LINEPLOTS"],
        "iris.csv",
        "iris-lineplots.txt",
    );
    assert_plot(
        &["lines", "-H", "-d,", "-t", "IRIS-LINEPLOTS"],
        "iris.csv",
        "iris-lineplots.txt",
    );
}

#[test]
fn scatterplot() {
    assert_plot(
        &["scatter", "-H", "-d,", "-t", "IRIS-SCATTER"],
        "iris.csv",
        "iris-scatter.txt",
    );
    assert_plot(
        &["s", "-H", "-d,", "-t", "IRIS-SCATTER"],
        "iris.csv",
        "iris-scatter.txt",
    );
}

#[test]
fn density_plot() {
    assert_plot(
        &["density", "-H", "-d,", "-t", "IRIS-DENSITY"],
        "iris.csv",
        "iris-density.txt",
    );
    assert_plot(
        &["d", "-H", "-d,", "-t", "IRIS-DENSITY"],
        "iris.csv",
        "iris-density.txt",
    );
}

#[test]
fn single_series_is_rejected() {
    assert_error(
        &["lineplots"],
        "simple.tsv",
        "There is only one series of input data",
    );
    assert_error(
        &["scatter"],
        "simple.tsv",
        "There is only one series of input data",
    );
    assert_error(
        &["density"],
        "simple.tsv",
        "There is only one series of input data",
    );
}

#[test]
fn odd_series_for_xyxy_is_rejected() {
    assert_error(
        &["lineplots", "--fmt", "xyxy", "-H"],
        "simple-xyxy-odd.tsv",
        "In the xyxy format, the number of series must be even",
    );
}
