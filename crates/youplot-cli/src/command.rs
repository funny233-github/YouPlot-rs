use crate::cli::Cli;
use crate::{decode_input, read_stdin_bytes, OutTarget, Options};
use youplot_core::backend::{self, Rendered};
use youplot_core::csv;
use youplot_core::dsv;
use youplot_core::dsv::build_data;
use youplot_core::{Data, Error, Params, Result};
use std::io::BufRead;
use std::sync::atomic::{AtomicBool, Ordering};

static STOP: AtomicBool = AtomicBool::new(false);

extern "C" {
    fn signal(sig: i32, handler: usize) -> usize;
}

extern "C" fn on_sigint(_: i32) {
    STOP.store(true, Ordering::SeqCst);
}


fn make_plot(
    command: &str,
    data: Option<&Data>,
    params: &mut Params,
    options: &Options,
) -> Result<Rendered> {
    backend::create_plot(
        command,
        data,
        params,
        &options.fmt(),
        options.reverse(),
        options.color_names(),
    )
}

pub fn run(cli: Cli) -> i32 {
    match run_inner(cli) {
        Ok(()) => 0,
        Err(Error::Argument(m)) => {
            eprintln!("\u{1b}[35m{}\u{1b}[0m", m);
            1
        }
        Err(Error::Message(m)) => {
            if !m.is_empty() {
                eprintln!("{}", m);
            } else if std::env::var("YOUPLOT_DEBUG_ERR").is_ok() {
                eprintln!("MARKER");
            }
            1
        }
    }
}

fn output_target(options: &Options) -> OutTarget {
    match &options.output {
        None => OutTarget::Stderr,
        Some(p) if p == "-" => OutTarget::Stdout,
        Some(p) => OutTarget::File(p.clone()),
    }
}

fn pass_target(options: &Options) -> Option<OutTarget> {
    match &options.pass {
        None => None,
        Some(p) if p == "-" => Some(OutTarget::Stdout),
        Some(p) => Some(OutTarget::File(p.clone())),
    }
}

fn forced_color(options: &Options, target: &OutTarget) -> bool {
    options.force_color.unwrap_or_else(|| target.is_tty())
}

fn write_data(options: &Options, bytes: &[u8]) -> Result<()> {
    if let Some(target) = pass_target(options) {
        target
            .write_all(bytes)
            .map_err(|e| Error::Message(e.to_string()))?;
    }
    Ok(())
}

fn run_inner(cli: Cli) -> Result<()> {
    let mut options = cli.options;
    let mut params = cli.params;
    let command = cli.command;

    if matches!(
        command.as_str(),
        "colors" | "color" | "colours" | "colour"
    ) {
        let rendered = make_plot(&command, None, &mut params, &options)?;
        let out = output_target(&options);
        let color = forced_color(&options, &out);
        let s = backend::render_to_string(&rendered, color);
        out.write_all(s.as_bytes())
            .map_err(|e| Error::Message(e.to_string()))?;
        return Ok(());
    }

    if options.progressive() {
        return run_progressive(&command, &mut options, &mut params);
    }

    let raw = read_stdin_bytes().map_err(|e| Error::Message(e.to_string()))?;
    if raw.is_empty() {
        return Ok(());
    }
    write_data(&options, &raw)?;
    let input = match options.encoding.as_deref() {
        Some(enc) => decode_input(&raw, Some(enc))?,
        None => match String::from_utf8(raw.clone()) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Failed to parse the text. ");
                eprintln!(
                    "Please try to set the correct character encoding with --encoding option."
                );
                eprintln!();
                return Err(Error::marker());
            }
        },
    };
    let data = parse_dsv(&input, &mut options)?;
    if options.debug() {
        println!("{}", inspect_data(&data));
    }
    let rendered = make_plot(&command, Some(&data), &mut params, &options)?;
    let out = output_target(&options);
    let color = forced_color(&options, &out);
    let s = backend::render_to_string(&rendered, color);
    out.write_all(s.as_bytes())
        .map_err(|e| Error::Message(e.to_string()))?;
    Ok(())
}

fn parse_dsv(input: &str, options: &mut Options) -> Result<Data> {
    match dsv::parse(input, &options.delimiter(), options.headers(), options.transpose()) {
        Ok(d) => Ok(d),
        Err(Error::Message(m)) if m.starts_with("__CSV__") => {
            eprintln!("Failed to parse the text. ");
            eprintln!("Please try to set the correct character encoding with --encoding option.");
            eprintln!("youplot/dsv.rb:1:in 'YouPlot::DSV.parse'");
            Err(Error::marker())
        }
        Err(e) => Err(e),
    }
}

