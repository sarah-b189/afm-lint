use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HeaderEntry {
    pub value: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct CharMetric {
    pub line: usize,
    pub width: Option<i64>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BBox {
    pub xmin: i64,
    pub ymin: i64,
    pub xmax: i64,
    pub ymax: i64,
}

/// Parsed structure of an AFM file. Parsing never fails outright: a
/// malformed file just ends up with fields left unset, and it's the
/// linter's job to turn that into findings the caller can act on.
#[derive(Debug, Default)]
pub struct AfmFile {
    pub header: HashMap<String, HeaderEntry>,
    pub char_metrics: Vec<CharMetric>,
    pub has_start_font_metrics: bool,
    pub has_end_font_metrics: bool,
    pub start_char_metrics_line: Option<usize>,
    pub end_char_metrics_line: Option<usize>,
    pub declared_char_count: Option<i64>,
    pub bbox: Option<BBox>,
    pub bbox_line: Option<usize>,
    pub last_line: usize,
}

pub fn parse(source: &str) -> AfmFile {
    let mut file = AfmFile::default();
    let mut in_char_metrics = false;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_no = idx + 1;
        file.last_line = line_no;
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if in_char_metrics {
            if line == "EndCharMetrics" {
                file.end_char_metrics_line = Some(line_no);
                in_char_metrics = false;
            } else {
                file.char_metrics.push(parse_char_metric_line(line, line_no));
            }
            continue;
        }

        let mut parts = line.splitn(2, char::is_whitespace);
        let keyword = parts.next().unwrap_or("");
        let rest = parts.next().unwrap_or("").trim();

        match keyword {
            "StartFontMetrics" => file.has_start_font_metrics = true,
            "EndFontMetrics" => file.has_end_font_metrics = true,
            "StartCharMetrics" => {
                in_char_metrics = true;
                file.start_char_metrics_line = Some(line_no);
                file.declared_char_count = rest.parse::<i64>().ok();
            }
            "FontBBox" => {
                file.bbox_line = Some(line_no);
                file.bbox = parse_bbox(rest);
            }
            _ => {
                file.header.insert(
                    keyword.to_string(),
                    HeaderEntry { value: rest.to_string(), line: line_no },
                );
            }
        }
    }

    file
}

fn parse_bbox(rest: &str) -> Option<BBox> {
    let nums: Vec<i64> = rest
        .split_whitespace()
        .filter_map(|tok| tok.parse::<i64>().ok())
        .collect();
    if nums.len() != 4 {
        return None;
    }
    Some(BBox { xmin: nums[0], ymin: nums[1], xmax: nums[2], ymax: nums[3] })
}

// A char metrics line looks like:
//   C 32 ; WX 278 ; N space ;
// Fields are separated by ';' and each field is "KEY value...".
fn parse_char_metric_line(line: &str, line_no: usize) -> CharMetric {
    let mut width = None;
    let mut name = None;

    for field in line.split(';') {
        let field = field.trim();
        if field.is_empty() {
            continue;
        }
        let mut tokens = field.split_whitespace();
        let key = tokens.next().unwrap_or("");
        match key {
            "WX" | "W0X" => width = tokens.next().and_then(|t| t.parse::<i64>().ok()),
            "N" => name = tokens.next().map(|t| t.to_string()),
            _ => {}
        }
    }

    CharMetric { line: line_no, width, name }
}
