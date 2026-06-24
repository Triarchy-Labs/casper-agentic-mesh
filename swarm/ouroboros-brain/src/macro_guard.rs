//! Macro Guard — Economic Event Shield for Ouroboros.
//! Blocks trading during FOMC, CPI, NFP, PPI events.
//! Port from Python `hyper/macro_guard.py` (1:1).

use serde::{Deserialize, Serialize};
use std::time::Duration;
use chrono::Datelike;

// ═══════════════════════════════════════════════════════════
// FOMC 2026 SCHEDULE (Federal Reserve official dates)
// ═══════════════════════════════════════════════════════════

const FOMC_DATES_2026: &[&str] = &[
    "2026-01-28", "2026-01-29",
    "2026-03-18", "2026-03-19",
    "2026-05-06", "2026-05-07",
    "2026-06-17", "2026-06-18",
    "2026-07-29", "2026-07-30",
    "2026-09-16", "2026-09-17",
    "2026-10-28", "2026-10-29",
    "2026-12-16", "2026-12-17",
];

// ═══════════════════════════════════════════════════════════
// EVENT TYPES
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct MacroEvent {
    pub name: String,
    pub date: String,
    pub impact: String,    // "CRITICAL", "HIGH", "MEDIUM"
    pub hours_until: f64,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroLevel {
    Clear,
    Caution,
    Lockdown,
}

impl std::fmt::Display for MacroLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MacroLevel::Clear => write!(f, "✅ CLEAR"),
            MacroLevel::Caution => write!(f, "⚠️ CAUTION"),
            MacroLevel::Lockdown => write!(f, "🔒 LOCKDOWN"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MacroGuardResult {
    pub shield_active: bool,
    pub level: MacroLevel,
    pub reason: String,
    pub events: Vec<MacroEvent>,
    pub factor_score: f64,  // Factor 15 contribution
}

// ═══════════════════════════════════════════════════════════
// FOREX FACTORY CALENDAR (web scraping)
// ═══════════════════════════════════════════════════════════

#[derive(Deserialize)]
struct ForexEvent {
    title: Option<String>,
    date: Option<String>,
    impact: Option<String>,
    country: Option<String>,
}

async fn scrape_forex_calendar() -> Vec<MacroEvent> {
    let client = reqwest::Client::new();
    let url = "https://nfs.faireconomy.media/ff_calendar_thisweek.json";

    let resp = match client.get(url)
        .header("User-Agent", "Mozilla/5.0")
        .timeout(Duration::from_secs(10))
        .send().await
    {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let data: Vec<ForexEvent> = match resp.json().await {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let now = chrono::Utc::now();
    let mut events = Vec::new();

    for item in &data {
        let impact = item.impact.as_deref().unwrap_or("").to_lowercase();
        if impact != "high" && impact != "medium" { continue; }

        let country = item.country.as_deref().unwrap_or("");
        if country != "USD" && country != "All" { continue; }

        let title = item.title.as_deref().unwrap_or("Unknown").to_string();
        let date_str = item.date.as_deref().unwrap_or("");

        let hours = if let Ok(event_dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
            let delta = event_dt.signed_duration_since(now);
            delta.num_minutes() as f64 / 60.0
        } else if date_str.len() >= 19 {
            if let Ok(event_dt) = chrono::NaiveDateTime::parse_from_str(&date_str[..19], "%Y-%m-%dT%H:%M:%S") {
                let delta = event_dt - now.naive_utc();
                delta.num_minutes() as f64 / 60.0
            } else { 999.0 }
        } else { 999.0 };

        if hours > -24.0 && hours < 72.0 {
            events.push(MacroEvent {
                name: title,
                date: date_str.chars().take(10).collect(),
                impact: impact.to_uppercase(),
                hours_until: (hours * 10.0).round() / 10.0,
                active: hours > -4.0 && hours < 0.0,
            });
        }
    }

    events
}

// ═══════════════════════════════════════════════════════════
// MAIN CHECK
// ═══════════════════════════════════════════════════════════

/// Master macro guard check — call once per cycle.
/// Returns whether trading should be cautious or locked down.
pub async fn macro_guard_check() -> MacroGuardResult {
    let now = chrono::Utc::now();
    let today = now.format("%Y-%m-%d").to_string();
    let mut events = Vec::new();

    // === FOMC Events ===
    for &date_str in FOMC_DATES_2026 {
        if let Ok(event_dt) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            let event_noon = event_dt.and_hms_opt(18, 0, 0).unwrap(); // 6PM UTC = typical FOMC release
            let delta_hours = (event_noon - now.naive_utc()).num_minutes() as f64 / 60.0;

            if delta_hours > -24.0 && delta_hours < 48.0 {
                events.push(MacroEvent {
                    name: "FOMC".into(),
                    date: date_str.into(),
                    impact: "CRITICAL".into(),
                    hours_until: (delta_hours * 10.0).round() / 10.0,
                    active: delta_hours > -4.0 && delta_hours < 4.0,
                });
            }
        }
    }

    // === CPI heuristic: 10th-13th of month ===
    let month_day = now.day();
    if (9..=14).contains(&month_day) {
        events.push(MacroEvent {
            name: "CPI_WINDOW".into(),
            date: today.clone(),
            impact: "HIGH".into(),
            hours_until: 0.0,
            active: true,
        });
    }

    // === NFP heuristic: first Friday of month ===
    let first_day = now.date_naive().with_day(1).unwrap_or(now.date_naive());
    let weekday = first_day.weekday().num_days_from_monday();
    let days_to_friday = (4 + 7 - weekday) % 7;
    let nfp_date = first_day + chrono::Duration::days(days_to_friday as i64);
    let nfp_noon = nfp_date.and_hms_opt(12, 30, 0).unwrap(); // 12:30 UTC = typical NFP
    let nfp_delta = (nfp_noon - now.naive_utc()).num_minutes() as f64 / 60.0;

    if nfp_delta > -24.0 && nfp_delta < 48.0 {
        events.push(MacroEvent {
            name: "NFP".into(),
            date: nfp_date.format("%Y-%m-%d").to_string(),
            impact: "HIGH".into(),
            hours_until: (nfp_delta * 10.0).round() / 10.0,
            active: nfp_delta > -4.0 && nfp_delta < 0.0,
        });
    }

    // === Web scraping (Forex Factory) ===
    let web_events = scrape_forex_calendar().await;
    events.extend(web_events);

    // === Decision ===
    if events.is_empty() {
        return MacroGuardResult {
            shield_active: false,
            level: MacroLevel::Clear,
            reason: "No major events upcoming".into(),
            events,
            factor_score: 0.0,
        };
    }

    let critical: Vec<_> = events.iter().filter(|e| e.impact == "CRITICAL").collect();
    let active: Vec<_> = events.iter().filter(|e| e.active).collect();
    let upcoming: Vec<_> = events.iter()
        .filter(|e| e.hours_until > 0.0 && e.hours_until < 24.0)
        .collect();

    let (level, reason, factor_score) = if !active.is_empty() {
        (MacroLevel::Lockdown,
         format!("ACTIVE: {} happening NOW", active[0].name),
         -2.0)
    } else if !critical.is_empty() && critical.iter().any(|e| e.hours_until > 0.0 && e.hours_until < 24.0) {
        (MacroLevel::Lockdown,
         format!("FOMC in {:.1}h", critical[0].hours_until),
         -1.5)
    } else if !upcoming.is_empty() {
        (MacroLevel::Caution,
         format!("{} events within 24h", upcoming.len()),
         -0.5)
    } else {
        (MacroLevel::Clear,
         format!("{} events tracked (not imminent)", events.len()),
         0.0)
    };

    let shield_active = matches!(level, MacroLevel::Lockdown | MacroLevel::Caution);
    tracing::info!("MacroGuard: {} — {}", level, reason);

    // Keep top 5 events
    events.truncate(5);

    MacroGuardResult {
        shield_active,
        level,
        reason,
        events,
        factor_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fomc_schedule_exists() {
        assert!(!FOMC_DATES_2026.is_empty());
        assert!(FOMC_DATES_2026.len() >= 16); // 8 meetings × 2 days
    }

    #[test]
    fn test_macro_level_display() {
        assert_eq!(format!("{}", MacroLevel::Lockdown), "🔒 LOCKDOWN");
        assert_eq!(format!("{}", MacroLevel::Clear), "✅ CLEAR");
    }
}
