use crate::plot::{texture_decode, Color, LookupKind};
use crate::ruby::{round_int, str_len};

#[derive(Clone)]
pub struct Canvas {
    pub width: i64,
    pub height: i64,
    pub pixel_width: f64,
    pub pixel_height: f64,
    pub origin_x: f64,
    pub origin_y: f64,
    pub plot_width: f64,
    pub plot_height: f64,
    pub x_pixel_per_char: i64,
    pub y_pixel_per_char: i64,
    pub kind: CanvasKind,
    pub grid: Vec<u32>,
    pub colors: Vec<u8>,
    pub max_density: u32,
}

#[derive(Clone)]
pub enum CanvasKind {
    Braille,
    Lookup(LookupKind),
    Density,
}

pub fn create(
    canvas_type: &str,
    width: i64,
    height: i64,
    origin_x: f64,
    origin_y: f64,
    plot_width: f64,
    plot_height: f64,
) -> Result<Canvas, String> {
    let (kind, xp, yp, fill) = match canvas_type {
        "ascii" => (CanvasKind::Lookup(LookupKind::Ascii), 3, 3, 0u32),
        "block" => (CanvasKind::Lookup(LookupKind::Block), 2, 2, 0u32),
        "braille" => (CanvasKind::Braille, 2, 4, 0x2800u32),
        "density" => (CanvasKind::Density, 1, 2, 0u32),
        "dot" => (CanvasKind::Lookup(LookupKind::Dot), 1, 2, 0u32),
        _ => return Err(format!("unknown canvas type: {}", canvas_type)),
    };
    let (width, height) = if canvas_type == "density" {
        (width.max(5), height.max(5))
    } else {
        (width, height)
    };
    let pixel_width = match kind {
        CanvasKind::Braille => width as f64 * xp as f64,
        CanvasKind::Lookup(_) | CanvasKind::Density => width as f64 * xp as f64,
    };
    let pixel_height = height as f64 * yp as f64;
    let n = (width * height) as usize;
    Ok(Canvas {
        width,
        height,
        pixel_width,
        pixel_height,
        origin_x,
        origin_y,
        plot_width,
        plot_height,
        x_pixel_per_char: xp,
        y_pixel_per_char: yp,
        kind,
        grid: vec![fill; n],
        colors: vec![0u8; n],
        max_density: 1,
    })
}

