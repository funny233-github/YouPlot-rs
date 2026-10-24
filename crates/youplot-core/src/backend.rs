use crate::aggregation;
use crate::plot::barplot::{self, transform_name};
use crate::plot::boxplot;
use crate::plot::grid;
use crate::plot::renderer::text_color;
use crate::plot::{param_color, Border, Color, Plot};
use crate::ruby::{self, round_int, str_len, Num};
use crate::stats;
use crate::{Data, Error, Params, Result};

pub enum Rendered {
    Plot(Box<Plot>),
    Text(String),
}

/// Dispatch a YouPlot command to the plotting backend.
/// `fmt` defaults to `"xyy"`; `reverse` applies to `count`;
/// `color_names` applies to `colors`.
pub fn create_plot(
    command: &str,
    data: Option<&Data>,
    params: &mut Params,
    fmt: &str,
    reverse: bool,
    color_names: bool,
) -> Result<Rendered> {
    match command {
        "bar" | "barplot" => {
            let data = data.unwrap();
            barplot_impl(data, params, Some(fmt), false, false)
        }
        "count" | "c" => {
            let data = data.unwrap();
            barplot_impl(data, params, None, true, reverse)
        }
        "hist" | "histogram" => {
            let data = data.unwrap();
            histogram(data, params)
        }
        "line" | "lineplot" | "l" => {
            let data = data.unwrap();
            line(data, params, fmt)
        }
        "lines" | "lineplots" | "ls" => {
            let data = data.unwrap();
            plot_fmt(data, fmt, Method::Line, params)
        }
        "scatter" | "s" => {
            let data = data.unwrap();
            plot_fmt(data, fmt, Method::Scatter, params)
        }
        "density" | "d" => {
            let data = data.unwrap();
            plot_fmt(data, fmt, Method::Density, params)
        }
        "box" | "boxplot" => {
            let data = data.unwrap();
            boxplot_impl(data, params)
        }
        "colors" | "color" | "colours" | "colour" => {
            Ok(Rendered::Text(colors(color_names)))
        }
        _ => Err(Error::Message(format!("unrecognized plot_type: {}", command))),
    }
}

fn num_val(v: &str) -> f64 {
    ruby::to_f(v)
}

fn cell_f(cell: &Option<String>) -> Result<f64> {
    // Ruby's NilClass#to_f is 0.0.
    Ok(match cell {
        Some(s) => num_val(s),
        None => 0.0,
    })
}

fn integer_literal(v: &Option<String>) -> bool {
    let Some(s) = v else { return false };
    if s.is_empty() {
        return false;
    }
    let b = s.as_bytes();
    let mut i = 0;
    if b[0] == b'+' || b[0] == b'-' {
        i = 1;
    }
    if i >= b.len() {
        return false;
    }
    b[i..].iter().all(|c| c.is_ascii_digit())
}

fn integer_display_values(values: &[f64], raw: &[Option<String>]) -> bool {
    values
        .iter()
        .all(|v| v.is_finite() && *v == (*v as i64) as f64)
        && raw.iter().all(integer_literal)
}

fn title_default(params: &mut Params, headers: &Option<Vec<Option<String>>>, idx: usize) {
    if params.title.is_none() {
        if let Some(h) = headers {
            params.title = h.get(idx).cloned().flatten();
        }
    }
}

