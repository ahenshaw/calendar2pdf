use crate::events::Event;
// use azul_text_layout::text_layout::ResolvedTextLayoutOptions;
// use azul_text_layout::{
//     text_layout::{position_words, split_text_into_words, words_to_scaled_words},
//     text_shaping::get_font_metrics_freetype,
// };
use itertools::Itertools;

use glyph_brush_layout::ab_glyph::Font;
use glyph_brush_layout::GlyphPositioner;

use chrono::naive::{Days, NaiveDate};
use printpdf::path::PaintMode::{self, FillStroke, Stroke};
use printpdf::*;
use std::collections::HashMap;

const COLS: u32 = 4;
const WIDTH: f32 = 612.;
const HEIGHT: f32 = 792.;
const GUTTER: f32 = 10.;
const SUMMARY_GUTTER: f32 = 5.;
const MARGIN: f32 = 36.;
const DAY_WIDTH: f32 = 10.;
const TITLE_HEIGHT: f32 = 16.;
const FONT_HEIGHT: f32 = 5.;
const BASE: f32 = 2.;

const ROW: f32 = (HEIGHT - TITLE_HEIGHT - MARGIN * 2.0) / (12.0 * 32.0 / COLS as f32);
const CW: f32 = (WIDTH - MARGIN * 2.0 - GUTTER * (COLS as f32 - 1.0)) / COLS as f32;
const SUMMARY: f32 = (CW - 2. * DAY_WIDTH) / 2. - SUMMARY_GUTTER;

pub fn create_pdf() -> (PdfDocumentReference, PdfLayerReference, IndirectFontRef) {
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
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();

    (doc, canvas, font)
}

pub fn calc_line_breaks(text: &str, width: f32, height: f32) -> Vec<String> {
    let font_bytes = include_bytes!("Helvetica.ttf");

    // load the font reference for glyph_brush_layout
    let gbl_font = glyph_brush_layout::ab_glyph::FontRef::try_from_slice(font_bytes).unwrap();
    // put it into a slice of glyph_brush_layout font references
    let gbl_fonts = &[gbl_font];
    let glyphs = glyph_brush_layout::Layout::default().calculate_glyphs(
        gbl_fonts,
        &glyph_brush_layout::SectionGeometry {
            bounds: (width, height),
            ..Default::default()
        },
        &[glyph_brush_layout::SectionText {
            text,
            scale: gbl_fonts[0].pt_to_px_scale(5.0).unwrap(),
            font_id: glyph_brush_layout::FontId(0),
        }],
    );

    let mut line_indices = glyphs
        .iter()
        .enumerate() // enumerate will give us the start index into the sample text of the start of the line
        .group_by(|(_, glyph)| glyph.glyph.position.y) // group by "y" which is effectively equivalent to the index of the line
        .into_iter()
        .map(|(_, mut group)| group.next().unwrap().0)
        .collect::<Vec<_>>();
    line_indices.push(text.len());

    line_indices.iter().tuple_windows().map(|(start, end)| text[*start..*end].trim_end().to_string()).collect()



}

pub fn write_events(
    canvas: &PdfLayerReference,
    events: &Vec<Event>,
    pos_map: &HashMap<NaiveDate, (f32, f32)>,
    font: &IndirectFontRef,
) {
    for event in events {
        // text box
        let fill = if event.id == 0 {
            Color::Rgb(Rgb::new(0.7, 0.9, 0.7, None))
        } else {
            Color::Rgb(Rgb::new(0.9, 0.7, 0.7, None))
        };
        let text = event.summary.clone();
        let lines = calc_line_breaks(&text, (SUMMARY - 1.) * 1.33, event.num_days as f32 * ROW);

        for day in 0..event.num_days {
            if let Some((x, y)) =
                pos_map.get(&event.start.checked_add_days(Days::new(day as u64)).unwrap())
            {
                let shade_height = if day == 0 { ROW - 1. } else { ROW };
                let start_y = if day == event.num_days - 1 {
                    y + 1.
                } else {
                    *y
                };
                canvas.set_fill_color(fill.clone());
                rect(
                    &canvas,
                    x + 1. + (SUMMARY_GUTTER + SUMMARY) * event.id as f32,
                    start_y,
                    SUMMARY,
                    shade_height,
                    PaintMode::Fill,
                );
                canvas.set_fill_color(Color::Rgb(Rgb::new(0., 0., 0., None)));
                if (day as usize) <= lines.len() {
                    canvas.use_text(
                        &lines[day as usize],
                        FONT_HEIGHT,
                        Pt(x + 2. + (SUMMARY + SUMMARY_GUTTER) * event.id as f32).into(),
                        Pt(y + BASE).into(),
                        &font,
                    );

                }
            }
        }
    }
}

pub fn base_calendar(
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
                FONT_HEIGHT,
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
                        FONT_HEIGHT,
                        Pt(left + 2.).into(),
                        Pt(bottom + BASE).into(),
                        &font,
                    );

                    canvas.use_text(
                        &format!("{}", day_of_week.chars().next().unwrap()),
                        FONT_HEIGHT,
                        Pt(left + DAY_WIDTH + 2.).into(),
                        Pt(bottom + BASE).into(),
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
