use crate::config;
use crate::Options;
use youplot_core::plot::border_types;
use youplot_core::plot::canvas::canvas_types;
use youplot_core::{ParamColor, Params, VERSION};

const COMMON_HELP: &str = "\
Common options:
    -O, --pass [FILE]       file to output input data to [stdout]
                            for inserting YouPlot in the middle of Unix pipes
    -o, --output [FILE]     file to output plots to [stdout]
                            If no option is specified, plot will print to stderr
    -d, --delimiter DELIM   use DELIM instead of [TAB] for field delimiter
    -H, --headers           specify that the input has header row
    -T, --transpose         transpose the axes of the input data
    -t, --title STR         print string on the top of plot
        --xlabel STR        print string on the bottom of the plot
        --ylabel STR        print string on the far left of the plot
    -w, --width INT         number of characters per row
    -h, --height INT        number of rows
    -b, --border STR        specify the style of the bounding box
                            (solid, corners, barplot)
    -m, --margin INT        number of spaces to the left of the plot
        --padding INT       space of the left and right of the plot
    -c, --color VAL         color of the drawing
        --[no-]labels       hide the labels
    -p, --progress          progressive mode [experimental]
    -C, --color-output      colorize even if writing to a pipe
    -M, --monochrome        no colouring even if writing to a tty
        --encoding STR      specify the input encoding
        --help              print sub-command help menu
        --config FILE       specify a config file
        --debug             print preprocessed data
";

fn main_help() -> String {
    format!(
        "
Program: YouPlot (Tools for plotting on the terminal)
Version: {} (using UnicodePlot {})
Source:  https://github.com/red-data-tools/YouPlot

Usage:   uplot <command> [options] <in.tsv>

Commands:
    barplot    bar           draw a horizontal barplot
    histogram  hist          draw a horizontal histogram
    lineplot   line          draw a line chart
    lineplots  lines         draw a line chart with multiple series
    scatter    s             draw a scatter plot
    density    d             draw a density plot
    boxplot    box           draw a horizontal boxplot
    count      c             draw a barplot based on the number of
                             occurrences (slow)
    colors     color         show the list of available colors

General options:
    --config                 print config file info
    --help                   print command specific help menu
    --version                print the version of YouPlot

",
        VERSION,
        crate::UNICODE_PLOT_VERSION
    )
}

fn group_options(command: &str) -> Option<String> {
    let s = match command {
        "barplot" | "bar" => "        --xscale STR        axis scaling (log, ln, log10, lg, log2, lb)
        --fmt STR           xy : header is like x, y...
                            yx : header is like y, x...
        --symbol STR        character to be used to plot the bars
"
        .to_string(),
        "count" | "c" => "        --xscale STR        axis scaling (log, ln, log10, lg, log2, lb)
        --symbol STR        character to be used to plot the bars
    -r, --reverse           reverse the result of comparisons
"
        .to_string(),
        "histogram" | "hist" => "    -n, --nbins INT         approximate number of bins
        --closed STR        side of the intervals to be closed [left]
        --symbol STR        character to be used to plot the bars
"
        .to_string(),
        "lineplot" | "line" | "l" => format!(
            "        --xlim FLOAT,FLOAT  plotting range for the x coordinate
        --ylim FLOAT,FLOAT  plotting range for the y coordinate
        --fmt STR           xy : header is like x, y...
                            yx : header is like y, x...
        --[no-]grid         draws grid-lines at the origin
        --canvas STR        type of canvas
                            ({})
",
            canvas_types().join(", ")
        ),
        "lineplots" | "lines" | "ls" | "scatter" | "s" | "density" | "d" => format!(
            "        --xlim FLOAT,FLOAT  plotting range for the x coordinate
        --ylim FLOAT,FLOAT  plotting range for the y coordinate
        --fmt STR           xyxy : header is like x1, y1, x2, y2, x3, y3...
                            xyy  : header is like x, y1, y2, y2, y3...
        --[no-]grid         draws grid-lines at the origin
        --canvas STR        type of canvas
                            ({})
",
            canvas_types().join(", ")
        ),
        "boxplot" | "box" => "        --xlim FLOAT,FLOAT  plotting range for the x coordinate
"
        .to_string(),
        "colors" | "color" | "colours" | "colour" => "    -n, --names             show color names only
"
        .to_string(),
        _ => return None,
    };
    Some(s)
}

