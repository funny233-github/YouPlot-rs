mod common;
use common::{fixture, run};

fn text(b: &[u8]) -> String { String::from_utf8_lossy(b).into_owned() }

#[test]
fn config_file_sets_width() {
    let r = run_in_fixture(
        &["line", "--config", "cfg.yml"],
        "simple.tsv",
        |dir| std::fs::write(dir.join("cfg.yml"), "width: 17\n").unwrap(),
    );
    assert_eq!(r.code(), 0, "{}", text(r.stderr()));
    assert_eq!(r.stderr(), fixture("simple-lineplot-width-17.txt").as_slice());
}

fn run_in_fixture<F: FnOnce(&std::path::Path)>(
    args: &[&str],
    stdin_fixture: &str,
    setup: F,
) -> common::Run {
    common::run_in(args, &fixture(stdin_fixture), setup)
}

#[test]
fn config_is_discovered_in_cwd() {
    let r = run_in_fixture(&["line"], "simple.tsv", |dir| {
        std::fs::write(dir.join(".youplot.yml"), "width: 17\n").unwrap();
    });
    assert_eq!(r.code(), 0, "{}", text(r.stderr()));
    assert_eq!(r.stderr(), fixture("simple-lineplot-width-17.txt").as_slice());
}

#[test]
fn cli_overrides_config() {
    let r = run_in_fixture(
        &["line", "-w", "40", "--config", "cfg.yml"],
        "simple.tsv",
        |dir| std::fs::write(dir.join("cfg.yml"), "width: 17\n").unwrap(),
    );
    assert_eq!(r.code(), 0, "{}", text(r.stderr()));
    assert_eq!(r.stderr(), fixture("simple-lineplot.txt").as_slice());
}

#[test]
fn unknown_config_key_is_an_error() {
    let r = run_in_fixture(&["line", "--config", "cfg.yml"], "simple.tsv", |dir| {
        std::fs::write(dir.join("cfg.yml"), "nope: 1\n").unwrap();
    });
    assert_ne!(r.code(), 0);
    assert!(text(r.stderr()).contains("Unknown option/param in config file: nope"));
}

#[test]
fn config_info_without_file() {
    let r = run(&["--config"], b"");
    assert_eq!(r.code(), 0);
    assert!(text(r.stdout()).contains("Configuration file not found."));
    assert!(text(r.stdout()).contains(".youplot.yml"));
}

#[test]
fn config_info_with_file() {
    let r = common::run_in(&["--config"], b"", |dir| {
        std::fs::write(dir.join(".youplotrc"), "width: 40\n").unwrap();
    });
    assert_eq!(r.code(), 0);
    let out = text(r.stdout());
    assert!(out.contains("config file :"));
    assert!(out.contains("\"width\"=>40"));
}
