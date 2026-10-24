use crate::csv;
use crate::{Data, Error, Result};

pub fn parse(
    input: &str,
    delimiter: &str,
    headers: bool,
    transpose: bool,
) -> Result<Data> {
    let mut arr = csv::parse(input, delimiter)
        .map_err(|e| Error::Message(format!("__CSV__{}", e.message)))?;

    arr.retain(|row| !(row.is_empty() || row.iter().all(|v| v.is_none())));

    let header_row = get_headers(&arr, headers, transpose);
    let series = get_series(&arr, headers, transpose);
    build_data(header_row, series)
}

pub fn transpose2(arr: &[Vec<Option<String>>]) -> Vec<Vec<Option<String>>> {
    let max = arr.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut out = Vec::with_capacity(max);
    for i in 0..max {
        let mut col = Vec::with_capacity(arr.len());
        for row in arr {
            col.push(row.get(i).cloned().unwrap_or(None));
        }
        out.push(col);
    }
    out
}

pub fn build_data(
    headers: Option<Vec<Option<String>>>,
    series: Vec<Vec<Option<String>>>,
) -> Result<Data> {
    let Some(headers) = headers else {
        return Ok(Data { headers: None, series });
    };

    // Warnings, matching DSV.validate_headers.
    if headers.iter().any(|h| h.is_none()) {
        eprintln!("\u{1b}[35mHeaders contains nil in it.\u{1b}[0m");
    }
    if headers.iter().any(|h| h.as_deref() == Some("")) {
        eprintln!("\u{1b}[35mHeaders contains \"\" in it.\u{1b}[0m");
    }

    let h_size = headers.len();
    let s_size = series.len();
    if h_size > s_size {
        eprintln!("\u{1b}[35mThe number of headers is greater than the number of series.\u{1b}[0m");
        return Err(Error::Message(String::new()));
    } else if h_size < s_size {
        eprintln!("\u{1b}[35mThe number of headers is less than the number of series.\u{1b}[0m");
        return Err(Error::Message(String::new()));
    }
    Ok(Data {
        headers: Some(headers),
        series,
    })
}

fn get_headers(
    arr: &[Vec<Option<String>>],
    headers: bool,
    transpose: bool,
) -> Option<Vec<Option<String>>> {
    if !headers {
        return None;
    }
    if transpose {
        return Some(arr.iter().map(|r| r.first().cloned().unwrap_or(None)).collect());
    }
    Some(arr.first().cloned().unwrap_or_default())
}

fn get_series(
    arr: &[Vec<Option<String>>],
    headers: bool,
    transpose: bool,
) -> Vec<Vec<Option<String>>> {
    if !headers {
        return if transpose {
            arr.to_vec()
        } else {
            transpose2(arr)
        };
    }
    if arr.len() == 1 {
        let n = arr[0].len();
        return vec![Vec::new(); n];
    }
    if transpose {
        return arr
            .iter()
            .map(|row| row.iter().skip(1).cloned().collect())
            .collect();
    }
    transpose2(&arr[1..])
}
