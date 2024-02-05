use calendar2pdf::events::get_events;
use calendar2pdf::printable::{base_calendar, create_pdf, write_events};
use clap::Parser;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

fn main() {
    let cli = Cli::parse();
    let outpath = cli.output.unwrap();
    let (doc, canvas, font) = create_pdf();
    let pos_map = base_calendar(&canvas, &font);
    if let Ok(events) = get_events(&cli.calendars) {
        for event in &events {
            println!("{:?}", event);
        }
        write_events(&canvas, &events, &pos_map, &font);
    }
    doc.save(&mut BufWriter::new(File::create(outpath).unwrap()))
        .unwrap();
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// file-paths for calendar files (*.ics)
    #[arg(required = true)]
    calendars: Vec<PathBuf>,

    /// output file-path
    #[arg(short, long, value_name = "output", default_value("test.pdf"))]
    output: Option<PathBuf>,

    /// Calendar year
    #[arg(short, long, value_name = "year", default_value("2024"))]
    year: Option<usize>,

    /// Number of columns
    #[arg(short, long, value_parser = clap::value_parser!(u16).range(3..5))]
    columns: Option<usize>,
}
