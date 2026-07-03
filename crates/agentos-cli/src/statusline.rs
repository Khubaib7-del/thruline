//! Claude Code statusline: context %, estimated prompts remaining, and an
//! estimated usage-window reset time. Everything here is an estimate and is
//! rendered as one (`~`); fail-open like the hooks — errors print a minimal
//! line, never break the host UI.

use chrono::{DateTime, Duration, DurationRound, Local, Utc};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

const DEFAULT_CONTEXT_LIMIT: u64 = 200_000;
/// Claude Pro/Max usage windows roll every 5 hours.
const USAGE_WINDOW_HOURS: i64 = 5;

#[derive(Deserialize, Default)]
struct Input {
    #[serde(default)]
    transcript_path: Option<PathBuf>,
    #[serde(default)]
    model: Option<Model>,
}

#[derive(Deserialize, Default)]
struct Model {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    display_name: Option<String>,
}

pub fn run() {
    let mut raw = String::new();
    let _ = std::io::stdin().read_to_string(&mut raw);
    let input: Input = serde_json::from_str(&raw).unwrap_or_default();

    let mut parts: Vec<String> = Vec::new();
    if let Some(name) = input.model.as_ref().and_then(|m| m.display_name.as_ref()) {
        parts.push(name.clone());
    }
    if let Some(tp) = &input.transcript_path {
        let limit = context_limit(input.model.as_ref().and_then(|m| m.id.as_deref()));
        if let Some(stats) = context_stats(tp, limit) {
            if stats.percent <= 100 {
                parts.push(format!(
                    "ctx {}% ({}k/{}k)",
                    stats.percent,
                    stats.used / 1000,
                    limit / 1000
                ));
                match stats.prompts_left {
                    // Early-session extrapolation produces silly numbers
                    // ("~1199 prompts left"); past 99 the estimate carries
                    // no information, so cap the display.
                    Some(p) if p > 99 => parts.push("99+ prompts left".into()),
                    Some(p) => parts.push(format!("~{p} prompts left")),
                    None => {}
                }
            } else {
                // Usage beyond the assumed limit means our limit table is
                // wrong for this model — admit it instead of showing >100%.
                parts.push(format!("ctx {}k (model limit unknown)", stats.used / 1000));
            }
        }
        if let Some(reset) = reset_estimate(tp, Utc::now()) {
            parts.push(format!(
                "window resets ~{}",
                reset.with_timezone(&Local).format("%H:%M")
            ));
        }
    }
    if parts.is_empty() {
        parts.push("agentos".into());
    }
    println!("{}", parts.join(" | "));
}

fn context_limit(model_id: Option<&str>) -> u64 {
    match model_id {
        Some(id) if id.contains("[1m]") => 1_000_000,
        _ => DEFAULT_CONTEXT_LIMIT,
    }
}

struct ContextStats {
    used: u64,
    percent: u64,
    prompts_left: Option<u64>,
}

/// Walk the transcript JSONL and take per-assistant-turn context totals
/// (input + cache + output tokens of each main-chain assistant message).
fn turn_totals(lines: impl Iterator<Item = String>) -> Vec<u64> {
    let mut totals = Vec::new();
    for line in lines {
        let Ok(v) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if v["isSidechain"].as_bool() == Some(true) {
            continue;
        }
        let usage = &v["message"]["usage"];
        if !usage.is_object() {
            continue;
        }
        let n = |k: &str| usage[k].as_u64().unwrap_or(0);
        let total = n("input_tokens")
            + n("cache_creation_input_tokens")
            + n("cache_read_input_tokens")
            + n("output_tokens");
        if total > 0 {
            totals.push(total);
        }
    }
    totals
}

fn stats_from_totals(totals: &[u64], limit: u64) -> Option<ContextStats> {
    let used = *totals.last()?;
    let percent = (used * 100) / limit.max(1);
    // Average growth over the last few turns predicts prompts remaining.
    let deltas: Vec<u64> = totals
        .windows(2)
        .rev()
        .take(10)
        .map(|w| w[1].saturating_sub(w[0]))
        .filter(|d| *d > 0)
        .collect();
    let prompts_left = if deltas.is_empty() {
        None
    } else {
        let avg = deltas.iter().sum::<u64>() / deltas.len() as u64;
        (avg > 0).then(|| limit.saturating_sub(used) / avg)
    };
    Some(ContextStats {
        used,
        percent,
        prompts_left,
    })
}