fn sub_help(command: &str) -> String {
    let group = group_options(command).unwrap_or_default();
    format!(
        "\nUsage: YouPlot {} [options] <in.tsv>\n\nOptions for {}:\n{}\n{}\n",
        command, command, group, COMMON_HELP
    )
}

pub fn is_known_command(c: &str) -> bool {
    matches!(
        c,
        "barplot"
            | "bar"
            | "histogram"
            | "hist"
            | "lineplot"
            | "line"
            | "l"
            | "lineplots"
            | "lines"
            | "ls"
            | "scatter"
            | "s"
            | "density"
            | "d"
            | "boxplot"
            | "box"
            | "count"
            | "c"
            | "colors"
            | "color"
            | "colours"
            | "colour"
    )
}

pub struct Cli {
    pub command: String,
    pub options: Options,
    pub params: Params,
}

#[derive(Clone, Copy, PartialEq)]
enum ArgKind {
    None,
    Optional,
    Required,
}

#[derive(Clone)]
struct Spec {
    long: &'static str,
    short: Option<char>,
    arg: ArgKind,
    id: &'static str,
}

fn base_specs(config_required: bool, help_id: &'static str) -> Vec<Spec> {
    vec![
        Spec { long: "pass", short: Some('O'), arg: ArgKind::Optional, id: "pass" },
        Spec { long: "output", short: Some('o'), arg: ArgKind::Optional, id: "output" },
        Spec { long: "delimiter", short: Some('d'), arg: ArgKind::Required, id: "delimiter" },
        Spec { long: "headers", short: Some('H'), arg: ArgKind::None, id: "headers" },
        Spec { long: "transpose", short: Some('T'), arg: ArgKind::None, id: "transpose" },
        Spec { long: "title", short: Some('t'), arg: ArgKind::Required, id: "title" },
        Spec { long: "xlabel", short: None, arg: ArgKind::Required, id: "xlabel" },
        Spec { long: "ylabel", short: None, arg: ArgKind::Required, id: "ylabel" },
        Spec { long: "width", short: Some('w'), arg: ArgKind::Required, id: "width" },
        Spec { long: "height", short: Some('h'), arg: ArgKind::Required, id: "height" },
        Spec { long: "border", short: Some('b'), arg: ArgKind::Required, id: "border" },
        Spec { long: "margin", short: Some('m'), arg: ArgKind::Required, id: "margin" },
        Spec { long: "padding", short: None, arg: ArgKind::Required, id: "padding" },
        Spec { long: "color", short: Some('c'), arg: ArgKind::Required, id: "color" },
        Spec { long: "labels", short: None, arg: ArgKind::None, id: "labels" },
        Spec { long: "progress", short: Some('p'), arg: ArgKind::None, id: "progress" },
        Spec { long: "color-output", short: Some('C'), arg: ArgKind::None, id: "color_output" },
        Spec { long: "monochrome", short: Some('M'), arg: ArgKind::None, id: "monochrome" },
        Spec { long: "encoding", short: None, arg: ArgKind::Required, id: "encoding" },
        Spec { long: "help", short: None, arg: ArgKind::None, id: help_id },
        Spec {
            long: "config",
            short: None,
            arg: if config_required { ArgKind::Required } else { ArgKind::None },
            id: if config_required { "config_file" } else { "config_info" },
        },
        Spec { long: "debug", short: None, arg: ArgKind::None, id: "debug" },
        Spec { long: "version", short: Some('v'), arg: ArgKind::None, id: "version" },
    ]
}

