pub mod barplot;
pub mod boxplot;
pub mod canvas;
pub mod grid;
pub mod renderer;

pub use barplot::Barplot;
pub use boxplot::Boxplot;
pub use canvas::{Canvas, CanvasKind};
pub use grid::GridPlot;
pub use renderer::render;

use crate::ruby::{round_half_even_int, round_int, str_len};
use crate::Params;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Color {
    Name(String),
    Index(u8),
}

impl Color {
    pub fn named(n: &str) -> Self {
        Color::Name(n.to_string())
    }
    pub fn normal() -> Self {
        Color::Name("normal".to_string())
    }
    pub fn light_black() -> Self {
        Color::Name("light_black".to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Border {
    Solid,
    Corners,
    Barplot,
}

pub fn border_from_str(s: &str) -> Option<Border> {
    match s {
        "solid" => Some(Border::Solid),
        "corners" => Some(Border::Corners),
        "barplot" => Some(Border::Barplot),
        _ => None,
    }
}

pub struct BorderChars {
    pub tl: &'static str,
    pub tr: &'static str,
    pub bl: &'static str,
    pub br: &'static str,
    pub t: &'static str,
    pub l: &'static str,
    pub b: &'static str,
    pub r: &'static str,
}

pub fn border_chars(b: Border) -> BorderChars {
    match b {
        Border::Solid => BorderChars {
            tl: "┌",
            tr: "┐",
            bl: "└",
            br: "┘",
            t: "─",
            l: "│",
            b: "─",
            r: "│",
        },
        Border::Corners => BorderChars {
            tl: "┌",
            tr: "┐",
            bl: "└",
            br: "┘",
            t: " ",
            l: " ",
            b: " ",
            r: " ",
        },
        Border::Barplot => BorderChars {
            tl: "┌",
            tr: "┐",
            bl: "└",
            br: "┘",
            t: " ",
            l: "┤",
            b: " ",
            r: " ",
        },
    }
}

pub fn border_types() -> &'static [&'static str] {
    &["solid", "corners", "barplot"]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LookupKind {
    Ascii,
    Block,
    Dot,
}

pub fn texture_decode(kind: LookupKind, code: usize) -> char {
    match kind {
        LookupKind::Ascii => ascii_decode()[code.min(0x1ff)],
        LookupKind::Block => {
            const BLOCK_DECODE: [char; 16] = [
                ' ', '▗', '▖', '▄', '▝', '▐', '▞', '▟', '▘', '▚', '▌', '▙', '▀', '▜', '▛', '█',
            ];
            BLOCK_DECODE[code & 0xf]
        }
        LookupKind::Dot => {
            const DOT_DECODE: [char; 4] = [' ', '.', '\'', ':'];
            DOT_DECODE[code & 0x3]
        }
    }
}

fn ascii_decode() -> &'static Vec<char> {
    use std::sync::OnceLock;
    static DECODE: OnceLock<Vec<char>> = OnceLock::new();
    DECODE.get_or_init(|| {
        let pairs: &[(u32, char)] = &[
            (0b101_000_000, '"'),
            (0b111_111_111, '@'),
            (0b010_000_000, '\''),
            (0b010_100_010, '('),
            (0b010_001_010, ')'),
            (0b000_010_000, '*'),
            (0b010_111_010, '+'),
            (0b000_010_010, ','),
            (0b000_100_100, ','),
            (0b000_001_001, ','),
            (0b000_111_000, '-'),
            (0b000_000_010, '.'),
            (0b000_000_100, '.'),
            (0b000_000_001, '.'),
            (0b001_010_100, '/'),
            (0b010_100_000, '/'),
            (0b001_010_110, '/'),
            (0b011_010_010, '/'),
            (0b001_010_010, '/'),
            (0b110_010_111, '1'),
            (0b010_000_010, ':'),
            (0b111_000_111, '='),
            (0b111_010_111, 'I'),
            (0b100_100_111, 'L'),
            (0b111_010_010, 'T'),
            (0b101_101_010, 'V'),
            (0b101_010_101, 'X'),
            (0b101_010_010, 'Y'),
            (0b110_100_110, '['),
            (0b010_001_000, '\\'),
            (0b100_010_001, '\\'),
            (0b110_010_010, '\\'),
            (0b100_010_011, '\\'),
            (0b100_010_010, '\\'),
            (0b011_001_011, ']'),
            (0b010_101_000, '^'),
            (0b000_000_111, '_'),
            (0b100_000_000, '`'),
            (0b110_010_011, 'l'),
            (0b000_111_100, 'r'),
            (0b000_101_010, 'v'),
            (0b011_110_011, '{'),
            (0b010_010_010, '|'),
            (0b100_100_100, '|'),
            (0b001_001_001, '|'),
            (0b110_011_110, '}'),
        ];
        let order: [u32; 46] = [
            0x0002, 0x00d2, 0x0113, 0x00a0, 0x0088, 0x002a, 0x0100, 0x0197, 0x0012, 0x0193,
            0x0092, 0x0082, 0x008a, 0x0054, 0x0004, 0x01d2, 0x01ff, 0x0124, 0x00a8, 0x0056,
            0x0001, 0x01c7, 0x0052, 0x0080, 0x0009, 0x00cb, 0x0007, 0x003c, 0x0111, 0x0140,
            0x0024, 0x0127, 0x0192, 0x0010, 0x019e, 0x01a6, 0x01d7, 0x0155, 0x00a2, 0x00ba,
            0x0112, 0x0049, 0x00f3, 0x0152, 0x0038, 0x016a,
        ];
        let mut map = std::collections::HashMap::new();
        for &(k, v) in pairs {
            map.insert(k, v);
        }
        let mut out = Vec::with_capacity(512);
        out.push(' ');
        for i in 1u32..=0b111_111_111 {
            let mut best = order[0];
            let mut bestd = (i ^ best).count_ones();
            for &k in &order[1..] {
                let d = (i ^ k).count_ones();
                if d < bestd {
                    best = k;
                    bestd = d;
                }
            }
            out.push(*map.get(&best).unwrap_or(&' '));
        }
        out
    })
}

pub enum PlotKind {
    Bar(Barplot),
    Grid(GridPlot),
    Box(Boxplot),
}

pub struct Plot {
    pub title: Option<String>,
    pub xlabel: Option<String>,
    pub ylabel: Option<String>,
    pub border: Border,
    pub margin: i64,
    pub padding: i64,
    pub labels_left: BTreeMap<usize, Option<String>>,
    pub colors_left: BTreeMap<usize, Color>,
    pub labels_right: BTreeMap<usize, Option<String>>,
    pub colors_right: BTreeMap<usize, Color>,
    pub decorations: BTreeMap<&'static str, Option<String>>,
    pub colors_deco: BTreeMap<&'static str, Color>,
    pub show_labels: bool,
    pub auto_color: usize,
    pub kind: PlotKind,
}

pub const COLOR_CYCLE: [&str; 6] = ["green", "blue", "red", "magenta", "yellow", "cyan"];

impl Plot {
    pub fn new(border: Border, margin: i64, padding: i64, labels: bool, kind: PlotKind) -> Self {
        Plot {
            title: None,
            xlabel: None,
            ylabel: None,
            border,
            margin,
            padding,
            labels_left: BTreeMap::new(),
            colors_left: BTreeMap::new(),
            labels_right: BTreeMap::new(),
            colors_right: BTreeMap::new(),
            decorations: BTreeMap::new(),
            colors_deco: BTreeMap::new(),
            show_labels: labels,
            auto_color: 0,
            kind,
        }
    }

    pub fn title_given(&self) -> bool {
        self.title.as_deref().map_or(false, |s| !s.is_empty())
    }
    pub fn xlabel_given(&self) -> bool {
        self.xlabel.as_deref().map_or(false, |s| !s.is_empty())
    }
    pub fn ylabel_given(&self) -> bool {
        self.ylabel.as_deref().map_or(false, |s| !s.is_empty())
    }
    pub fn ylabel_length(&self) -> usize {
        self.ylabel.as_deref().map(str_len).unwrap_or(0)
    }
    pub fn show_labels(&self) -> bool {
        self.show_labels
    }
    pub fn n_rows(&self) -> i64 {
        match &self.kind {
            PlotKind::Bar(b) => b.bars.len() as i64,
            PlotKind::Grid(g) => g.canvas.height,
            PlotKind::Box(b) => 3 * b.data.len() as i64,
        }
    }
    pub fn n_columns(&self) -> i64 {
        match &self.kind {
            PlotKind::Bar(b) => b.width,
            PlotKind::Grid(g) => g.canvas.width,
            PlotKind::Box(b) => b.width,
        }
    }

    pub fn annotate(&mut self, loc: &str, value: Option<String>, color: Color) -> Result<(), String> {
        match loc {
            "l" => {
                for row in 0..self.n_rows() {
                    let empty = self
                        .labels_left
                        .get(&(row as usize))
                        .map_or(true, |v| v.as_deref().unwrap_or("").is_empty());
                    if empty {
                        self.labels_left.insert(row as usize, value);
                        self.colors_left.insert(row as usize, color);
                        break;
                    }
                }
            }
            "r" => {
                for row in 0..self.n_rows() {
                    let empty = self
                        .labels_right
                        .get(&(row as usize))
                        .map_or(true, |v| v.as_deref().unwrap_or("").is_empty());
                    if empty {
                        self.labels_right.insert(row as usize, value);
                        self.colors_right.insert(row as usize, color);
                        break;
                    }
                }
            }
            "t" | "b" | "tl" | "tr" | "bl" | "br" => {
                let key: &'static str = match loc {
                    "t" => "t",
                    "b" => "b",
                    "tl" => "tl",
                    "tr" => "tr",
                    "bl" => "bl",
                    "br" => "br",
                    _ => unreachable!(),
                };
                self.decorations.insert(key, value);
                self.colors_deco.insert(key, color);
            }
            _ => return Err(format!("unknown location to annotate ({} for :t, :b, :l, :r, :tl, :tr, :bl, or :br)", loc)),
        }
        Ok(())
    }

    pub fn annotate_row(&mut self, loc: &str, row: i64, value: Option<String>, color: Color) {
        match loc {
            "l" => {
                self.labels_left.insert(row as usize, value);
                self.colors_left.insert(row as usize, color);
            }
            "r" => {
                self.labels_right.insert(row as usize, value);
                self.colors_right.insert(row as usize, color);
            }
            _ => {}
        }
    }

    pub fn next_color(&mut self) -> Color {
        let c = Color::named(COLOR_CYCLE[self.auto_color]);
        self.auto_color = (self.auto_color + 1) % COLOR_CYCLE.len();
        c
    }

    pub fn print_row(&self, row: i64, emit: &mut dyn FnMut(&Color, &str)) {
        match &self.kind {
            PlotKind::Bar(b) => b.print_row(row, emit),
            PlotKind::Grid(g) => g.canvas.print_row(row, |c, s| emit(c, s)),
            PlotKind::Box(b) => b.print_row(row, emit),
        }
    }
}

// ---------------------------------------------------------------------------
// parameter helpers
// ---------------------------------------------------------------------------

pub fn default_width(params: &Params) -> i64 {
    params.width.map(|v| v as i64).unwrap_or(40)
}
pub fn default_height(params: &Params) -> i64 {
    params.height.map(|v| v as i64).unwrap_or(15)
}
pub fn default_margin(params: &Params) -> Result<i64, String> {
    let m = params.margin.map(|v| v as i64).unwrap_or(3);
    if m < 0 {
        return Err("margin must be >= 0".to_string());
    }
    Ok(m)
}
pub fn default_padding(params: &Params) -> i64 {
    params.padding.map(|v| v as i64).unwrap_or(1)
}
pub fn default_labels(params: &Params) -> bool {
    params.labels.unwrap_or(true)
}
pub fn default_border(params: &Params, def: Border) -> Result<Border, String> {
    match &params.border {
        None => Ok(def),
        Some(s) => border_from_str(s).ok_or_else(|| format!("unknown border type: {}", s)),
    }
}

pub fn param_color(params: &Params, def: &str) -> Color {
    match &params.color {
        None => Color::named(def),
        Some(crate::ParamColor::Name(s)) => Color::Name(s.clone()),
        Some(crate::ParamColor::Number(n)) => {
            // Numeric colors are invalid for styles but Ruby passes them on;
            // treat as a text color index.
            if *n >= 0 && *n <= 255 {
                Color::Index(*n as u8)
            } else {
                Color::named(def)
            }
        }
    }
}

pub fn apply_common(plot: &mut Plot, params: &Params) {
    plot.title = params.title.clone();
    plot.xlabel = params.xlabel.clone();
    plot.ylabel = params.ylabel.clone();
}

pub fn roundable_num(v: f64) -> crate::ruby::Num {
    if crate::ruby::roundable(v) {
        crate::ruby::Num::Int(round_int(v))
    } else {
        crate::ruby::Num::Float(v)
    }
}

pub fn even_round_int(v: f64) -> i64 {
    round_half_even_int(v)
}
