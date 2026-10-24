//! Port of enumerable-statistics histogram/percentile used by UnicodePlot.

use crate::ruby::{round_digits, RoundMode};

#[derive(Debug)]
pub struct Histogram {
    pub edge: Vec<f64>,
    pub weights: Vec<i64>,
    pub closed_right: bool,
}

impl Histogram {
    pub fn closed_right(&self) -> bool {
        self.closed_right
    }
}

pub fn percentile(sorted: &[f64], q: f64) -> f64 {
    let n = sorted.len();
    assert!(n > 0);
    if n == 1 {
        return sorted[0];
    }
    let d = (n as f64 - 1.0) * q / 100.0;
    let i = d.trunc();
    let f = d - i;
    let l = i as usize;
    let x0 = sorted[l];
    if f == 0.0 || l == n - 1 {
        return x0;
    }
    let x1 = sorted[l + 1];
    x0 * (1.0 - f) + x1 * f
}

pub fn percentiles(values: &[f64], qs: &[f64]) -> Vec<f64> {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    qs.iter().map(|&q| percentile(&sorted, q)).collect()
}

pub fn sturges(n: usize) -> i64 {
    if n == 0 {
        1
    } else {
        ((n as f64).log2().ceil() + 1.0) as i64
    }
}

pub fn histogram(values: &[f64], nbins: Option<i64>, closed: Option<&str>) -> Result<Histogram, String> {
    let left_p = match closed {
        None => true,
        Some("right") => false,
        Some(_) => {
            return Err(format!(
                "invalid value for :closed keyword ({} for :left or :right)",
                closed.unwrap()
            ))
        }
    };
    let n = values.len();
    let bins: i64 = match nbins {
        None => sturges(n),
        Some(v) => v,
    };
    if n == 0 && bins < 0 {
        return Err(format!("nbins must be >= 0 for an empty array, got {}", bins));
    }
    if n > 0 && bins < 1 {
        return Err(format!(
            "nbins must be >= 1 for a non-empty array, got {}",
            bins
        ));
    }
    if n == 0 {
        return Ok(Histogram {
            edge: vec![0.0],
            weights: vec![],
            closed_right: !left_p,
        });
    }
    let mut lo = values[0];
    let mut hi = values[0];
    for &v in values {
        if v < lo {
            lo = v;
        }
        if v > hi {
            hi = v;
        }
    }

    let edge = calculate_edge(lo, hi, bins, left_p);
    let n_bins = edge.len() - 1;
    let mut weights = vec![0i64; n_bins];
    for &x in values {
        let bi = bin_index(&edge, x, left_p);
        if bi >= 0 && (bi as usize) < n_bins {
            weights[bi as usize] += 1;
        }
    }
    Ok(Histogram { edge, weights, closed_right: !left_p })
}

fn bin_index(edge: &[f64], x: f64, left_p: bool) -> i64 {
    let mut lo: i64 = -1;
    let mut hi: i64 = edge.len() as i64;
    if left_p {
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            let y = edge[mid as usize];
            if y <= x {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        lo
    } else {
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            let y = edge[mid as usize];
            if y < x {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        hi - 1
    }
}

fn calculate_edge(lo: f64, hi: f64, nbins: i64, left_p: bool) -> Vec<f64> {
    let mut start: f64;
    let mut step: f64;
    let mut divisor: f64;
    let mut len: i64;

    if hi == lo {
        start = hi;
        step = 1.0;
        divisor = 1.0;
        len = 1;
    } else {
        let bw = (hi - lo) / nbins as f64;
        let lbw = bw.log10();
        if lbw >= 0.0 {
            step = 10f64.powf(lbw.floor());
            let r = bw / step;
            if r <= 1.1 {
                // do nothing
            } else if r <= 2.2 {
                step *= 2.0;
            } else if r <= 5.5 {
                step *= 5.0;
            } else {
                step *= 10.0;
            }
            divisor = 1.0;
            start = step * (lo / step).floor();
            len = ((hi - start) / step).ceil() as i64;
        } else {
            divisor = 10f64.powf(-lbw.floor());
            let r = bw * divisor;
            if r <= 1.1 {
                // do nothing
            } else if r <= 2.2 {
                divisor /= 2.0;
            } else if r <= 5.5 {
                divisor /= 5.0;
            } else {
                divisor /= 10.0;
            }
            step = 1.0;
            start = (lo * divisor).floor();
            len = (hi * divisor - start).ceil() as i64;
        }
    }

    if left_p {
        while lo < start / divisor {
            start -= step;
        }
        while (start + ((len - 1) as f64) * step) / divisor <= hi {
            len += 1;
        }
    } else {
        while lo <= start / divisor {
            start -= step;
        }
        while (start + ((len - 1) as f64) * step) / divisor < hi {
            len += 1;
        }
    }

    let mut edge = Vec::with_capacity(len as usize);
    for _ in 0..len {
        edge.push(start / divisor);
        start += step;
    }
    edge
}

/// Used by the internal xlim/ylim computation helper in Utils.
pub fn round_up_subtick(x: f64, m: f64) -> f64 {
    if x == 0.0 {
        0.0
    } else if x > 0.0 {
        round_digits(x, ndigits_for(m), RoundMode::Ceil)
    } else {
        -round_digits(-x, ndigits_for(m), RoundMode::Floor)
    }
}

pub fn round_down_subtick(x: f64, m: f64) -> f64 {
    if x == 0.0 {
        0.0
    } else if x > 0.0 {
        round_digits(x, ndigits_for(m), RoundMode::Floor)
    } else {
        -round_digits(-x, ndigits_for(m), RoundMode::Ceil)
    }
}

fn ndigits_for(m: f64) -> i32 {
    crate::ruby::ceil_neg_log10(m) + 1
}
