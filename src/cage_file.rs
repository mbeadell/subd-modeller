use crate::{Cage, Point};
use std::error::Error;
use std::io::BufRead;

pub const CAGE_HEADER: &str = "subd-cube 1";

pub fn parse_width(value: &str) -> Result<f64, Box<dyn Error>> {
    let width: f64 = value.parse()?;
    if (1e-9..=1e12).contains(&width) {
        Ok(width)
    } else {
        Err("cage width must be between 1e-9 and 1e12 mm".into())
    }
}

/// Read a saved cage without requiring access to a filesystem.
pub fn read_cage(input: impl BufRead) -> Result<(Cage, Option<f64>), Box<dyn Error>> {
    let mut lines = input.lines();
    let header = lines.next().transpose()?;
    let explicit = header.as_deref() == Some("subd-quads 1");
    let sectioned = header.as_deref() == Some("subd-sections 1");
    let width_mm = match header.as_deref() {
        Some(CAGE_HEADER) => None,
        Some("subd-cube 2" | "subd-sections 1" | "subd-quads 1") => {
            let line = lines.next().transpose()?.ok_or("missing cage width")?;
            Some(parse_width(
                line.strip_prefix("width-mm ").ok_or("expected width-mm")?,
            )?)
        }
        _ => return Err("invalid cage header".into()),
    };
    let mut points = Vec::new();
    let mut quads = Vec::new();
    let mut creases = Vec::new();
    for line in lines {
        let line = line?;
        // Editor metadata is optional and does not alter geometry exchange.
        if line.starts_with("# editor-state ") {
            continue;
        }
        if let Some(edge) = line.strip_prefix("c ") {
            if creases.len() == 252 {
                return Err("too many creases".into());
            }
            let mut indices = edge.split_whitespace();
            let a = indices.next().ok_or("missing crease endpoint")?.parse()?;
            let b = indices.next().ok_or("missing crease endpoint")?.parse()?;
            let weight = indices
                .next()
                .map(str::parse)
                .transpose()?
                .unwrap_or(f64::INFINITY);
            if indices.next().is_some() {
                return Err("expected two crease endpoints and optional weight".into());
            }
            creases.push((a, b, weight));
            continue;
        }
        if let Some(face) = line.strip_prefix("q ").filter(|_| explicit) {
            if quads.len() == 126 {
                return Err("too many quad faces".into());
            }
            let indices = face
                .split_whitespace()
                .map(str::parse)
                .collect::<Result<Vec<usize>, _>>()?;
            let quad = indices
                .try_into()
                .map_err(|_| "expected four quad indices")?;
            quads.push(quad);
            continue;
        }
        if points.len() == if explicit { 128 } else { 32 } {
            return Err("too many control points".into());
        }
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
    let mut cage = if explicit {
        Cage::from_quads(points, quads)?
    } else {
        if !sectioned && points.len() != 8 {
            return Err("cube cages require eight control points".into());
        }
        Cage::from_sections(points)?
    };
    cage.set_crease_weights(creases)?;
    Ok((cage, width_mm))
}
