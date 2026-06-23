use crate::core::date_utils::{parse_date, ExtractedDate, get_yesterday};
use crate::excel::reader::read_date_value;

pub fn extract_date(file_path: &str, date_column: &str, force_precision: &str) -> ExtractedDate {
    if date_column.is_empty() {
        return get_yesterday();
    }

    let raw_value = read_date_value(file_path, date_column);
    let raw_str = match raw_value {
        Some(v) => v,
        None => return get_yesterday(),
    };

    let extracted = match parse_date(&raw_str) {
        Some(e) => e,
        None => return get_yesterday(),
    };

    let mut result = extracted;
    if force_precision == "day" {
        result.precision = "day".to_string();
    } else if force_precision == "month" {
        result.precision = "month".to_string();
    }

    result
}
