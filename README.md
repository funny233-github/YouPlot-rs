# YouPlot-rs

A Rust port of [YouPlot](https://github.com/red-data-tools/YouPlot), including a
faithful port of the `unicode_plot.rb` rendering engine. Plot output is
byte-for-byte compatible with the Ruby implementation, including Ruby's
`Float#to_s` / `Float#round` quirks, ANSI styling and progressive mode.

The project is split into a library crate and a CLI crate:

```
Cargo.toml                     workspace
crates/
  youplot-core/                library: no argument parsing, no terminal I/O
    src/
      backend.rs               YouPlot command -> plot object
      csv.rs  dsv.rs           DSV parsing (Ruby CSV semantics, nil fields)
      aggregation.rs           `count` tally + natural ordering
      stats.rs                 histogram edges/weights, percentiles
      ruby.rs                  Ruby Float/String compatibility layer
      plot/                    UnicodePlot port
        canvas.rs              ascii / block / braille / density / dot canvases
        barplot.rs  boxplot.rs grid.rs
        renderer.rs            borders, labels, ANSI styled printer
  youplot-cli/                 binary crate: `uplot` and `youplot`
    src/
      cli.rs                   OptionParser-compatible argument parsing + help
      config.rs                YAML config file subset + search paths
      command.rs               stdin/stdout, -o / -O, progressive mode
      bin/uplot.rs, bin/youplot.rs
    tests/                     golden binary tests, one file per feature
```

## Build / run

```sh
cargo build --release
target/release/uplot line < data.tsv              # plot to stderr
target/release/uplot bar -H -d, < data.csv
target/release/uplot line -o - < data.tsv         # plot to stdout
target/release/uplot colors
```

`uplot` and `youplot` are identical entry points.

## Features

| Feature | Core | CLI golden tests |
| --- | --- | --- |
| bar / barplot | `plot/barplot.rs`, `backend::barplot_impl` | `tests/barplot.rs` |
| histogram / hist | `stats.rs`, `backend::histogram` | `tests/histogram.rs` |
| line / lineplot | `plot/grid.rs`, `plot/canvas.rs` | `tests/lineplot.rs` |
| lineplots / scatter / density | `backend::plot_xyy`, `plot_xyxy` | `tests/multi_series.rs` |
| boxplot / box | `plot/boxplot.rs` | `tests/boxplot.rs` |
| count / c | `aggregation.rs` | `tests/count.rs` |
| colors | `backend::colors` | `tests/colors.rs` |
| DSV / CSV / transpose / headers | `csv.rs`, `dsv.rs` | `tests/dsv.rs` |
| `-o` / `-O` output routing | `youplot-cli/command.rs` | `tests/output.rs` |
| config files / `--config` | `youplot-cli/config.rs` | `tests/config.rs` |
| progressive mode | `youplot-cli/command.rs` | `tests/progressive.rs` |
| `-C` / `-M` / debug | `youplot-cli/command.rs` | `tests/colors_output.rs` |
| help / version / errors | `youplot-cli/cli.rs` | `tests/cli.rs` |
| Ruby numeric compatibility | `ruby.rs` | `youplot-core/tests/ruby_compat.rs` |

## Tests

```sh
cargo test                                   # everything
cargo test -p youplot-core                   # numeric/DSV/stats unit tests
cargo test -p youplot-cli --test lineplot    # one feature
cargo test -p youplot-cli --test histogram
cargo test -p youplot-cli --test config
```

The CLI tests in `crates/youplot-cli/tests/fixtures/` are the golden outputs
from the original Ruby YouPlot test suite. They compare the exact bytes of
stdout/stderr, so they cover both plot geometry and formatting.

## Compatibility notes

- Empty unquoted CSV fields are `nil` (rendered as `0.0` when converted to a
  number), quoted `""` is an empty string.
- Float formatting and rounding follow `ruby/numeric.c`, including the
  scaled-arithmetic artifacts of `Float#round/ceil/floor(ndigits)`.
- `--encoding` supports the labels understood by `encoding_rs`; UTF-16LE/BE
  keep a leading BOM like Ruby does.
- Progressive mode forces colored rendering into a StringIO-like buffer, then
  writes the same cursor movement/cleanup sequences as Ruby.
- Extreme magnitudes (>1e24) inherit Ruby's own floating-point rounding
  quirks; this is intentional.
