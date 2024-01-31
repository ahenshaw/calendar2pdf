use anyhow::Result;
use chrono::naive::NaiveDate;
use chrono::Duration;
use icalendar::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Event {
    start: NaiveDate,
    end: NaiveDate,
    summary: String,
}

fn get_events(calend_datear_file_path: &Path) -> Result<Vec<Event>> {
    let mut events = vec![];
    let data = std::fs::read_to_string(calend_datear_file_path)?;
    let data = parser::unfold(&data);

    let components = parser::read_calendar_simple(&data).unwrap();
    for component in components {
        for component in component.components {
            let mut start: Option<String> = None;
            let mut end: Option<String> = None;
            let mut summary: Option<String> = None;
            for prop in component.properties {
                match prop.name.as_str() {
                    "DTSTART" => start = Some(prop.val.to_string()),
                    "DTEND" => end = Some(prop.val.to_string()),
                    "SUMMARY" => summary = Some(prop.val.to_string()),
                    _ => (),
                }
            }
            match (&start, &end, &summary) {
                (Some(start), Some(end), Some(summary)) => {
                    let (start, _) = NaiveDate::parse_and_remainder(&start, "%Y%m%d").unwrap();
                    let (end, _) = NaiveDate::parse_and_remainder(&end, "%Y%m%d").unwrap();
                    events.push(Event {
                        start,
                        end,
                        summary: summary.to_string(),
                    });
                }
                _ => (),
            }
        }
    }
    Ok(events)
}

fn populate_days(events: &Vec<Event>) -> HashMap<NaiveDate, Vec<String>> {
    let mut days = HashMap::new();
    for event in events {
        if !event.summary.starts_with("Canceled: ") {
            let mut day = event.start;
            while day <= event.end {
                days.entry(day)
                    .or_insert(Vec::new())
                    .push(event.summary.clone());
                day += Duration::days(1);
            }
        }
    }
    days
}

fn main() {
    if let Ok(mut events) = get_events(&PathBuf::from(
        "C:/Users/ah6/Documents/Henshaw Andrew Calendar.ics",
    )) {
        let days = populate_days(&events);
        for (day, events) in days {
            println!("{} {:?}", day, events);
        }
    }
}