fn barplot_impl(
    data: &Data,
    params: &mut Params,
    fmt: Option<&str>,
    count: bool,
    reverse: bool,
) -> Result<Rendered> {
    let headers = &data.headers;
    let series = &data.series;

    let labels: Vec<Option<String>>;
    let bars: Vec<Num>;

    let mut series_for_count: Vec<Vec<Option<String>>> = Vec::new();
    let mut counted = false;

    if count {
        if series.is_empty() {
            return Err(Error::marker());
        }
        let (labels0, counts) = aggregation::count_values(&series[0], reverse);
        series_for_count.push(labels0.clone());
        series_for_count.push(counts.iter().map(|c| Some(c.to_string())).collect());
        counted = true;
        labels = labels0;
        bars = counts.iter().map(|&c| Num::Int(c)).collect();
    } else if series.len() == 1 {
        title_default(params, headers, 0);
        labels = (1..=series[0].len())
            .map(|i| Some(i.to_string()))
            .collect();
        let mut values = Vec::with_capacity(series[0].len());
        for cell in &series[0] {
            values.push(cell_f(cell)?);
        }
        if integer_display_values(&values, &series[0]) {
            bars = values.iter().map(|v| Num::Int(*v as i64)).collect();
        } else {
            bars = values.iter().map(|v| Num::Float(*v)).collect();
        }
    } else {
        let fmt = fmt.unwrap_or("xyy");
        let (x_col, y_col) = if fmt == "yx" { (1, 0) } else { (0, 1) };
        if x_col >= series.len() || y_col >= series.len() {
            return Err(Error::marker());
        }
        title_default(params, headers, y_col);
        labels = series[x_col].clone();
        let mut values = Vec::with_capacity(series[y_col].len());
        for cell in &series[y_col] {
            values.push(cell_f(cell)?);
        }
        if count || integer_display_values(&values, &series[y_col]) {
            bars = values.iter().map(|v| Num::Int(*v as i64)).collect();
        } else {
            bars = values.iter().map(|v| Num::Float(*v)).collect();
        }
    }

    if counted {
        // `params.title = headers[0]` in Ruby overrides a CLI title.
        if let Some(h) = headers {
            params.title = h.first().cloned().flatten();
        }
        let _ = series_for_count;
    }

    let transform = params.xscale.clone();
    let border = crate::plot::default_border(params, Border::Barplot).map_err(Error::Argument)?;
    let margin = crate::plot::default_margin(params).map_err(Error::Argument)?;
    let padding = crate::plot::default_padding(params);
    let labels_flag = crate::plot::default_labels(params);
    let width = crate::plot::default_width(params);
    let color = param_color(params, "green");
    let symbol = params.symbol.clone().unwrap_or_else(|| "■".to_string());

    let plot = barplot::barplot(
        labels,
        bars,
        width,
        color,
        symbol,
        transform,
        border,
        margin,
        padding,
        labels_flag,
        params,
    )
    .map_err(Error::Argument)?;
    Ok(Rendered::Plot(Box::new(plot)))
}

