use crate::afm::AfmFile;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug)]
pub struct Finding {
    pub line: usize,
    pub severity: Severity,
    pub message: String,
}

// Keys the Adobe AFM spec calls out as required for a well-formed font
// program, beyond FontName itself. Missing one isn't fatal to parsing,
// but it means downstream tools may fall back to guesses.
const RECOMMENDED_KEYS: &[&str] = &[
    "FontBBox",
    "UnderlinePosition",
    "UnderlineThickness",
    "Version",
    "Ascender",
    "Descender",
];

pub fn lint(afm: &AfmFile, lenient: bool) -> Vec<Finding> {
    let mut findings = Vec::new();

    if !afm.has_start_font_metrics {
        findings.push(Finding {
            line: 1,
            severity: Severity::Error,
            message: "missing StartFontMetrics header".to_string(),
        });
    }

    if !afm.has_end_font_metrics {
        findings.push(Finding {
            line: afm.last_line.max(1),
            severity: Severity::Error,
            message: "missing EndFontMetrics footer".to_string(),
        });
    }

    if !afm.header.contains_key("FontName") {
        findings.push(Finding {
            line: 1,
            severity: Severity::Error,
            message: "missing required FontName entry".to_string(),
        });
    }

    for key in RECOMMENDED_KEYS {
        let present = afm.header.contains_key(*key) || (*key == "FontBBox" && afm.bbox.is_some());
        if !present {
            let severity = if lenient { Severity::Warning } else { Severity::Error };
            findings.push(Finding {
                line: afm.start_char_metrics_line.unwrap_or(1),
                severity,
                message: format!("missing recommended entry {key}"),
            });
        }
    }

    if let (Some(start), None) = (afm.start_char_metrics_line, afm.end_char_metrics_line) {
        findings.push(Finding {
            line: start,
            severity: Severity::Error,
            message: "StartCharMetrics is never closed with EndCharMetrics".to_string(),
        });
    }

    if let Some(declared) = afm.declared_char_count {
        let actual = afm.char_metrics.len() as i64;
        if declared != actual {
            findings.push(Finding {
                line: afm.start_char_metrics_line.unwrap_or(1),
                severity: Severity::Error,
                message: format!(
                    "StartCharMetrics declares {declared} glyphs but {actual} were found"
                ),
            });
        }
    }

    if let (Some(bbox), Some(line)) = (afm.bbox, afm.bbox_line) {
        if bbox.xmin >= bbox.xmax || bbox.ymin >= bbox.ymax {
            let severity = if lenient { Severity::Warning } else { Severity::Error };
            findings.push(Finding {
                line,
                severity,
                message: "FontBBox has zero or negative area".to_string(),
            });
        }
    }

    check_char_metrics(afm, lenient, &mut findings);

    findings.sort_by_key(|f| f.line);
    findings
}

fn check_char_metrics(afm: &AfmFile, lenient: bool, findings: &mut Vec<Finding>) {
    let mut seen_names: HashMap<&str, usize> = HashMap::new();

    for metric in &afm.char_metrics {
        let name = match &metric.name {
            Some(name) => name,
            None => {
                findings.push(Finding {
                    line: metric.line,
                    severity: Severity::Error,
                    message: "character metrics line is missing an N (glyph name) field"
                        .to_string(),
                });
                continue;
            }
        };

        let width = match metric.width {
            Some(width) => width,
            None => {
                findings.push(Finding {
                    line: metric.line,
                    severity: Severity::Error,
                    message: format!("glyph '{name}' is missing a WX (width) field"),
                });
                continue;
            }
        };

        if let Some(first_line) = seen_names.get(name.as_str()) {
            findings.push(Finding {
                line: metric.line,
                severity: Severity::Error,
                message: format!("glyph '{name}' is already defined at line {first_line}"),
            });
        } else {
            seen_names.insert(name.as_str(), metric.line);
        }

        if width < 0 {
            let severity = if lenient { Severity::Warning } else { Severity::Error };
            findings.push(Finding {
                line: metric.line,
                severity,
                message: format!("glyph '{name}' has a negative advance width ({width})"),
            });
        } else if width == 0 && name != "space" && !lenient {
            findings.push(Finding {
                line: metric.line,
                severity: Severity::Warning,
                message: format!("glyph '{name}' has a zero advance width"),
            });
        }
    }
}
