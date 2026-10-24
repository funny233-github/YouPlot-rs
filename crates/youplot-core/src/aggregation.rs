use std::collections::HashMap;

/// Port of YouPlot::Aggregation.count_values. Returns [labels, counts].
pub fn count_values(arr: &[Option<String>], reverse: bool) -> (Vec<Option<String>>, Vec<i64>) {
    let mut tally: Vec<(Option<String>, i64)> = Vec::new();
    let mut index: HashMap<Option<String>, usize> = HashMap::new();
    for v in arr {
        if let Some(&i) = index.get(v) {
            tally[i].1 += 1;
        } else {
            index.insert(v.clone(), tally.len());
            tally.push((v.clone(), 1));
        }
    }

    let mut cache: HashMap<String, SortKey> = HashMap::new();
    tally.sort_by(|a, b| {
        let r = b.1.cmp(&a.1);
        if r.is_ne() {
            r
        } else {
            natural_compare(a.0.as_deref(), b.0.as_deref(), &mut cache)
        }
    });

    if reverse {
        tally.reverse();
    }

    let labels = tally.iter().map(|x| x.0.clone()).collect();
    let counts = tally.iter().map(|x| x.1).collect();
    (labels, counts)
}

#[derive(Clone)]
struct SortKey {
    kind: KeyKind,
    string: String,
    numeric: Option<f64>,
    tokens: Option<Vec<(bool, String)>>,
}

#[derive(Clone, PartialEq)]
enum KeyKind {
    Text,
    Numeric,
    Mixed,
}

fn natural_sort_key(value: Option<&str>, cache: &mut HashMap<String, SortKey>) -> SortKey {
    let str = value.unwrap_or("");
    if let Some(k) = cache.get(str) {
        return k.clone();
    }
    let has_digit = str.bytes().any(|b| b.is_ascii_digit());
    let key = if has_digit {
        if let Some(n) = parse_numeric(str) {
            SortKey {
                kind: KeyKind::Numeric,
                string: str.to_string(),
                numeric: Some(n),
                tokens: None,
            }
        } else {
            SortKey {
                kind: KeyKind::Mixed,
                string: str.to_string(),
                numeric: None,
                tokens: None,
            }
        }
    } else {
        SortKey {
            kind: KeyKind::Text,
            string: str.to_string(),
            numeric: None,
            tokens: None,
        }
    };
    cache.insert(str.to_string(), key.clone());
    key
}

fn ensure_tokens(key: &mut SortKey) -> Vec<(bool, String)> {
    if key.tokens.is_none() {
        key.tokens = Some(natural_tokens(&key.string));
    }
    key.tokens.clone().unwrap()
}

fn parse_numeric(s: &str) -> Option<f64> {
    let b = s.as_bytes();
    if b.is_empty() {
        return None;
    }
    let mut i = 0;
    if b[i] == b'+' || b[i] == b'-' {
        i += 1;
    }
    let start_digits = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let int_digits = i - start_digits;
    let mut frac_digits = 0;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let s2 = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        frac_digits = i - s2;
    }
    if i != b.len() {
        return None;
    }
    if int_digits == 0 && frac_digits == 0 {
        return None;
    }
    s.parse::<f64>().ok()
}

fn natural_tokens(s: &str) -> Vec<(bool, String)> {
    // Ruby: str.scan(/\d+|\D+/)
    let mut out = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let digit = chars[i].is_ascii_digit();
        let start = i;
        while i < chars.len() && chars[i].is_ascii_digit() == digit {
            i += 1;
        }
        out.push((digit, chars[start..i].iter().collect()));
    }
    out
}

fn natural_compare(
    a: Option<&str>,
    b: Option<&str>,
    cache: &mut HashMap<String, SortKey>,
) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let mut ka = natural_sort_key(a, cache);
    let mut kb = natural_sort_key(b, cache);

    if ka.kind == KeyKind::Text && kb.kind == KeyKind::Text {
        return ka.string.cmp(&kb.string);
    }
    if ka.kind == KeyKind::Numeric && kb.kind == KeyKind::Numeric {
        let r = ka.numeric.unwrap().partial_cmp(&kb.numeric.unwrap()).unwrap_or(Ordering::Equal);
        if r != Ordering::Equal {
            return r;
        }
        return ka.string.cmp(&kb.string);
    }

    let ta = ensure_tokens(&mut ka);
    let tb = ensure_tokens(&mut kb);
    let max = ta.len().max(tb.len());
    for i in 0..max {
        let xa = ta.get(i);
        let xb = tb.get(i);
        let Some(xa) = xa else { return Ordering::Less };
        let Some(xb) = xb else { return Ordering::Greater };
        let r = if xa.0 && xb.0 {
            compare_integer_strings(&xa.1, &xb.1)
        } else {
            xa.1.cmp(&xb.1)
        };
        if r != Ordering::Equal {
            return r;
        }
    }
    ka.string.cmp(&kb.string)
}

fn compare_integer_strings(a: &str, b: &str) -> std::cmp::Ordering {
    fn strip(s: &str) -> &str {
        let t = s.trim_start_matches('0');
        if t.is_empty() {
            "0"
        } else {
            t
        }
    }
    let aa = strip(a);
    let bb = strip(b);
    let r = aa.len().cmp(&bb.len());
    if r.is_ne() {
        return r;
    }
    let r = aa.cmp(bb);
    if r.is_ne() {
        return r;
    }
    a.cmp(b)
}
