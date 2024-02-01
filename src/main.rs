use anyhow::Result;
use chrono::naive::NaiveDate;
use chrono::Duration;
use icalendar::parser;
use printpdf::path::PaintMode;
use printpdf::{Color, PdfDocument, Pt, Rect, Rgb};
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

const COLS: u32 = 4;
const WIDTH: f32 = 612.;
const HEIGHT: f32 = 792.;
const GUTTER: f32 = 10.;
const MARGIN: f32 = 36.;
const MONTH_WIDTH: f32 = 12.;
const DAY_WIDTH: f32 = 10.;
// const LINE_GAP: f32 = 4.0;
const ROW: f32 = (HEIGHT - MARGIN * 2.0) / (12.0 * 31.0 / COLS as f32);
const CW: f32 = (WIDTH - MARGIN * 2.0 - GUTTER * (COLS as f32 - 1.0)) / COLS as f32;

fn write_to_pdf(outpath: &Path) {
    let (doc, page1, layer1) = PdfDocument::new(
        "Calendar".to_string(),
        Pt(WIDTH).into(),
        Pt(HEIGHT).into(),
        "Layer 1",
    );
    let font_file = File::open("assets/fonts/DejaVuSans.ttf").unwrap();
    let font = doc.add_external_font(font_file).unwrap();

    let canvas = doc.get_page(page1).get_layer(layer1);
    canvas.set_outline_thickness(0.01);
    canvas.set_outline_color(Color::Rgb(Rgb::new(0.8, 0.8, 0.8, None)));

    let mut left = MARGIN;
    let mut month = 1;
    for _ in 0..COLS {
        let mut bottom = HEIGHT - MARGIN - ROW;
        for _ in 1..=12 / COLS {
            // month
            canvas.add_rect(
                Rect::new(
                    Pt(left).into(),
                    Pt(bottom - (30. * ROW)).into(),
                    Pt(left + MONTH_WIDTH).into(),
                    Pt(bottom + ROW).into(),
                )
                .with_mode(PaintMode::Stroke),
            );

            for day in 1..=31 {
                // text
                canvas.add_rect(
                    Rect::new(
                        Pt(left + MONTH_WIDTH + DAY_WIDTH).into(),
                        Pt(bottom).into(),
                        Pt(left + CW).into(),
                        Pt(bottom + ROW).into(),
                    )
                    .with_mode(PaintMode::Stroke),
                );
                // day number
                canvas.add_rect(
                    Rect::new(
                        Pt(left + MONTH_WIDTH).into(),
                        Pt(bottom).into(),
                        Pt(left + MONTH_WIDTH + DAY_WIDTH).into(),
                        Pt(bottom + ROW).into(),
                    )
                    .with_mode(PaintMode::Stroke),
                );
                let text = format!("{}", day);
                canvas.use_text(
                    &text,
                    6.0,
                    Pt(left + MONTH_WIDTH).into(),
                    Pt(bottom + 2.0).into(),
                    &font,
                );
                bottom -= ROW;
            }
            month += 1;
        }
        left += CW + GUTTER;
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
