mod patch;
pub use patch::{BicubicPatch, SurfacePatch};

use std::collections::{BTreeMap, BTreeSet};
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
    CreasedSpline,
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
                Self::CreasedSpline => "spline STEP requires closed crease rings without branches",
                Self::TooManyLevels => "subdivision level must be between 0 and 6",
            }
        )
    }
}

impl std::error::Error for Error {}

/// A closed, oriented quad cage.
#[derive(Clone, Debug)]
pub struct Cage {
    points: Vec<Point>,
    quads: Option<Vec<Quad>>,
    creases: BTreeSet<EdgeKey>,
}

#[derive(Debug, PartialEq)]
pub struct Mesh {
    pub points: Vec<Point>,
    pub quads: Vec<Quad>,
}

impl Cage {
    pub fn cube() -> Self {
        Self {
            quads: None,
            creases: BTreeSet::new(),
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
        Ok(Self {
            points,
            quads: None,
            creases: BTreeSet::new(),
        })
    }

    pub fn from_quads(points: Vec<Point>, quads: Vec<Quad>) -> Result<Self, Error> {
        if !(8..=128).contains(&points.len()) || !(6..=126).contains(&quads.len()) {
            return Err(Error::InvalidTopology);
        }
        for &point in &points {
            validate_point(point)?;
        }
        let mut rings = vec![BTreeMap::new(); points.len()];
        for &[a, b, c, d] in &quads {
            let mut vertices = [a, b, c, d];
            vertices.sort_unstable();
            if vertices
                .windows(2)
                .any(|pair| matches!(pair, [x, y] if x == y))
            {
                return Err(Error::InvalidTopology);
            }
            for (vertex, from, to) in [(a, d, b), (b, a, c), (c, b, d), (d, c, a)] {
                let ring = rings.get_mut(vertex).ok_or(Error::InvalidTopology)?;
                if ring.insert(from, to).is_some() {
                    return Err(Error::InvalidTopology);
                }
            }
        }
        for ring in rings {
            if ring.len() < 3 {
                return Err(Error::InvalidTopology);
            }
            let (&start, _) = ring.first_key_value().ok_or(Error::InvalidTopology)?;
            let mut next = start;
            for index in 0..ring.len() {
                next = *ring.get(&next).ok_or(Error::InvalidTopology)?;
                if next == start && index + 1 != ring.len() {
                    return Err(Error::InvalidTopology);
                }
            }
            if next != start {
                return Err(Error::InvalidTopology);
            }
        }
        Ok(Self {
            points,
            quads: Some(quads),
            creases: BTreeSet::new(),
        })
    }

    pub fn creases(&self) -> &BTreeSet<EdgeKey> {
        &self.creases
    }

    pub fn set_creases(&mut self, edges: impl IntoIterator<Item = EdgeKey>) -> Result<(), Error> {
        let mesh = self.preview(0)?;
        let mut valid = BTreeSet::new();
        for [a, b, c, d] in mesh.quads {
            valid.extend([
                edge_key(a, b),
                edge_key(b, c),
                edge_key(c, d),
                edge_key(d, a),
            ]);
        }
        let creases: BTreeSet<_> = edges.into_iter().map(|(a, b)| edge_key(a, b)).collect();
        if !creases.is_subset(&valid) {
            return Err(Error::InvalidTopology);
        }
        self.creases = creases;
        Ok(())
    }

    pub fn explicit_quads(&self) -> Option<&[Quad]> {
        self.quads.as_deref()
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
        self.refined(levels).map(|(mesh, _)| mesh)
    }

    fn refined(&self, levels: u8) -> Result<(Mesh, BTreeSet<EdgeKey>), Error> {
        if levels > 6 {
            return Err(Error::TooManyLevels);
        }
        let quads = if let Some(quads) = &self.quads {
            quads.clone()
        } else {
            let end = self.points.len() - 4;
            let mut quads = vec![[0, 3, 2, 1], [end, end + 1, end + 2, end + 3]];
            for start in (0..end).step_by(4) {
                for (a, b) in [(0, 1), (1, 2), (2, 3), (3, 0)] {
                    quads.push([start + a, start + b, start + b + 4, start + a + 4]);
                }
            }
            quads
        };
        let mut mesh = Mesh {
            points: self.points.clone(),
            quads,
        };
        let mut creases = self.creases.clone();
        for _ in 0..levels {
            (mesh, creases) = refine_with_creases(&mesh, &creases)?;
        }
        Ok((mesh, creases))
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
fn crease_neighbors(creases: &BTreeSet<EdgeKey>) -> BTreeMap<usize, Vec<usize>> {
    let mut sharp_neighbors: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for &(a, b) in creases {
        sharp_neighbors.entry(a).or_default().push(b);
        sharp_neighbors.entry(b).or_default().push(a);
    }
    sharp_neighbors
}

// Hard-crease masks from DeRose, Kass and Truong (1998).
// https://graphics.pixar.com/library/Geri/paper.pdf
fn refine_with_creases(
    mesh: &Mesh,
    creases: &BTreeSet<EdgeKey>,
) -> Result<(Mesh, BTreeSet<EdgeKey>), Error> {
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
    let sharp_neighbors = crease_neighbors(creases);
    for (vertex, (&position, ring)) in mesh.points.iter().zip(rings).enumerate() {
        match sharp_neighbors
            .get(&vertex)
            .map(Vec::as_slice)
            .unwrap_or_default()
        {
            &[a, b] => {
                points.push(scale(
                    add(scale(position, 6.), add(point(a)?, point(b)?)),
                    0.125,
                ));
                continue;
            }
            neighbors if neighbors.len() >= 3 => {
                points.push(position);
                continue;
            }
            _ => {}
        }
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

    let mut child_creases = BTreeSet::new();
    for (&(a, b), edge) in &mut edges {
        if edge.face_count != 2 {
            return Err(Error::InvalidTopology);
        }
        edge.refined_index = points.len();
        let sum = add(point(a)?, point(b)?);
        if creases.contains(&(a, b)) {
            points.push(scale(sum, 0.5));
            child_creases.extend([
                edge_key(a, edge.refined_index),
                edge_key(b, edge.refined_index),
            ]);
        } else {
            points.push(scale(add(sum, edge.face_sum), 0.25));
        }
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
    Ok((Mesh { points, quads }, child_creases))
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
