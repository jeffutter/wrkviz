mod parser;
mod report;

use plotters::prelude::*;
use std::env;
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let filename = &args.get(1);
    if filename.is_none() {
        println!("Filename not provided as an argument.");
        return Ok(());
    }
    let filen = filename.unwrap();

    let mut stdin = io::stdin();
    let mut buffer = String::new();
    if stdin.read_to_string(&mut buffer).is_err() {
        println!("Unable to read input");
        return Ok(());
    }
    let (_rest, reports) = parser::parse(&*buffer).unwrap();

    let root = BitMapBackend::new(filen, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Latency", ("sans-serif", 30).into_font())
        .margin(5)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(reports.min_x()..reports.max_x(), 0f32..reports.max_y())?;

    chart
        .configure_mesh()
        .x_desc("Percentile")
        .y_desc("Milliseconds")
        .draw()?;

    for (idx, report) in reports.iter().enumerate() {
        let color = Palette99::pick(idx);

        let mut data = report
            .detailed_latency
            .iter()
            .map(|(ms, pct, _, _)| (pct * 100.0, *ms));

        chart
            .draw_series(LineSeries::new(&mut data, &color))?
            .label(format!("{} req/sec", report.req_s))
            .legend(move |(x, y)| {
                let color = Palette99::pick(idx);
                PathElement::new(vec![(x, y), (x + 20, y)], color)
            });
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}
