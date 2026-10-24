//! Emulation of the small set of Ruby numeric / string semantics that
//! YouPlot's output depends on.

/// Ruby's `Float#to_s`.
pub fn float_to_s(x: f64) -> String {
    if x.is_nan() {
        return "NaN".to_string();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    if x == 0.0 {
        return if x.is_sign_negative() { "-0.0" } else { "0.0" }.to_string();
    }

    let neg = x.is_sign_negative();
    let (digits, decpt) = shortest_digits(x.abs());
    let mut s = String::new();
    if neg {
        s.push('-');
    }

    let fixed = if decpt > 0 {
        (decpt as usize) < digits.len() || decpt <= 15
    } else {
        decpt > -4
    };
    if fixed {
        if decpt <= 0 {
            s.push_str("0.");
            for _ in 0..(-decpt) {
                s.push('0');
            }
            s.push_str(&digits);
        } else if (decpt as usize) >= digits.len() {
            s.push_str(&digits);
            for _ in 0..(decpt as usize - digits.len()) {
                s.push('0');
            }
            s.push_str(".0");
        } else {
            s.push_str(&digits[..decpt as usize]);
            s.push('.');
            s.push_str(&digits[decpt as usize..]);
        }
    } else {
        let exp = decpt - 1;
        s.push(digits.as_bytes()[0] as char);
        s.push('.');
        if digits.len() > 1 {
            s.push_str(&digits[1..]);
        } else {
            s.push('0');
        }
        s.push('e');
        if exp < 0 {
            s.push('-');
        } else {
            s.push('+');
        }
        let ae = exp.abs();
        if ae < 10 {
            s.push('0');
        }
        s.push_str(&ae.to_string());
    }
    s
}

/// Return (significant digits without dot, decimal point position).
/// Uses Rust's shortest round-trip float formatting.
fn shortest_digits(x: f64) -> (String, i32) {
    // Ruby's dtoa produces the closest shortest round-trip representation.
    // Rust's Display is shortest, but not always closest among shortest
    // candidates, so find the smallest precision that round-trips.
    let mut chosen = format!("{:e}", x);
    for p in 1..=17usize {
        let candidate = format!("{:.*e}", p - 1, x);
        if candidate.parse::<f64>().ok() == Some(x) {
            chosen = candidate;
            break;
        }
    }
    let (mant, exp) = chosen.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    let digits: String = mant.chars().filter(|c| *c != '.').collect();
    let digits = digits.trim_end_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    (digits.to_string(), exp + 1)
}

/// Ruby `Float#round` (no argument): half away from zero, returns integer.
pub fn round_int(x: f64) -> i64 {
    if !x.is_finite() {
        return 0;
    }
    x.round() as i64
}

/// Ruby `Float#round(half: :even)` (no decimals).
pub fn round_half_even_int(x: f64) -> i64 {
    x.round_ties_even() as i64
}

#[derive(Clone, Copy, PartialEq)]
pub enum RoundMode {
    Round,
    Ceil,
    Floor,
}

/// Ruby `Float#round(n)`, `#ceil(n)` and `#floor(n)` for `n > 0`,
/// exactly following ruby/numeric.c (including its floating-point quirks).
/// For `n <= 0` an integral float is returned (Ruby returns an Integer).
pub fn round_digits(x: f64, ndigits: i32, mode: RoundMode) -> f64 {
    if x == 0.0 {
        return if ndigits > 0 { x } else { 0.0 };
    }
    if !x.is_finite() {
        return x;
    }
    if ndigits == 0 {
        let v = match mode {
            RoundMode::Round => x.round(),
            RoundMode::Ceil => x.ceil(),
            RoundMode::Floor => x.floor(),
        };
        return if v == 0.0 { 0.0 } else { v };
    }
    if ndigits < 0 {
        return round_digits_negative(x, ndigits, mode);
    }
    let binexp = frexp_exp(x);
    if float_round_overflow(ndigits, binexp) {
        return x;
    }
    match mode {
        RoundMode::Floor => {
            if x > 0.0 && float_round_underflow(ndigits, binexp) {
                return 0.0;
            }
            let f = pow10i(ndigits);
            let mul = (x * f).floor();
            let res = (mul + 1.0) / f;
            if res > x {
                mul / f
            } else {
                res
            }
        }
        RoundMode::Ceil => {
            if x < 0.0 && float_round_underflow(ndigits, binexp) {
                return 0.0;
            }
            let f = pow10i(ndigits);
            (x * f).ceil() / f
        }
        RoundMode::Round => {
            if float_round_underflow(ndigits, binexp) {
                return 0.0;
            }
            let f = pow10i(ndigits);
            if ndigits > 14 {
                // Ruby switches to exact Rational arithmetic here.
                return round_by_rational_approx(x, ndigits);
            }
            round_half_up(x, f) / f
        }
    }
}

fn f64_to_bigint_trunc(x: f64) -> num_bigint::BigInt {
    use num_bigint::BigInt;
    if x == 0.0 || !x.is_finite() {
        return BigInt::from(0);
    }
    let neg = x < 0.0;
    let ax = x.abs();
    let bits = ax.to_bits();
    let exp_field = ((bits >> 52) & 0x7ff) as i32;
    let frac = bits & ((1u64 << 52) - 1);
    let (m, e2) = if exp_field == 0 {
        (frac, -1074i32)
    } else {
        (frac | (1u64 << 52), exp_field - 1075)
    };
    let mut v = BigInt::from(m);
    if e2 >= 0 {
        v <<= e2 as usize;
    } else {
        // m carries 52 fractional bits; shift them off (truncate toward zero).
        v >>= (-e2) as usize;
    }
    if neg {
        -v
    } else {
        v
    }
}

fn bigint_to_f64(v: &num_bigint::BigInt) -> f64 {
    v.to_string().parse::<f64>().unwrap_or(0.0)
}

fn round_digits_negative(x: f64, ndigits: i32, mode: RoundMode) -> f64 {
    use num_bigint::BigInt;
    use num_traits::{Signed, Zero};
    let n = match mode {
        RoundMode::Round => f64_to_bigint_trunc(x),
        RoundMode::Ceil => f64_to_bigint_trunc(x.ceil()),
        RoundMode::Floor => f64_to_bigint_trunc(x.floor()),
    };
    let k = (-ndigits) as u32;
    let f: BigInt = BigInt::from(10u32).pow(k);
    let q = match mode {
        RoundMode::Round => {
            let neg = n.is_negative();
            let a = n.abs();
            let mut q = &a / &f;
            let r = &a % &f;
            if &r * 2 >= f {
                q += 1;
            }
            if neg {
                -q
            } else {
                q
            }
        }
        RoundMode::Floor => {
            let mut q = &n / &f;
            let r = &n % &f;
            if !r.is_zero() && n.is_negative() {
                q -= 1;
            }
            q
        }
        RoundMode::Ceil => {
            let mut q = &n / &f;
            let r = &n % &f;
            if !r.is_zero() && n.is_positive() {
                q += 1;
            }
            q
        }
    };
    bigint_to_f64(&(q * f))
}

fn round_half_up(x: f64, s: f64) -> f64 {
    let f = (x * s).round();
    if s == 1.0 {
        return f;
    }
    if x > 0.0 {
        if (f + 0.5) / s <= x {
            f + 1.0
        } else {
            f
        }
    } else if (f - 0.5) / s >= x {
        f - 1.0
    } else {
        f
    }
}

fn frexp_exp(x: f64) -> i32 {
    let bits = x.abs().to_bits();
    let e = ((bits >> 52) & 0x7ff) as i32;
    if e == 0 {
        let m = bits & ((1u64 << 52) - 1);
        let h = 63 - m.leading_zeros() as i32;
        h + 1 - 1074
    } else {
        e - 1022
    }
}

fn float_round_overflow(ndigits: i32, binexp: i32) -> bool {
    // DBL_DIG (15) + 2
    ndigits >= 17 - if binexp > 0 { binexp / 4 } else { binexp / 3 - 1 }
}

fn float_round_underflow(ndigits: i32, binexp: i32) -> bool {
    ndigits < -(if binexp > 0 { binexp / 3 + 1 } else { binexp / 4 })
}

/// Ruby's exact-Rational path used for ndigits > 14.
fn round_by_rational_approx(x: f64, ndigits: i32) -> f64 {
    use num_bigint::BigUint;
    let neg = x.is_sign_negative();
    let ax = x.abs();
    let bits = ax.to_bits();
    let exp_field = ((bits >> 52) & 0x7ff) as i32;
    let frac = bits & ((1u64 << 52) - 1);
    let (m, e2) = if exp_field == 0 {
        (frac, -1074i32)
    } else {
        (frac | (1u64 << 52), exp_field - 1075)
    };
    let n = ndigits as u32;
    let mut num = BigUint::from(m);
    if n > 0 {
        num *= BigUint::from(5u32).pow(n);
    }
    let k = e2 + ndigits;
    let rounded: BigUint = if k >= 0 {
        num << (k as usize)
    } else {
        let s = (-k) as usize;
        let q = &num >> s;
        let back = &q << s;
        let rem = &num - back;
        let half = BigUint::from(1u32) << (s - 1);
        if rem >= half {
            q + BigUint::from(1u32)
        } else {
            q
        }
    };
    if rounded == BigUint::from(0u32) {
        return 0.0;
    }
    let digits = rounded.to_str_radix(10);
    let nd = ndigits as usize;
    let decimal = if digits.len() > nd {
        let split = digits.len() - nd;
        format!("{}.{}", &digits[..split], &digits[split..])
    } else {
        format!("0.{}{}", "0".repeat(nd - digits.len()), digits)
    };
    let v = decimal.parse::<f64>().unwrap_or(0.0);
    if neg {
        -v
    } else {
        v
    }
}

fn pow10i(n: i32) -> f64 {
    // C's pow(10, n), used by ruby/numeric.c.
    if n >= 0 {
        10f64.powf(n as f64)
    } else {
        10f64.powf(n as f64)
    }
}

/// Ruby `ceil_neg_log10`.
pub fn ceil_neg_log10(x: f64) -> i32 {
    let v = -x.log10();
    if roundable(v) {
        v.ceil() as i32
    } else {
        v.floor() as i32
    }
}

pub fn roundable(x: f64) -> bool {
    let t = x.trunc();
    t == x && t >= i64::MIN as f64 && t < i64::MAX as f64
}

pub fn float_round_log10(x: f64, m: f64) -> f64 {
    if x == 0.0 {
        0.0
    } else if x > 0.0 {
        round_digits(x, ceil_neg_log10(m) + 1, RoundMode::Round)
    } else {
        -round_digits(-x, ceil_neg_log10(m) + 1, RoundMode::Round)
    }
}

/// Ruby `Float#step(to, by:)` iteration values (inclusive with endpoint
/// snapping), used for origin grid lines.
pub fn float_step_values(beg: f64, end: f64, unit: f64) -> Vec<f64> {
    let mut out = Vec::new();
    if unit == 0.0 || unit.is_nan() {
        return out;
    }
    if unit.is_infinite() {
        if (unit > 0.0 && beg <= end) || (unit < 0.0 && beg >= end) {
            out.push(beg);
        }
        return out;
    }
    let size = float_step_size(beg, end, unit);
    if size < 0.0 {
        return out;
    }
    let n = size as i64;
    for i in 0..=n {
        let mut d = beg + unit * (i as f64);
        if i == n {
            if (unit > 0.0 && d > end) || (unit < 0.0 && d < end) {
                d = end;
            }
        }
        if (unit > 0.0 && d > end) || (unit < 0.0 && d < end) {
            break;
        }
        out.push(d);
    }
    out
}

fn float_step_size(beg: f64, end: f64, unit: f64) -> f64 {
    let epsilon = f64::EPSILON;
    let mut n = (end - beg) / unit;
    let mut err = (beg.abs() + end.abs() + (end - beg).abs()) / unit.abs() * epsilon;
    if err > 0.5 {
        err = 0.5;
    }
    if n > 0.0 {
        if n < 1.0 {
            n = 0.0;
        } else if n < 1.0 + err {
            n = 1.0;
        } else {
            n = (n + err).floor();
        }
    } else {
        if n > -1.0 {
            n = 0.0;
        } else if n > -1.0 - err {
            n = -1.0;
        } else {
            n = (n - err).ceil();
        }
    }
    n
}

pub fn str_len(s: &str) -> usize {
    s.chars().count()
}

/// Ruby `String#to_f` on a numeric string: parse leading float prefix, else 0.
pub fn to_f(s: &str) -> f64 {
    let t = s.trim_start();
    let bytes = t.as_bytes();
    let mut end = 0usize;
    let mut i = 0usize;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    let mut seen_digit = false;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
        seen_digit = true;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
            seen_digit = true;
        }
    }
    if seen_digit {
        end = i;
        if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
            let mut j = i + 1;
            if j < bytes.len() && (bytes[j] == b'+' || bytes[j] == b'-') {
                j += 1;
            }
            let mut exp_digits = false;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
                exp_digits = true;
            }
            if exp_digits {
                end = j;
            }
        }
    }
    t[..end].parse::<f64>().unwrap_or(0.0)
}

/// Ruby `String#to_i` (leading integer prefix).
pub fn to_i(s: &str) -> i64 {
    let t = s.trim_start();
    let bytes = t.as_bytes();
    let mut i = 0usize;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    let start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == start {
        return 0;
    }
    t[..i].parse::<i64>().unwrap_or(0)
}

/// Ruby `Integer#to_s` / `Float#to_s` dispatch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Num {
    Int(i64),
    Float(f64),
}

impl Num {
    pub fn to_f(self) -> f64 {
        match self {
            Num::Int(i) => i as f64,
            Num::Float(f) => f,
        }
    }
    pub fn to_i(self) -> i64 {
        match self {
            Num::Int(i) => i,
            Num::Float(f) => f as i64,
        }
    }
    pub fn to_s(self) -> String {
        match self {
            Num::Int(i) => i.to_string(),
            Num::Float(f) => float_to_s(f),
        }
    }
}

impl std::fmt::Display for Num {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_s())
    }
}
