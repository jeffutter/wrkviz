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

    let max_y = reports.iter().fold(f32::MIN, |acc, report| {
        let m = report
            .detailed_latency
            .iter()
            .scan(0f32, |prev, (_ms, _, count, _)| {
                let count_diff = (*count as f32) - *prev;
                *prev = *count as f32;

                Some(count_diff)
            })
            .fold(f32::MIN, |a, b| a.max(b));
        acc.max(m)
    }) * (reports.iter().len() as f32);

    let root = BitMapBackend::new(filen, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Latency", ("sans-serif", 30).into_font())
        .margin(5)
        .x_label_area_size(35)
        // .y_label_area_size(60)
        .build_cartesian_2d(0.0..reports.max_x(), (max_y * -1.0)..max_y)?;

    chart
        .configure_mesh()
        .x_desc("Milliseconds")
        .y_desc("Request Count")
        .x_label_formatter(&|v: &f32| (v.round() as usize).to_string())
        .draw()?;

    for (idx, report) in reports.iter().enumerate() {
        let base = idx as f32 * (max_y / 2f32);

        let data: Vec<(f32, f32)> = report
            .detailed_latency
            .iter()
            .scan(0f32, |prev, (ms, _, count, _)| {
                let count_diff = (*count as f32) - *prev;
                *prev = *count as f32;

                Some((*ms, count_diff))
            })
            .collect();

        chart.draw_series(AreaSeries::new(
            data.iter().map(|(x, y)| (*x, base + *y / 2.0)),
            base,
            Palette99::pick(idx),
        ))?;

        chart.draw_series(AreaSeries::new(
            data.iter().map(|(x, y)| (*x, base - *y / 2.0)),
            base,
            Palette99::pick(idx),
        ))?;
    }

    Ok(())
}
