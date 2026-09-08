// Golden-file tests: run the built binary against a fixture .afm file and
// compare its stdout byte-for-byte against a checked-in expected output.
// This exercises the parser and the rule engine together, the same way a
// user invoking the CLI would, instead of poking at their internals.

use std::path::Path;
use std::process::Command;

fn run(args: &[&str]) -> (String, i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_afm-lint"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to run afm-lint");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not valid utf-8");
    (stdout, output.status.code().unwrap_or(-1))
}

fn golden(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read golden file {}: {err}", path.display()))
}

#[test]
fn clean_file_reports_no_issues() {
    let (stdout, code) = run(&["tests/fixtures/clean.afm"]);
    assert_eq!(stdout, golden("clean.strict.stdout"));
    assert_eq!(code, 0);
}

#[test]
fn broken_file_strict() {
    let (stdout, code) = run(&["tests/fixtures/broken.afm"]);
    assert_eq!(stdout, golden("broken.strict.stdout"));
    assert_eq!(code, 1);
}

#[test]
fn broken_file_lenient_still_fails_on_structural_errors() {
    let (stdout, code) = run(&["--lenient", "tests/fixtures/broken.afm"]);
    assert_eq!(stdout, golden("broken.lenient.stdout"));
    assert_eq!(code, 1);
}

#[test]
fn soft_issues_strict_fails() {
    let (stdout, code) = run(&["tests/fixtures/soft-issues.afm"]);
    assert_eq!(stdout, golden("soft-issues.strict.stdout"));
    assert_eq!(code, 1);
}

#[test]
fn soft_issues_lenient_downgrades_to_warnings_and_passes() {
    let (stdout, code) = run(&["--lenient", "tests/fixtures/soft-issues.afm"]);
    assert_eq!(stdout, golden("soft-issues.lenient.stdout"));
    assert_eq!(code, 0);
}

#[test]
fn multiple_file_arguments_are_all_linted_in_order() {
    let (stdout, code) = run(&["tests/fixtures/clean.afm", "tests/fixtures/broken.afm"]);
    let expected = format!("{}{}", golden("clean.strict.stdout"), golden("broken.strict.stdout"));
    assert_eq!(stdout, expected);
    assert_eq!(code, 1);
}

#[test]
fn directory_argument_lints_every_afm_file_inside_it() {
    let (stdout, code) = run(&["tests/fixtures/multi"]);
    assert_eq!(stdout, golden("multi.strict.stdout"));
    assert_eq!(code, 1);
}
