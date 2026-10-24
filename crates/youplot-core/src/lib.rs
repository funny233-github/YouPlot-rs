//! Core library for YouPlot: DSV parsing, aggregation and Unicode plotting.
//!
//! The CLI lives in the `youplot-cli` crate. Everything in this crate is
//! independent of argument parsing, configuration files and terminal I/O.

pub mod aggregation;
pub mod backend;
pub mod csv;
pub mod dsv;
pub mod plot;
pub mod ruby;
pub mod stats;

pub const VERSION: &str = "0.5.0";
pub const UNICODE_PLOT_VERSION: &str = "0.0.5";

#[derive(Debug, Clone)]
pub struct Data {
    pub headers: Option<Vec<Option<String>>>,
    pub series: Vec<Vec<Option<String>>>,
}

/// UnicodePlot parameters.
#[derive(Debug, Clone, Default)]
pub struct Params {
    pub title: Option<String>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub border: Option<String>,
    pub margin: Option<f64>,
    pub padding: Option<f64>,
    pub color: Option<ParamColor>,
    pub xlabel: Option<String>,
    pub ylabel: Option<String>,
    pub labels: Option<bool>,
    pub symbol: Option<String>,
    pub xscale: Option<String>,
    pub nbins: Option<f64>,
    pub closed: Option<String>,
    pub canvas: Option<String>,
    pub xlim: Option<Vec<f64>>,
    pub ylim: Option<Vec<f64>>,
    pub grid: Option<bool>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParamColor {
    Name(String),
    Number(i64),
}

#[derive(Debug)]
pub enum Error {
    /// An ArgumentError raised while building the plot.
    Argument(String),
    /// A generic fatal message; printed as-is. An empty message means the
    /// caller already emitted the user-facing warning and only needs exit 1.
    Message(String),
}

impl Error {
    pub fn argument(msg: impl Into<String>) -> Self {
        Error::Argument(msg.into())
    }
    pub fn message(msg: impl Into<String>) -> Self {
        Error::Message(msg.into())
    }
    pub fn marker() -> Self {
        Error::Message(String::new())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Argument(m) | Error::Message(m) => write!(f, "{}", m),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// A colored text output sink. Writes into a string buffer.
pub struct Out {
    pub buf: String,
    pub color: bool,
}

impl Out {
    pub fn new(color: bool) -> Self {
        Out {
            buf: String::new(),
            color,
        }
    }
    pub fn print(&mut self, s: &str) {
        self.buf.push_str(s);
    }
}