fn context_stats(transcript: &Path, limit: u64) -> Option<ContextStats> {
    let file = fs::File::open(transcript).ok()?;
    let lines = BufReader::new(file).lines().map_while(Result::ok);
    stats_from_totals(&turn_totals(lines), limit)
}

/// Estimated end of the current 5-hour usage window: earliest session start
/// among recently-active transcripts, floored to the hour, plus 5 hours.
/// Approximate by design — rendered with `~`.
fn reset_estimate(transcript: &Path, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let projects_dir = transcript.parent()?.parent()?;
    let window = Duration::hours(USAGE_WINDOW_HOURS);
    let mut earliest: Option<DateTime<Utc>> = None;

    for project in fs::read_dir(projects_dir).ok()?.flatten() {
        let Ok(entries) = fs::read_dir(project.path()) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "jsonl") {
                continue;
            }
            // mtime = last activity; only sessions active inside the window count.
            let Ok(meta) = entry.metadata() else { continue };
            let Ok(modified) = meta.modified() else {
                continue;
            };
            let modified: DateTime<Utc> = modified.into();
            if now - modified > window {
                continue;
            }
            if let Some(ts) = first_timestamp(&path) {
                if earliest.is_none_or(|e| ts < e) {
                    earliest = Some(ts);
                }
            }
        }
    }
    window_reset(earliest?, now)
}

/// Pure window arithmetic, separated for testing: a session start older than
/// the window can't tell us where the current block began — return None
/// rather than a fabricated time. Otherwise floor to the hour and add 5h.
fn window_reset(earliest_recent: DateTime<Utc>, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let window = Duration::hours(USAGE_WINDOW_HOURS);
    if earliest_recent <= now - window {
        return None;
    }
    let floored = earliest_recent
        .duration_trunc(Duration::hours(1))
        .unwrap_or(earliest_recent);
    Some(floored + window)
}

fn first_timestamp(path: &Path) -> Option<DateTime<Utc>> {
    let file = fs::File::open(path).ok()?;
    for line in BufReader::new(file).lines().map_while(Result::ok).take(5) {
        let Ok(v) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if let Some(ts) = v["timestamp"].as_str() {
            if let Ok(parsed) = DateTime::parse_from_rfc3339(ts) {
                return Some(parsed.with_timezone(&Utc));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn usage_line(input: u64, cache_read: u64, output: u64) -> String {
        format!(
            r#"{{"message":{{"usage":{{"input_tokens":{input},"cache_read_input_tokens":{cache_read},"output_tokens":{output}}}}},"timestamp":"2026-07-03T10:00:00Z"}}"#
        )
    }

    #[test]
    fn turn_totals_sum_all_token_kinds_and_skip_sidechains() {
        let lines = vec![
            r#"{"type":"user","message":{"role":"user"}}"#.to_string(),
            usage_line(1000, 50_000, 500),
            format!(r#"{{"isSidechain":true,{}"#, &usage_line(9, 9, 9)[1..]),
            usage_line(1200, 80_000, 800),
        ];
        let totals = turn_totals(lines.into_iter());
        assert_eq!(totals, vec![51_500, 82_000]);
    }

    #[test]
    fn stats_report_percent_and_prompts_left() {
        let stats = stats_from_totals(&[50_000, 80_000, 110_000], 200_000).unwrap();
        assert_eq!(stats.used, 110_000);
        assert_eq!(stats.percent, 55);
        // avg growth 30k, remaining 90k -> 3 prompts
        assert_eq!(stats.prompts_left, Some(3));
    }

    #[test]
    fn one_meelion_models_get_bigger_limit() {
        assert_eq!(context_limit(Some("claude-sonnet-5[1m]")), 1_000_000);
        assert_eq!(context_limit(Some("claude-fable-5")), 200_000);
        assert_eq!(context_limit(None), 200_000);
    }

    #[test]
    fn window_reset_floors_to_hour_or_declines_to_guess() {
        let now = Utc.with_ymd_and_hms(2026, 7, 3, 12, 40, 0).unwrap();
        // Session started 12:10 → window 12:00–17:00.
        let started = Utc.with_ymd_and_hms(2026, 7, 3, 12, 10, 0).unwrap();
        assert_eq!(
            window_reset(started, now),
            Some(Utc.with_ymd_and_hms(2026, 7, 3, 17, 0, 0).unwrap())
        );
        // A session started 9 hours ago can't define the current window —
        // no answer beats a wrong answer.
        let stale = Utc.with_ymd_and_hms(2026, 7, 3, 3, 0, 0).unwrap();
        assert_eq!(window_reset(stale, now), None);
    }
}
