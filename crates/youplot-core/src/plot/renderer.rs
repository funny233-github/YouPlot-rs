use super::{border_chars, Color, Plot};
use crate::ruby::{round_int, str_len};
use crate::Out;

pub fn text_color(color: &Color) -> String {
    match color {
        Color::Index(i) => format!("\u{1b}[38;5;{}m", i),
        Color::Name(name) => {
            let code = match name.as_str() {
                "black" => "\u{1b}[30m",
                "red" => "\u{1b}[31m",
                "green" => "\u{1b}[32m",
                "yellow" => "\u{1b}[33m",
                "blue" => "\u{1b}[34m",
                "magenta" => "\u{1b}[35m",
                "cyan" => "\u{1b}[36m",
                "white" => "\u{1b}[37m",
                "gray" => "\u{1b}[90m",
                "light_black" => "\u{1b}[90m",
                "light_red" => "\u{1b}[91m",
                "light_green" => "\u{1b}[92m",
                "light_yellow" => "\u{1b}[93m",
                "light_blue" => "\u{1b}[94m",
                "light_magenta" => "\u{1b}[95m",
                "light_cyan" => "\u{1b}[96m",
                "normal" => "\u{1b}[0m",
                "default" => "\u{1b}[39m",
                "bold" => "\u{1b}[1m",
                "underline" => "\u{1b}[4m",
                "blink" => "\u{1b}[5m",
                "reverse" => "\u{1b}[7m",
                "hidden" => "\u{1b}[8m",
                "nothing" => "",
                _ => "\u{1b}[39m",
            };
            code.to_string()
        }
    }
}

fn disable_text_style(color: &Color) -> String {
    match color {
        Color::Index(_) => "\u{1b}[39m".to_string(),
        Color::Name(name) => match name.as_str() {
            "bold" => "\u{1b}[22m".to_string(),
            "underline" => "\u{1b}[24m".to_string(),
            "blink" => "\u{1b}[25m".to_string(),
            "reverse" => "\u{1b}[27m".to_string(),
            "hidden" => "\u{1b}[28m".to_string(),
            "normal" | "default" | "nothing" => String::new(),
            _ => "\u{1b}[39m".to_string(),
        },
    }
}

pub fn print_styled(out: &mut Out, args: &[&str], bold: bool, color: &Color) {
    if !out.color {
        for a in args {
            out.print(a);
        }
        return;
    }
    let mut color = color.clone();
    if bold && color == Color::named("bold") {
        color = Color::named("nothing");
    }
    let mut enable = text_color(&color);
    if bold {
        enable.push_str("\u{1b}[1m");
    }
    let mut disable = String::new();
    if bold {
        disable.push_str("\u{1b}[22m");
    }
    disable.push_str(&disable_text_style(&color));

    let mut s = String::new();
    for a in args {
        s.push_str(a);
    }
    let mut first = true;
    for line in each_line(&s) {
        if !first {
            out.print("\n");
        }
        first = false;
        if line.is_empty() {
            continue;
        }
        out.print(&enable);
        out.print(line);
        out.print(&disable);
    }
}

fn each_line(s: &str) -> Vec<&str> {
    if s.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut start = 0;
    for (i, b) in s.bytes().enumerate() {
        if b == b'\n' {
            out.push(&s[start..=i]);
            start = i + 1;
        }
    }
    if start < s.len() {
        out.push(&s[start..]);
    }
    out
}

pub fn nocolor_string(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            let mut j = i + 2;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 2 && j < bytes.len() && bytes[j] == b'm' {
                i = j + 1;
                continue;
            }
        }
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

struct Renderer<'a> {
    plot: &'a Plot,
    out: Out,
    b: super::BorderChars,
    border_length: i64,
    max_len_l: usize,
    max_len_r: usize,
    plot_offset: usize,
    plot_padding: String,
    border_padding: String,
    y_lab_row: i64,
}

pub fn render(plot: &Plot, color: bool) -> String {
    let mut r = Renderer {
        plot,
        out: Out::new(color),
        b: border_chars(plot.border),
        border_length: 0,
        max_len_l: 0,
        max_len_r: 0,
        plot_offset: 0,
        plot_padding: String::new(),
        border_padding: String::new(),
        y_lab_row: 0,
    };
    r.init_render();
    r.render_top();
    r.render_rows();
    r.render_bottom();
    r.out.print("\n");
    r.out.buf
}

impl<'a> Renderer<'a> {
    fn push(&mut self, s: &str) {
        self.out.print(s);
    }
    fn pad(&mut self, n: i64) {
        if n > 0 {
            self.push(&" ".repeat(n as usize));
        }
    }
    fn styled(&mut self, args: &[&str], color: &Color) {
        print_styled(&mut self.out, args, false, color);
    }
    fn styled_bold(&mut self, args: &[&str], color: &Color) {
        print_styled(&mut self.out, args, true, color);
    }

