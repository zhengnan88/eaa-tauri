use std::path::Path;
use calamine::{Reader, open_workbook, Data, CellErrorType};

/// Check if file is an Excel file by extension
pub fn is_excel_file(file_path: &Path) -> bool {
    match file_path.extension().and_then(|e| e.to_str()) {
        Some(ext) => {
            let ext = ext.to_lowercase();
            ext == "xlsx" || ext == "xlsm" || ext == "xls"
        }
        None => false,
    }
}

/// Convert a calamine Data value to string
fn data_to_string(data: &Data) -> Option<String> {
    match data {
        Data::Empty => None,
        Data::String(s) => Some(s.clone()),
        Data::Float(f) => {
            if *f == (*f as i64) as f64 && (*f as i64).abs() < i64::MAX {
                Some((*f as i64).to_string())
            } else {
                Some(f.to_string())
            }
        }
        Data::Int(i) => Some(i.to_string()),
        Data::Bool(b) => Some(b.to_string()),
        Data::DateTime(_) => {
            // Use Display trait for DateTime formatting
            Some(data.to_string())
        }
        Data::DateTimeIso(s) => Some(s.clone()),
        Data::DurationIso(s) => Some(s.clone()),
        Data::Error(e) => {
            match e {
                CellErrorType::Null => None,
                CellErrorType::Div0 => Some("#DIV/0!".to_string()),
                CellErrorType::Value => Some("#VALUE!".to_string()),
                CellErrorType::Ref => Some("#REF!".to_string()),
                CellErrorType::Name => Some("#NAME?".to_string()),
                CellErrorType::Num => Some("#NUM!".to_string()),
                CellErrorType::NA => Some("#N/A".to_string()),
                CellErrorType::GettingData => Some("...".to_string()),
            }
        }
    }
}

/// Read headers from an Excel file
/// Returns (headers, detected_row) where detected_row is 1-based
pub fn read_headers(file_path: &str, feature_row: u32) -> Option<(Vec<String>, u32)> {
    let path = Path::new(file_path);
    if !path.exists() {
        return None;
    }

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext != "xlsx" && ext != "xlsm" && ext != "xls" {
        return None;
    }

    let mut workbook: calamine::Xlsx<_> = open_workbook(file_path).ok()?;
    let sheet_names = workbook.sheet_names().to_owned();
    let sheet_name = sheet_names.first()?;
    let range = workbook.worksheet_range(sheet_name).ok()?;

    let start_row = (feature_row.max(1) - 1) as usize;
    let total_rows = range.height();
    let total_cols = range.width();

    // Try to find a valid header row (start from feature_row, try up to 3 rows)
    for row_idx in start_row..(start_row + 3).min(total_rows) {
        let headers: Vec<String> = (0..total_cols)
            .filter_map(|col| {
                range.get((row_idx, col)).and_then(|d| data_to_string(d))
            })
            .collect();

        let non_empty = headers.iter().filter(|h| !h.trim().is_empty()).count();
        if non_empty >= 3 {
            return Some((headers, (row_idx + 1) as u32));
        }
    }

    // Fallback: try row 0 if feature_row didn't work
    if start_row > 0 && total_rows > 0 {
        let headers: Vec<String> = (0..total_cols)
            .filter_map(|col| {
                range.get((0, col)).and_then(|d| data_to_string(d))
            })
            .collect();
        let non_empty = headers.iter().filter(|h| !h.trim().is_empty()).count();
        if non_empty >= 3 {
            return Some((headers, 1));
        }
    }

    None
}

/// Read the first non-empty value from a specific column
/// The column is identified by its header name in the first rows
pub fn read_date_value(file_path: &str, column_name: &str) -> Option<String> {
    let path = Path::new(file_path);
    if !path.exists() || column_name.is_empty() {
        return None;
    }

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext != "xlsx" && ext != "xlsm" && ext != "xls" {
        return None;
    }

    let mut workbook: calamine::Xlsx<_> = open_workbook(file_path).ok()?;
    let sheet_names = workbook.sheet_names().to_owned();
    let sheet_name = sheet_names.first()?;
    let range = workbook.worksheet_range(sheet_name).ok()?;

    let total_rows = range.height();
    let total_cols = range.width();

    if total_rows == 0 || total_cols == 0 {
        return None;
    }

    // Find column index by searching header rows (rows 0-2)
    let mut col_idx = None;
    for row in 0..3.min(total_rows) {
        for col in 0..total_cols {
            if let Some(s) = range.get((row, col)).and_then(|d| data_to_string(d)) {
                if s == column_name {
                    col_idx = Some(col);
                    break;
                }
            }
        }
        if col_idx.is_some() {
            break;
        }
    }

    let col = col_idx?;

    // Read first non-empty value from that column (skip header rows)
    for row in 1..total_rows {
        if let Some(s) = range.get((row, col)).and_then(|d| data_to_string(d)) {
            if !s.trim().is_empty() {
                return Some(s);
            }
        }
    }

    None
}

/// Get file info (row count, column count, file size)
pub fn get_file_info(file_path: &str) -> Option<(usize, usize, u64)> {
    let path = Path::new(file_path);
    if !path.exists() {
        return None;
    }

    let file_size = std::fs::metadata(path).ok()?.len();

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext != "xlsx" && ext != "xlsm" && ext != "xls" {
        return None;
    }

    let mut workbook: calamine::Xlsx<_> = open_workbook(file_path).ok()?;
    let sheet_names = workbook.sheet_names().to_owned();
    let sheet_name = sheet_names.first()?;
    let range = workbook.worksheet_range(sheet_name).ok()?;

    Some((range.height(), range.width(), file_size))
}
