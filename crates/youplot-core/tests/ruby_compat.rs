use youplot_core::ruby::{
    ceil_neg_log10, float_round_log10, float_step_values, float_to_s, round_digits,
    round_half_even_int, to_f, to_i, Num, RoundMode,
};

#[test]
fn float_to_s_matches_ruby() {
    for (v, s) in [
        (1.0, "1.0"),
        (-0.0, "-0.0"),
        (0.1, "0.1"),
        (100.0, "100.0"),
        (1e-4, "0.0001"),
        (1e-5, "1.0e-05"),
        (1e14, "100000000000000.0"),
        (1e15, "1.0e+15"),
        (1e16, "1.0e+16"),
        (1234567890123456.0, "1.234567890123456e+15"),
        (1652326326377310.8, "1652326326377310.8"),
        (f64::INFINITY, "Infinity"),
        (f64::NEG_INFINITY, "-Infinity"),
        (f64::NAN, "NaN"),
    ] {
        assert_eq!(float_to_s(v), s, "float_to_s({v:?})");
    }
}

#[test]
fn rounding_matches_ruby_quirks() {
    assert_eq!(float_to_s(round_digits(2.675, 2, RoundMode::Round)), "2.68");
    assert_eq!(float_to_s(round_digits(1.005, 2, RoundMode::Round)), "1.01");
    assert_eq!(float_to_s(round_digits(0.25, 1, RoundMode::Round)), "0.3");
    // Float#round uses scaled arithmetic, including its surprising results.
    assert_eq!(
        float_to_s(round_digits(963549136834475.0, 1, RoundMode::Round)),
        "963549136834475.2"
    );
    assert_eq!(
        float_to_s(round_digits(999999999999999.0, 1, RoundMode::Round)),
        "999999999999999.2"
    );
    assert_eq!(float_to_s(round_digits(2.675, 2, RoundMode::Ceil)), "2.68");
    assert_eq!(float_to_s(round_digits(2.675, 2, RoundMode::Floor)), "2.67");
    assert_eq!(float_to_s(round_digits(1234.5, -1, RoundMode::Round)), "1230.0");
    assert_eq!(float_to_s(round_digits(1250.0, -2, RoundMode::Round)), "1300.0");
}

#[test]
fn negative_zero_and_even_rounding() {
    assert_eq!(float_to_s(round_digits(-0.0000316, 3, RoundMode::Round)), "-0.0");
    assert_eq!(round_half_even_int(2.5), 2);
    assert_eq!(round_half_even_int(3.5), 4);
    assert_eq!(round_half_even_int(-2.5), -2);
}

#[test]
fn string_number_parsing() {
    assert_eq!(to_f("1e3"), 1000.0);
    assert_eq!(to_f("1.5abc"), 1.5);
    assert_eq!(to_f("abc"), 0.0);
    assert_eq!(to_f("  -2.25"), -2.25);
    assert_eq!(to_i("-12x"), -12);
    assert_eq!(to_i("abc"), 0);
}

#[test]
fn util_helpers() {
    assert_eq!(ceil_neg_log10(1.0), 0);
    assert_eq!(ceil_neg_log10(0.1), 1);
    assert_eq!(float_to_s(float_round_log10(0.0, 1.0)), "0.0");
    assert_eq!(float_to_s(float_round_log10(-0.5, 1.0)), "-0.5");
}

#[test]
fn float_step_includes_endpoints_like_ruby() {
    let v = float_step_values(0.0, 1.0, 0.1);
    assert_eq!(v.len(), 11);
    assert_eq!(v[0], 0.0);
    assert_eq!(*v.last().unwrap(), 1.0);
    let v = float_step_values(0.1, 0.3, 0.1);
    assert_eq!(v.len(), 3);
    assert_eq!(*v.last().unwrap(), 0.3);
}

#[test]
fn num_display() {
    assert_eq!(Num::Int(12).to_s(), "12");
    assert_eq!(Num::Float(12.0).to_s(), "12.0");
}