    fn init_render(&mut self) {
        self.border_length = self.plot.n_columns();
        self.max_len_l = if self.plot.show_labels() && !self.plot.labels_left.is_empty() {
            self.plot
                .labels_left
                .values()
                .map(|l| str_len(&nocolor_string(l.as_deref().unwrap_or(""))))
                .max()
                .unwrap_or(0)
        } else {
            0
        };
        self.max_len_r = if self.plot.show_labels() && !self.plot.labels_right.is_empty() {
            self.plot
                .labels_right
                .values()
                .map(|l| str_len(&nocolor_string(l.as_deref().unwrap_or(""))))
                .max()
                .unwrap_or(0)
        } else {
            0
        };
        if self.plot.show_labels() && self.plot.ylabel_given() {
            self.max_len_l += self.plot.ylabel_length() + 1;
        }
        self.plot_offset = self.max_len_l + self.plot.margin as usize + self.plot.padding as usize;
        self.plot_padding = " ".repeat(self.plot.padding.max(0) as usize);
        self.border_padding = " ".repeat(self.plot_offset);
        self.y_lab_row = round_int(self.plot.n_rows() as f64 / 2.0) - 1;
    }

    fn render_top(&mut self) {
        let title = self.plot.title.clone().unwrap_or_default();
        self.print_title(self.border_padding.clone(), &title, self.border_length, Color::normal(), true);
        if self.plot.title_given() {
            self.push("\n");
        }

        if self.plot.show_labels() {
            let tl = self
                .plot
                .decorations
                .get("tl")
                .and_then(|v| v.clone())
                .unwrap_or_default();
            let t = self
                .plot
                .decorations
                .get("t")
                .and_then(|v| v.clone())
                .unwrap_or_default();
            let tr = self
                .plot
                .decorations
                .get("tr")
                .and_then(|v| v.clone())
                .unwrap_or_default();
            if !tl.is_empty() || !tr.is_empty() || !t.is_empty() {
                let tl_col = self
                    .plot
                    .colors_deco
                    .get("tl")
                    .cloned()
                    .unwrap_or_else(Color::light_black);
                let t_col = self
                    .plot
                    .colors_deco
                    .get("t")
                    .cloned()
                    .unwrap_or_else(Color::light_black);
                let tr_col = self
                    .plot
                    .colors_deco
                    .get("tr")
                    .cloned()
                    .unwrap_or_else(Color::light_black);
                let tl_len = str_len(&tl) as f64;
                let t_len = str_len(&t) as f64;
                let tr_len = str_len(&tr) as f64;
                let padding = self.border_padding.clone();
                self.styled(&[&padding, &tl], &tl_col);
                let cnt = round_int(self.border_length as f64 / 2.0 - t_len / 2.0 - tl_len);
                let pad = if cnt > 0 { " ".repeat(cnt as usize) } else { String::new() };
                self.styled(&[&pad, &t], &t_col);
                let cnt = self.border_length as f64 - tr_len - tl_len - t_len + 2.0 - cnt as f64;
                let cnt = round_int(cnt);
                let pad = if cnt > 0 { " ".repeat(cnt as usize) } else { String::new() };
                self.styled_bold(&[&pad, &tr, "\n"], &tr_col);
            }
        }

        let border_padding = self.border_padding.clone();
        let line = self.b.t.repeat(self.border_length.max(0) as usize);
        let b = self.b.clone_for_use();
        self.styled(
            &[&border_padding, b.tl, &line, b.tr],
            &Color::light_black(),
        );
        let max_len_r = " ".repeat(self.max_len_r);
        let plot_padding = self.plot_padding.clone();
        self.push(&max_len_r);
        self.push(&plot_padding);
        self.push("\n");
    }

    fn render_rows(&mut self) {
        for row in 0..self.plot.n_rows() {
            self.render_row(row);
        }
    }

