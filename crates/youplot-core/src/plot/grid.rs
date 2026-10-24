use super::canvas::Canvas;
use super::{Color, Plot, PlotKind};
use crate::ruby::float_step_values;
use crate::stats::{round_down_subtick, round_up_subtick};
use crate::Params;

pub fn plotting_range_narrow(xmin: f64, xmax: f64) -> (f64, f64) {
    let diff = xmax - xmin;
    let xmax = round_up_subtick(xmax, diff);
    let xmin = round_down_subtick(xmin, diff);
    (xmin, xmax)
}

pub fn extend_limits(values: &[f64], limits: [f64; 2]) -> Result<(f64, f64), String> {
    let mut mi = limits[0].min(limits[1]);
    let mut ma = limits[0].max(limits[1]);
    if mi == 0.0 && ma == 0.0 {
        if values.is_empty() {
            return Err("undefined method 'to_f' for nil".to_string());
        }
        let mut vmin = values[0];
        let mut vmax = values[0];
        for &v in values {
            if v < vmin {
                vmin = v;
            }
            if v > vmax {
                vmax = v;
            }
        }
        mi = vmin;
        ma = vmax;
    }
    let diff = ma - mi;
    if diff == 0.0 {
        ma = mi + 1.0;
        mi -= 1.0;
    }
    if limits == [0.0, 0.0] {
        Ok(plotting_range_narrow(mi, ma))
    } else {
        Ok((mi, ma))
    }
}

pub struct GridPlot {
    pub canvas: Canvas,
}

pub fn new_grid_plot(
    x: &[f64],
    y: &[f64],
    canvas_type: &str,
    grid_default: bool,
    params: &Params,
) -> Result<Plot, String> {
    if x.len() != y.len() {
        return Err("x and y must be the same length".to_string());
    }
    if x.is_empty() {
        return Err("x and y must not be empty".to_string());
    }
    let xlim = params.xlim.clone().unwrap_or_else(|| vec![0.0, 0.0]);
    let ylim = params.ylim.clone().unwrap_or_else(|| vec![0.0, 0.0]);
    if xlim.len() != 2 || ylim.len() != 2 {
        return Err("xlim and ylim must be 2-length arrays".to_string());
    }
    let width = super::default_width(params).max(5);
    let height = super::default_height(params).max(2);
    let (min_x, max_x) = extend_limits(x, [xlim[0], xlim[1]])?;
    let (min_y, max_y) = extend_limits(y, [ylim[0], ylim[1]])?;
    let origin_x = min_x;
    let origin_y = min_y;
    let plot_width = max_x - origin_x;
    let plot_height = max_y - origin_y;
    let mut canvas = super::canvas::create(
        canvas_type, width, height, origin_x, origin_y, plot_width, plot_height,
    )?;

    let border = super::default_border(params, super::Border::Solid)?;
    let margin = super::default_margin(params)?;
    let padding = super::default_padding(params);
    let labels = super::default_labels(params);
    let mut plot = Plot::new(
        border,
        margin,
        padding,
        labels,
        PlotKind::Grid(GridPlot { canvas: canvas.clone() }),
    );
    super::apply_common(&mut plot, params);

    plot.annotate_row(
        "l",
        0,
        Some(super::roundable_num(max_y).to_s()),
        Color::light_black(),
    );
    plot.annotate_row(
        "l",
        height - 1,
        Some(super::roundable_num(min_y).to_s()),
        Color::light_black(),
    );
    plot.annotate(
        "bl",
        Some(super::roundable_num(min_x).to_s()),
        Color::light_black(),
    )?;
    plot.annotate(
        "br",
        Some(super::roundable_num(max_x).to_s()),
        Color::light_black(),
    )?;

    let grid = params.grid.unwrap_or(grid_default);
    if grid {
        if min_y < 0.0 && 0.0 < max_y {
            let step = plot_width / (canvas.width as f64 * canvas.x_pixel_per_char as f64 - 1.0);
            for i in float_step_values(min_x, max_x, step) {
                canvas.point(i, 0.0, &Color::normal());
            }
        }
        if min_x < 0.0 && 0.0 < max_x {
            let step = plot_height / (canvas.height as f64 * canvas.y_pixel_per_char as f64 - 1.0);
            for i in float_step_values(min_y, max_y, step) {
                canvas.point(0.0, i, &Color::normal());
            }
        }
    }

    // Put the canvas with grid lines back into the plot.
    if let PlotKind::Grid(g) = &mut plot.kind {
        g.canvas = canvas;
    }
    Ok(plot)
}

pub fn grid_canvas(plot: &Plot) -> &Canvas {
    match &plot.kind {
        PlotKind::Grid(g) => &g.canvas,
        _ => unreachable!(),
    }
}

pub fn grid_canvas_mut(plot: &mut Plot) -> &mut Canvas {
    match &mut plot.kind {
        PlotKind::Grid(g) => &mut g.canvas,
        _ => unreachable!(),
    }
}

pub fn add_line(
    plot: &mut Plot,
    x: &[f64],
    y: &[f64],
    color: Color,
    name: Option<String>,
) -> Result<(), String> {
    let (mut x, mut y) = (x.to_vec(), y.to_vec());
    if x.len() == 1 && y.len() == 1 {
        let intercept = x[0];
        let slope = y[0];
        let canvas = grid_canvas(plot);
        let xmin = canvas.origin_x;
        let xmax = canvas.origin_x + canvas.plot_width;
        let ymin = canvas.origin_y;
        let ymax = canvas.origin_y + canvas.plot_height;
        let _ = (ymin, ymax);
        x = vec![xmin, xmax];
        y = vec![intercept + xmin * slope, intercept + xmax * slope];
    }
    let color = if matches!(&color, Color::Name(n) if n == "auto") {
        plot.next_color()
    } else {
        color
    };
    if let Some(name) = &name {
        if !name.is_empty() {
            plot.annotate("r", Some(name.clone()), color.clone())?;
        }
    }
    grid_canvas_mut(plot).lines(&x, &y, &color)
}

pub fn add_points(
    plot: &mut Plot,
    x: &[f64],
    y: &[f64],
    color: Color,
    name: Option<String>,
) -> Result<(), String> {
    let color = if matches!(&color, Color::Name(n) if n == "auto") {
        plot.next_color()
    } else {
        color
    };
    if let Some(name) = &name {
        if !name.is_empty() {
            plot.annotate("r", Some(name.clone()), color.clone())?;
        }
    }
    grid_canvas_mut(plot).points(x, y, &color)
}

pub fn lineplot(
    x: &[f64],
    y: &[f64],
    params: &Params,
    color: Color,
    name: Option<String>,
) -> Result<Plot, String> {
    let canvas = params.canvas.as_deref().unwrap_or("braille");
    let mut plot = new_grid_plot(x, y, canvas, true, params)?;
    add_line(&mut plot, x, y, color, name)?;
    Ok(plot)
}

pub fn scatterplot(
    x: &[f64],
    y: &[f64],
    params: &Params,
    color: Color,
    name: Option<String>,
) -> Result<Plot, String> {
    let canvas = params.canvas.as_deref().unwrap_or("braille");
    let mut plot = new_grid_plot(x, y, canvas, true, params)?;
    add_points(&mut plot, x, y, color, name)?;
    Ok(plot)
}

pub fn densityplot(
    x: &[f64],
    y: &[f64],
    params: &Params,
    color: Color,
    name: Option<String>,
) -> Result<Plot, String> {
    let mut plot = new_grid_plot(x, y, "density", false, params)?;
    add_points(&mut plot, x, y, color, name)?;
    Ok(plot)
}