fn command_specs(command: &str) -> Vec<Spec> {
    match command {
        "barplot" | "bar" => vec![
            Spec { long: "xscale", short: None, arg: ArgKind::Required, id: "xscale" },
            Spec { long: "fmt", short: None, arg: ArgKind::Required, id: "fmt" },
            Spec { long: "symbol", short: None, arg: ArgKind::Required, id: "symbol" },
        ],
        "count" | "c" => vec![
            Spec { long: "xscale", short: None, arg: ArgKind::Required, id: "xscale" },
            Spec { long: "symbol", short: None, arg: ArgKind::Required, id: "symbol" },
            Spec { long: "reverse", short: Some('r'), arg: ArgKind::None, id: "reverse" },
        ],
        "histogram" | "hist" => vec![
            Spec { long: "nbins", short: Some('n'), arg: ArgKind::Required, id: "nbins" },
            Spec { long: "closed", short: None, arg: ArgKind::Required, id: "closed" },
            Spec { long: "symbol", short: None, arg: ArgKind::Required, id: "symbol" },
        ],
        "lineplot" | "line" | "l" => vec![
            Spec { long: "xlim", short: None, arg: ArgKind::Required, id: "xlim" },
            Spec { long: "ylim", short: None, arg: ArgKind::Required, id: "ylim" },
            Spec { long: "fmt", short: None, arg: ArgKind::Required, id: "fmt" },
            Spec { long: "grid", short: None, arg: ArgKind::None, id: "grid" },
            Spec { long: "canvas", short: None, arg: ArgKind::Required, id: "canvas" },
        ],
        "lineplots" | "lines" | "ls" | "scatter" | "s" | "density" | "d" => vec![
            Spec { long: "xlim", short: None, arg: ArgKind::Required, id: "xlim" },
            Spec { long: "ylim", short: None, arg: ArgKind::Required, id: "ylim" },
            Spec { long: "fmt", short: None, arg: ArgKind::Required, id: "fmt" },
            Spec { long: "grid", short: None, arg: ArgKind::None, id: "grid" },
            Spec { long: "canvas", short: None, arg: ArgKind::Required, id: "canvas" },
        ],
        "boxplot" | "box" => {
            vec![Spec { long: "xlim", short: None, arg: ArgKind::Required, id: "xlim" }]
        }
        "colors" | "color" | "colours" | "colour" => {
            vec![Spec { long: "names", short: Some('n'), arg: ArgKind::None, id: "names" }]
        }
        _ => Vec::new(),
    }
}

struct Ctx<'a> {
    options: &'a mut Options,
    params: &'a mut Params,
}

fn find_long<'a>(specs: &'a [Spec], name: &str) -> std::result::Result<&'a Spec, String> {
    if let Some(s) = specs.iter().find(|s| s.long == name) {
        return Ok(s);
    }
    let matches: Vec<&Spec> = specs.iter().filter(|s| s.long.starts_with(name)).collect();
    if matches.len() == 1 {
        Ok(matches[0])
    } else if matches.len() > 1 {
        Err(format!("ambiguous option: --{}", name))
    } else {
        Err(format!("invalid option: --{}", name))
    }
}

fn find_short(specs: &[Spec], c: char) -> Option<&Spec> {
    specs.iter().find(|s| s.short == Some(c))
}

fn parse_number(value: &str, opt: &str) -> std::result::Result<f64, String> {
    let t = value.replace('_', "");
    if let Ok(v) = t.parse::<f64>() {
        return Ok(v);
    }
    Err(format!("invalid argument: {} {}", opt, value))
}

fn parse_array(value: &str) -> Vec<String> {
    value.split(',').map(|s| s.trim().to_string()).collect()
}

