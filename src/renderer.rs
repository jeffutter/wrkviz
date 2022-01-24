use crate::line;
use crate::report;
use crate::violin;

#[derive(clap::ArgEnum, Clone, Debug)]
pub enum RendererInput {
    Violin,
    Line,
}

pub enum Renderer<'a> {
    Violin(violin::Violin<'a>),
    Line(line::Line<'a>),
}

impl<'a> Renderer<'a> {
    pub fn new(input: RendererInput, reports: report::Reports, filename: &'a str) -> Self {
        match input {
            RendererInput::Violin => Renderer::Violin(violin::Violin::new(reports, filename)),
            RendererInput::Line => Renderer::Line(line::Line::new(reports, filename)),
        }
    }

    pub fn render(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self {
            Self::Violin(violin) => violin.render(),
            Self::Line(line) => line.render(),
        }
    }
}