fn histogram(data: &Data, params: &mut Params) -> Result<Rendered> {
    if let Some(h) = &data.headers {
        if params.title.is_none() {
            params.title = h.first().cloned().flatten();
        }
    }
    let mut values = Vec::new();
    if data.series.is_empty() {
        return Err(Error::marker());
    }
    for cell in &data.series[0] {
        values.push(cell_f(cell)?);
    }

    let nbins = params.nbins.map(|v| v as i64);
    let hist = stats::histogram(&values, nbins, params.closed.as_deref())
        .map_err(Error::Argument)?;

    let symbol = params.symbol.clone().unwrap_or_else(|| "▇".to_string());
    let bin_width = if hist.edge.len() > 1 {
        hist.edge[1] - hist.edge[0]
    } else {
        return Err(Error::marker());
    };

    let mut pad_left = 0usize;
    let mut pad_right = 0usize;
    let mut parts: Vec<(String, usize, usize, String, usize, usize)> = Vec::new();
    for &e in &hist.edge {
        let val1 = ruby::float_round_log10(e, bin_width);
        let val2 = ruby::float_round_log10(val1 + bin_width, bin_width);
        let (a1l, a1r) = split_len(&ruby::float_to_s(val1));
        let (a2l, a2r) = split_len(&ruby::float_to_s(val2));
        pad_left = pad_left.max(a1l).max(a2l);
        pad_right = pad_right.max(a1r).max(a2r);
        parts.push((ruby::float_to_s(val1), a1l, a1r, ruby::float_to_s(val2), a2l, a2r));
    }

    let l_str = if hist.closed_right() { "(" } else { "[" };
    let r_str = if hist.closed_right() { "]" } else { ")" };

    let mut labels: Vec<Option<String>> = Vec::with_capacity(hist.weights.len());
    for (v1, a1l, a1r, v2, a2l, a2r) in parts.iter().take(hist.weights.len()) {
        let mut label = String::new();
        label.push_str("\u{1b}[90m");
        label.push_str(l_str);
        label.push_str("\u{1b}[0m");
        label.push_str(&" ".repeat(pad_left.saturating_sub(*a1l)));
        label.push_str(v1);
        label.push_str(&" ".repeat(pad_right.saturating_sub(*a1r)));
        label.push_str("\u{1b}[90m, \u{1b}[0m");
        label.push_str(&" ".repeat(pad_left.saturating_sub(*a2l)));
        label.push_str(v2);
        label.push_str(&" ".repeat(pad_right.saturating_sub(*a2r)));
        label.push_str("\u{1b}[90m");
        label.push_str(r_str);
        label.push_str("\u{1b}[0m");
        labels.push(Some(label));
    }

    let transform = params.xscale.clone();
    if params.xlabel.is_none() {
        params.xlabel = Some(transform_name(transform.as_deref(), "Frequency"));
    }
    let border = crate::plot::default_border(params, Border::Barplot).map_err(Error::Argument)?;
    let margin = crate::plot::default_margin(params).map_err(Error::Argument)?;
    let padding = crate::plot::default_padding(params);
    let labels_flag = crate::plot::default_labels(params);
    let width = crate::plot::default_width(params);
    let color = param_color(params, "green");
    let bars: Vec<Num> = hist.weights.iter().map(|&w| Num::Int(w)).collect();

    let plot = barplot::barplot(
        labels,
        bars,
        width,
        color,
        symbol,
        transform,
        border,
        margin,
        padding,
        labels_flag,
        params,
    )
    .map_err(Error::Argument)?;
    Ok(Rendered::Plot(Box::new(plot)))
}

fn split_len(s: &str) -> (usize, usize) {
    match s.split_once('.') {
        Some((a, b)) => (str_len(a), str_len(b)),
        None => (str_len(s), 0),
    }
}

fn values_of(series: &[Vec<Option<String>>]) -> Result<Vec<Vec<f64>>> {
    let mut out = Vec::with_capacity(series.len());
    for s in series {
        let mut v = Vec::with_capacity(s.len());
        for cell in s {
            v.push(cell_f(cell)?);
        }
        out.push(v);
    }
    Ok(out)
}

fn minmax(v: &[f64]) -> Result<(f64, f64)> {
    if v.is_empty() {
        return Err(Error::marker());
    }
    let mut mi = v[0];
    let mut ma = v[0];
    for &x in v {
        if x < mi {
            mi = x;
        }
        if x > ma {
            ma = x;
        }
    }
    Ok((mi, ma))
}

fn set_xlim_ylim(params: &mut Params, x: &[f64], ys: &[f64]) -> Result<()> {
    if params.xlim.is_none() {
        let (mi, ma) = minmax(x)?;
        params.xlim = Some(vec![mi, ma]);
    }
    if params.ylim.is_none() {
        let (mi, ma) = minmax(ys)?;
        params.ylim = Some(vec![mi, ma]);
    }
    Ok(())
}

const CANVAS_COLORS: [&str; 8] = [
    "normal", "blue", "red", "magenta", "green", "cyan", "yellow", "white",
];

fn warn_numeric_line_color() {
    eprintln!("YouPlot: Line colors cannot be specified by numerical values.");
    eprintln!();
    eprintln!("For more information, please see the following issue.");
    eprintln!("https://github.com/red-data-tools/unicode_plot.rb/issues/34");
}

