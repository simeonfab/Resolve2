//! Quick capture: turn "Pay rent every month 9am p1 #Home @bills remind 30m" into a task.

use crate::model::Recurrence;
use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Weekday};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Capture {
    pub title: String,
    pub due: Option<NaiveDate>,
    pub time: Option<NaiveTime>,
    pub priority: Option<u8>,
    pub project: Option<String>,
    pub section: Option<String>,
    pub labels: Vec<String>,
    pub recurrence: Recurrence,
    pub reminder_min: Option<i64>,
    pub duration_min: Option<u32>,
}

impl Capture {
    /// The "Resolve understood" chips for Add task, without emoji (B-E11): (text, kind).
    pub fn plain_chips(&self) -> Vec<(String, Recognised)> {
        let mut v = vec![];
        if self.recurrence.is_some() {
            v.push((format!("↻ {}", self.recurrence.label()), Recognised::When));
        }
        match (self.due, self.time) {
            (Some(d), Some(t)) => v.push((format!("{} {}", crate::model::fmt_date_short(d), crate::model::fmt_time(t)), Recognised::When)),
            (Some(d), None) => v.push((crate::model::fmt_date_short(d), Recognised::When)),
            (None, Some(t)) => v.push((crate::model::fmt_time(t), Recognised::When)),
            _ => {}
        }
        if let Some(p) = self.priority {
            v.push((format!("Priority {p}"), Recognised::Priority));
        }
        if let Some(p) = &self.project {
            v.push((format!("Project: {p}"), Recognised::Project));
        }
        for l in &self.labels {
            v.push((format!("@{l}"), Recognised::Label));
        }
        if let Some(r) = self.reminder_min {
            v.push((format!("Reminder {r} min before"), Recognised::When));
        }
        if let Some(d) = self.duration_min {
            v.push((format!("{d} min"), Recognised::When));
        }
        v
    }

    pub fn chips(&self) -> Vec<String> {
        let mut v = vec![];
        if let Some(d) = self.due {
            let mut s = crate::model::fmt_date_short(d);
            if let Some(t) = self.time {
                s.push(' ');
                s.push_str(&crate::model::fmt_time(t));
            }
            v.push(format!("📅 {s}"));
        } else if let Some(t) = self.time {
            v.push(format!("🕑 {}", crate::model::fmt_time(t)));
        }
        if self.recurrence.is_some() {
            v.push(format!("🔁 {}", self.recurrence.label()));
        }
        if let Some(p) = self.priority {
            v.push(format!("⚑ P{p}"));
        }
        if let Some(p) = &self.project {
            v.push(format!("# {p}"));
        }
        for l in &self.labels {
            v.push(format!("@{l}"));
        }
        if let Some(r) = self.reminder_min {
            v.push(format!("⏰ {r} min before"));
        }
        if let Some(d) = self.duration_min {
            v.push(format!("⏱ {d} min"));
        }
        v
    }
}

fn weekday(s: &str) -> Option<Weekday> {
    Some(match s {
        "mon" | "monday" => Weekday::Mon,
        "tue" | "tues" | "tuesday" => Weekday::Tue,
        "wed" | "weds" | "wednesday" => Weekday::Wed,
        "thu" | "thur" | "thurs" | "thursday" => Weekday::Thu,
        "fri" | "friday" => Weekday::Fri,
        "sat" | "saturday" => Weekday::Sat,
        "sun" | "sunday" => Weekday::Sun,
        _ => return None,
    })
}

fn month(s: &str) -> Option<u32> {
    let m = match s.get(..3)? {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => return None,
    };
    // reject words like "market" that merely start with a month
    let full = ["january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december"];
    if s.len() <= 4 || full.contains(&s) || s == "sept" {
        Some(m)
    } else {
        None
    }
}

/// Next date (today counts if `allow_today`) falling on `wd`.
pub fn next_weekday(from: NaiveDate, wd: Weekday, allow_today: bool) -> NaiveDate {
    let mut d = if allow_today { from } else { from + Duration::days(1) };
    while d.weekday() != wd {
        d += Duration::days(1);
    }
    d
}

