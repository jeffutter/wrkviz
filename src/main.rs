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

#[macro_use]
extern crate self_update;

#[derive(clap::ArgEnum, Clone, Debug)]
enum RenderEnum {
    Violin,
    Line,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    update: bool,

    #[clap(short, long, required_unless_present = "update")]
    filename: Option<String>,

    #[clap(arg_enum, short, long, default_value_t = RenderEnum::Line)]
    renderer: RenderEnum,
}

fn update() -> Result<(), Box<dyn std::error::Error>> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("jeffutter")
        .repo_name("wrkviz")
        .bin_name("wrkviz")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    println!("Update status: `{}`!", status.version());

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.update {
        return update();
    };

    let mut stdin = io::stdin();
    let mut buffer = String::new();
    if stdin.read_to_string(&mut buffer).is_err() {
        println!("Unable to read input");
        return Ok(());
    }
    let (_rest, reports) = parser::parse(&*buffer).unwrap();

    let filename = &args.filename.unwrap();
    let renderer: Box<dyn Renderer> = match args.renderer {
        RenderEnum::Violin => Box::new(Violin::new(reports, filename)),
        RenderEnum::Line => Box::new(Line::new(reports, filename)),
    };

    renderer.render()
}
