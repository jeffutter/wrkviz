use std::{ops::Index, slice::Iter, slice::IterMut};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Report {
    pub website: String,
    pub req_s: f32,
    pub hdr_histogram: Vec<(f32, f32)>,
    pub detailed_latency: Vec<(f32, f32, u32, f32)>,
    pub duration: u32,
    pub connections: u32,
    pub threads: u32,
    pub filename: Option<String>,
}

impl Report {
    fn min_pct(&self) -> f32 {
        self.detailed_latency
            .iter()
            .fold(f32::MAX, |a, (_ms, pct, _count, _)| a.min(*pct * 100.0))
    }

    fn max_pct(&self) -> f32 {
        self.detailed_latency
            .iter()
            .fold(f32::MIN, |a, (_ms, pct, _count, _)| a.max(*pct * 100.00))
    }

    fn max_latency(&self) -> f32 {
        self.detailed_latency
            .iter()
            .fold(f32::MIN, |a, (ms, _pct, _count, _)| a.max(*ms))
    }

    pub fn set_filename(&mut self, filename: String) {
        self.filename = Some(filename);
    }
}

#[derive(Debug, Clone)]
pub struct Reports(Vec<Report>);

impl Reports {
    pub fn new(reports: Vec<Report>) -> Self {
        Self(reports)
    }

    pub fn min_pct(&self) -> f32 {
        self.0
            .iter()
            .fold(f32::MAX, |a, report| report.min_pct().min(a))
    }

    pub fn max_pct(&self) -> f32 {
        self.0
            .iter()
            .fold(f32::MIN, |a, report| report.max_pct().max(a))
    }

    pub fn max_latency(&self) -> f32 {
        self.0
            .iter()
            .fold(f32::MIN, |a, report| report.max_latency().max(a))
    }

    pub fn iter(&self) -> Iter<Report> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Report> {
        self.0.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Index<usize> for Reports {
    type Output = Report;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

impl IntoIterator for Reports {
    type Item = Report;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<Report> for Reports {
    fn from_iter<I: IntoIterator<Item = Report>>(iter: I) -> Self {
        let mut c = Vec::new();

        for i in iter {
            c.push(i);
        }

        Reports(c)
    }
}
