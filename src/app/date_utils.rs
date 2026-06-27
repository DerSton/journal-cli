//! Utility functions for resolving, formatting, and parsing localized date formats.
//!
//! Evaluates the system's locale settings at runtime to detect the preferred date separator
//! and field order, offering localized date entry and presentation.

use crate::model::get_system_locale;
use chrono::NaiveDate;

/// Resolves the user's system locale date presentation format.
///
/// Formats a test date (`2023-11-22`) in the current system locale, inspects the
/// resulting string to find the separator (e.g. `.`, `/`, `-`) and field order, and constructs
/// a user placeholder (e.g. `"DD.MM.YYYY"`) and a prioritized list of `strftime` patterns.
///
/// # Returns
///
/// A tuple containing:
/// - A helper placeholder string for input fields (e.g. `"DD.MM.YYYY"`).
/// - A vector of format strings (`Vec<String>`), where the first format is the detected primary format.
pub fn get_date_format_info() -> (String, Vec<String>) {
    let locale = get_system_locale();

    // We format a test date: 2023-11-22 (distinct values for Y, M, D)
    let test_date = chrono::NaiveDate::from_ymd_opt(2023, 11, 22).unwrap();
    let formatted = test_date.format_localized("%x", locale).to_string();

    let has_4_digit_year = formatted.contains("2023");

    // Find non-numeric characters to detect the separator
    let mut separators: Vec<char> = formatted.chars().filter(|c| !c.is_numeric()).collect();
    separators.dedup();
    let sep = separators.first().copied().unwrap_or('-');

    let parts: Vec<&str> = formatted.split(sep).collect();

    let mut format_parts = Vec::new();
    let mut placeholder_parts = Vec::new();

    for part in parts {
        let part_trimmed = part.trim();
        if part_trimmed == "22" {
            format_parts.push("%d");
            placeholder_parts.push("DD");
        } else if part_trimmed == "11" {
            format_parts.push("%m");
            placeholder_parts.push("MM");
        } else if part_trimmed == "2023" {
            format_parts.push("%Y");
            placeholder_parts.push("YYYY");
        } else if part_trimmed == "23" {
            format_parts.push("%y");
            placeholder_parts.push("YY");
        }
    }

    if format_parts.len() != 3 {
        return (
            "YYYY-MM-DD".to_string(),
            vec![
                "%Y-%m-%d".to_string(),
                "%d.%m.%Y".to_string(),
                "%m/%d/%Y".to_string(),
                "%d/%m/%Y".to_string(),
            ],
        );
    }

    let sep_str = sep.to_string();
    let primary_format = format_parts.join(&sep_str);
    let placeholder = placeholder_parts.join(&sep_str);

    let mut formats = vec![primary_format];

    let alt_year_format = if has_4_digit_year {
        format_parts
            .iter()
            .map(|&p| if p == "%Y" { "%y" } else { p })
            .collect::<Vec<_>>()
            .join(&sep_str)
    } else {
        format_parts
            .iter()
            .map(|&p| if p == "%y" { "%Y" } else { p })
            .collect::<Vec<_>>()
            .join(&sep_str)
    };
    formats.push(alt_year_format);

    let fallbacks = vec![
        "%Y-%m-%d".to_string(),
        "%d.%m.%Y".to_string(),
        "%m/%d/%Y".to_string(),
        "%d/%m/%Y".to_string(),
    ];
    for fallback in fallbacks {
        if !formats.contains(&fallback) {
            formats.push(fallback);
        }
    }

    (placeholder, formats)
}

/// Formats a date to a string using the primary detected system format.
pub fn format_localized_date(date: NaiveDate) -> String {
    let (_, formats) = get_date_format_info();
    date.format(&formats[0]).to_string()
}

/// Parses a localized date string using the detected system date formats and standard fallbacks.
///
/// # Examples
///
/// ```
/// use journal_cli::app::parse_localized_date;
/// use chrono::NaiveDate;
///
/// // Parsing ISO date format fallback:
/// let parsed = parse_localized_date("2023-11-22").unwrap();
/// assert_eq!(parsed, NaiveDate::from_ymd_opt(2023, 11, 22).unwrap());
///
/// // Parsing European date format fallback:
/// let parsed_eu = parse_localized_date("22.11.2023").unwrap();
/// assert_eq!(parsed_eu, NaiveDate::from_ymd_opt(2023, 11, 22).unwrap());
/// ```
pub fn parse_localized_date(s: &str) -> Option<NaiveDate> {
    let (_, formats) = get_date_format_info();
    for fmt in &formats {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Some(d);
        }
    }
    None
}