fn apply_value(
    spec: &Spec,
    value: Option<&str>,
    opt_display: &str,
    ctx: &mut Ctx,
) -> std::result::Result<(), String> {
    let v = value;
    let need = |v: Option<&str>| -> std::result::Result<String, String> {
        v.map(|s| s.to_string())
            .ok_or_else(|| format!("missing argument: {}", opt_display))
    };
    match spec.id {
        "pass" => {
            ctx.options.pass = Some(v.unwrap_or("-").to_string());
        }
        "output" => {
            ctx.options.output = Some(v.unwrap_or("-").to_string());
        }
        "delimiter" => ctx.options.delimiter = Some(need(v)?),
        "headers" => ctx.options.headers = Some(true),
        "transpose" => ctx.options.transpose = Some(true),
        "title" => ctx.params.title = Some(need(v)?),
        "xlabel" => ctx.params.xlabel = Some(need(v)?),
        "ylabel" => ctx.params.ylabel = Some(need(v)?),
        "width" => ctx.params.width = Some(parse_number(&need(v)?, opt_display)?),
        "height" => ctx.params.height = Some(parse_number(&need(v)?, opt_display)?),
        "border" => ctx.params.border = Some(need(v)?),
        "margin" => ctx.params.margin = Some(parse_number(&need(v)?, opt_display)?),
        "padding" => ctx.params.padding = Some(parse_number(&need(v)?, opt_display)?),
        "color" => {
            let s = need(v)?;
            ctx.params.color = Some(if !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) {
                ParamColor::Number(s.parse::<i64>().unwrap_or(0))
            } else {
                ParamColor::Name(s)
            });
        }
        "labels" => ctx.params.labels = Some(true),
        "progress" => ctx.options.progressive = Some(true),
        "color_output" => ctx.options.force_color = Some(true),
        "monochrome" => ctx.options.force_color = Some(false),
        "encoding" => ctx.options.encoding = Some(need(v)?),
        "debug" => ctx.options.debug = Some(true),
        "fmt" => ctx.options.fmt = Some(need(v)?),
        "symbol" => ctx.params.symbol = Some(need(v)?),
        "xscale" => ctx.params.xscale = Some(need(v)?),
        "nbins" => ctx.params.nbins = Some(parse_number(&need(v)?, opt_display)?),
        "closed" => ctx.params.closed = Some(need(v)?),
        "canvas" => ctx.params.canvas = Some(need(v)?),
        "xlim" => {
            let s = need(v)?;
            let vals = parse_array(&s)
                .iter()
                .map(|x| x.parse::<f64>().unwrap_or(0.0))
                .collect();
            ctx.params.xlim = Some(vals);
        }
        "ylim" => {
            let s = need(v)?;
            let vals = parse_array(&s)
                .iter()
                .map(|x| x.parse::<f64>().unwrap_or(0.0))
                .collect();
            ctx.params.ylim = Some(vals);
        }
        "grid" => ctx.params.grid = Some(true),
        "no_grid" => ctx.params.grid = Some(false),
        "no_labels" => ctx.params.labels = Some(false),
        "reverse" => ctx.options.reverse = Some(true),
        "names" => ctx.options.color_names = Some(true),
        "config_file" => ctx.options.config_file = Some(need(v)?),
        _ => {}
    }
    Ok(())
}

