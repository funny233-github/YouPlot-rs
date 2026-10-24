fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match youplot_cli::cli::parse(args) {
        Ok(cli) => std::process::exit(youplot_cli::command::run(cli)),
        Err(e) => {
            eprintln!("YouPlot: {}", e);
            std::process::exit(1);
        }
    }
}
