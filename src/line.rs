use crate::report::Reports;
use plotters::prelude::*;

pub struct Line<'a> {
    reports: Reports,
    filename: &'a str,
}

impl<'a> Line<'a> {
    pub fn new(reports: Reports, filename: &'a str) -> Self {
        Self { reports, filename }
    }

    pub fn render(&self) -> Result<(), Box<dyn std::error::Error>> {
        let size = (960, 720);
        let root = SVGBackend::new(self.filename, size).into_drawing_area();

        let mut chart = ChartBuilder::on(&root)
            .caption("Latency", ("sans-serif", 30).into_font())
            .margin(5)
            .x_label_area_size(35)
            .y_label_area_size(60)
            .build_cartesian_2d(
                self.reports.min_pct()..self.reports.max_pct(),
                0f32..self.reports.max_latency(),
            )?;

        chart
            .configure_mesh()
            .x_desc("Percentile")
            .y_desc("Milliseconds")
            .draw()?;

        for (idx, report) in self.reports.iter().enumerate() {
            let color = Palette99::pick(idx);

            let mut data = report
                .detailed_latency
                .iter()
                .map(|(ms, pct, _, _)| (pct * 100.0, *ms));

            let label = match &report.filename {
                Some(filename) => {
                    format!(
                        "{}, {} connections {} req/sec",
                        filename, report.connections, report.req_s
                    )
                }
                None => {
                    format!(
                        "{}, {} connections {} req/sec",
                        report.website, report.connections, report.req_s
                    )
                }
            };

            chart
                .draw_series(LineSeries::new(&mut data, &color))?
                .label(label)
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
}