/// Parse tokens; returns the index of the first unconsumed token.
fn parse_tokens(
    tokens: &[String],
    start: usize,
    specs: &[Spec],
    ctx: &mut Ctx,
    stop_at_non_option: bool,
    phase1: bool,
    command: Option<&str>,
) -> std::result::Result<usize, String> {
    let mut i = start;
    while i < tokens.len() {
        let tok = &tokens[i];
        if tok == "--" {
            return Ok(i + 1);
        }
        let is_option = tok.len() > 1 && tok.starts_with('-');
        if !is_option {
            if stop_at_non_option {
                return Ok(i);
            }
            i += 1;
            continue;
        }

        if let Some(body) = tok.strip_prefix("--") {
            let (name, inline) = match body.split_once('=') {
                Some((a, b)) => (a, Some(b.to_string())),
                None => (body, None),
            };
            // --no-labels / --no-grid
            if let Some(base) = name.strip_prefix("no-") {
                if base == "labels" || base == "grid" {
                    if let Some(v) = inline {
                        return Err(format!(
                            "needless argument: --{}={}",
                            name, v
                        ));
                    }
                    let spec = Spec {
                        long: if base == "labels" { "labels" } else { "grid" },
                        short: None,
                        arg: ArgKind::None,
                        id: if base == "labels" { "no_labels" } else { "no_grid" },
                    };
                    let display = format!("--{}", name);
                    apply_value(&spec, None, &display, ctx)?;
                    i += 1;
                    continue;
                }
            }

            let spec = find_long(specs, name)?.clone();
            let display = format!("--{}", spec.long);
            if spec.id == "version" {
                println!("YouPlot {}", VERSION);
                std::process::exit(0);
            }
            if spec.id == "help" {
                if phase1 {
                    print!("{}", main_help());
                } else {
                    print!("{}", sub_help(command.unwrap_or("")));
                }
                std::process::exit(0);
            }
            if spec.id == "config_info" {
                show_config_info();
                std::process::exit(0);
            }
            match spec.arg {
                ArgKind::None => {
                    if let Some(v) = inline {
                        return Err(format!(
                            "needless argument: --{}={}",
                            spec.long, v
                        ));
                    }
                    apply_value(&spec, None, &display, ctx)?;
                    i += 1;
                }
                ArgKind::Required => {
                    let (value, used) = match inline {
                        Some(v) => {
                            if v.is_empty() {
                                return Err(format!(
                                    "missing argument: {}",
                                    display
                                ));
                            }
                            (v, 1usize)
                        }
                        None => {
                            if i + 1 >= tokens.len() {
                                return Err(format!(
                                    "missing argument: {}",
                                    display
                                ));
                            }
                            (tokens[i + 1].clone(), 2usize)
                        }
                    };
                    apply_value(&spec, Some(&value), &display, ctx)?;
                    i += used;
                }
                ArgKind::Optional => {
                    let (value, used) = match inline {
                        Some(v) => (Some(v), 1usize),
                        None => {
                            if i + 1 < tokens.len()
                                && (!tokens[i + 1].starts_with('-') || tokens[i + 1] == "-")
                            {
                                (Some(tokens[i + 1].clone()), 2usize)
                            } else {
                                (None, 1usize)
                            }
                        }
                    };
                    apply_value(&spec, value.as_deref(), &display, ctx)?;
                    i += used;
                }
            }
            continue;
        }

        // Short option(s).
        let chars: Vec<char> = tok.chars().skip(1).collect();
        let mut pos = 0usize;
        let mut consumed_next = false;
        while pos < chars.len() {
            let c = chars[pos];
            let Some(spec) = find_short(specs, c) else {
                let long_matches: Vec<&Spec> = specs
                    .iter()
                    .filter(|s| s.long.starts_with(c))
                    .collect();
                if long_matches.len() > 1 {
                    return Err(format!("ambiguous option: -{}", c));
                }
                return Err(format!("invalid option: -{}", c));
            };
            let spec = spec.clone();
            let display = format!("-{}", c);
            let rest: String = chars[pos + 1..].iter().collect();
            if spec.id == "version" {
                println!("YouPlot {}", VERSION);
                std::process::exit(0);
            }
            if spec.id == "help" {
                if phase1 {
                    print!("{}", main_help());
                } else {
                    print!("{}", sub_help(command.unwrap_or("")));
                }
                std::process::exit(0);
            }
            if spec.id == "config_info" {
                show_config_info();
                std::process::exit(0);
            }
            match spec.arg {
                ArgKind::None => {
                    if !rest.is_empty() {
                        // Treat the rest as another bundled short option.
                        pos += 1;
                        continue;
                    }
                    apply_value(&spec, None, &display, ctx)?;
                    pos += 1;
                }
                ArgKind::Required => {
                    let (value, next_used) = if !rest.is_empty() {
                        (rest, false)
                    } else if i + 1 < tokens.len() {
                        (tokens[i + 1].clone(), true)
                    } else {
                        return Err(format!("missing argument: {}", display));
                    };
                    apply_value(&spec, Some(&value), &display, ctx)?;
                    if next_used {
                        consumed_next = true;
                    }
                    pos = chars.len();
                }
                ArgKind::Optional => {
                    let (value, next_used) = if !rest.is_empty() {
                        (Some(rest), false)
                    } else if i + 1 < tokens.len()
                        && (!tokens[i + 1].starts_with('-') || tokens[i + 1] == "-")
                    {
                        (Some(tokens[i + 1].clone()), true)
                    } else {
                        (None, false)
                    };
                    apply_value(&spec, value.as_deref(), &display, ctx)?;
                    if next_used {
                        consumed_next = true;
                    }
                    pos = chars.len();
                }
            }
        }
        i += if consumed_next { 2 } else { 1 };
    }
    Ok(i)
}

