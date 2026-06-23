use crate::core::config::Rule;
use crate::core::config::AppSettings;

pub fn normalize_column(name: &str, ignore_case: bool, ignore_space: bool) -> String {
    let mut result = name.to_string();
    if ignore_space {
        result = result.trim().to_string();
    }
    if ignore_case {
        result = result.to_lowercase();
    }
    result
}

pub fn match_all<'a>(headers: &[String], rules: &'a [Rule], settings: &AppSettings) -> Option<&'a Rule> {
    let normalized_headers: Vec<String> = headers
        .iter()
        .map(|h| normalize_column(h, settings.ignore_case, settings.ignore_space))
        .collect();

    for rule in rules {
        if rule_matches(rule, &normalized_headers, settings) {
            return Some(rule);
        }
    }
    None
}

pub fn rule_matches(rule: &Rule, normalized_headers: &[String], settings: &AppSettings) -> bool {
    for col in &rule.feature_columns {
        let normalized_col = normalize_column(col, settings.ignore_case, settings.ignore_space);
        if !normalized_headers.contains(&normalized_col) {
            return false;
        }
    }
    true
}