/// Replicates the errors UnicodePlot raises for non-style canvas colors.
fn color_option(params: &Params, line_method: bool) -> Result<Color> {
    let c = param_color_auto(params);
    match &c {
        Color::Index(_) => {
            if line_method {
                warn_numeric_line_color();
                Err(Error::marker())
            } else {
                Err(Error::Message(
                    "nil can't be coerced into Integer (TypeError)".to_string(),
                ))
            }
        }
        Color::Name(n) if n == "auto" || CANVAS_COLORS.contains(&n.as_str()) => Ok(c),
        Color::Name(_) => Err(Error::Message(
            "nil can't be coerced into Integer (TypeError)".to_string(),
        )),
    }
}

fn param_color_auto(params: &Params) -> Color {
    match &params.color {
        None => Color::named("auto"),
        Some(crate::ParamColor::Name(n)) => Color::Name(n.clone()),
        Some(crate::ParamColor::Number(n)) => {
            if *n >= 0 && *n <= 255 {
                Color::Index(*n as u8)
            } else {
                Color::named("auto")
            }
        }
    }
}

fn name_of(headers: &Option<Vec<Option<String>>>, i: usize) -> Option<String> {
    headers.as_ref().and_then(|h| h.get(i)).cloned().flatten()
}

#[derive(Clone, Copy)]
enum Method {
    Line,
    Scatter,
    Density,
}

fn plot_xyy(data: &Data, method: Method, params: &mut Params) -> Result<Rendered> {
    let headers = &data.headers;
    let mut series = values_of(&data.series)?;
    let series0 = series[0].clone();

    if params.name.is_none() {
        params.name = name_of(headers, 1);
    }
    if params.xlabel.is_none() {
        params.xlabel = name_of(headers, 0);
    }
    let mut ys: Vec<f64> = Vec::new();
    for s in &series[1..] {
        ys.extend_from_slice(s);
    }
    set_xlim_ylim(params, &series0, &ys)?;

    let color = color_option(params, matches!(method, Method::Line))?;
    let name = params.name.clone();
    let x0 = series[0].clone();
    let y0 = series[1].clone();
    let mut plot = match method {
        Method::Line => grid::lineplot(&x0, &y0, params, color, name),
        Method::Scatter => grid::scatterplot(&x0, &y0, params, color, name),
        Method::Density => grid::densityplot(&x0, &y0, params, color, name),
    }
    .map_err(Error::Argument)?;

    for i in 2..series.len() {
        let xi = series[0].clone();
        let yi = series[i].clone();
        let name = name_of(headers, i);
        match method {
            Method::Line => grid::add_line(&mut plot, &xi, &yi, Color::named("auto"), name),
            Method::Scatter => {
                grid::add_points(&mut plot, &xi, &yi, Color::named("auto"), name)
            }
            Method::Density => grid::add_points(&mut plot, &xi, &yi, Color::named("auto"), name),
        }
        .map_err(Error::Argument)?;
    }
    series.clear();
    Ok(Rendered::Plot(Box::new(plot)))
}

