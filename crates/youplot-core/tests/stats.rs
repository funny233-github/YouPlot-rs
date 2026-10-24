use youplot_core::stats;

#[test]
fn percentile_interpolates() {
    let vals = [1.0, 2.0, 3.0, 4.0];
    let p = stats::percentiles(&vals, &[0.0, 25.0, 50.0, 75.0, 100.0]);
    assert_eq!(p, vec![1.0, 1.75, 2.5, 3.25, 4.0]);
}

#[test]
fn histogram_edges_and_counts() {
    let vals = [-10.0, 0.0, 10.0];
    let h = stats::histogram(&vals, None, None).unwrap();
    assert_eq!(h.edge, vec![-10.0, 0.0, 10.0, 20.0]);
    assert_eq!(h.weights, vec![1, 1, 1]);
    assert!(!h.closed_right());
}

#[test]
fn histogram_rejects_unknown_closed() {
    let err = stats::histogram(&[1.0, 2.0], None, Some("left")).unwrap_err();
    assert!(err.contains("invalid value for :closed keyword"));
}

#[test]
fn histogram_single_value_extends_one_bin() {
    let h = stats::histogram(&[5.0], None, None).unwrap();
    assert_eq!(h.edge, vec![5.0, 6.0]);
    assert_eq!(h.weights, vec![1]);
}

#[test]
fn sturges_bin_count() {
    assert_eq!(stats::sturges(1), 1);
    assert_eq!(stats::sturges(2), 2);
    assert_eq!(stats::sturges(100), 8);
}
