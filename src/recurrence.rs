use anyhow::{Result, bail, ensure};
use chrono::{DateTime, Local, LocalResult, TimeZone, Utc};
use croner::parser::{CronParser, Seconds, Year};

const MAX_INDIVIDUAL_MISSED: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissedPolicy {
    Skip,
    Consolidate,
    Sequence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissedRange {
    pub count: u64,
    pub from: i64,
    pub through: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryPlan {
    pub individual: Vec<i64>,
    pub consolidated: Option<MissedRange>,
    pub new_cursor: i64,
}

pub fn next_local_occurrence(expression: &str, after_micros: i64) -> Result<i64> {
    next_occurrence_in(expression, after_micros, &Local)
}

fn next_occurrence_in<Tz: TimeZone>(
    expression: &str,
    after_micros: i64,
    timezone: &Tz,
) -> Result<i64> {
    let cron = parse_cron(expression)?;
    let after = DateTime::<Utc>::from_timestamp_micros(after_micros)
        .ok_or_else(|| anyhow::anyhow!("timestamp is out of range"))?;
    let local_after = after.with_timezone(timezone).naive_local();
    let mut civil_cursor = local_after;

    // Search over matching civil times, then resolve each against the OS timezone.
    for _ in 0..5_000_000 {
        let candidate = cron.find_next_occurrence(&civil_cursor, false)?;
        match timezone.from_local_datetime(&candidate) {
            LocalResult::Single(value) if value.timestamp_micros() > after_micros => {
                return Ok(value.timestamp_micros());
            }
            LocalResult::Ambiguous(first, _) if first.timestamp_micros() > after_micros => {
                return Ok(first.timestamp_micros());
            }
            LocalResult::Single(_) | LocalResult::Ambiguous(_, _) | LocalResult::None => {
                civil_cursor = candidate;
            }
        }
    }
    bail!("no matching local occurrence found within search limit")
}

pub fn recover_missed(
    expression: &str,
    cursor_micros: i64,
    through_micros: i64,
    policy: MissedPolicy,
) -> Result<RecoveryPlan> {
    ensure!(
        through_micros >= cursor_micros,
        "recovery range is inverted"
    );
    if policy == MissedPolicy::Skip {
        return Ok(RecoveryPlan {
            individual: Vec::new(),
            consolidated: None,
            new_cursor: through_micros,
        });
    }
    let mut cursor = cursor_micros;
    let mut individual = Vec::new();
    let mut consolidated: Option<MissedRange> = None;

    loop {
        let next = next_local_occurrence(expression, cursor)?;
        if next > through_micros {
            break;
        }
        cursor = next;
        match policy {
            MissedPolicy::Skip => {
                unreachable!("skip policy returned before enumerating occurrences")
            }
            MissedPolicy::Consolidate => add_to_range(&mut consolidated, next),
            MissedPolicy::Sequence if individual.len() < MAX_INDIVIDUAL_MISSED => {
                individual.push(next);
            }
            MissedPolicy::Sequence => add_to_range(&mut consolidated, next),
        }
    }

    Ok(RecoveryPlan {
        individual,
        consolidated,
        new_cursor: cursor.max(through_micros),
    })
}

fn add_to_range(range: &mut Option<MissedRange>, timestamp: i64) {
    match range {
        Some(range) => {
            range.count += 1;
            range.through = timestamp;
        }
        None => {
            *range = Some(MissedRange {
                count: 1,
                from: timestamp,
                through: timestamp,
            })
        }
    }
}

pub fn parse_cron(expression: &str) -> Result<croner::Cron> {
    let fields: Vec<_> = expression.split_whitespace().collect();
    ensure!(
        fields.len() == 5,
        "cron expression must contain exactly five fields"
    );
    ensure!(
        expression
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, ' ' | '\t' | '*' | ',' | '-' | '/')),
        "cron expression contains unsupported characters"
    );
    let parser = CronParser::builder()
        .seconds(Seconds::Disallowed)
        .year(Year::Disallowed)
        .build();
    Ok(parser.parse(expression)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::Europe::Stockholm;

    fn micros(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> i64 {
        Stockholm
            .with_ymd_and_hms(year, month, day, hour, minute, second)
            .earliest()
            .unwrap()
            .timestamp_micros()
    }

    #[test]
    fn rejects_extended_or_non_five_field_cron() {
        assert!(parse_cron("0 0 1 * 1").is_ok());
        assert!(parse_cron("0 0 0 1 * *").is_err());
        assert!(parse_cron("0 0 L * *").is_err());
        assert!(parse_cron("0 0 * JAN *").is_err());
    }

    #[test]
    fn restricted_day_of_month_and_weekday_use_traditional_or() {
        let parser = CronParser::builder()
            .seconds(Seconds::Disallowed)
            .year(Year::Disallowed)
            .build();
        let cron = parser.parse("0 0 1 * 1").unwrap();
        let sunday = chrono::NaiveDate::from_ymd_opt(2025, 1, 5)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let next = cron.find_next_occurrence(&sunday, false).unwrap();
        assert_eq!(
            next.date(),
            chrono::NaiveDate::from_ymd_opt(2025, 1, 6).unwrap()
        );
    }

    // Covers: BDD-JRN-001-09 BDD-JRN-002-16
    #[test]
    fn resolves_spring_gap_by_skipping_nonexistent_local_time() {
        let after = micros(2025, 3, 30, 1, 59, 59);
        let next = next_occurrence_in("30 2 * * *", after, &Stockholm).unwrap();
        assert_eq!(next, micros(2025, 3, 31, 2, 30, 0));
    }

    // Covers: BDD-JRN-001-09 BDD-JRN-002-16
    #[test]
    fn emits_only_first_copy_of_autumn_overlap() {
        let after = micros(2025, 10, 26, 1, 59, 59);
        let next = next_occurrence_in("30 2 * * *", after, &Stockholm).unwrap();
        assert_eq!(next, micros(2025, 10, 26, 2, 30, 0));
        let after_first = micros(2025, 10, 26, 2, 30, 0);
        assert_eq!(
            next_occurrence_in("30 2 * * *", after_first, &Stockholm).unwrap(),
            micros(2025, 10, 27, 2, 30, 0)
        );
    }

    // Covers: BDD-JRN-001-07 BDD-JRN-002-11
    #[test]
    fn recovery_caps_individual_runs_and_consolidates_the_excess() {
        let from = chrono::NaiveDate::from_ymd_opt(2025, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp_micros();
        let through = from + 120 * 60 * 1_000_000;
        let plan = recover_missed("* * * * *", from, through, MissedPolicy::Sequence).unwrap();
        assert_eq!(plan.individual.len(), 100);
        assert_eq!(plan.consolidated.unwrap().count, 20);
        assert_eq!(plan.new_cursor, through);
    }

    #[test]
    fn recovery_skip_advances_cursor_and_consolidate_reports_exact_range() {
        let from = chrono::NaiveDate::from_ymd_opt(2025, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp_micros();
        let through = from + 3 * 60 * 1_000_000;
        let skipped = recover_missed("* * * * *", from, through, MissedPolicy::Skip).unwrap();
        assert!(skipped.individual.is_empty());
        assert!(skipped.consolidated.is_none());
        assert_eq!(skipped.new_cursor, through);

        let merged = recover_missed("* * * * *", from, through, MissedPolicy::Consolidate).unwrap();
        let range = merged.consolidated.unwrap();
        assert_eq!(range.count, 3);
        assert_eq!(range.from, from + 60 * 1_000_000);
        assert_eq!(range.through, through);
    }
}