pub fn parse_time(s: &str) -> Option<NaiveTime> {
    let s = s.trim().to_lowercase();
    let (body, pm, am) = if let Some(b) = s.strip_suffix("pm") {
        (b.to_string(), true, false)
    } else if let Some(b) = s.strip_suffix("am") {
        (b.to_string(), false, true)
    } else {
        (s.clone(), false, false)
    };
    let body = body.trim();
    let (h, m) = if let Some((h, m)) = body.split_once(':').or_else(|| body.split_once('.')) {
        (h.parse::<u32>().ok()?, m.parse::<u32>().ok()?)
    } else {
        if !(pm || am) {
            // a bare number like "17" is not a time unless written 17:00
            return None;
        }
        (body.parse::<u32>().ok()?, 0)
    };
    let h = if pm && h < 12 {
        h + 12
    } else if am && h == 12 {
        0
    } else {
        h
    };
    NaiveTime::from_hms_opt(h, m, 0)
}

pub fn parse_date(s: &str, today: NaiveDate) -> Option<NaiveDate> {
    let s = s.trim().to_lowercase();
    match s.as_str() {
        "today" | "tod" => return Some(today),
        "tomorrow" | "tmr" | "tom" | "tmrw" => return Some(today + Duration::days(1)),
        "yesterday" => return Some(today - Duration::days(1)),
        _ => {}
    }
    if let Some(wd) = weekday(&s) {
        return Some(next_weekday(today, wd, false));
    }
    if let Ok(d) = NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
        return Some(d);
    }
    // day/month (UK order) with optional year
    let parts: Vec<&str> = s.split(['/', '.']).collect();
    if parts.len() >= 2 && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())) {
        let d: u32 = parts[0].parse().ok()?;
        let m: u32 = parts[1].parse().ok()?;
        let y: i32 = if parts.len() > 2 {
            let y: i32 = parts[2].parse().ok()?;
            if y < 100 {
                2000 + y
            } else {
                y
            }
        } else {
            today.year()
        };
        let mut date = NaiveDate::from_ymd_opt(y, m, d)?;
        if parts.len() == 2 && date < today {
            date = NaiveDate::from_ymd_opt(y + 1, m, d)?;
        }
        return Some(date);
    }
    None
}

fn day_number(s: &str) -> Option<u32> {
    let t = s.trim_end_matches(|c: char| c.is_alphabetic());
    let n: u32 = t.parse().ok()?;
    let suffix = &s[t.len()..];
    if (1..=31).contains(&n) && ["", "st", "nd", "rd", "th"].contains(&suffix) {
        Some(n)
    } else {
        None
    }
}

fn duration_minutes(s: &str) -> Option<i64> {
    let s = s.to_lowercase();
    for (suf, mult) in [("min", 1), ("mins", 1), ("m", 1), ("h", 60), ("hr", 60), ("hrs", 60), ("hour", 60), ("hours", 60), ("d", 1440)] {
        if let Some(n) = s.strip_suffix(suf) {
            if let Ok(v) = n.parse::<f64>() {
                return Some((v * mult as f64).round() as i64);
            }
        }
    }
    None
}

pub fn parse(input: &str, today: NaiveDate) -> Capture {
    parse_used(input, today).0
}

/// What a recognised word means, for highlighting it as you type (B-E11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recognised {
    /// A date, time, repeat, reminder or duration (amber).
    When,
    /// p1 to p4 (terracotta).
    Priority,
    /// #Project (green).
    Project,
    /// @label (amber).
    Label,
}

/// The byte ranges of the words the parser recognised, and what each is.
pub fn recognised(input: &str, today: NaiveDate) -> Vec<(std::ops::Range<usize>, Recognised)> {
    let (_, used) = parse_used(input, today);
    let mut out = vec![];
    let mut at = 0;
    for (i, tok) in input.split_whitespace().enumerate() {
        let start = at + input[at..].find(tok).unwrap_or(0);
        at = start + tok.len();
        if !used.get(i).copied().unwrap_or(false) {
            continue;
        }
        let low = tok.to_lowercase();
        let kind = if tok.starts_with('#') {
            Recognised::Project
        } else if tok.starts_with('@') {
            Recognised::Label
        } else if low.len() == 2 && (low.starts_with('p') || low.starts_with('!')) && low[1..].parse::<u8>().is_ok() {
            Recognised::Priority
        } else {
            Recognised::When
        };
        out.push((start..at, kind));
    }
    out
}

