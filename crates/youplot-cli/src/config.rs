//! Minimal YAML subset for YouPlot config files, following Psych's
//! YAML 1.1-ish scalar rules (yes/no/on/off are booleans, 0-prefixed octal).

use crate::Options;
use youplot_core::{Error, Params, Result};

#[derive(Debug, Clone)]
pub enum Yaml {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Array(Vec<Yaml>),
}

impl Yaml {
    pub fn to_f64(&self) -> Option<f64> {
        match self {
            Yaml::Int(i) => Some(*i as f64),
            Yaml::Float(f) => Some(*f),
            Yaml::Str(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }
    pub fn to_string_value(&self) -> Option<String> {
        match self {
            Yaml::Str(s) => Some(s.clone()),
            Yaml::Int(i) => Some(i.to_string()),
            Yaml::Float(f) => Some(youplot_core::ruby::float_to_s(*f)),
            Yaml::Bool(b) => Some(b.to_string()),
            _ => None,
        }
    }
    pub fn to_bool(&self) -> Option<bool> {
        match self {
            Yaml::Bool(b) => Some(*b),
            Yaml::Str(s) => match s.as_str() {
                "true" | "yes" | "on" => Some(true),
                "false" | "no" | "off" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
    pub fn to_f64_vec(&self) -> Option<Vec<f64>> {
        match self {
            Yaml::Array(a) => a.iter().map(|v| v.to_f64()).collect(),
            Yaml::Str(s) => s
                .trim()
                .trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .map(|p| p.trim().parse::<f64>().ok())
                .collect(),
            _ => None,
        }
    }

    pub fn inspect(&self) -> String {
        match self {
            Yaml::Null => "nil".to_string(),
            Yaml::Bool(b) => b.to_string(),
            Yaml::Int(i) => i.to_string(),
            Yaml::Float(f) => youplot_core::ruby::float_to_s(*f),
            Yaml::Str(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            Yaml::Array(a) => {
                let items: Vec<String> = a.iter().map(|v| v.inspect()).collect();
                format!("[{}]", items.join(", "))
            }
        }
    }
}

pub struct Config {
    pub path: String,
    pub entries: Vec<(String, Yaml)>,
}

impl Config {
    pub fn get(&self, key: &str) -> Option<&Yaml> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }
    pub fn inspect(&self) -> String {
        let items: Vec<String> = self
            .entries
            .iter()
            .map(|(k, v)| format!("\"{}\"=>{}", k, v.inspect()))
            .collect();
        format!("{{{}}}", items.join(", "))
    }
}

pub fn candidate_paths() -> Vec<String> {
    let mut paths = Vec::new();
    if let Ok(v) = std::env::var("MYYOUPLOTRC") {
        paths.push(v);
    }
    paths.push(".youplot.yml".to_string());
    paths.push(".youplotrc".to_string());
    if let Ok(home) = std::env::var("HOME") {
        paths.push(format!("{}/.youplotrc", home));
        paths.push(format!("{}/.youplot.yml", home));
        paths.push(format!("{}/.config/youplot/youplotrc", home));
        paths.push(format!("{}/.config/youplot/youplot.yml", home));
    }
    paths
}

pub fn find_config_file() -> Option<String> {
    for file in candidate_paths() {
        let path = expand_path(&file);
        if std::path::Path::new(&path).exists() {
            std::env::set_var("MYYOUPLOTRC", &path);
            return Some(path);
        }
    }
    None
}

fn expand_path(p: &str) -> String {
    if p == "~" {
        return std::env::var("HOME").unwrap_or_else(|_| p.to_string());
    }
    if let Some(rest) = p.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}/{}", home, rest);
        }
    }
    if p.starts_with('/') {
        return p.to_string();
    }
    match std::env::current_dir() {
        Ok(d) => d.join(p).to_string_lossy().to_string(),
        Err(_) => p.to_string(),
    }
}

pub fn load(path: &str) -> Result<Config> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        Error::Message(format!("{} @ rb_sysopen - {}", io_message(&e), path))
    })?;
    let entries = parse_yaml(&text).map_err(Error::Message)?;
    Ok(Config {
        path: path.to_string(),
        entries,
    })
}

fn io_message(e: &std::io::Error) -> String {
    match e.kind() {
        std::io::ErrorKind::NotFound => "No such file or directory".to_string(),
        std::io::ErrorKind::PermissionDenied => "Permission denied".to_string(),
        _ => e.to_string(),
    }
}