fn show_config_info() {
    let mut options = Options::default();
    let mut params = Params::default();
    let cfg_path = config::find_config_file();
    let loaded = match &cfg_path {
        Some(p) => match config::load(p) {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("YouPlot: {}", e);
                std::process::exit(1);
            }
        },
        None => None,
    };
    if let Some(c) = loaded {
        let _ = config::apply_config(&mut options, &mut params, &c);
        println!("config file : {}", c.path);
        println!("{}", c.inspect());
    } else {
        let paths = config::candidate_paths();
        print!("Configuration file not found.
It should be a YAML file, like this example:
  width : 40
  height : 20
By default, YouPlot will look for the configuration file in these locations:
");
        for p in paths {
            println!("  {}", p);
        }
        println!("If you have the file elsewhere, you can specify its location with the `MYYOUPLOTRC` environment variable.");
    }
}

pub fn parse(argv: Vec<String>) -> std::result::Result<Cli, String> {
    let mut options = Options::default();
    let mut params = Params::default();

    // Phase 1: main options, stopping at the first non-option.
    let main_specs = base_specs(false, "help");
    let mut ctx = Ctx {
        options: &mut options,
        params: &mut params,
    };
    let idx = parse_tokens(&argv, 0, &main_specs, &mut ctx, true, true, None)?;

    if idx >= argv.len() {
        // No command: help goes to stderr.
        eprint!("{}", main_help());
        std::process::exit(0);
    }
    let command = argv[idx].clone();
    let mut pos = idx + 1;

    if command == "help" {
        print!("{}", main_help());
        std::process::exit(0);
    }
    if command == "config" {
        show_config_info();
        std::process::exit(0);
    }
    if !is_known_command(&command) {
        return Err(format!("unrecognized command '{}'", command));
    }

    // Phase 2: command-specific options.
    let mut specs = base_specs(true, "help");
    specs.extend(command_specs(&command));
    let mut ctx = Ctx {
        options: &mut options,
        params: &mut params,
    };
    let _ = parse_tokens(
        &argv,
        pos,
        &specs,
        &mut ctx,
        false,
        false,
        Some(&command),
    )?;
    pos = argv.len();
    let _ = pos;

    // Load config, then resolve.
    let cfg_path = match &options.config_file {
        Some(p) => Some(p.clone()),
        None => config::find_config_file(),
    };
    if let Some(path) = cfg_path {
        match config::load(&path) {
            Ok(c) => {
                config::apply_config(&mut options, &mut params, &c).map_err(|e| e.to_string())?;
            }
            Err(e) => {
                eprintln!("YouPlot: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(Cli {
        command,
        options,
        params,
    })
}

pub fn border_help() -> String {
    border_types().join(", ")
}
