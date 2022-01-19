mod line;
mod parser;
mod renderer;
mod report;
mod violin;

use crate::line::Line;
use crate::renderer::Renderer;
use crate::violin::Violin;
use clap::Parser;
use std::io::{self, Read};

#[derive(clap::ArgEnum, Clone, Debug)]
enum RenderEnum {
    Violin,
    Line,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(required = true)]
    filename: String,

    #[clap(arg_enum, short, long, default_value_t = RenderEnum::Line)]
    renderer: RenderEnum,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut stdin = io::stdin();
    let mut buffer = String::new();
    if stdin.read_to_string(&mut buffer).is_err() {
        println!("Unable to read input");
        return Ok(());
    }
    let (_rest, reports) = parser::parse(&*buffer).unwrap();

    let renderer: Box<dyn Renderer> = match args.renderer {
        RenderEnum::Violin => Box::new(Violin::new(reports, &args.filename)),
        RenderEnum::Line => Box::new(Line::new(reports, &args.filename)),
    };

    renderer.render()
}
