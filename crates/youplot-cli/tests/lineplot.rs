mod common;
use common::assert_plot;

#[test]
fn default_lineplot_and_aliases() {
    assert_plot(&["line"], "simple.tsv", "simple-lineplot.txt");
    assert_plot(&["lineplot"], "simple.tsv", "simple-lineplot.txt");
    assert_plot(&["l"], "simple.tsv", "simple-lineplot.txt");
}

#[test]
fn transpose() {
    assert_plot(&["line", "--transpose"], "simpleT.tsv", "simple-lineplot.txt");
    assert_plot(&["line", "-T"], "simpleT.tsv", "simple-lineplot.txt");
}

#[test]
fn labels_and_titles() {
    assert_plot(&["line", "--xlabel", "X-LABEL"], "simple.tsv", "simple-lineplot-xlabel.txt");
    assert_plot(&["line", "--ylabel", "Y-LABEL"], "simple.tsv", "simple-lineplot-ylabel.txt");
}

#[test]
fn dimensions_and_spacing() {
    assert_plot(&["line", "--width", "17"], "simple.tsv", "simple-lineplot-width-17.txt");
    assert_plot(&["line", "-w", "17"], "simple.tsv", "simple-lineplot-width-17.txt");
    assert_plot(&["line", "--height", "17"], "simple.tsv", "simple-lineplot-height-17.txt");
    assert_plot(&["line", "-h", "17"], "simple.tsv", "simple-lineplot-height-17.txt");
    assert_plot(&["line", "--margin", "17"], "simple.tsv", "simple-lineplot-margin-17.txt");
    assert_plot(&["line", "-m", "17"], "simple.tsv", "simple-lineplot-margin-17.txt");
    assert_plot(&["line", "--padding", "17"], "simple.tsv", "simple-lineplot-padding-17.txt");
}

#[test]
fn borders() {
    assert_plot(&["line", "--border", "corners"], "simple.tsv", "simple-lineplot-border-corners.txt");
    assert_plot(&["line", "-b", "corners"], "simple.tsv", "simple-lineplot-border-corners.txt");
    assert_plot(&["line", "--border", "barplot"], "simple.tsv", "simple-lineplot-border-barplot.txt");
}

#[test]
fn canvases() {
    assert_plot(&["line", "--canvas", "braille"], "simple.tsv", "simple-lineplot.txt");
    assert_plot(&["line", "--canvas", "ascii"], "simple.tsv", "simple-lineplot-canvas-ascii.txt");
    assert_plot(&["line", "--canvas", "dot"], "simple.tsv", "simple-lineplot-canvas-dot.txt");
    assert_plot(&["line", "--canvas", "density"], "simple.tsv", "simple-lineplot-canvas-density.txt");
}

#[test]
fn limits_and_grid() {
    assert_plot(&["line", "--xlim", "-1,5"], "simple.tsv", "simple-lineplot-xlim--1-5.txt");
    assert_plot(&["line", "--ylim", "-25,50"], "simple.tsv", "simple-lineplot-ylim--25-50.txt");
    assert_plot(
        &["line", "--xlim", "-1,5", "--ylim", "-25,50"],
        "simple.tsv",
        "simple-lineplot-xlim--1-5-ylim--25-50.txt",
    );
    assert_plot(&["line", "--grid"], "simple.tsv", "simple-lineplot.txt");
    assert_plot(&["line", "--no-grid"], "simple.tsv", "simple-lineplot-no-grid.txt");
}

#[test]
fn iris_lineplot() {
    assert_plot(
        &["line", "-H", "-d,", "-t", "IRIS-LINEPLOT"],
        "iris.csv",
        "iris-lineplot.txt",
    );
}
