use crate::models::analytics::{EventTypeCount, TimeSeriesPoint, HourlyCount};
use chrono::{DateTime, Utc, Datelike};
use std::collections::HashMap;

/// Compute percentage breakdown for a list of (label, count) pairs.
pub fn compute_percentages(counts: Vec<(String, i64)>) -> Vec<EventTypeCount> {
    let total: i64 = counts.iter().map(|(_, c)| c).sum();
    counts
        .into_iter()
        .map(|(event_type, count)| EventTypeCount {
            percentage: if total > 0 {
                (count as f64 / total as f64) * 100.0
            } else {
                0.0
            },
            event_type,
            count,
        })
        .collect()
}

/// Fill gaps in a daily time series so every date in [start, end] has an entry.
pub fn fill_time_series_gaps(
    data: Vec<TimeSeriesPoint>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Vec<TimeSeriesPoint> {
    let map: HashMap<String, i64> = data.into_iter().map(|p| (p.date, p.count)).collect();

    let mut result = Vec::new();
    let mut current = start.date_naive();
    let end_date = end.date_naive();

    while current <= end_date {
        let key = current.format("%Y-%m-%d").to_string();
        result.push(TimeSeriesPoint {
            count: *map.get(&key).unwrap_or(&0),
            date: key,
        });
        current = current.succ_opt().unwrap_or(end_date);
    }

    result
}

/// Build a 24-slot hourly distribution from a list of (hour, count) rows.
pub fn build_hourly_distribution(rows: Vec<(i32, i64)>) -> Vec<HourlyCount> {
    let map: HashMap<i32, i64> = rows.into_iter().collect();
    (0..24)
        .map(|hour| HourlyCount {
            count: *map.get(&hour).unwrap_or(&0),
            hour,
        })
        .collect()
}

/// Calculate average, rounding to 2 decimal places.
pub fn safe_average(total: i64, count: i64) -> f64 {
    if count == 0 {
        return 0.0;
    }
    let avg = total as f64 / count as f64;
    (avg * 100.0).round() / 100.0
}

/// Serialize a Vec of rows to CSV string.
/// `headers` is the header row; `rows` is an iterator of comma-separated value strings.
pub fn to_csv(headers: &[&str], rows: Vec<Vec<String>>) -> String {
    let mut out = String::new();
    out.push_str(&headers.join(","));
    out.push('\n');
    for row in rows {
        out.push_str(&row.join(","));
        out.push('\n');
    }
    out
}

/// Escape a CSV field value (wrap in quotes if it contains commas or quotes).
pub fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
