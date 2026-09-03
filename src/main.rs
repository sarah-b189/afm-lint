mod afm;
mod lint;

use lint::Severity;
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut lenient = false;
    let mut path: Option<String> = None;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--lenient" => lenient = true,
            "--help" | "-h" => {
                print_usage();
                return ExitCode::SUCCESS;
            }
            other if other.starts_with('-') => {
                eprintln!("unknown flag: {other}");
                print_usage();
                return ExitCode::from(2);
            }
            other => path = Some(other.to_string()),
        }
    }

    let path = match path {
        Some(p) => p,
        None => {
            print_usage();
            return ExitCode::from(2);
        }
    };

    let source = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("{path}: {err}");
            return ExitCode::from(2);
        }
    };

    let parsed = afm::parse(&source);
    let findings = lint::lint(&parsed, lenient);

    let mut had_error = false;
    for finding in &findings {
        let tag = match finding.severity {
            Severity::Error => {
                had_error = true;
                "error"
            }
            Severity::Warning => "warning",
        };
        println!("{path}:{}: {tag}: {}", finding.line, finding.message);
    }

    if findings.is_empty() {
        println!("{path}: no issues found");
    }

    if had_error {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn print_usage() {
    eprintln!("usage: afm-lint [--lenient] <file.afm>");
}
