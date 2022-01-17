use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{
        digit1, line_ending, multispace0, multispace1, not_line_ending, space0, space1,
    },
    combinator::{map_res, recognize},
    multi::{many1, separated_list1},
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult,
};
use plotters::prelude::*;
use std::env;
use std::io::{self, Read};

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Report<'a> {
    website: &'a str,
    req_s: f32,
    hdr_histogram: Vec<(f32, f32)>,
    detailed_latency: Vec<(f32, f32, u32, f32)>,
    duration: u32,
}

fn start_line(s: &str) -> IResult<&str, (u32, &str)> {
    let (rest, _) = tag("Running ")(s)?;
    let (rest, seconds) = terminated(map_res(digit1, |s: &str| s.parse::<u32>()), tag("s "))(rest)?;
    let (rest, (_, website, _)) = tuple((tag("test @ "), not_line_ending, line_ending))(rest)?;
    Ok((rest, (seconds, website)))
}

fn decimal_float(s: &str) -> IResult<&str, f32> {
    map_res(recognize(tuple((digit1, tag("."), digit1))), |s: &str| {
        s.parse::<f32>()
    })(s)
}

fn milliseconds(s: &str) -> IResult<&str, f32> {
    let (rest, (v, unit)) = tuple((
        decimal_float,
        alt((tag("s"), tag("ms"), tag("us"), tag("ns"))),
    ))(s)?;

    let ms = match unit {
        "s" => v * 1000.0,
        "ms" => v,
        "us" => v / 1000.0,
        "ns" => v / 1000000.0,
        _ => unreachable!(),
    };

    Ok((rest, ms))
}

fn hdr_histogram(s: &str) -> IResult<&str, Vec<(f32, f32)>> {
    separated_list1(
        multispace1,
        separated_pair(
            preceded(space0, terminated(decimal_float, tag("%"))),
            space1,
            milliseconds,
        ),
    )(s)
}

fn detailed_latency(s: &str) -> IResult<&str, Vec<(f32, f32, u32, f32)>> {
    separated_list1(
        multispace1,
        tuple((
            preceded(multispace0, terminated(decimal_float, space0)),
            preceded(multispace0, terminated(decimal_float, space0)),
            preceded(
                multispace0,
                terminated(map_res(digit1, |s: &str| s.parse::<u32>()), space0),
            ),
            preceded(multispace0, terminated(decimal_float, space0)),
        )),
    )(s)
}

fn report(s: &str) -> IResult<&str, Report> {
    let (rest, (duration, website)) = start_line(s)?;
    let (rest, _) = take_until("Latency Distribution (HdrHistogram - Recorded Latency)")(rest)?;
    let (rest, _) = terminated(not_line_ending, line_ending)(rest)?;
    let (rest, hdr_histogram) = hdr_histogram(rest)?;
    let (rest, _) = take_until("Detailed Percentile spectrum")(rest)?;
    let (rest, _) = terminated(not_line_ending, line_ending)(rest)?;
    let (rest, _) = terminated(not_line_ending, line_ending)(rest)?;
    let (rest, detailed_latency) = detailed_latency(rest)?;
    let (rest, _) = terminated(not_line_ending, line_ending)(rest)?; // inf percentile
    let (rest, _) = take_until("Requests/sec:")(rest)?;
    let (rest, req_s) = preceded(
        tuple((tag("Requests/sec:"), multispace0)),
        terminated(decimal_float, line_ending),
    )(rest)?;
    let (rest, _) = take_until("Transfer/sec:")(rest)?;
    let (rest, _) = terminated(not_line_ending, line_ending)(rest)?;

    Ok((
        rest,
        Report {
            website,
            duration,
            hdr_histogram,
            detailed_latency,
            req_s,
        },
    ))
}

fn parse(s: &str) -> IResult<&str, Vec<Report>> {
    many1(report)(s)
}

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
    let (_rest, reports) = parse(&*buffer).unwrap();

    let min_x = reports.iter().fold(f32::MAX, |a, report| {
        let b = report
            .detailed_latency
            .iter()
            .fold(f32::MAX, |a, (_, b, _, _)| a.min(*b * 100.0));

        a.min(b)
    });
    let max_x = reports.iter().fold(f32::MIN, |a, report| {
        let b = report
            .detailed_latency
            .iter()
            .fold(f32::MIN, |a, (_, b, _, _)| a.max(*b * 100.0));

        a.max(b)
    });
    let max_y = reports.iter().fold(f32::MIN, |a, report| {
        let b = report
            .detailed_latency
            .iter()
            .fold(f32::MIN, |a, (b, _, _, _)| a.max(*b));

        a.max(b)
    });

    let root = BitMapBackend::new(filen, (640, 480)).into_drawing_area();
    let _x = root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Latency", ("sans-serif", 30).into_font())
        .margin(5)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(min_x..max_x, 0f32..max_y)?;

    chart
        .configure_mesh()
        .x_desc("Percentile")
        .y_desc("Milliseconds")
        .draw()?;

    for (idx, report) in reports.iter().enumerate() {
        let color = Palette99::pick(idx);

        let mut data = report
            .detailed_latency
            .iter()
            .map(|(ms, pct, _, _)| (pct * 100.0, *ms));

        chart
            .draw_series(LineSeries::new(&mut data, &color))?
            .label(format!("{} req/sec", report.req_s))
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

    // println!("\n{:?}\n\n{:?}", x, rest);

    Ok(())
}
