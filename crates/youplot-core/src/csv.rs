//! Minimal CSV parser emulating the parts of Ruby's CSV behaviour that
//! YouPlot relies on. Empty unquoted fields become `None` (Ruby `nil`).

#[derive(Debug)]
pub struct CsvError {
    pub message: String,
}

type Row = Vec<Option<String>>;

pub fn parse(input: &str, delimiter: &str) -> Result<Vec<Row>, CsvError> {
    if delimiter.is_empty() {
        return Err(CsvError {
            message: ":col_sep must be 1 or more characters".to_string(),
        });
    }
    let delim: Vec<char> = delimiter.chars().collect();
    let chars: Vec<char> = input.chars().collect();
    let mut rows: Vec<Row> = Vec::new();
    let mut row: Row = Vec::new();
    let mut i = 0usize;
    let n = chars.len();
    let mut at_field_start = true;
    let mut line_no = 1usize;

    while i < n {
        let c = chars[i];

        // End of record at a field boundary.
        if at_field_start && (c == '\n' || (c == '\r' && i + 1 < n && chars[i + 1] == '\n')) {
            if !row.is_empty() {
                row.push(None);
            }
            rows.push(std::mem::take(&mut row));
            i += if c == '\r' { 2 } else { 1 };
            line_no += 1;
            continue;
        }

        if at_field_start {
            if starts_with(&chars, i, &delim) {
                row.push(None);
                i += delim.len();
                continue;
            }
            if c == '"' {
                // Quoted field.
                i += 1;
                let mut field = String::new();
                let mut closed = false;
                while i < n {
                    let c = chars[i];
                    if c == '"' {
                        if i + 1 < n && chars[i + 1] == '"' {
                            field.push('"');
                            i += 2;
                            continue;
                        }
                        closed = true;
                        i += 1;
                        break;
                    }
                    if c == '\n' {
                        line_no += 1;
                    }
                    field.push(c);
                    i += 1;
                }
                if !closed {
                    return Err(CsvError {
                        message: format!("Unclosed quoted field in line {}.", line_no),
                    });
                }
                row.push(Some(field));
                if i >= n {
                    break;
                }
                if starts_with(&chars, i, &delim) {
                    i += delim.len();
                    continue;
                }
                if chars[i] == '\n' || (chars[i] == '\r' && i + 1 < n && chars[i + 1] == '\n') {
                    rows.push(std::mem::take(&mut row));
                    i += if chars[i] == '\r' { 2 } else { 1 };
                    line_no += 1;
                    continue;
                }
                return Err(CsvError {
                    message: format!(
                        "Any value after quoted field isn't allowed in line {}.",
                        line_no
                    ),
                });
            }
        }

        // Unquoted field.
        let mut field = String::new();
        let mut illegal_quote = false;
        while i < n {
            let c = chars[i];
            if c == '\n' || (c == '\r' && i + 1 < n && chars[i + 1] == '\n') {
                break;
            }
            if starts_with(&chars, i, &delim) {
                break;
            }
            if c == '"' {
                illegal_quote = true;
            }
            field.push(c);
            i += 1;
        }
        if illegal_quote {
            return Err(CsvError {
                message: format!("Illegal quoting in line {}.", line_no),
            });
        }
        if field.is_empty() {
            row.push(None);
        } else {
            row.push(Some(field));
        }
        if i >= n {
            break;
        }
        if starts_with(&chars, i, &delim) {
            i += delim.len();
            at_field_start = true;
            continue;
        }
        // Newline
        if chars[i] == '\r' {
            i += 1;
        }
        i += 1;
        rows.push(std::mem::take(&mut row));
        line_no += 1;
        at_field_start = true;
    }

    if !row.is_empty() {
        rows.push(row);
    }
    Ok(rows)
}

fn starts_with(chars: &[char], i: usize, pat: &[char]) -> bool {
    if i + pat.len() > chars.len() {
        return false;
    }
    for (k, p) in pat.iter().enumerate() {
        if chars[i + k] != *p {
            return false;
        }
    }
    true
}
