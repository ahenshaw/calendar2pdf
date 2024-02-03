use anyhow::Result;
use chrono::naive::NaiveDate;
use chrono::Duration;
use icalendar::parser;
use printpdf::path::PaintMode::{self, FillStroke, Stroke};
use printpdf::*;
use std::collections::HashMap;
// use std::fmt::write;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    if let Ok(events) = get_events(&PathBuf::from("data/KHW Travel Calendar.ics")) {
        let days = populate_days(&events);
        for (day, events) in days {
            println!("{} {:?}", day, events);
        }
    }
    let outpath = PathBuf::from("test.pdf");
    let (mut doc, canvas, font) = create_pdf();
    let _label_pos = base_calendar(&mut doc, &canvas, &font);

    doc.save(&mut BufWriter::new(File::create(outpath).unwrap()))
        .unwrap();
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
const DAY_WIDTH: f32 = 10.;
const TITLE_HEIGHT: f32 = 24.;
// const LINE_GAP: f32 = 4.0;
const ROW: f32 = (HEIGHT - TITLE_HEIGHT - MARGIN * 2.0) / (12.0 * 32.0 / COLS as f32);
const CW: f32 = (WIDTH - MARGIN * 2.0 - GUTTER * (COLS as f32 - 1.0)) / COLS as f32;

fn write_events(
    canvas: &PdfLayerReference,
    days: HashMap<NaiveDate, Vec<String>>,
    pos_map: HashMap<NaiveDate, (f32, f32)>,
    font: &IndirectFontRef,
) {
    S
}

fn create_pdf() -> (PdfDocumentReference, PdfLayerReference, IndirectFontRef) {
    let (mut doc, page1, layer1) = PdfDocument::new(
        "Calendar".to_string(),
        Pt(WIDTH).into(),
        Pt(HEIGHT).into(),
        "Layer 1",
    );
    doc = doc.with_conformance(PdfConformance::Custom(CustomPdfConformance {
        requires_icc_profile: false,
        requires_xmp_metadata: false,
        ..Default::default()
    }));
    let canvas = doc.get_page(page1).get_layer(layer1);
    let font_file = File::open("assets/fonts/DejaVuSans.ttf").unwrap();
    let font = doc.add_external_font(font_file).unwrap();

    (doc, canvas, font)
}
fn base_calendar(
    doc: &mut PdfDocumentReference,
    canvas: &PdfLayerReference,
    font: &IndirectFontRef,
) -> HashMap<NaiveDate, (f32, f32)> {
    let year = 2024;
    let mut left = MARGIN;
    let mut month = 1u32;
    let mut shade = false;
    let mut label_position = HashMap::new();

    canvas.set_outline_thickness(0.0);
    canvas.set_outline_color(Color::Rgb(Rgb::new(0.9, 0.9, 0.9, None)));

    for _ in 0..COLS {
        let mut bottom = HEIGHT - MARGIN - ROW - TITLE_HEIGHT;
        let months_in_col = 12 / COLS;
        for _ in 0..months_in_col {
            let month_name = MONTHS[(month - 1) as usize];
            let month_start = bottom + ROW;
            canvas.set_fill_color(Color::Rgb(Rgb::new(0.7, 0.7, 0.9, None)));
            rect(&canvas, left, bottom, CW, ROW, FillStroke);

            canvas.set_fill_color(Color::Rgb(Rgb::new(0., 0., 0., None)));
            canvas.use_text(
                &format!("{}", month_name),
                5.0,
                Pt(left + 5. * DAY_WIDTH).into(),
                Pt(bottom + 2.0).into(),
                &font,
            );
            bottom -= ROW;

            for day in 1..=31 {
                // check if current day okay
                if let Some(today) = NaiveDate::from_ymd_opt(year, month, day) {
                    let day_of_week = today.format("%a").to_string();
                    if day_of_week == "Mon" {
                        shade = !shade
                    }
                    label_position.insert(today, (left + 2. * DAY_WIDTH, bottom));
                    // text box
                    rect(
                        &canvas,
                        left + DAY_WIDTH,
                        bottom,
                        CW - DAY_WIDTH,
                        ROW,
                        Stroke,
                    );
                    // day number
                    canvas.set_fill_color(Color::Rgb(Rgb::new(0.9, 0.9, 0.9, None)));
                    rect(
                        &canvas,
                        left,
                        bottom,
                        DAY_WIDTH,
                        ROW,
                        if shade { FillStroke } else { Stroke },
                    );
                    rect(
                        &canvas,
                        left + DAY_WIDTH,
                        bottom,
                        DAY_WIDTH,
                        ROW,
                        if shade { FillStroke } else { Stroke },
                    );
                    canvas.set_fill_color(Color::Rgb(Rgb::new(0., 0., 0., None)));

                    canvas.use_text(
                        &format!("{}", day),
                        5.0,
                        Pt(left + 2.).into(),
                        Pt(bottom + 2.).into(),
                        &font,
                    );

                    canvas.use_text(
                        &format!("{}", day_of_week.chars().next().unwrap()),
                        5.0,
                        Pt(left + DAY_WIDTH + 2.).into(),
                        Pt(bottom + 2.0).into(),
                        &font,
                    );
                }
                bottom -= ROW;
            }
            // month
            canvas.save_graphics_state();
            canvas.set_outline_color(Color::Rgb(Rgb::new(0.4, 0.4, 0.4, None)));
            rect(
                &canvas,
                left,
                bottom + ROW,
                CW,
                month_start - bottom - ROW,
                Stroke,
            );
            canvas.restore_graphics_state();
            month += 1;
        }
        left += CW + GUTTER;
    }
    label_position
}

pub fn write_month(
    canvas: &PdfLayerReference,
    index: u32,
    x: f32,
    top: f32,
    bottom: f32,
    font: &IndirectFontRef,
) {
    let name = MONTHS[index as usize].to_lowercase();
    const MONTH_FONT_SIZE: f32 = 11.;
    let start = (top + bottom) / 2. + (name.len() as f32) * MONTH_FONT_SIZE / 2.;
    dbg!((top, bottom, &start));

    canvas.begin_text_section();
    canvas.set_font(&font, MONTH_FONT_SIZE);
    canvas.set_line_height(MONTH_FONT_SIZE);
    canvas.set_text_cursor(Pt(x).into(), Pt(start).into());
    for c in name.chars() {
        canvas.write_text(format!("{c}"), &font);
        canvas.add_line_break();
    }
    canvas.end_text_section();
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

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn rect(
    canvas: &PdfLayerReference,
    left: f32,
    bottom: f32,
    width: f32,
    height: f32,
    mode: PaintMode,
) {
    canvas.add_rect(
        Rect::new(
            Pt(left).into(),
            Pt(bottom).into(),
            Pt(left + width).into(),
            Pt(bottom + height).into(),
        )
        .with_mode(mode),
    );
}
