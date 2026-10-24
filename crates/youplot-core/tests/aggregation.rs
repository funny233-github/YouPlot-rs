use youplot_core::aggregation::count_values;

fn s(v: &str) -> Option<String> {
    Some(v.to_string())
}

#[test]
fn counts_sort_by_frequency_then_natural_name() {
    let data = vec![s("b"), s("a"), s("a"), s("chr10"), s("chr2"), s("chr2")];
    let (labels, counts) = count_values(&data, false);
    assert_eq!(labels, vec![s("a"), s("chr2"), s("b"), s("chr10")]);
    assert_eq!(counts, vec![2, 2, 1, 1]);
}

#[test]
fn reverse_flips_order() {
    let data = vec![s("x"), s("x"), s("y")];
    let (labels, counts) = count_values(&data, true);
    assert_eq!(labels, vec![s("y"), s("x")]);
    assert_eq!(counts, vec![1, 2]);
}

#[test]
fn numeric_labels_sort_numerically() {
    let data = vec![s("10"), s("2"), s("2"), s("1")];
    let (labels, _) = count_values(&data, false);
    assert_eq!(labels, vec![s("2"), s("1"), s("10")]);
}

#[test]
fn nil_labels_are_grouped() {
    let data = vec![None, None, s("a")];
    let (labels, counts) = count_values(&data, false);
    assert_eq!(counts, vec![2, 1]);
    assert_eq!(labels[0], None);
}
