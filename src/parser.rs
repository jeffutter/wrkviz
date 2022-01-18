use crate::report::{Report, Reports};
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

pub fn parse(s: &str) -> IResult<&str, Reports> {
    let (rest, reports) = many1(report)(s)?;

    Ok((rest, Reports::new(reports)))
}