impl Canvas {
    fn index_at(&self, x: i64, y: i64) -> Option<usize> {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return None;
        }
        Some((y * self.width + x) as usize)
    }

    pub fn point(&mut self, x: f64, y: f64, color: &Color) {
        if !(self.origin_x <= x
            && x <= self.origin_x + self.plot_width
            && self.origin_y <= y
            && y <= self.origin_y + self.plot_height)
        {
            return;
        }
        let plot_offset_x = x - self.origin_x;
        let pixel_x = plot_offset_x / self.plot_width * self.pixel_width;
        let plot_offset_y = y - self.origin_y;
        let pixel_y = self.pixel_height - plot_offset_y / self.plot_height * self.pixel_height;
        self.pixel(pixel_x.floor() as i64, pixel_y.floor() as i64, color);
    }

    pub fn points(&mut self, x: &[f64], y: &[f64], color: &Color) -> Result<(), String> {
        if x.len() != y.len() {
            return Err("x and y must be the same length".to_string());
        }
        if x.is_empty() {
            return Err("x and y must not be empty".to_string());
        }
        for i in 0..x.len() {
            self.point(x[i], y[i], color);
        }
        Ok(())
    }

    pub fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, color: &Color) {
        if (x1 < self.origin_x && x2 < self.origin_x)
            || (x1 > self.origin_x + self.plot_width && x2 > self.origin_x + self.plot_width)
        {
            return;
        }
        if (y1 < self.origin_y && y2 < self.origin_y)
            || (y1 > self.origin_y + self.plot_height && y2 > self.origin_y + self.plot_height)
        {
            return;
        }

        let toff = x1 - self.origin_x;
        let px1 = toff / self.plot_width * self.pixel_width;
        let toff = x2 - self.origin_x;
        let px2 = toff / self.plot_width * self.pixel_width;

        let toff = y1 - self.origin_y;
        let py1 = self.pixel_height - toff / self.plot_height * self.pixel_height;
        let toff = y2 - self.origin_y;
        let py2 = self.pixel_height - toff / self.plot_height * self.pixel_height;

        let dx = px2 - px1;
        let dy = py2 - py1;
        let nsteps = if dx.abs() > dy.abs() { dx.abs() } else { dy.abs() };
        let inc_x = dx / nsteps;
        let inc_y = dy / nsteps;

        let mut cur_x = px1;
        let mut cur_y = py1;
        self.pixel(cur_x.floor() as i64, cur_y.floor() as i64, color);
        let n = nsteps as i64;
        for _ in 1..=n {
            cur_x += inc_x;
            cur_y += inc_y;
            self.pixel(cur_x.floor() as i64, cur_y.floor() as i64, color);
        }
    }

    pub fn lines(&mut self, x: &[f64], y: &[f64], color: &Color) -> Result<(), String> {
        if x.len() != y.len() {
            return Err("x and y must be the same length".to_string());
        }
        if x.is_empty() {
            return Err("x and y must not be empty".to_string());
        }
        for i in 0..x.len() - 1 {
            self.line(x[i], y[i], x[i + 1], y[i + 1], color);
        }
        Ok(())
    }

    fn pixel(&mut self, pixel_x: i64, pixel_y: i64, color: &Color) {
        if pixel_x < 0
            || (pixel_x as f64) > self.pixel_width
            || pixel_y < 0
            || (pixel_y as f64) > self.pixel_height
        {
            return;
        }
        let mut pixel_x = pixel_x;
        let mut pixel_y = pixel_y;
        if (pixel_x as f64) >= self.pixel_width {
            pixel_x -= 1;
        }
        if (pixel_y as f64) >= self.pixel_height {
            pixel_y -= 1;
        }

        match self.kind {
            CanvasKind::Braille => {
                let tx = pixel_x as f64 / self.pixel_width * self.width as f64;
                let mut char_x = tx.floor() as i64 + 1;
                let char_x_off = pixel_x % 2 + 1;
                if char_x < round_int(tx) + 1 && char_x_off == 1 {
                    char_x += 1;
                }
                let char_y_off = pixel_y % 4 + 1;
                let char_y = (pixel_y - (char_y_off - 1)) / 4 + 1;
                if let Some(index) = self.index_at(char_x - 1, char_y - 1) {
                    let sign = BRAILLE_SIGNS[(char_x_off - 1) as usize][(char_y_off - 1) as usize];
                    self.grid[index] |= sign;
                    self.colors[index] |= color_encode(color);
                }
            }
            CanvasKind::Lookup(kind) => {
                let tx = pixel_x as f64 / self.pixel_width * self.width as f64;
                let mut char_x = tx.floor() as i64 + 1;
                let char_x_off = pixel_x % self.x_pixel_per_char + 1;
                if char_x < round_int(tx) + 1 && char_x_off == 1 {
                    char_x += 1;
                }
                let char_y_off = pixel_y % self.y_pixel_per_char + 1;
                let char_y = (pixel_y - (char_y_off - 1)) / self.y_pixel_per_char + 1;
                if let Some(index) = self.index_at(char_x - 1, char_y - 1) {
                    self.grid[index] |= lookup_encode(kind, (char_x_off - 1) as usize, (char_y_off - 1) as usize);
                    self.colors[index] |= color_encode(color);
                }
            }
            CanvasKind::Density => {
                let char_x = (pixel_x as f64 / self.pixel_width * self.width as f64).floor() as i64;
                let char_y =
                    (pixel_y as f64 / self.pixel_height * self.height as f64).floor() as i64;
                if let Some(index) = self.index_at(char_x, char_y) {
                    self.grid[index] += 1;
                    self.max_density = self.max_density.max(self.grid[index]);
                    self.colors[index] |= color_encode(color);
                }
            }
        }
    }

    pub fn char_at(&self, x: i64, y: i64) -> char {
        match self.index_at(x, y) {
            Some(i) => char::from_u32(self.grid[i]).unwrap_or(' '),
            None => ' ',
        }
    }

    pub fn color_at(&self, x: i64, y: i64) -> Color {
        match self.index_at(x, y) {
            Some(i) => color_decode(self.colors[i]),
            None => Color::Name("normal".to_string()),
        }
    }

    pub fn print_row<F>(&self, row_index: i64, mut emit: F)
    where
        F: FnMut(&Color, &str),
    {
        let y = row_index;
        match self.kind {
            CanvasKind::Braille => {
                for x in 0..self.width {
                    let ch = self.char_at(x, y);
                    let c = self.color_at(x, y);
                    let mut b = [0u8; 4];
                    emit(&c, ch.encode_utf8(&mut b));
                }
            }
            CanvasKind::Lookup(kind) => {
                for x in 0..self.width {
                    let code = self.grid[self.index_at(x, y).unwrap()] as usize;
                    let ch = texture_decode(kind, code);
                    let c = self.color_at(x, y);
                    let mut b = [0u8; 4];
                    emit(&c, ch.encode_utf8(&mut b));
                }
            }
            CanvasKind::Density => {
                let signs = [" ", "░", "▒", "▓", "█"];
                let val_scale = 4.0 / self.max_density as f64;
                for x in 0..self.width {
                    let count = self.grid[self.index_at(x, y).unwrap()];
                    let idx = round_int(count as f64 * val_scale).clamp(0, 4) as usize;
                    let c = self.color_at(x, y);
                    emit(&c, signs[idx]);
                }
            }
        }
    }
}

