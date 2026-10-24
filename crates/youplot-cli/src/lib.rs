//! Command-line interface for YouPlot. All plotting logic lives in
//! `youplot-core`; this crate owns argument parsing, config files, terminal
//! I/O and progressive mode.
//!
//! A Rust port of YouPlot (<https://github.com/red-data-tools/YouPlot>,
//! MIT, Copyright (c) 2020 kojix2, 2025 Red Data Tools). See `NOTICE.md`.

pub mod cli;
pub mod command;
pub mod config;

pub use youplot_core as core;
pub use youplot_core::{UNICODE_PLOT_VERSION, VERSION};

use std::io::{self, IsTerminal, Read, Write};

/// Command line options that are not plot parameters.
#[derive(Debug, Clone, Default)]
pub struct Options {
    pub delimiter: Option<String>,
    pub transpose: Option<bool>,
    pub headers: Option<bool>,
    pub pass: Option<String>,
    pub output: Option<String>,
    pub fmt: Option<String>,
    pub progressive: Option<bool>,
    pub encoding: Option<String>,
    pub reverse: Option<bool>,
    pub color_names: Option<bool>,
    pub debug: Option<bool>,
    /// `-C` forces color, `-M` forces monochrome; last one wins.
    pub force_color: Option<bool>,
    pub config_file: Option<String>,
}

impl Options {
    pub fn delimiter(&self) -> String {
        self.delimiter.clone().unwrap_or_else(|| "\t".to_string())
    }
    pub fn transpose(&self) -> bool {
        self.transpose.unwrap_or(false)
    }
    pub fn headers(&self) -> bool {
        self.headers.unwrap_or(false)
    }
    pub fn fmt(&self) -> String {
        self.fmt.clone().unwrap_or_else(|| "xyy".to_string())
    }
    pub fn progressive(&self) -> bool {
        self.progressive.unwrap_or(false)
    }
    pub fn reverse(&self) -> bool {
        self.reverse.unwrap_or(false)
    }
    pub fn color_names(&self) -> bool {
        self.color_names.unwrap_or(false)
    }
    pub fn debug(&self) -> bool {
        self.debug.unwrap_or(false)
    }
}

/// Where plot / data output goes.
#[derive(Debug, Clone)]
pub enum OutTarget {
    Stdout,
    Stderr,
    File(String),
}

impl OutTarget {
    pub fn write_all(&self, bytes: &[u8]) -> io::Result<()> {
        match self {
            OutTarget::Stdout => {
                let mut out = io::stdout().lock();
                out.write_all(bytes)?;
                out.flush()
            }
            OutTarget::Stderr => {
                let mut out = io::stderr().lock();
                out.write_all(bytes)?;
                out.flush()
            }
            OutTarget::File(path) => {
                let mut f = std::fs::File::create(path)?;
                f.write_all(bytes)?;
                f.flush()
            }
        }
    }

    pub fn is_tty(&self) -> bool {
        match self {
            OutTarget::Stdout => io::stdout().is_terminal(),
            OutTarget::Stderr => io::stderr().is_terminal(),
            OutTarget::File(_) => false,
        }
    }
}

pub fn read_stdin_bytes() -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;
    Ok(buf)
}

pub fn decode_input(
    input: &[u8],
    encoding: Option<&str>,
) -> std::result::Result<String, youplot_core::Error> {
    use youplot_core::Error;
    match encoding {
        None => match std::str::from_utf8(input) {
            Ok(s) => Ok(s.to_string()),
            Err(_) => Err(Error::Message("invalid byte sequence in UTF-8".to_string())),
        },
        Some(label) => {
            let enc = encoding_rs::Encoding::for_label(label.as_bytes()).ok_or_else(|| {
                Error::Message(format!("unknown encoding name - {}", label))
            })?;
            let (cow, _, had_errors) = enc.decode(input);
            if had_errors {
                return Err(Error::Message("invalid byte sequence".to_string()));
            }
            let mut out = cow.into_owned();
            let lower = label.to_ascii_lowercase().replace('_', "-");
            let bom = matches!(lower.as_str(), "utf-16le" | "utf-16be");
            if bom {
                let has_bom = (input.starts_with(&[0xFF, 0xFE]) && lower == "utf-16le")
                    || (input.starts_with(&[0xFE, 0xFF]) && lower == "utf-16be");
                if has_bom && !out.starts_with('\u{FEFF}') {
                    out.insert(0, '\u{FEFF}');
                }
            }
            Ok(out)
        }
    }
}
