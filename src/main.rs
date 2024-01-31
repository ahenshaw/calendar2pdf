use anyhow::Result;
use chrono::naive::NaiveDate;
use chrono::Duration;
use icalendar::parser;
use printpdf::path::PaintMode;
use printpdf::{Mm, PdfDocument, Pt, Rect};
use std::collections::HashMap;
// use std::fmt::write;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    // if let Ok(events) = get_events(&PathBuf::from("data/KHW Travel Calendar.ics")) {
    //     let days = populate_days(&events);
    //     for (day, events) in days {
    //         println!("{} {:?}", day, events);
    //     }
    // }
    write_to_pdf(&PathBuf::from("test.pdf"));
}

#[derive(Debug)]
pub struct Event {
    start: NaiveDate,
    end: NaiveDate,
    summary: String,
}

const COLS: usize = 3;
const WIDTH: f32 = 612.;
const HEIGHT: f32 = 792.;

fn write_to_pdf(outpath: &Path) {
    let (doc, page1, layer1) = PdfDocument::new(
        "Calendar".to_string(),
        Pt(WIDTH).into(),
        Pt(HEIGHT).into(),
        "Layer 1",
    );
    let canvas = doc.get_page(page1).get_layer(layer1);
    canvas.set_outline_thickness(0.05);
    for _col in 0..COLS {
        let rect = Rect::new(
            Pt(20.).into(),
            Pt(20.).into(),
            Pt(100.).into(),
            Pt(100.).into(),
        )
        .with_mode(PaintMode::Stroke);
        canvas.add_rect(rect);
    }

    doc.save(&mut BufWriter::new(File::create(outpath).unwrap()))
        .unwrap();
}

pub fn get_events(calendar_file_path: &Path) -> Result<Vec<Event>> {
    let mut events = vec![];
    let data = std::fs::read_to_string(calendar_file_path)?;
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

pub fn populate_days(events: &Vec<Event>) -> HashMap<NaiveDate, Vec<String>> {
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
