mod stl;

use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::process::ExitCode;
use subd_prototype::cage_file::{parse_width, read_cage, CAGE_HEADER};
use subd_prototype::{step, Cage, Point};

const USAGE: &str = "Usage:
  subd-prototype new CAGE [CAGE_WIDTH_MM]
  subd-prototype set CAGE VERTEX X Y Z OUTPUT_CAGE
  subd-prototype preview CAGE LEVEL OUTPUT_OBJ
  subd-prototype stl CAGE LEVEL OUTPUT_STL
  subd-prototype spline-preview CAGE OUTPUT_OBJ
  subd-prototype spline-step CAGE OUTPUT_STEP
  subd-prototype step CAGE LEVEL OUTPUT_STEP";

fn save(cage: &Cage, width_mm: Option<f64>, path: &str) -> Result<(), Box<dyn Error>> {
    let mut writer = output(path)?;
    if cage.explicit_quads().is_some() {
        let width = width_mm.ok_or("quad cages require millimetre scale")?;
        writeln!(writer, "subd-quads 1")?;
        writeln!(writer, "width-mm {width}")?;
    } else if cage.points().len() != 8 {
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
    if let Some(quads) = cage.explicit_quads() {
        for [a, b, c, d] in quads {
            writeln!(writer, "q {a} {b} {c} {d}")?;
        }
    }
    for (&(a, b), &weight) in cage.creases() {
        if weight.is_infinite() {
            writeln!(writer, "c {a} {b}")?;
        } else {
            writeln!(writer, "c {a} {b} {weight}")?;
        }
    }
    writer.flush()?;
    Ok(())
}

fn load(path: &str) -> Result<(Cage, Option<f64>), Box<dyn Error>> {
    read_cage(BufReader::new(File::open(path)?))
}

fn output(path: impl AsRef<Path>) -> std::io::Result<BufWriter<File>> {
    Ok(BufWriter::new(
        File::options().write(true).create_new(true).open(path)?,
    ))
}

// Compare the two endpoint derivatives without depending on model scale.
fn has_flat_corner(
    [[a, ab, ba, b], [ad, _, _, bc], [da, _, _, cb], [d, dc, cd, c]]: [[Point; 4]; 4],
) -> bool {
    [(a, ab, ad), (b, ba, bc), (c, cd, cb), (d, dc, da)]
        .into_iter()
        .any(|([x, y, z], [px, py, pz], [qx, qy, qz])| {
            let (u, v, w) = (px - x, py - y, pz - z);
            let (a, b, c) = (qx - x, qy - y, qz - z);
            let lengths = u.hypot(v).hypot(w) * a.hypot(b).hypot(c);
            let cross = (v * c - w * b).hypot(w * a - u * c).hypot(u * b - v * a);
            cross <= lengths * 2_f64.to_radians().sin()
        })
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
        [command, input, path] if command == "spline-preview" => {
            let (cage, width) = load(input)?;
            let scale = width.map_or(1., |width| width * 0.5);
            let patches = cage.surface_patches()?;
            let mut writer = output(path)?;
            for (index, patch) in patches.iter().enumerate() {
                if step::planar_frame(patch.control_points).is_none()
                    && has_flat_corner(patch.control_points)
                {
                    writeln!(writer, "# flat-corner {index}")?;
                }
                let basis = |t: f64| {
                    let s = 1. - t;
                    [s * s * s, 3. * s * s * t, 3. * s * t * t, t * t * t]
                };
                for v in [0., 0.5, 1.] {
                    for u in [0., 0.5, 1.] {
                        let mut point = [0.; 3];
                        for (row, bv) in patch.control_points.iter().zip(basis(v)) {
                            for (control, bu) in row.iter().zip(basis(u)) {
                                for (coordinate, component) in point.iter_mut().zip(control) {
                                    *coordinate += component * bu * bv * scale;
                                }
                            }
                        }
                        let [x, y, z] = point;
                        writeln!(writer, "v {x} {y} {z}")?;
                    }
                }
                for offset in [0, 1, 3, 4] {
                    let a = index * 9 + offset + 1;
                    writeln!(writer, "f {a} {} {} {}", a + 1, a + 4, a + 3)?;
                }
            }
            writer.flush()?;
            Ok(())
        }
        [command, input, level, path] if command == "stl" => {
            let (cage, width) = load(input)?;
            let width = width.ok_or("STL export requires a cage width in millimetres")?;
            let level = level.parse()?;
            if level > 5 {
                return Err("STL supports refinement levels 0 through 5".into());
            }
            stl::write(cage.preview(level)?, width * 0.5, output(path)?)?;
            Ok(())
        }
        [command, input, path] if command == "spline-step" => {
            let (cage, width) = load(input)?;
            let width = width.ok_or("STEP export requires a cage with a width in millimetres")?;
            let contents = step::encode_splines(&cage, width)?;
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
