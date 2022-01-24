use crate::line;
use crate::report;
use crate::violin;

#[derive(clap::ArgEnum, Clone, Debug)]
pub enum RendererInput {
    Violin,
    Line,
}

pub enum Renderer<'a, 'b> {
    Violin(violin::Violin<'a, 'b>),
    Line(line::Line<'a, 'b>),
}

impl<'a, 'b> Renderer<'a, 'b> {
    pub fn new(input: RendererInput, reports: report::Reports<'a>, filename: &'b str) -> Self {
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
