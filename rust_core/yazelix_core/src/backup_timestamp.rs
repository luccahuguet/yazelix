// Test lane: default
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn compact_utc_backup_timestamp() -> String {
    let epoch_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    compact_utc_backup_timestamp_from_epoch_secs(epoch_secs)
}

fn compact_utc_backup_timestamp_from_epoch_secs(epoch_secs: i64) -> String {
    let days = epoch_secs.div_euclid(86_400);
    let seconds_of_day = epoch_secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    format!("{year:04}{month:02}{day:02}_{hour:02}{minute:02}{second:02}")
}

fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };

    (year as i32, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Regression: backup names must stay human-readable instead of regressing to opaque epoch-only suffixes.
    #[test]
    fn formats_compact_utc_backup_timestamps() {
        assert_eq!(
            compact_utc_backup_timestamp_from_epoch_secs(0),
            "19700101_000000"
        );
        assert_eq!(
            compact_utc_backup_timestamp_from_epoch_secs(1_713_398_400),
            "20240418_000000"
        );
    }
}
