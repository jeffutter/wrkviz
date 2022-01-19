use std::{ops::Index, slice::Iter};

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
            .fold(f32::MAX, |a, (_, b, _, _)| a.min(*b))
    }

    fn max_x(&self) -> f32 {
        self.detailed_latency
            .iter()
            .fold(f32::MIN, |a, (ms, _pct, _count, _)| a.max(*ms))
    }

    fn max_y(&self) -> f32 {
        self.detailed_latency
            .iter()
            .fold(f32::MIN, |a, (_ms, _pct, count, _)| a.max(*count as f32))
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

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> Index<usize> for Reports<'a> {
    type Output = Report<'a>;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}
