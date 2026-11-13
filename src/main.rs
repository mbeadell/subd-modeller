mod step;

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::process::ExitCode;
use subd_prototype::{Cage, Point};

const CAGE_HEADER: &str = "subd-cube 1";
const USAGE: &str = "Usage:
  subd-prototype new CAGE [CAGE_WIDTH_MM]
  subd-prototype set CAGE VERTEX X Y Z OUTPUT_CAGE
  subd-prototype preview CAGE LEVEL OUTPUT_OBJ
  subd-prototype spline-step CAGE OUTPUT_STEP
  subd-prototype step CAGE LEVEL OUTPUT_STEP";

fn save(cage: &Cage, width_mm: Option<f64>, path: &str) -> Result<(), Box<dyn Error>> {
    let mut writer = output(path)?;
    if cage.points().len() != 8 {
        let width = width_mm.ok_or("sectioned cages require millimetre scale")?;
        writeln!(writer, "subd-sections 1")?;
        writeln!(writer, "width-mm {width}")?;
    } else if let Some(width) = width_mm {
        writeln!(writer, "subd-cube 2")?;
        writeln!(writer, "width-mm {width}")?;
    } else {
        writeln!(writer, "{CAGE_HEADER}")?;
    }
    for [x, y, z] in cage.points() {
        writeln!(writer, "{x} {y} {z}")?;
    }
    writer.flush()?;
    Ok(())
}

fn parse_width(value: &str) -> Result<f64, Box<dyn Error>> {
    let width: f64 = value.parse()?;
    if (1e-9..=1e12).contains(&width) {
        Ok(width)
    } else {
        Err("cage width must be between 1e-9 and 1e12 mm".into())
    }
}

fn load(path: &str) -> Result<(Cage, Option<f64>), Box<dyn Error>> {
    let mut lines = BufReader::new(File::open(path)?).lines();
    let header = lines.next().transpose()?;
    let sectioned = header.as_deref() == Some("subd-sections 1");
    let width_mm = match header.as_deref() {
        Some(CAGE_HEADER) => None,
        Some("subd-cube 2" | "subd-sections 1") => {
            let line = lines.next().transpose()?.ok_or("missing cage width")?;
            Some(parse_width(
                line.strip_prefix("width-mm ").ok_or("expected width-mm")?,
            )?)
        }
        _ => return Err("invalid cage header".into()),
    };
    let mut points = Vec::new();
    for line in lines {
        if points.len() == 32 {
            return Err("too many control points".into());
        }
        let line = line?;
        let mut fields = line.split_whitespace();
        let mut point: Point = [0.; 3];
        for coordinate in &mut point {
            *coordinate = fields.next().ok_or("missing coordinate")?.parse()?;
        }
        if fields.next().is_some() {
            return Err("too many coordinates".into());
        }
        points.push(point);
    }
    if !sectioned && points.len() != 8 {
        return Err("cube cages require eight control points".into());
    }
    Ok((Cage::from_sections(points)?, width_mm))
}

fn output(path: impl AsRef<Path>) -> std::io::Result<BufWriter<File>> {
    Ok(BufWriter::new(
        File::options().write(true).create_new(true).open(path)?,
    ))
}

fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    match args {
        [command, path] if command == "new" => save(&Cage::cube(), None, path),
        [command, path, width] if command == "new" => {
            save(&Cage::cube(), Some(parse_width(width)?), path)
        }
        [command, input, vertex, x, y, z, path] if command == "set" => {
            let (mut cage, width_mm) = load(input)?;
            cage.set_point(vertex.parse()?, [x.parse()?, y.parse()?, z.parse()?])?;
            save(&cage, width_mm, path)
        }
        [command, input, level, path] if command == "preview" => {
            let (cage, width_mm) = load(input)?;
            let mesh = cage.preview(level.parse()?)?;
            let scale = width_mm.map_or(1.0, |width| width * 0.5);
            let mut writer = output(path)?;
            for [x, y, z] in &mesh.points {
                writeln!(writer, "v {} {} {}", x * scale, y * scale, z * scale)?;
            }
            for [a, b, c, d] in &mesh.quads {
                writeln!(writer, "f {} {} {} {}", a + 1, b + 1, c + 1, d + 1)?;
            }
            writer.flush()?;
            Ok(())
        }
        [command, input, path] if command == "spline-step" => {
            let (cage, width) = load(input)?;
            let width = width.ok_or("STEP export requires a cage with a width in millimetres")?;
            let contents = step::encode_splines(cage.preview(3)?, width, cage.surface_patches()?)?;
            let mut writer = output(path)?;
            writer.write_all(contents.as_bytes())?;
            writer.flush()?;
            Ok(())
        }
        [command, input, level, path] if command == "step" => {
            let (cage, width) = load(input)?;
            let width = width.ok_or("STEP export requires a cage with a width in millimetres")?;
            let level = level.parse()?;
            if level > 3 {
                return Err("STEP preview supports levels 0 through 3".into());
            }
            let contents = step::encode(cage.preview(level)?, width)?;
            let mut writer = output(path)?;
            writer.write_all(contents.as_bytes())?;
            writer.flush()?;
            Ok(())
        }
        [] => {
            println!("{USAGE}");
            Ok(())
        }
        [flag] if flag == "--help" || flag == "-h" => {
            println!("{USAGE}");
            Ok(())
        }
        _ => Err(USAGE.into()),
    }
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