fn parse_yaml(text: &str) -> std::result::Result<Vec<(String, Yaml)>, String> {
    let mut out = Vec::new();
    for raw in text.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "---" || trimmed == "..." {
            continue;
        }
        // Only top-level scalar mappings are supported (enough for YouPlot).
        let Some(colon) = find_colon(trimmed) else {
            return Err(format!("syntax error: {}", line));
        };
        let key = unquote(trimmed[..colon].trim());
        let mut value = trimmed[colon + 1..].trim().to_string();
        // strip comments outside quotes
        value = strip_comment(&value).trim().to_string();
        if value.is_empty() {
            // Could be a block sequence; YouPlot configs only use inline arrays.
            out.push((key, Yaml::Null));
        } else {
            out.push((key, parse_scalar(&value)));
        }
    }
    Ok(out)
}

fn find_colon(s: &str) -> Option<usize> {
    let mut in_single = false;
    let mut in_double = false;
    for (i, c) in s.char_indices() {
        match c {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            ':' if !in_single && !in_double => return Some(i),
            _ => {}
        }
    }
    None
}

fn strip_comment(s: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = s[i..].chars().next().unwrap();
        match c {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '#' if !in_single && !in_double => {
                if i == 0 || bytes[i - 1].is_ascii_whitespace() {
                    return &s[..i];
                }
            }
            _ => {}
        }
        i += c.len_utf8();
    }
    s
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        return unescape_double(&s[1..s.len() - 1]);
    }
    if s.len() >= 2 && s.starts_with('\'') && s.ends_with('\'') {
        return s[1..s.len() - 1].replace("''", "'");
    }
    s.to_string()
}

fn unescape_double(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn parse_scalar(s: &str) -> Yaml {
    let s = s.trim();
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len() - 1];
        if inner.trim().is_empty() {
            return Yaml::Array(Vec::new());
        }
        return Yaml::Array(split_commas(inner).into_iter().map(|p| parse_scalar(&p)).collect());
    }
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        return Yaml::Str(unescape_double(&s[1..s.len() - 1]));
    }
    if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
        return Yaml::Str(s[1..s.len() - 1].replace("''", "'"));
    }
    let lower = s.to_ascii_lowercase();
    if matches!(lower.as_str(), "~" | "null") {
        return Yaml::Null;
    }
    if matches!(lower.as_str(), "true" | "yes" | "on") {
        return Yaml::Bool(true);
    }
    if matches!(lower.as_str(), "false" | "no" | "off") {
        return Yaml::Bool(false);
    }
    if let Some(v) = parse_int(s) {
        return Yaml::Int(v);
    }
    if let Ok(v) = s.parse::<f64>() {
        if s.chars().any(|c| c == '.' || c == 'e' || c == 'E') {
            return Yaml::Float(v);
        }
    }
    Yaml::Str(s.to_string())
}