    fn render_row(&mut self, row: i64) {
        let mut left_str = self
            .plot
            .labels_left
            .get(&(row as usize))
            .cloned()
            .flatten()
            .unwrap_or_default();
        let left_col = self
            .plot
            .colors_left
            .get(&(row as usize))
            .cloned()
            .unwrap_or_else(Color::light_black);
        let mut right_str = self
            .plot
            .labels_right
            .get(&(row as usize))
            .cloned()
            .flatten()
            .unwrap_or_default();
        let right_col = self
            .plot
            .colors_right
            .get(&(row as usize))
            .cloned()
            .unwrap_or_else(Color::light_black);
        let mut left_len = str_len(&nocolor_string(&left_str));
        let mut right_len = str_len(&nocolor_string(&right_str));

        if !self.out.color {
            left_str = nocolor_string(&left_str);
            right_str = nocolor_string(&right_str);
            left_len = str_len(&left_str);
            right_len = str_len(&right_str);
        }

        self.pad(self.plot.margin);
        if self.plot.show_labels() {
            if row == self.y_lab_row {
                let ylabel = self.plot.ylabel.clone().unwrap_or_default();
                self.styled(&[&ylabel], &Color::normal());
                self.pad(self.max_len_l as i64 - self.plot.ylabel_length() as i64 - left_len as i64);
            } else {
                self.pad(self.max_len_l as i64 - left_len as i64);
            }
            self.styled(&[&left_str], &left_col);
        }

        let pid = self.plot_padding.clone();
        let left_border = self.b.l;
        self.styled(&[&pid, left_border], &Color::light_black());

        // Canvas row.
        match &self.plot.kind {
            super::PlotKind::Bar(b) => b.print_row_out(&mut self.out, row),
            super::PlotKind::Box(b) => b.print_row_out(&mut self.out, row),
            super::PlotKind::Grid(g) => {
                let mut chars: Vec<(Color, String)> = Vec::new();
                g.canvas.print_row(row, |c, s| {
                    chars.push((c.clone(), s.to_string()));
                });
                for (c, s) in chars {
                    self.styled(&[&s], &c);
                }
            }
        }

        let right_border = self.b.r;
        self.styled(&[right_border], &Color::light_black());
        if self.plot.show_labels() {
            let pid = self.plot_padding.clone();
            self.push(&pid);
            self.styled(&[&right_str], &right_col);
            self.pad(self.max_len_r as i64 - right_len as i64);
        }
        self.push("\n");
    }

    fn render_bottom(&mut self) {
        let border_padding = self.border_padding.clone();
        let line = self.b.b.repeat(self.border_length.max(0) as usize);
        let b = self.b.clone_for_use();
        self.styled(
            &[&border_padding, b.bl, &line, b.br],
            &Color::light_black(),
        );
        let max_len_r = " ".repeat(self.max_len_r);
        let plot_padding = self.plot_padding.clone();
        self.push(&max_len_r);
        self.push(&plot_padding);

        if self.plot.show_labels() {
            let bl = self
                .plot
                .decorations
                .get("bl")
                .and_then(|v| v.clone())
                .unwrap_or_default();
            let bm = self
                .plot
                .decorations
                .get("b")
                .and_then(|v| v.clone())
                .unwrap_or_default();
            let br = self
                .plot
                .decorations
                .get("br")
                .and_then(|v| v.clone())
                .unwrap_or_default();
            if !bl.is_empty() || !br.is_empty() || !bm.is_empty() {
                self.push("\n");
                let bl_col = self
                    .plot
                    .colors_deco
                    .get("bl")
                    .cloned()
                    .unwrap_or_else(Color::light_black);
                let bm_col = self
                    .plot
                    .colors_deco
                    .get("b")
                    .cloned()
                    .unwrap_or_else(Color::light_black);
                let br_col = self
                    .plot
                    .colors_deco
                    .get("br")
                    .cloned()
                    .unwrap_or_else(Color::light_black);
                let bl_len = str_len(&bl) as f64;
                let bm_len = str_len(&bm) as f64;
                let br_len = str_len(&br) as f64;
                let padding = self.border_padding.clone();
                self.styled(&[&padding, &bl], &bl_col);
                let cnt = round_int(self.border_length as f64 / 2.0 - bm_len / 2.0 - bl_len);
                let pad = if cnt > 0 { " ".repeat(cnt as usize) } else { String::new() };
                self.styled(&[&pad, &bm], &bm_col);
                let cnt2 = self.border_length as f64 - br_len - bl_len - bm_len + 2.0 - cnt as f64;
                let cnt2 = round_int(cnt2);
                let pad = if cnt2 > 0 { " ".repeat(cnt2 as usize) } else { String::new() };
                self.styled(&[&pad, &br], &br_col);
            }
            if self.plot.xlabel_given() {
                self.push("\n");
            }
            let xlabel = self.plot.xlabel.clone().unwrap_or_default();
            self.print_title(self.border_padding.clone(), &xlabel, self.border_length, Color::normal(), false);
        }
    }

    fn print_title(&mut self, padding: String, title: &str, p_width: i64, color: Color, bold: bool) {
        if title.is_empty() {
            return;
        }
        let mut offset = round_int(p_width as f64 / 2.0 - str_len(title) as f64 / 2.0);
        if offset < 0 {
            offset = 0;
        }
        let tpad = " ".repeat(offset as usize);
        if bold {
            self.styled_bold(&[&padding, &tpad, title], &color);
        } else {
            self.styled(&[&padding, &tpad, title], &color);
        }
    }
}

// Small helper so we can hold border chars while mutably borrowing self.
impl super::BorderChars {
    fn clone_for_use(&self) -> SuperBorderChars {
        SuperBorderChars {
            tl: self.tl,
            tr: self.tr,
            bl: self.bl,
            br: self.br,
        }
    }
}

pub struct SuperBorderChars {
    pub tl: &'static str,
    pub tr: &'static str,
    pub bl: &'static str,
    pub br: &'static str,
}
