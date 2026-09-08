mod afm;
mod lint;

use lint::Severity;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut lenient = false;
    let mut inputs: Vec<String> = Vec::new();

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
            other => inputs.push(other.to_string()),
        }
    }

    if inputs.is_empty() {
        print_usage();
        return ExitCode::from(2);
    }

    let mut paths = Vec::new();
    let mut had_io_error = false;
    for input in &inputs {
        if let Err(err) = collect_afm_paths(Path::new(input), &mut paths) {
            eprintln!("{input}: {err}");
            had_io_error = true;
        }
    }

    if paths.is_empty() {
        if !had_io_error {
            eprintln!("no .afm files found in the given path(s)");
        }
        return ExitCode::from(2);
    }

    let mut had_error = false;
    for path in &paths {
        match lint_file(path, lenient) {
            Ok(file_had_error) => had_error |= file_had_error,
            Err(err) => {
                eprintln!("{}: {err}", path.display());
                had_io_error = true;
            }
        }
    }

    if had_io_error {
        ExitCode::from(2)
    } else if had_error {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn lint_file(path: &Path, lenient: bool) -> io::Result<bool> {
    let source = fs::read_to_string(path)?;
    let parsed = afm::parse(&source);
    let findings = lint::lint(&parsed, lenient);
    let display = path.display();

    let mut had_error = false;
    for finding in &findings {
        let tag = match finding.severity {
            Severity::Error => {
                had_error = true;
                "error"
            }
            Severity::Warning => "warning",
        };
        println!("{display}:{}: {tag}: {}", finding.line, finding.message);
    }

    if findings.is_empty() {
        println!("{display}: no issues found");
    }

    Ok(had_error)
}

// A bare file argument is linted as given, even if its extension isn't
// `.afm` - the user named it explicitly. A directory is walked recursively
// and only files ending in `.afm` are collected, with entries sorted so
// output order doesn't depend on the filesystem's directory listing order.
fn collect_afm_paths(path: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    let metadata = fs::metadata(path)?;
    if metadata.is_dir() {
        let mut entries: Vec<PathBuf> =
            fs::read_dir(path)?.filter_map(|entry| entry.ok()).map(|entry| entry.path()).collect();
        entries.sort();
        for entry in entries {
            if entry.is_dir() {
                collect_afm_paths(&entry, out)?;
            } else {
                let is_afm = entry
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("afm"));
                if is_afm {
                    out.push(entry);
                }
            }
        }
    } else {
        out.push(path.to_path_buf());
    }
    Ok(())
}

fn print_usage() {
    eprintln!("usage: afm-lint [--lenient] <file.afm | directory>...");
}