fn plot_xyxy(data: &Data, method: Method, params: &mut Params) -> Result<Rendered> {
    let headers = &data.headers;
    let series = values_of(&data.series)?;
    let mut pairs: Vec<(Vec<f64>, Vec<f64>)> = Vec::new();
    for chunk in series.chunks(2) {
        if chunk.len() == 2 {
            pairs.push((chunk[0].clone(), chunk[1].clone()));
        } else {
            pairs.push((chunk[0].clone(), Vec::new()));
        }
    }
    if params.name.is_none() {
        params.name = name_of(headers, 0);
    }
    let mut xs: Vec<f64> = Vec::new();
    let mut ys: Vec<f64> = Vec::new();
    for (x, y) in &pairs {
        xs.extend_from_slice(x);
        ys.extend_from_slice(y);
    }
    set_xlim_ylim(params, &xs, &ys)?;

    if pairs.is_empty() {
        return Err(Error::marker());
    }
    let color = color_option(params, matches!(method, Method::Line))?;
    let name = params.name.clone();
    let (x0, y0) = pairs.remove(0);
    let mut plot = match method {
        Method::Line => grid::lineplot(&x0, &y0, params, color, name),
        Method::Scatter => grid::scatterplot(&x0, &y0, params, color, name),
        Method::Density => grid::densityplot(&x0, &y0, params, color, name),
    }
    .map_err(Error::Argument)?;

    for (i, (xi, yi)) in pairs.iter().enumerate() {
        let name = name_of(headers, (i + 1) * 2);
        match method {
            Method::Line => grid::add_line(&mut plot, xi, yi, Color::named("auto"), name),
            Method::Scatter => grid::add_points(&mut plot, xi, yi, Color::named("auto"), name),
            Method::Density => grid::add_points(&mut plot, xi, yi, Color::named("auto"), name),
        }
        .map_err(Error::Argument)?;
    }
    Ok(Rendered::Plot(Box::new(plot)))
}

fn plot_fmt(data: &Data, fmt: &str, method: Method, params: &mut Params) -> Result<Rendered> {
    check_series_size(data, fmt)?;
    match fmt {
        "xyy" => plot_xyy(data, method, params),
        "xyxy" => plot_xyxy(data, method, params),
        "yx" => Err(Error::Argument("Incorrect format: yx".to_string())),
        _ => Err(Error::Argument(format!("Unknown format: {}", fmt))),
    }
}

fn line(data: &Data, params: &mut Params, fmt: &str) -> Result<Rendered> {
    let headers = &data.headers;
    let series = &data.series;
    if series.len() == 1 {
        if params.ylabel.is_none() {
            params.ylabel = name_of(headers, 0);
        }
        let mut y = Vec::with_capacity(series[0].len());
        for cell in &series[0] {
            y.push(cell_f(cell)?);
        }
        let x: Vec<f64> = (1..=y.len()).map(|i| i as f64).collect();
        let color = color_option(params, true)?;
        let name = params.name.clone();
        let plot = grid::lineplot(&x, &y, params, color, name).map_err(Error::Argument)?;
        return Ok(Rendered::Plot(Box::new(plot)));
    }

    let (x_col, y_col) = if fmt == "yx" { (1, 0) } else { (0, 1) };
    if x_col >= series.len() || y_col >= series.len() {
        return Err(Error::marker());
    }
    if let Some(h) = headers {
        if params.xlabel.is_none() {
            params.xlabel = h.get(x_col).cloned().flatten();
        }
        if params.ylabel.is_none() {
            params.ylabel = h.get(y_col).cloned().flatten();
        }
    }
    let mut x = Vec::with_capacity(series[x_col].len());
    for cell in &series[x_col] {
        x.push(cell_f(cell)?);
    }
    let mut y = Vec::with_capacity(series[y_col].len());
    for cell in &series[y_col] {
        y.push(cell_f(cell)?);
    }
    let color = color_option(params, true)?;
    let name = params.name.clone();
    let plot = grid::lineplot(&x, &y, params, color, name).map_err(Error::Argument)?;
    Ok(Rendered::Plot(Box::new(plot)))
}

fn inspect_headers(headers: &Option<Vec<Option<String>>>) -> String {
    match headers {
        None => "nil".to_string(),
        Some(h) => {
            let items: Vec<String> = h
                .iter()
                .map(|v| match v {
                    None => "nil".to_string(),
                    Some(s) => format!("\"{}\"", ruby_inspect_escape(s)),
                })
                .collect();
            format!("[{}]", items.join(", "))
        }
    }
}

fn ruby_inspect_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\x1b' => out.push_str("\\e"),
            _ => out.push(c),
        }
    }
    out
}

