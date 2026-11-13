mod patch;
pub use patch::{BicubicPatch, SurfacePatch};

use std::collections::BTreeMap;
use std::fmt;

pub type Point = [f64; 3];
type Quad = [usize; 4];
type EdgeKey = (usize, usize);

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidPoint,
    InvalidVertex,
    InvalidTopology,
    TooManyLevels,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::InvalidPoint => "expected finite coordinates within ±1e12",
                Self::InvalidVertex => "vertex index is out of range",
                Self::InvalidTopology => "expected a closed quad mesh",
                Self::TooManyLevels => "subdivision level must be between 0 and 6",
            }
        )
    }
}

impl std::error::Error for Error {}

/// A closed cage with four control points per cross-section.
#[derive(Clone, Debug)]
pub struct Cage {
    points: Vec<Point>,
}

#[derive(Debug, PartialEq)]
pub struct Mesh {
    pub points: Vec<Point>,
    pub quads: Vec<Quad>,
}

impl Cage {
    pub fn cube() -> Self {
        Self {
            points: vec![
                [-1., -1., -1.],
                [1., -1., -1.],
                [1., 1., -1.],
                [-1., 1., -1.],
                [-1., -1., 1.],
                [1., -1., 1.],
                [1., 1., 1.],
                [-1., 1., 1.],
            ],
        }
    }

    pub fn from_points(points: [Point; 8]) -> Result<Self, Error> {
        Self::from_sections(points.to_vec())
    }

    pub fn from_sections(points: Vec<Point>) -> Result<Self, Error> {
        if !(8..=32).contains(&points.len()) || !points.len().is_multiple_of(4) {
            return Err(Error::InvalidTopology);
        }
        for &point in &points {
            validate_point(point)?;
        }
        Ok(Self { points })
    }

    pub fn points(&self) -> &[Point] {
        &self.points
    }

    pub fn set_point(&mut self, vertex: usize, point: Point) -> Result<(), Error> {
        validate_point(point)?;
        *self.points.get_mut(vertex).ok_or(Error::InvalidVertex)? = point;
        Ok(())
    }

    /// Finite Catmull–Clark refinement for display.
    pub fn preview(&self, levels: u8) -> Result<Mesh, Error> {
        if levels > 6 {
            return Err(Error::TooManyLevels);
        }
        let end = self.points.len() - 4;
        let mut quads = vec![[0, 3, 2, 1], [end, end + 1, end + 2, end + 3]];
        for start in (0..end).step_by(4) {
            for (a, b) in [(0, 1), (1, 2), (2, 3), (3, 0)] {
                quads.push([start + a, start + b, start + b + 4, start + a + 4]);
            }
        }
        let mut mesh = Mesh {
            points: self.points.clone(),
            quads,
        };
        for _ in 0..levels {
            mesh = refine(&mesh)?;
        }
        Ok(mesh)
    }
}

fn validate_point(point: Point) -> Result<(), Error> {
    if point.into_iter().all(|coordinate| coordinate.abs() <= 1e12) {
        Ok(())
    } else {
        Err(Error::InvalidPoint)
    }
}

fn add(a: Point, b: Point) -> Point {
    let [ax, ay, az] = a;
    let [bx, by, bz] = b;
    [ax + bx, ay + by, az + bz]
}

fn scale([x, y, z]: Point, factor: f64) -> Point {
    [x * factor, y * factor, z * factor]
}

fn edge_key(a: usize, b: usize) -> EdgeKey {
    (a.min(b), a.max(b))
}

#[derive(Default)]
struct VertexRing {
    face_sum: Point,
    midpoint_sum: Point,
    face_count: u32,
}

#[derive(Default)]
struct Edge {
    face_sum: Point,
    face_count: u32,
    refined_index: usize,
}