fn split_commas(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut depth = 0;
    for c in s.chars() {
        match c {
            '\'' if !in_double => {
                in_single = !in_single;
                cur.push(c);
            }
            '"' if !in_single => {
                in_double = !in_double;
                cur.push(c);
            }
            '[' if !in_single && !in_double => {
                depth += 1;
                cur.push(c);
            }
            ']' if !in_single && !in_double => {
                depth -= 1;
                cur.push(c);
            }
            ',' if !in_single && !in_double && depth == 0 => {
                out.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

fn parse_int(s: &str) -> Option<i64> {
    let t = s.replace('_', "");
    let (sign, body) = if let Some(rest) = t.strip_prefix('-') {
        (-1, rest)
    } else if let Some(rest) = t.strip_prefix('+') {
        (1, rest)
    } else {
        (1, t.as_str())
    };
    if body.is_empty() {
        return None;
    }
    if let Some(hex) = body.strip_prefix("0x").or_else(|| body.strip_prefix("0X")) {
        return i64::from_str_radix(hex, 16).ok().map(|v| v * sign);
    }
    if let Some(oct) = body.strip_prefix("0o").or_else(|| body.strip_prefix("0O")) {
        return i64::from_str_radix(oct, 8).ok().map(|v| v * sign);
    }
    if body.len() > 1 && body.starts_with('0') && body.bytes().all(|b| b.is_ascii_digit()) {
        return i64::from_str_radix(&body[1..], 8).ok().map(|v| v * sign);
    }
    body.parse::<i64>().ok().map(|v| v * sign)
}

pub const OPTION_KEYS: &[&str] = &[
    "delimiter",
    "transpose",
    "headers",
    "pass",
    "output",
    "fmt",
    "progressive",
    "encoding",
    "reverse",
    "color_names",
    "debug",
];

pub const PARAM_KEYS: &[&str] = &[
    "title",
    "width",
    "height",
    "border",
    "margin",
    "padding",
    "color",
    "xlabel",
    "ylabel",
    "labels",
    "symbol",
    "xscale",
    "nbins",
    "closed",
    "canvas",
    "xlim",
    "ylim",
    "grid",
    "name",
];

pub fn apply_config(options: &mut Options, params: &mut Params, config: &Config) -> Result<()> {
    // Validate keys first.
    for (k, _) in &config.entries {
        if !OPTION_KEYS.contains(&k.as_str()) && !PARAM_KEYS.contains(&k.as_str()) {
            return Err(Error::Message(format!(
                "Unknown option/param in config file: {}",
                k
            )));
        }
    }
    if options.delimiter.is_none() {
        if let Some(v) = config.get("delimiter").and_then(|v| v.to_string_value()) {
            options.delimiter = Some(v);
        }
    }
    if options.transpose.is_none() {
        if let Some(v) = config.get("transpose").and_then(|v| v.to_bool()) {
            options.transpose = Some(v);
        }
    }
    if options.headers.is_none() {
        if let Some(v) = config.get("headers").and_then(|v| v.to_bool()) {
            options.headers = Some(v);
        }
    }
    if options.pass.is_none() {
        if let Some(v) = config.get("pass").and_then(|v| v.to_string_value()) {
            options.pass = Some(v);
        }
    }
    if options.output.is_none() {
        if let Some(v) = config.get("output").and_then(|v| v.to_string_value()) {
            options.output = Some(v);
        }
    }
    if options.fmt.is_none() {
        if let Some(v) = config.get("fmt").and_then(|v| v.to_string_value()) {
            options.fmt = Some(v);
        }
    }
    if options.progressive.is_none() {
        if let Some(v) = config.get("progressive").and_then(|v| v.to_bool()) {
            options.progressive = Some(v);
        }
    }
    if options.encoding.is_none() {
        if let Some(v) = config.get("encoding").and_then(|v| v.to_string_value()) {
            options.encoding = Some(v);
        }
    }
    if options.reverse.is_none() {
        if let Some(v) = config.get("reverse").and_then(|v| v.to_bool()) {
            options.reverse = Some(v);
        }
    }
    if options.color_names.is_none() {
        if let Some(v) = config.get("color_names").and_then(|v| v.to_bool()) {
            options.color_names = Some(v);
        }
    }
    if options.debug.is_none() {
        if let Some(v) = config.get("debug").and_then(|v| v.to_bool()) {
            options.debug = Some(v);
        }
    }

    if params.title.is_none() {
        params.title = config.get("title").and_then(|v| v.to_string_value());
    }
    for (slot, key) in [
        (&mut params.width, "width"),
        (&mut params.height, "height"),
        (&mut params.margin, "margin"),
        (&mut params.padding, "padding"),
        (&mut params.nbins, "nbins"),
    ] {
        if slot.is_none() {
            *slot = config.get(key).and_then(|v| v.to_f64());
        }
    }
    if params.border.is_none() {
        params.border = config.get("border").and_then(|v| v.to_string_value());
    }
    if params.color.is_none() {
        if let Some(v) = config.get("color") {
            params.color = match v {
                Yaml::Int(i) => Some(youplot_core::ParamColor::Number(*i)),
                Yaml::Str(s) => Some(youplot_core::ParamColor::Name(s.clone())),
                _ => None,
            };
        }
    }
    if params.xlabel.is_none() {
        params.xlabel = config.get("xlabel").and_then(|v| v.to_string_value());
    }
    if params.ylabel.is_none() {
        params.ylabel = config.get("ylabel").and_then(|v| v.to_string_value());
    }
    if params.labels.is_none() {
        params.labels = config.get("labels").and_then(|v| v.to_bool());
    }
    if params.symbol.is_none() {
        params.symbol = config.get("symbol").and_then(|v| v.to_string_value());
    }
    if params.xscale.is_none() {
        params.xscale = config.get("xscale").and_then(|v| v.to_string_value());
    }
    if params.closed.is_none() {
        params.closed = config.get("closed").and_then(|v| v.to_string_value());
    }
    if params.canvas.is_none() {
        params.canvas = config.get("canvas").and_then(|v| v.to_string_value());
    }
    if params.xlim.is_none() {
        params.xlim = config.get("xlim").and_then(|v| v.to_f64_vec());
    }
    if params.ylim.is_none() {
        params.ylim = config.get("ylim").and_then(|v| v.to_f64_vec());
    }
    if params.grid.is_none() {
        params.grid = config.get("grid").and_then(|v| v.to_bool());
    }
    if params.name.is_none() {
        params.name = config.get("name").and_then(|v| v.to_string_value());
    }
    Ok(())
}