const BRAILLE_SIGNS: [[u32; 4]; 2] = [
    [0x2801, 0x2802, 0x2804, 0x2840],
    [0x2808, 0x2810, 0x2820, 0x2880],
];

fn lookup_encode(kind: LookupKind, x: usize, y: usize) -> u32 {
    match kind {
        LookupKind::Ascii => ASCII_SIGNS[x][y],
        LookupKind::Block => BLOCK_SIGNS[x][y],
        LookupKind::Dot => DOT_SIGNS[x][y],
    }
}

const ASCII_SIGNS: [[u32; 3]; 3] = [
    [0b100_000_000, 0b000_100_000, 0b000_000_100],
    [0b010_000_000, 0b000_010_000, 0b000_000_010],
    [0b001_000_000, 0b000_001_000, 0b000_000_001],
];
const BLOCK_SIGNS: [[u32; 2]; 2] = [[0b1000, 0b0010], [0b0100, 0b0001]];
const DOT_SIGNS: [[u32; 2]; 1] = [[0b10, 0b01]];

pub fn color_encode(color: &Color) -> u8 {
    match color {
        Color::Name(n) => match n.as_str() {
            "normal" => 0b000,
            "blue" => 0b001,
            "red" => 0b010,
            "magenta" => 0b011,
            "green" => 0b100,
            "cyan" => 0b101,
            "yellow" => 0b110,
            "white" => 0b111,
            _ => 0,
        },
        Color::Index(_) => 0,
    }
}

fn color_decode(v: u8) -> Color {
    let name = match v & 0b111 {
        0b000 => "normal",
        0b001 => "blue",
        0b010 => "red",
        0b011 => "magenta",
        0b100 => "green",
        0b101 => "cyan",
        0b110 => "yellow",
        0b111 => "white",
        _ => "normal",
    };
    Color::Name(name.to_string())
}

pub fn canvas_types() -> &'static [&'static str] {
    &["ascii", "block", "braille", "density", "dot"]
}

pub fn str_width(s: &str) -> usize {
    str_len(s)
}
