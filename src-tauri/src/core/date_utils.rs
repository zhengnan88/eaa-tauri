use chrono::{NaiveDate, Datelike};

#[derive(Debug, Clone)]
pub struct ExtractedDate {
    pub date_value: NaiveDate,
    pub precision: String,
}

pub fn parse_date(value: &str) -> Option<ExtractedDate> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    // Try numeric formats
    if let Ok(num) = value.parse::<i64>() {
        let s = num.to_string();
        if s.len() == 8 {
            if let Ok(d) = NaiveDate::parse_from_str(&s, "%Y%m%d") {
                return Some(ExtractedDate {
                    date_value: d,
                    precision: "day".to_string(),
                });
            }
        }
        if s.len() == 6 {
            let formatted = format!("{}01", s);
            if let Ok(d) = NaiveDate::parse_from_str(&formatted, "%Y%m%d") {
                return Some(ExtractedDate {
                    date_value: d,
                    precision: "month".to_string(),
                });
            }
        }
    }

    // Try date range format: YYYY-MM-DD~YYYY-MM-DD
    if value.find('~').is_some() {
        let parts: Vec<&str> = value.split('~').collect();
        if parts.len() == 2 {
            if let Ok(d) = NaiveDate::parse_from_str(parts[0], "%Y-%m-%d") {
                return Some(ExtractedDate {
                    date_value: d,
                    precision: "month".to_string(),
                });
            }
        }
    }

    // Try Chinese format: YYYY年MM月DD日
    let chinese_day_pattern = regex_lite::Regex::new(r"(\d{4})年(\d{1,2})月(\d{1,2})日").ok();
    if let Some(re) = &chinese_day_pattern {
        if let Some(caps) = re.captures(value) {
            let y: u32 = caps[1].parse().ok()?;
            let m: u32 = caps[2].parse().ok()?;
            let d: u32 = caps[3].parse().ok()?;
            if let Some(date) = NaiveDate::from_ymd_opt(y as i32, m, d) {
                return Some(ExtractedDate {
                    date_value: date,
                    precision: "day".to_string(),
                });
            }
        }
    }

    // Try standard formats
    let day_formats = ["%Y-%m-%d", "%Y/%m/%d", "%Y年%m月%d日"];
    for fmt in &day_formats {
        if let Ok(d) = NaiveDate::parse_from_str(value, fmt) {
            return Some(ExtractedDate {
                date_value: d,
                precision: "day".to_string(),
            });
        }
    }

    // Try month formats by appending day
    let padded = format!("{}-01", value);
    if let Ok(d) = NaiveDate::parse_from_str(&padded, "%Y-%m-%d") {
        return Some(ExtractedDate {
            date_value: d,
            precision: "month".to_string(),
        });
    }

    // Try Chinese month format
    if let Some(caps) = regex_lite::Regex::new(r"(\d{4})年(\d{1,2})月$").ok().and_then(|re| re.captures(value)) {
        let y: i32 = caps[1].parse().ok()?;
        let m: u32 = caps[2].parse().ok()?;
        if let Some(d) = NaiveDate::from_ymd_opt(y, m, 1) {
            return Some(ExtractedDate {
                date_value: d,
                precision: "month".to_string(),
            });
        }
    }

    None
}

pub fn parse_date_from_cell(value: &str) -> Option<ExtractedDate> {
    parse_date(value)
}

pub fn format_date(extracted: &ExtractedDate, pattern: &str) -> String {
    let d = &extracted.date_value;
    let mut result = pattern.to_string();
    result = result.replace("yyyy", &format!("{:04}", d.year()));
    result = result.replace("MM", &format!("{:02}", d.month()));
    result = result.replace("dd", &format!("{:02}", d.day()));
    result = result.replace("HH", "00");
    result = result.replace("mm", "00");
    result = result.replace("ss", "00");
    result
}

pub fn get_yesterday() -> ExtractedDate {
    use chrono::Duration;
    let today = chrono::Local::now().date_naive();
    ExtractedDate {
        date_value: today - Duration::days(1),
        precision: "day".to_string(),
    }
}
