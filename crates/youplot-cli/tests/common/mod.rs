#![allow(dead_code)]

//! Shared helpers for the golden CLI tests.
//!
//! Each test runs the real `uplot` binary with a fixture on stdin and compares
//! the exact bytes written to stdout/stderr with the golden output produced by
//! the original Ruby YouPlot implementation.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

pub fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixture_path(name)).unwrap_or_else(|e| panic!("fixture {name}: {e}"))
}

pub struct Run {
    pub output: Output,
    pub dir: PathBuf,
}

impl Run {
    pub fn code(&self) -> i32 {
        self.output.status.code().unwrap_or(-1)
    }
    pub fn stdout(&self) -> &[u8] {
        &self.output.stdout
    }
    pub fn stderr(&self) -> &[u8] {
        &self.output.stderr
    }
    pub fn file(&self, name: &str) -> Vec<u8> {
        std::fs::read(self.dir.join(name)).unwrap_or_default()
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn temp_dir() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        "youplot-test-{}-{}",
        std::process::id(),
        n
    ));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Run uplot with `args`, feeding `stdin` (fixture name or raw bytes).
pub fn run(args: &[&str], stdin: &[u8]) -> Run {
    run_in(args, stdin, |_| {})
}

/// Run uplot in a temp directory after running `setup` in that directory.
pub fn run_in<F: FnOnce(&std::path::Path)>(
    args: &[&str],
    stdin: &[u8],
    setup: F,
) -> Run {
    let dir = temp_dir();
    setup(&dir);
    let stdin_path = dir.join("stdin");
    let mut f = std::fs::File::create(&stdin_path).unwrap();
    f.write_all(stdin).unwrap();
    drop(f);

    let output = Command::new(env!("CARGO_BIN_EXE_uplot"))
        .args(args)
        .current_dir(&dir)
        .env("HOME", &dir)
        .env_remove("MYYOUPLOTRC")
        .stdin(std::fs::File::open(&stdin_path).unwrap())
        .output()
        .unwrap();
    Run { output, dir }
}

pub fn run_fixture(args: &[&str], stdin_fixture: &str) -> Run {
    run(args, &fixture(stdin_fixture))
}

fn show(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Assert that a plot goes to stderr by default and matches the golden file.
pub fn assert_plot(args: &[&str], stdin_fixture: &str, golden: &str) {
    let r = run_fixture(args, stdin_fixture);
    assert_eq!(r.code(), 0, "args={args:?} stderr={}", show(r.stderr()));
    let expected = fixture(golden);
    assert_eq!(
        show(r.stderr()),
        show(&expected),
        "stderr mismatch for args={args:?} stdin={stdin_fixture} golden={golden}"
    );
    assert_eq!(r.stdout(), b"", "unexpected stdout for args={args:?}");
}

/// Assert a command fails and contains a message on stderr.
pub fn assert_error(args: &[&str], stdin_fixture: &str, needle: &str) {
    let r = run_fixture(args, stdin_fixture);
    assert_ne!(r.code(), 0, "expected failure for args={args:?}");
    assert!(
        show(r.stderr()).contains(needle),
        "stderr for {args:?} did not contain {needle:?}:\n{}",
        show(r.stderr())
    );
}