fn check_series_size(data: &Data, fmt: &str) -> Result<()> {
    let series = &data.series;
    if series.len() == 1 {
        eprintln!("YouPlot: There is only one series of input data. Please check the delimiter.");
        eprintln!();
        eprintln!(
            "Headers: \u{1b}[35m{}\u{1b}[0m",
            inspect_headers(&data.headers)
        );
        let first = series[0]
            .first()
            .and_then(|v| v.clone())
            .unwrap_or_default();
        let last = series[0]
            .last()
            .and_then(|v| v.clone())
            .unwrap_or_default();
        eprintln!(
            "The first item is: \u{1b}[35m\"{}\"\u{1b}[0m",
            first
        );
        eprintln!("The last item is : \u{1b}[35m\"{}\"\u{1b}[0m", last);
        return Err(Error::marker());
    }
    if fmt == "xyxy" && !series.len().is_multiple_of(2) {
        eprintln!("YouPlot: In the xyxy format, the number of series must be even.");
        eprintln!();
        eprintln!(
            "Number of series: \u{1b}[35m{}\u{1b}[0m",
            series.len()
        );
        eprintln!(
            "Headers: \u{1b}[35m{}\u{1b}[0m",
            inspect_headers(&data.headers)
        );
        return Err(Error::marker());
    }
    Ok(())
}

fn boxplot_impl(data: &Data, params: &mut Params) -> Result<Rendered> {
    if data.series.is_empty() {
        return Err(Error::marker());
    }
    let headers = match &data.headers {
        Some(h) => h.clone(),
        None => (1..=data.series.len()).map(|i| Some(i.to_string())).collect(),
    };
    let plot = boxplot::boxplot(headers, &data.series, params).map_err(Error::Argument)?;
    Ok(Rendered::Plot(Box::new(plot)))
}

const TEXT_COLORS: &[(&str, &str)] = &[
    ("black", "\u{1b}[30m"),
    ("red", "\u{1b}[31m"),
    ("green", "\u{1b}[32m"),
    ("yellow", "\u{1b}[33m"),
    ("blue", "\u{1b}[34m"),
    ("magenta", "\u{1b}[35m"),
    ("cyan", "\u{1b}[36m"),
    ("white", "\u{1b}[37m"),
    ("gray", "\u{1b}[90m"),
    ("light_black", "\u{1b}[90m"),
    ("light_red", "\u{1b}[91m"),
    ("light_green", "\u{1b}[92m"),
    ("light_yellow", "\u{1b}[93m"),
    ("light_blue", "\u{1b}[94m"),
    ("light_magenta", "\u{1b}[95m"),
    ("light_cyan", "\u{1b}[96m"),
    ("normal", "\u{1b}[0m"),
    ("default", "\u{1b}[39m"),
    ("bold", "\u{1b}[1m"),
    ("underline", "\u{1b}[4m"),
    ("blink", "\u{1b}[5m"),
    ("reverse", "\u{1b}[7m"),
    ("hidden", "\u{1b}[8m"),
    ("nothing", ""),
];

fn colors(names_only: bool) -> String {
    let mut s = String::new();
    for (k, v) in TEXT_COLORS {
        s.push_str(v);
        s.push_str(k);
        if !names_only {
            s.push_str("\t  ●");
        }
        s.push_str("\u{1b}[0m");
        s.push('\t');
    }
    for i in 0..=255u16 {
        s.push_str(&format!("\u{1b}[38;5;{}m", i));
        s.push_str(&i.to_string());
        if !names_only {
            s.push_str("\t  ●");
        }
        s.push_str("\u{1b}[0m");
        s.push('\t');
    }
    s.push('\n');
    s
}

pub fn render_to_string(rendered: &Rendered, color: bool) -> String {
    match rendered {
        Rendered::Plot(p) => crate::plot::render(p, color),
        Rendered::Text(s) => s.clone(),
    }
}

pub fn _unused(text: &str, color: &Color) -> String {
    let _ = text_color(color);
    format!("{}{}", text_color(color), text)
}

pub fn _round(v: f64) -> i64 {
    round_int(v)
}