fn parse_used(input: &str, today: NaiveDate) -> (Capture, Vec<bool>) {
    let mut c = Capture::default();
    let toks: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
    let low: Vec<String> = toks.iter().map(|s| s.to_lowercase().trim_end_matches([',', ';']).to_string()).collect();
    let mut used = vec![false; toks.len()];
    let n = toks.len();
    let mut i = 0;
    while i < n {
        let w = low[i].as_str();
        let next = low.get(i + 1).map(|s| s.as_str()).unwrap_or("");
        let next2 = low.get(i + 2).map(|s| s.as_str()).unwrap_or("");
        let mut take = 0usize;

        if (w.len() == 2 && w.starts_with('p')) || (w.len() == 2 && w.starts_with('!')) {
            if let Ok(p) = w[1..].parse::<u8>() {
                if (1..=4).contains(&p) {
                    c.priority = Some(p);
                    take = 1;
                }
            }
        }
        if take == 0 && w.starts_with('#') && w.len() > 1 {
            let raw = toks[i][1..].replace('_', " ");
            if let Some((p, s)) = raw.split_once('/') {
                c.project = Some(p.to_string());
                c.section = Some(s.to_string());
            } else {
                c.project = Some(raw);
            }
            take = 1;
        }
        if take == 0 && w.starts_with('@') && w.len() > 1 {
            c.labels.push(toks[i][1..].to_string());
            take = 1;
        }
        if take == 0 && (w == "every" || w == "ev") {
            let (rec, k, start): (Option<Recurrence>, usize, Option<NaiveDate>) = match next {
                "day" => (Some(Recurrence::Daily), 2, Some(today)),
                "weekday" | "workday" => (Some(Recurrence::Weekdays), 2, None),
                "week" => (Some(Recurrence::Weekly), 2, Some(today)),
                "month" => (Some(Recurrence::Monthly), 2, Some(today)),
                "year" => (Some(Recurrence::Yearly), 2, Some(today)),
                "other" => match next2 {
                    "day" => (Some(Recurrence::EveryDays(2)), 3, Some(today)),
                    "week" => (Some(Recurrence::EveryWeeks(2)), 3, Some(today)),
                    _ => (None, 0, None),
                },
                _ => {
                    if let Some(wd) = weekday(next) {
                        (Some(Recurrence::Weekly), 2, Some(next_weekday(today, wd, true)))
                    } else if let Ok(k) = next.parse::<u32>() {
                        match next2 {
                            "days" | "day" => (Some(Recurrence::EveryDays(k)), 3, Some(today)),
                            "weeks" | "week" => (Some(Recurrence::EveryWeeks(k)), 3, Some(today)),
                            _ => (None, 0, None),
                        }
                    } else {
                        (None, 0, None)
                    }
                }
            };
            if let Some(r) = rec {
                c.recurrence = r;
                if c.due.is_none() {
                    c.due = start.or_else(|| {
                        let mut d = today;
                        while matches!(d.weekday(), Weekday::Sat | Weekday::Sun) {
                            d += Duration::days(1);
                        }
                        Some(d)
                    });
                }
                take = k;
            }
        }
        if take == 0 {
            let rec = match w {
                "daily" => Some(Recurrence::Daily),
                "weekly" => Some(Recurrence::Weekly),
                "monthly" => Some(Recurrence::Monthly),
                "yearly" | "annually" => Some(Recurrence::Yearly),
                _ => None,
            };
            if let Some(r) = rec {
                c.recurrence = r;
                if c.due.is_none() {
                    c.due = Some(today);
                }
                take = 1;
            }
        }
        if take == 0 && (w == "remind" || w == "reminder") {
            let arg = if next == "me" { next2 } else { next };
            let skip = if next == "me" { 3 } else { 2 };
            if let Some(m) = duration_minutes(arg) {
                c.reminder_min = Some(m);
                take = skip;
            } else {
                c.reminder_min = Some(0);
                take = 1;
            }
        }
        if take == 0 && w == "for" {
            if let Some(m) = duration_minutes(next) {
                c.duration_min = Some(m.max(5) as u32);
                take = 2;
            }
        }
        if take == 0 && w == "tonight" {
            c.due = Some(today);
            c.time = c.time.or(NaiveTime::from_hms_opt(20, 0, 0));
            take = 1;
        }
        if take == 0 && w == "next" {
            match next {
                "week" => {
                    c.due = Some(next_weekday(today, Weekday::Mon, false));
                    take = 2;
                }
                "month" => {
                    c.due = today.checked_add_months(chrono::Months::new(1)).map(|d| d.with_day(1).unwrap());
                    take = 2;
                }
                _ => {
                    if let Some(wd) = weekday(next) {
                        // the given weekday in next week (weeks start on Monday)
                        let next_monday = next_weekday(today, Weekday::Mon, false);
                        c.due = Some(next_weekday(next_monday, wd, true));
                        take = 2;
                    }
                }
            }
        }
        if take == 0 && w == "in" {
            if let Ok(k) = next.parse::<i64>() {
                let d = match next2 {
                    "day" | "days" => Some(today + Duration::days(k)),
                    "week" | "weeks" => Some(today + Duration::days(7 * k)),
                    "month" | "months" => today.checked_add_months(chrono::Months::new(k as u32)),
                    _ => None,
                };
                if d.is_some() {
                    c.due = d;
                    take = 3;
                }
            }
        }
        if take == 0 && w == "at" {
            if let Some(t) = parse_time(next) {
                c.time = Some(t);
                take = 2;
            }
        }
        if take == 0 {
            if let Some(t) = parse_time(w) {
                c.time = Some(t);
                take = 1;
            }
        }
        if take == 0 {
            // "5 jan" / "jan 5" / "5th january"
            if let (Some(d), Some(m)) = (day_number(w), month(next)) {
                let mut date = NaiveDate::from_ymd_opt(today.year(), m, d);
                if let Some(x) = date {
                    if x < today {
                        date = NaiveDate::from_ymd_opt(today.year() + 1, m, d);
                    }
                }
                if date.is_some() {
                    c.due = date;
                    take = 2;
                }
            } else if let (Some(m), Some(d)) = (month(w), day_number(next)) {
                let mut date = NaiveDate::from_ymd_opt(today.year(), m, d);
                if let Some(x) = date {
                    if x < today {
                        date = NaiveDate::from_ymd_opt(today.year() + 1, m, d);
                    }
                }
                if date.is_some() {
                    c.due = date;
                    take = 2;
                }
            }
        }
        if take == 0 {
            // a date word, but don't swallow "on" etc.
            let word = if w == "on" || w == "by" || w == "due" { next } else { w };
            if let Some(d) = parse_date(word, today) {
                c.due = Some(d);
                take = if word == w { 1 } else { 2 };
            }
        }
        if take > 0 {
            for u in used.iter_mut().skip(i).take(take) {
                *u = true;
            }
            i += take;
        } else {
            i += 1;
        }
    }
    let title: Vec<&str> = toks.iter().zip(used.iter()).filter(|(_, u)| !**u).map(|(t, _)| t.as_str()).collect();
    c.title = title.join(" ").trim().trim_end_matches([',', '-']).trim().to_string();
    if c.due.is_none() && c.time.is_some() {
        c.due = Some(today);
    }
    (c, used)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basics() {
        let t = NaiveDate::from_ymd_opt(2026, 11, 5).unwrap();
        let c = parse("Pay rent tomorrow 9am p1 #Home @bills", t);
        assert_eq!(c.title, "Pay rent");
        assert_eq!(c.due, Some(NaiveDate::from_ymd_opt(2026, 11, 6).unwrap()));
        assert_eq!(c.time, NaiveTime::from_hms_opt(9, 0, 0));
        assert_eq!(c.priority, Some(1));
        assert_eq!(c.project.as_deref(), Some("Home"));
        let c = parse("Water plants every week", t);
        assert_eq!(c.recurrence, Recurrence::Weekly);
        assert_eq!(c.title, "Water plants");
        let c = parse("Dentist 12/11 at 3:30pm remind 30m", t);
        assert_eq!(c.due, Some(NaiveDate::from_ymd_opt(2026, 11, 12).unwrap()));
        assert_eq!(c.reminder_min, Some(30));
        assert_eq!(c.title, "Dentist");
    }
}
