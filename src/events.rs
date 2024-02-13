use anyhow::Result;
use chrono::naive::NaiveDate;
use icalendar::parser;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Event {
    pub id: usize,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub num_days: i64,
    pub summary: String,
}

pub fn get_events(calendars: &Vec<PathBuf>) -> Result<Vec<Event>> {
    let mut events = vec![];
    for (id, calendar) in calendars.iter().enumerate() {
        let data = std::fs::read_to_string(calendar)?;
        let data = parser::unfold(&data);

        let components = parser::read_calendar_simple(&data).unwrap();
        for component in components {
            for component in component.components {
                let mut start: Option<String> = None;
                let mut end: Option<String> = None;
                let mut summary: Option<String> = None;
                let mut location: Option<String> = None;
                for prop in component.properties {
                    match prop.name.as_str() {
                        "DTSTART" => start = Some(prop.val.to_string()),
                        "DTEND" => end = Some(prop.val.to_string()),
                        "SUMMARY" => summary = Some(prop.val.to_string()),
                        "LOCATION" => location = Some(prop.val.to_string()),
                        _ => (),
                    }
                }
                match (&start, &end, &summary) {
                    (Some(start), Some(end), Some(summary)) => {
                        let (start, _) = NaiveDate::parse_and_remainder(&start, "%Y%m%d").unwrap();
                        let (end, _) = NaiveDate::parse_and_remainder(&end, "%Y%m%d").unwrap();
                        let num_days = (end - start).num_days();
                        let mut summary = summary.to_string();
                        if let Some(location) = location {
                            if location.len() > 0 {
                                summary.push_str(&format!(" at {location}"));
                            }
                        }

                        if !summary.starts_with("Canceled: ") {
                            events.push(Event {
                                id,
                                start,
                                end,
                                num_days,
                                summary,
                            });
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    Ok(events)
}
