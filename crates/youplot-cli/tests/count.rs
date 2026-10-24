mod common;
use common::assert_plot;

#[test]
fn count_and_alias() {
    assert_plot(&["count"], "simple.tsv", "simple-count.txt");
    assert_plot(&["c"], "simple.tsv", "simple-count.txt");
}

#[test]
fn iris_count_with_headers() {
    assert_plot(
        &["count", "-H", "-d,"],
        "iris.csv",
        "iris-count.txt",
    );
}

#[test]
fn reverse_flips_label_order() {
    let input = b"a\na\nb\nb\nb\nc\n";
    let r = common::run(&["count"], input);
    assert_eq!(r.code(), 0);
    let out = String::from_utf8_lossy(r.stderr()).into_owned();
    let pos_b = out.find(" b ").unwrap();
    let pos_a = out.find(" a ").unwrap();
    let pos_c = out.find(" c ").unwrap();
    assert!(pos_b < pos_a && pos_a < pos_c, "{out}");

    let r = common::run(&["count", "-r"], input);
    let out = String::from_utf8_lossy(r.stderr()).into_owned();
    let pos_b = out.find(" b ").unwrap();
    let pos_a = out.find(" a ").unwrap();
    let pos_c = out.find(" c ").unwrap();
    assert!(pos_c < pos_a && pos_a < pos_b, "{out}");
}