// Catmull and Clark (1978), closed-surface refinement rules.
// https://doi.org/10.1016/0010-4485(78)90110-0
fn refine(mesh: &Mesh) -> Result<Mesh, Error> {
    let point = |id| mesh.points.get(id).copied().ok_or(Error::InvalidTopology);
    let mut rings: Vec<_> = mesh.points.iter().map(|_| VertexRing::default()).collect();
    let mut edges = BTreeMap::<EdgeKey, Edge>::new();
    let mut face_points = Vec::new();

    for &[a, b, c, d] in &mesh.quads {
        let center = scale(
            add(add(point(a)?, point(b)?), add(point(c)?, point(d)?)),
            0.25,
        );
        face_points.push(center);

        for (from, to) in [(a, b), (b, c), (c, d), (d, a)] {
            let edge = edges.entry(edge_key(from, to)).or_default();
            edge.face_sum = add(edge.face_sum, center);
            edge.face_count += 1;

            // On an oriented closed mesh each incident edge is outgoing once.
            let ring = rings.get_mut(from).ok_or(Error::InvalidTopology)?;
            ring.face_sum = add(ring.face_sum, center);
            ring.midpoint_sum = add(ring.midpoint_sum, scale(add(point(from)?, point(to)?), 0.5));
            ring.face_count += 1;
        }
    }

    let mut points = Vec::new();
    for (&position, ring) in mesh.points.iter().zip(rings) {
        if ring.face_count < 3 {
            return Err(Error::InvalidTopology);
        }
        let n = f64::from(ring.face_count);
        let average_face = scale(ring.face_sum, 1.0 / n);
        let average_midpoint = scale(ring.midpoint_sum, 1.0 / n);
        let weighted = add(
            add(average_face, scale(average_midpoint, 2.0)),
            scale(position, n - 3.0),
        );
        points.push(scale(weighted, 1.0 / n));
    }

    for (&(a, b), edge) in &mut edges {
        if edge.face_count != 2 {
            return Err(Error::InvalidTopology);
        }
        edge.refined_index = points.len();
        points.push(scale(add(add(point(a)?, point(b)?), edge.face_sum), 0.25));
    }

    let edge_index = |a, b| {
        edges
            .get(&edge_key(a, b))
            .map(|edge| edge.refined_index)
            .ok_or(Error::InvalidTopology)
    };
    let first_face_point = points.len();
    points.extend(face_points);
    let mut quads = Vec::new();
    for (face, &[a, b, c, d]) in mesh.quads.iter().enumerate() {
        let center = first_face_point + face;
        for (previous, vertex, next) in [(d, a, b), (a, b, c), (b, c, d), (c, d, a)] {
            quads.push([
                vertex,
                edge_index(vertex, next)?,
                center,
                edge_index(previous, vertex)?,
            ]);
        }
    }
    Ok(Mesh { points, quads })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn first_refinement_matches_cube_geometry() {
        let mesh = Cage::cube().preview(1).unwrap();
        assert_eq!(mesh.points.len(), 26);
        assert_eq!(mesh.quads.len(), 24);
        for coordinate in mesh.points[0] {
            assert!((coordinate + 5.0 / 9.0).abs() < 1e-12);
        }
        assert_eq!(mesh.points[8], [0.0, -0.75, -0.75]);
        assert_eq!(mesh.points[20], [0.0, 0.0, -1.0]);
    }

    #[test]
    fn edited_cube_remains_closed_and_oriented() {
        let mut cage = Cage::cube();
        cage.set_point(6, [1.5, 1., 2.]).unwrap();
        let mesh = cage.preview(3).unwrap();
        assert_eq!(mesh.quads.len(), 384);
        assert_ne!(mesh, Cage::cube().preview(3).unwrap());

        let mut directions = BTreeMap::new();
        for [a, b, c, d] in mesh.quads {
            for (from, to) in [(a, b), (b, c), (c, d), (d, a)] {
                assert!(from < mesh.points.len() && to < mesh.points.len());
                assert_ne!(from, to);
                let counts = directions.entry(edge_key(from, to)).or_insert((0, 0));
                if from < to {
                    counts.0 += 1;
                } else {
                    counts.1 += 1;
                }
            }
        }
        assert!(directions.values().all(|&counts| counts == (1, 1)));
    }
}
