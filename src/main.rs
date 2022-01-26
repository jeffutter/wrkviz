mod line;
mod parser;
mod renderer;
mod report;
mod violin;

use clap::Parser;
use renderer::{Renderer, RendererInput};
use report::Reports;
use std::fs;
use std::path::PathBuf;

#[macro_use]
extern crate self_update;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    update: bool,

    #[clap(parse(from_os_str), required_unless_present = "update")]
    files: Vec<PathBuf>,

    #[clap(short, long, required_unless_present = "update")]
    filename: Option<String>,

    #[clap(arg_enum, short, long, default_value_t = RendererInput::Line)]
    renderer: RendererInput,
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

    let reports: Reports = args
        .files
        .iter()
        .flat_map(|path| {
            let data = fs::read_to_string(&path).unwrap();

            let (_rest, mut reports) = parser::parse(&data).unwrap();

            if let Some(filename) = path.file_name() {
                reports.iter_mut().for_each(|report| {
                    report.set_filename(filename.to_string_lossy().to_string());
                })
            }

            reports
        })
        .collect();

    let filename = &args.filename.unwrap();
    let renderer = Renderer::new(args.renderer, reports, filename);

    renderer.render()
}
