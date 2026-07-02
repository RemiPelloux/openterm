use super::types::ResultRecord;

const DAY_MS: f64 = 86_400_000.0;
const HALF_LIFE_DAYS: f64 = 30.0;
const BASE_LOG_ODDS: f64 = -1.4;
const FRESHNESS_WEIGHT: f64 = 2.2;
const ACCESS_WEIGHT: f64 = 1.1;
const FAILURE_WEIGHT: f64 = 0.8;
const COMPACTNESS_WEIGHT: f64 = 0.4;
const ACCESS_SATURATION: f64 = 3.0;
const SIZE_SCALE_BYTES: f64 = 1_048_576.0;

pub struct RetentionScore {
    pub value: f64,
    pub freshness: f64,
    pub access: f64,
    pub diagnostic: f64,
}

struct Features {
    freshness: f64,
    access: f64,
    diagnostic: f64,
    compactness: f64,
}

pub fn score(record: &ResultRecord, now_ms: u64) -> RetentionScore {
    if record.pinned {
        return RetentionScore {
            value: 1.0,
            freshness: 1.0,
            access: 1.0,
            diagnostic: diagnostic_value(record.exit_code),
        };
    }
    let features = features(record, now_ms);
    let log_odds = BASE_LOG_ODDS
        + FRESHNESS_WEIGHT * features.freshness
        + ACCESS_WEIGHT * features.access
        + FAILURE_WEIGHT * features.diagnostic
        + COMPACTNESS_WEIGHT * features.compactness;
    RetentionScore {
        value: logistic(log_odds),
        freshness: features.freshness,
        access: features.access,
        diagnostic: features.diagnostic,
    }
}

fn features(record: &ResultRecord, now_ms: u64) -> Features {
    let age_ms = bounded_u32(now_ms.saturating_sub(record.started_ms));
    let age_days = f64::from(age_ms) / DAY_MS;
    let freshness = 2.0_f64.powf(-age_days / HALF_LIFE_DAYS);
    let access_count = bounded_u32(record.access_count);
    let access = 1.0 - (-f64::from(access_count) / ACCESS_SATURATION).exp();
    let raw_bytes = record.stdout_bytes.saturating_add(record.stderr_bytes);
    let bytes = f64::from(bounded_u32(raw_bytes));
    Features {
        freshness,
        access,
        diagnostic: diagnostic_value(record.exit_code),
        compactness: 1.0 / (1.0 + bytes / SIZE_SCALE_BYTES).sqrt(),
    }
}

fn diagnostic_value(exit_code: i32) -> f64 {
    if exit_code == 0 { 0.25 } else { 1.0 }
}

fn logistic(value: f64) -> f64 {
    1.0 / (1.0 + (-value).exp())
}

fn bounded_u32(value: u64) -> u32 {
    value.try_into().unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::score;
    use crate::results::types::ResultRecord;

    #[test]
    fn recent_results_outscore_stale_results() {
        let recent = record(1_000, 0, 0, false);
        let stale = record(1_000, 0, 0, false);
        assert!(score(&recent, 2_000).value > score(&stale, 10_000_000_000).value);
    }

    #[test]
    fn access_failure_and_pinning_increase_retention() {
        let ordinary = record(1_000, 0, 0, false);
        let valuable = record(1_000, 1, 8, false);
        let pinned = record(1_000, 0, 0, true);
        assert!(score(&valuable, 2_000).value > score(&ordinary, 2_000).value);
        assert!((score(&pinned, 2_000).value - 1.0).abs() < f64::EPSILON);
    }

    fn record(started_ms: u64, exit_code: i32, access_count: u64, pinned: bool) -> ResultRecord {
        ResultRecord {
            id: "res_test".into(),
            command: "ai-run".into(),
            executable: "true".into(),
            argv: vec!["true".into()],
            cwd: "/tmp".into(),
            exit_code,
            started_ms,
            duration_ms: 1,
            stdout_bytes: 100,
            stderr_bytes: 0,
            evidence_kind: "raw-command-output".into(),
            pinned,
            trashed_ms: None,
            access_count,
            last_accessed_ms: None,
        }
    }
}
