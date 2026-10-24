use youplot_core::csv;
use youplot_core::dsv;

#[test]
fn csv_treats_empty_unquoted_field_as_nil() {
    let rows = csv::parse("a\t\tb\n", "\t").unwrap();
    assert_eq!(rows, vec![vec![Some("a".into()), None, Some("b".into())]]);
}

#[test]
fn csv_quoted_empty_is_empty_string() {
    let rows = csv::parse("a,\"\"\n", ",").unwrap();
    assert_eq!(rows, vec![vec![Some("a".into()), Some(String::new())]]);
}

#[test]
fn csv_quoted_fields_with_delimiters_and_newlines() {
    let rows = csv::parse("\"a,b\",\"c\nd\"\n", ",").unwrap();
    assert_eq!(
        rows,
        vec![vec![Some("a,b".into()), Some("c\nd".into())]]
    );
}

#[test]
fn csv_blank_line_is_empty_row() {
    let rows = csv::parse("a\n\nb\n", "\t").unwrap();
    assert_eq!(rows, vec![vec![Some("a".into())], vec![], vec![Some("b".into())]]);
}

#[test]
fn dsv_header_and_transpose() {
    let d = dsv::parse("x\ty\n1\t2\n3\t4\n", "\t", true, false).unwrap();
    assert_eq!(d.headers.unwrap(), vec![Some("x".into()), Some("y".into())]);
    assert_eq!(d.series.len(), 2);
    assert_eq!(d.series[0], vec![Some("1".into()), Some("3".into())]);
    assert_eq!(d.series[1], vec![Some("2".into()), Some("4".into())]);

    let d = dsv::parse("x\t1\t3\ny\t2\t4\n", "\t", true, true).unwrap();
    assert_eq!(d.headers.unwrap(), vec![Some("x".into()), Some("y".into())]);
    assert_eq!(d.series[0], vec![Some("1".into()), Some("3".into())]);
}

#[test]
fn dsv_without_headers_transposes() {
    let d = dsv::parse("a\tb\n1\t2\n", "\t", false, false).unwrap();
    assert!(d.headers.is_none());
    assert_eq!(d.series, vec![vec![Some("a".into()), Some("1".into())], vec![Some("b".into()), Some("2".into())]]);
}
