mod parser;
mod report;

use csaps::CubicSmoothingSpline;
use itertools::Itertools;
use plotters::prelude::*;
use std::env;
use std::io::{self, Read};

const DARK_BLUE: RGBColor = RGBColor(31, 120, 180);

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

    let x_range = 0.0..reports.max_x();
    let y_range = -0.5..reports.len() as f32 - 0.5;

    let size = (960, 300 + (18 * reports.len() as u32));
    let root = SVGBackend::new(filen, size).into_drawing_area();

    let mut chart = ChartBuilder::on(&root)
        .caption("Latency", ("sans-serif", 30).into_font())
        .margin((5).percent())
        .set_label_area_size(LabelAreaPosition::Left, (10).percent_width().min(60))
        .set_label_area_size(LabelAreaPosition::Bottom, (5).percent_width().min(40))
        .build_cartesian_2d(x_range, y_range)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .y_desc("Input")
        .x_desc("Latency (ms)")
        .y_label_formatter(&|v: &f32| reports[v.round() as usize].website.to_string())
        .y_labels(reports.len())
        .x_label_formatter(&|v: &f32| (v.round() as usize).to_string())
        .draw()?;

    for (idx, report) in reports.iter().enumerate() {
        let base = idx as f32;

        let report_max_y = report
            .detailed_latency
            .iter()
            .scan(0f32, |prev, (_ms, _, count, _)| {
                let count_diff = (*count as f32) - *prev;
                *prev = *count as f32;

                Some(count_diff)
            })
            .fold(f32::MIN, |a, b| a.max(b));

        let scaler = |i: f32| (i / report_max_y) * 0.9;

        let mut data: Vec<(f32, f32)> = report
            .detailed_latency
            .iter()
            .scan(0f32, |prev, (ms, _, count, _)| {
                let count_diff = (*count as f32) - *prev;
                *prev = *count as f32;

                Some((*ms, count_diff))
            })
            .group_by(|(x, _y)| *x)
            .into_iter()
            .map(|(x, ys)| {
                let s = ys.into_iter().map(|(_, ys)| ys).sum();

                (x, scaler(s))
            })
            .collect();

        data.sort_by(|(x1, _), (x2, _)| x1.partial_cmp(x2).unwrap());

        let xs: Vec<f32> = data.iter().map(|(x, _y)| *x).collect();
        let ys: Vec<f32> = data.iter().map(|(_x, y)| *y).collect();

        let smooth_ys = CubicSmoothingSpline::new(&xs, &ys)
            .with_smooth(0.99)
            .make()
            .unwrap()
            .evaluate(&xs)
            .unwrap();

        let smoothdata: Vec<(f32, f32)> = xs
            .iter()
            .zip(smooth_ys.iter())
            .map(|(x, y)| (*x, *y))
            .collect();

        chart.draw_series(AreaSeries::new(
            smoothdata.iter().map(|(x, y)| (*x, base + *y / 2.0)),
            base,
            &DARK_BLUE, // Palette99::pick(idx),
        ))?;

        chart.draw_series(AreaSeries::new(
            smoothdata.iter().map(|(x, y)| (*x, base - *y / 2.0)),
            base,
            &DARK_BLUE, // Palette99::pick(idx),
        ))?;
    }

    Ok(())
}
