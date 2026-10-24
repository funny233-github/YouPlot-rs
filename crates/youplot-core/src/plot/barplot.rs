use super::{Color, Plot, PlotKind};
use crate::ruby::{round_int, str_len, Num};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scale {
    Log,
    Log10,
    Log2,
}

impl Scale {
    pub fn parse(s: &str) -> Result<Scale, String> {
        match s {
            "log" | "ln" => Ok(Scale::Log),
            "log10" | "lg" => Ok(Scale::Log10),
            "log2" | "lb" => Ok(Scale::Log2),
            _ => Err("func must be callable".to_string()),
        }
    }
    pub fn apply(self, x: f64) -> Result<f64, String> {
        let v = match self {
            Scale::Log => x.ln(),
            Scale::Log10 => x.log10(),
            Scale::Log2 => x.log2(),
        };
        if v.is_nan() {
            return Err("Domain error - Numerical argument is out of domain".to_string());
        }
        Ok(v)
    }
    pub fn name(self) -> &'static str {
        match self {
            Scale::Log => "log",
            Scale::Log10 => "log10",
            Scale::Log2 => "log2",
        }
    }
}

pub fn transform_name(scale: Option<&str>, basename: &str) -> String {
    match scale {
        None => basename.to_string(),
        Some(s) => format!("{} [{}]", basename, s),
    }
}

pub struct Barplot {
    pub bars: Vec<Num>,
    pub symbol: String,
    pub max_freq: f64,
    pub max_len: usize,
    pub width: i64,
    pub color: Color,
    pub transform: Option<Scale>,
}

impl Barplot {
    pub fn new(
        bars: Vec<Num>,
        width: i64,
        color: Color,
        symbol: String,
        transform: Option<String>,
    ) -> Result<Self, String> {
        if str_len(&symbol) > 1 {
            return Err("symbol must be a single character".to_string());
        }
        if bars.is_empty() {
            return Err("undefined method 'to_s' for nil".to_string());
        }
        let scale = match transform.as_deref() {
            Some(s) => Some(Scale::parse(s)?),
            None => None,
        };
        let (max_freq, i) = find_max(&bars, scale)?;
        let max_len = str_len(&bars[i].to_s());
        let width = width.max(max_len as i64 + 7).max(10);
        Ok(Barplot {
            bars,
            symbol,
            max_freq,
            max_len,
            width,
            color,
            transform: scale,
        })
    }

    pub fn print_row_out(&self, out: &mut crate::Out, row_index: i64) {
        let bar = self.bars[row_index as usize];
        let max_bar_width = (self.width - 2 - self.max_len as i64).max(1);
        let val = match self.transform {
            Some(t) => t.apply(bar.to_f()).unwrap_or(f64::NAN),
            None => bar.to_f(),
        };
        let bar_len = if self.max_freq > 0.0 {
            round_int(val.max(0.0) / self.max_freq * max_bar_width as f64)
        } else {
            0
        };
        let bar_str = if self.max_freq > 0.0 {
            self.symbol.repeat(bar_len.max(0) as usize)
        } else {
            String::new()
        };
        let bar_lbl = bar.to_s();
        super::renderer::print_styled(out, &[&bar_str], false, &self.color);
        let pair = format!(" {}", bar_lbl);
        super::renderer::print_styled(out, &[&pair], false, &Color::normal());
        let pan_len = (max_bar_width + 1 + self.max_len as i64
            - bar_len
            - str_len(&bar_lbl) as i64)
            .max(0);
        if pan_len > 0 {
            out.print(&" ".repeat(pan_len as usize));
        }
    }

    pub fn print_row(&self, row_index: i64, emit: &mut dyn FnMut(&Color, &str)) {
        let bar = self.bars[row_index as usize];
        let max_bar_width = (self.width - 2 - self.max_len as i64).max(1);
        let val = match self.transform {
            Some(t) => t.apply(bar.to_f()).unwrap_or(f64::NAN),
            None => bar.to_f(),
        };
        let bar_len = if self.max_freq > 0.0 {
            round_int(val.max(0.0) / self.max_freq * max_bar_width as f64)
        } else {
            0
        };
        let bar_str = if self.max_freq > 0.0 {
            self.symbol.repeat(bar_len.max(0) as usize)
        } else {
            String::new()
        };
        let bar_lbl = bar.to_s();
        emit(&self.color, &bar_str);
        emit(&Color::normal(), " ");
        emit(&Color::normal(), &bar_lbl);
        let pan_len = (max_bar_width + 1 + self.max_len as i64
            - bar_len
            - str_len(&bar_lbl) as i64)
            .max(0);
        if pan_len > 0 {
            emit(&Color::normal(), &" ".repeat(pan_len as usize));
        }
    }
}

pub fn find_max(bars: &[Num], transform: Option<Scale>) -> Result<(f64, usize), String> {
    let mut i = 0usize;
    let mut j = 0usize;
    let mut max = transform_value(transform, bars[0])?;
    while j < bars.len() {
        let v = transform_value(transform, bars[j])?;
        if v > max {
            i = j;
            max = v;
        }
        j += 1;
    }
    Ok((max, i))
}

pub fn transform_value(transform: Option<Scale>, bar: Num) -> Result<f64, String> {
    match transform {
        Some(t) => t.apply(bar.to_f()),
        None => Ok(bar.to_f()),
    }
}

/// Build a barplot.
#[allow(clippy::too_many_arguments)]
pub fn barplot(
    labels: Vec<Option<String>>,
    bars: Vec<Num>,
    width: i64,
    color: Color,
    symbol: String,
    transform: Option<String>,
    border: super::Border,
    margin: i64,
    padding: i64,
    show_labels: bool,
    params: &crate::Params,
) -> Result<Plot, String> {
    if labels.len() != bars.len() {
        return Err("The given vectors must be of the same length".to_string());
    }
    if let Some(min) = bars.iter().map(|b| b.to_f()).reduce(f64::min) {
        if min < 0.0 {
            return Err("All values have to be positive. Negative bars are not supported.".to_string());
        }
    } else {
        return Err("undefined method 'to_s' for nil".to_string());
    }
    let transform_name_src = transform.clone();
    let bar = Barplot::new(bars, width, color, symbol, transform)?;
    let mut plot = Plot::new(border, margin, padding, show_labels, PlotKind::Bar(bar));
    super::apply_common(&mut plot, params);
    if plot.xlabel.is_none() {
        plot.xlabel = Some(transform_name(transform_name_src.as_deref(), ""));
    }
    for (i, key) in labels.into_iter().enumerate() {
        plot.annotate_row("l", i as i64, key, Color::normal());
    }
    Ok(plot)
}
