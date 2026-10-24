use super::{Color, Plot, PlotKind};
use crate::ruby::round_half_even_int;
use crate::stats::percentiles;
use crate::Params;

pub struct Boxplot {
    pub data: Vec<[f64; 5]>,
    pub width: i64,
    pub color: Color,
    pub min_x: f64,
    pub max_x: f64,
}

impl Boxplot {
    pub fn print_row_out(&self, out: &mut crate::Out, row_index: i64) {
        let line = self.line_for_row(row_index);
        super::renderer::print_styled(out, &[&line], false, &self.color);
    }

    fn line_for_row(&self, row_index: i64) -> String {
        let series = &self.data[(row_index / 3) as usize];
        let series_row = (row_index % 3) as usize;
        let min_char = ["╷", "├", "╵"][series_row];
        let line_char = [" ", "─", " "][series_row];
        let left_box_char = ["┌", "┤", "└"][series_row];
        let line_box_char = ["─", " ", "─"][series_row];
        let median_char = ["┬", "│", "┴"][series_row];
        let right_box_char = ["┐", "├", "┘"][series_row];
        let max_char = ["╷", "┤", "╵"][series_row];
        let transformed = self.transform(series);
        let mut line: Vec<&str> = vec![" "; self.width as usize];
        line[(transformed[0] - 1) as usize] = min_char;
        line[(transformed[1] - 1) as usize] = left_box_char;
        line[(transformed[2] - 1) as usize] = median_char;
        line[(transformed[3] - 1) as usize] = right_box_char;
        line[(transformed[4] - 1) as usize] = max_char;
        for i in transformed[0]..(transformed[1] - 1) { line[i.max(0) as usize] = line_char; }
        for i in transformed[1]..(transformed[2] - 1) { line[i.max(0) as usize] = line_box_char; }
        for i in transformed[2]..(transformed[3] - 1) { line[i.max(0) as usize] = line_box_char; }
        for i in transformed[3]..(transformed[4] - 1) { line[i.max(0) as usize] = line_char; }
        line.join("")
    }

    pub fn print_row(&self, row_index: i64, emit: &mut dyn FnMut(&Color, &str)) {
        let series = &self.data[(row_index / 3) as usize];
        let series_row = (row_index % 3) as usize;

        let min_char = ["╷", "├", "╵"][series_row];
        let line_char = [" ", "─", " "][series_row];
        let left_box_char = ["┌", "┤", "└"][series_row];
        let line_box_char = ["─", " ", "─"][series_row];
        let median_char = ["┬", "│", "┴"][series_row];
        let right_box_char = ["┐", "├", "┘"][series_row];
        let max_char = ["╷", "┤", "╵"][series_row];

        let transformed = self.transform(series);
        let mut line: Vec<&str> = vec![" "; self.width as usize];
        line[(transformed[0] - 1) as usize] = min_char;
        line[(transformed[1] - 1) as usize] = left_box_char;
        line[(transformed[2] - 1) as usize] = median_char;
        line[(transformed[3] - 1) as usize] = right_box_char;
        line[(transformed[4] - 1) as usize] = max_char;

        for i in transformed[0]..(transformed[1] - 1) {
            line[i.max(0) as usize] = line_char;
        }
        for i in transformed[1]..(transformed[2] - 1) {
            line[i.max(0) as usize] = line_box_char;
        }
        for i in transformed[2]..(transformed[3] - 1) {
            line[i.max(0) as usize] = line_box_char;
        }
        for i in transformed[3]..(transformed[4] - 1) {
            line[i.max(0) as usize] = line_char;
        }

        emit(&self.color, &line.join(""));
    }

    fn transform(&self, values: &[f64; 5]) -> [i64; 5] {
        let mut out = [0i64; 5];
        for (i, &val) in values.iter().enumerate() {
            let v = (val - self.min_x) / (self.max_x - self.min_x) * self.width as f64;
            out[i] = round_half_even_int(v).clamp(1, self.width);
        }
        out
    }
}

#[allow(clippy::too_many_arguments)]
pub fn boxplot(
    headers: Vec<Option<String>>,
    series: &[Vec<Option<String>>],
    params: &Params,
) -> Result<Plot, String> {
    let mut data_values: Vec<Vec<f64>> = Vec::new();
    for s in series {
        let mut v = Vec::with_capacity(s.len());
        for cell in s {
            // Ruby's NilClass#to_f is 0.0.
            v.push(match cell {
                Some(text) => crate::ruby::to_f(text),
                None => 0.0,
            });
        }
        data_values.push(v);
    }
    if headers.len() != data_values.len() {
        return Err("wrong number of text".to_string());
    }

    let xlim = params.xlim.clone().unwrap_or_else(|| vec![0.0, 0.0]);
    if xlim.len() != 2 {
        return Err("xlim must be a length 2 array".to_string());
    }

    let mut flat: Vec<f64> = Vec::new();
    for s in &data_values {
        if s.is_empty() {
            return Err("unable to compute percentile(s) for an empty array".to_string());
        }
        let mut mi = s[0];
        let mut ma = s[0];
        for &v in s {
            if v < mi {
                mi = v;
            }
            if v > ma {
                ma = v;
            }
        }
        flat.push(mi);
        flat.push(ma);
    }
    let (min_x, max_x) = super::grid::extend_limits(&flat, [xlim[0], xlim[1]])?;

    let width = super::default_width(params).max(10);
    let border = super::default_border(params, super::Border::Corners)?;
    let margin = super::default_margin(params)?;
    let padding = super::default_padding(params);
    let labels = super::default_labels(params);
    let color = super::param_color(params, "green");

    let data: Vec<[f64; 5]> = data_values
        .iter()
        .map(|s| {
            let p = percentiles(s, &[0.0, 25.0, 50.0, 75.0, 100.0]);
            [p[0], p[1], p[2], p[3], p[4]]
        })
        .collect();

    let mut plot = Plot::new(
        border,
        margin,
        padding,
        labels,
        PlotKind::Box(Boxplot {
            data,
            width,
            color,
            min_x,
            max_x,
        }),
    );
    super::apply_common(&mut plot, params);

    let mean_x = (min_x + max_x) / 2.0;
    plot.annotate("bl", Some(super::roundable_num(min_x).to_s()), Color::light_black())?;
    plot.annotate("b", Some(super::roundable_num(mean_x).to_s()), Color::light_black())?;
    plot.annotate("br", Some(super::roundable_num(max_x).to_s()), Color::light_black())?;

    for (i, name) in headers.into_iter().enumerate() {
        if let Some(name) = name {
            if !name.is_empty() {
                plot.annotate_row("l", i as i64 * 3 + 1, Some(name), Color::normal());
            }
        }
    }

    Ok(plot)
}
