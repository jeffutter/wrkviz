use std::slice::Iter;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Report<'a> {
    pub website: &'a str,
    pub req_s: f32,
    pub hdr_histogram: Vec<(f32, f32)>,
    pub detailed_latency: Vec<(f32, f32, u32, f32)>,
    pub duration: u32,
}

impl Report<'_> {
    fn min_x(&self) -> f32 {
        self.detailed_latency
            .iter()
            .fold(f32::MAX, |a, (_, b, _, _)| a.min(*b * 100.0))
    }

    fn max_x(&self) -> f32 {
        self.detailed_latency
            .iter()
            .fold(f32::MIN, |a, (_, b, _, _)| a.max(*b * 100.0))
    }

    fn max_y(&self) -> f32 {
        self.detailed_latency
            .iter()
            .fold(f32::MIN, |a, (b, _, _, _)| a.max(*b))
    }
}

pub struct Reports<'a>(Vec<Report<'a>>);

impl<'a> Reports<'a> {
    pub fn new(reports: Vec<Report<'a>>) -> Self {
        Self(reports)
    }

    pub fn min_x(&self) -> f32 {
        self.0
            .iter()
            .fold(f32::MAX, |a, report| report.min_x().min(a))
    }

    pub fn max_x(&self) -> f32 {
        self.0
            .iter()
            .fold(f32::MIN, |a, report| report.max_x().max(a))
    }

    pub fn max_y(&self) -> f32 {
        self.0
            .iter()
            .fold(f32::MIN, |a, report| report.max_y().max(a))
    }

    pub fn iter(&self) -> Iter<Report> {
        self.0.iter()
    }
}