fn inspect_opt_string(v: &Option<String>) -> String {
    match v {
        None => "nil".to_string(),
        Some(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
    }
}

fn inspect_data(data: &Data) -> String {
    let headers = match &data.headers {
        None => "nil".to_string(),
        Some(h) => {
            let items: Vec<String> = h.iter().map(inspect_opt_string).collect();
            format!("[{}]", items.join(", "))
        }
    };
    let series: Vec<String> = data
        .series
        .iter()
        .map(|s| {
            let items: Vec<String> = s.iter().map(inspect_opt_string).collect();
            format!("[{}]", items.join(", "))
        })
        .collect();
    format!(
        "#<struct YouPlot::Data headers={}, series=[{}]>",
        headers,
        series.join(", ")
    )
}

// ---------------------------------------------------------------------------
// progressive mode
// ---------------------------------------------------------------------------

struct ProgressiveState {
    initialized: bool,
    headers: Option<Vec<Option<String>>>,
    series: Vec<Vec<Option<String>>>,
    header_consumed: bool,
    row_count: usize,
}

impl ProgressiveState {
    fn new() -> Self {
        ProgressiveState {
            initialized: false,
            headers: None,
            series: Vec::new(),
            header_consumed: false,
            row_count: 0,
        }
    }
}

fn run_progressive(command: &str, options: &mut Options, params: &mut Params) -> Result<()> {
    let target = output_target(options);
    if matches!(target, OutTarget::File(_)) {
        return Err(Error::Message(
            "In progressive mode, output to a file is not possible.".to_string(),
        ));
    }

    STOP.store(false, Ordering::SeqCst);
    unsafe {
        let h = on_sigint as extern "C" fn(i32);
        signal(2, h as usize);
    }

    write_target(&target, b"\x1b[?25l")?;

    let mut state = ProgressiveState::new();
    let mut last_rendered: usize = 0;

    let result = (|| -> Result<()> {
        let stdin = std::io::stdin();
        let mut reader = stdin.lock();
        let mut buf: Vec<u8> = Vec::new();
        loop {
            buf.clear();
            let n = reader
                .read_until(b'\n', &mut buf)
                .map_err(|e| Error::Message(e.to_string()))?;
            if n == 0 {
                break;
            }
            write_data(options, &buf)?;

            let rendered_lines =
                progressive_step(command, options, params, &mut state, &buf, &target)?;
            if rendered_lines > 0 {
                last_rendered = rendered_lines;
            }
            if STOP.load(Ordering::SeqCst) {
                break;
            }
            if rendered_lines > 0 {
                write_target(
                    &target,
                    format!("\x1b[{}F", rendered_lines).as_bytes(),
                )?;
            }
        }
        Ok(())
    })();

    // ensure cleanup
    if last_rendered > 0 {
        let _ = write_target(&target, format!("\x1b[{}E", last_rendered).as_bytes());
    }
    let _ = write_target(&target, b"\x1b[0J");
    let _ = write_target(&target, b"\x1b[?25h");

    result
}

fn progressive_step(
    command: &str,
    options: &mut Options,
    params: &mut Params,
    state: &mut ProgressiveState,
    raw_line: &[u8],
    target: &OutTarget,
) -> Result<usize> {
    let line = decode_input(raw_line, options.encoding.as_deref())?;
    let rows = csv::parse(&line, &options.delimiter());
    let row: Vec<Option<String>> = match rows {
        Ok(mut r) => {
            if r.is_empty() {
                Vec::new()
            } else {
                r.remove(0)
            }
        }
        Err(_) => {
            eprintln!("Failed to parse the text. ");
            eprintln!("Please try to set the correct character encoding with --encoding option.");
            eprintln!("youplot/command.rb:1:in 'YouPlot::Command#parse_progressive_row'");
            return Err(Error::marker());
        }
    };
    if row.is_empty() || row.iter().all(|v| v.is_none()) {
        return Ok(0);
    }
    if !state.initialized {
        state.initialized = true;
        state.headers = if options.headers() { Some(Vec::new()) } else { None };
        state.series = Vec::new();
        state.header_consumed = false;
        state.row_count = 0;
    }

    if options.headers() && !options.transpose() && !state.header_consumed {
        state.headers = Some(row);
        state.header_consumed = true;
        return Ok(0);
    }

    if options.headers() && options.transpose() {
        let mut headers = state.headers.take().unwrap_or_default();
        headers.push(row.first().cloned().unwrap_or(None));
        state.headers = Some(headers);
        state.series.push(row.into_iter().skip(1).collect());
    } else if options.transpose() {
        state.series.push(row);
    } else {
        append_columns(state, &row);
    }

    let data = build_data(state.headers.clone(), state.series.clone())?;
    let rendered = make_plot(command, Some(&data), params, options)?;
    output_progressive(&rendered, target)
}

fn append_columns(state: &mut ProgressiveState, row: &[Option<String>]) {
    if row.len() > state.series.len() {
        for i in state.series.len()..row.len() {
            let _ = i;
            state.series.push(vec![None; state.row_count]);
        }
    }
    for i in 0..state.series.len() {
        state.series[i].push(row.get(i).cloned().unwrap_or(None));
    }
    state.row_count += 1;
}

fn output_progressive(rendered: &Rendered, target: &OutTarget) -> Result<usize> {
    let s = backend::render_to_string(rendered, true);
    let lines = split_lines(&s);
    for line in &lines {
        let chomped = chomp(line);
        target
            .write_all(chomped.as_bytes())
            .map_err(|e| Error::Message(e.to_string()))?;
        target
            .write_all(b"\x1b[0K")
            .map_err(|e| Error::Message(e.to_string()))?;
        target
            .write_all(b"\n")
            .map_err(|e| Error::Message(e.to_string()))?;
    }
    target
        .write_all(b"\x1b[0J")
        .map_err(|e| Error::Message(e.to_string()))?;
    Ok(lines.len())
}

fn split_lines(s: &str) -> Vec<&str> {
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

fn chomp(s: &str) -> &str {
    if let Some(t) = s.strip_suffix("\r\n") {
        t
    } else if let Some(t) = s.strip_suffix('\n') {
        t
    } else {
        s
    }
}

fn write_target(target: &OutTarget, bytes: &[u8]) -> Result<()> {
    target
        .write_all(bytes)
        .map_err(|e| Error::Message(e.to_string()))
}
