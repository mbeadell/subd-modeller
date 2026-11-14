//! Exact uniform bicubic B-spline spans in regular Catmull–Clark regions.
//! The regular-grid equivalence follows Catmull and Clark (1978),
//! https://doi.org/10.1016/0010-4485(78)90110-0.

use std::collections::BTreeMap;

use crate::{Cage, Error, Point};

/// One unit knot span, with control points indexed by row then column.
#[derive(Debug)]
pub struct BicubicPatch {
    pub face: usize,
    pub control_points: [[Point; 4]; 4],
}

impl BicubicPatch {
    /// Convert the uniform B-spline span to an equivalent Bézier patch on
    /// [0, 1]². Rows run in v and columns run in u.
    pub fn bezier_control_points(&self) -> [[Point; 4]; 4] {
        fn convert([a, b, c, d]: [Point; 4]) -> [Point; 4] {
            use crate::{add, scale};
            [
                scale(add(add(a, scale(b, 4.0)), c), 1.0 / 6.0),
                scale(add(scale(b, 2.0), c), 1.0 / 3.0),
                scale(add(b, scale(c, 2.0)), 1.0 / 3.0),
                scale(add(add(b, scale(c, 4.0)), d), 1.0 / 6.0),
            ]
        }

        transpose(transpose(self.control_points.map(convert)).map(convert))
    }
}

fn transpose([a, b, c, d]: [[Point; 4]; 4]) -> [[Point; 4]; 4] {
    let [a0, a1, a2, a3] = a;
    let [b0, b1, b2, b3] = b;
    let [c0, c1, c2, c3] = c;
    let [d0, d1, d2, d3] = d;
    [
        [a0, b0, c0, d0],
        [a1, b1, c1, d1],
        [a2, b2, c2, d2],
        [a3, b3, c3, d3],
    ]
}

impl Cage {
    /// Extract exact spans from the refined control mesh. Faces touching an
    /// extraordinary vertex are omitted. Two refinements expose regular spans
    /// on the cube while retaining three unresolved faces per original corner.
    pub fn regular_patches(&self, levels: u8) -> Result<Vec<BicubicPatch>, Error> {
        if !self.creases().is_empty() {
            return Err(Error::CreasedSpline);
        }
        let mesh = self.preview(levels)?;
        let mut successors = BTreeMap::new();
        let mut valences = BTreeMap::new();
        for &[a, b, c, d] in &mesh.quads {
            for (from, to, next) in [(a, b, c), (b, c, d), (c, d, a), (d, a, b)] {
                successors.insert((from, to), next);
                *valences.entry(from).or_insert(0) += 1;
            }
        }

        let opposite = |a, b| {
            let c = successors
                .get(&(b, a))
                .copied()
                .ok_or(Error::InvalidTopology)?;
            let d = successors
                .get(&(a, c))
                .copied()
                .ok_or(Error::InvalidTopology)?;
            Ok::<_, Error>((c, d))
        };

        let mut patches = Vec::new();
        for (face, &[a, b, c, d]) in mesh.quads.iter().enumerate() {
            if [a, b, c, d]
                .iter()
                .any(|vertex| valences.get(vertex) != Some(&4))
            {
                continue;
            }

            let (south_a, south_b) = opposite(a, b)?;
            let (east_b, east_c) = opposite(b, c)?;
            let (north_c, north_d) = opposite(c, d)?;
            let (west_d, west_a) = opposite(d, a)?;
            let (_, southwest) = opposite(a, south_a)?;
            let (_, southeast) = opposite(b, east_b)?;
            let (_, northeast) = opposite(c, north_c)?;
            let (_, northwest) = opposite(d, west_d)?;

            let indices = [
                [southwest, south_a, south_b, southeast],
                [west_a, a, b, east_b],
                [west_d, d, c, east_c],
                [northwest, north_d, north_c, northeast],
            ];
            let mut control_points = [[[0.; 3]; 4]; 4];
            for (row, indices) in control_points.iter_mut().zip(indices) {
                for (point, index) in row.iter_mut().zip(indices) {
                    *point = mesh
                        .points
                        .get(index)
                        .copied()
                        .ok_or(Error::InvalidTopology)?;
                }
            }
            patches.push(BicubicPatch {
                face,
                control_points,
            });
        }
        Ok(patches)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing, clippy::unwrap_used)]
    use super::*;

    fn evaluate(patch: &BicubicPatch, u: f64, v: f64) -> Point {
        fn basis(t: f64) -> [f64; 4] {
            [
                (1.0 - t).powi(3) / 6.0,
                (3.0 * t.powi(3) - 6.0 * t * t + 4.0) / 6.0,
                (-3.0 * t.powi(3) + 3.0 * t * t + 3.0 * t + 1.0) / 6.0,
                t.powi(3) / 6.0,
            ]
        }
        let mut point = [0.; 3];
        for (row, y) in patch.control_points.iter().zip(basis(v)) {
            for (control, x) in row.iter().zip(basis(u)) {
                point = crate::add(point, crate::scale(*control, x * y));
            }
        }
        point
    }

    fn boundary(patch: &BicubicPatch, edge: usize, t: f64) -> Point {
        match edge {
            0 => evaluate(patch, t, 0.),
            1 => evaluate(patch, 1., t),
            2 => evaluate(patch, 1. - t, 1.),
            _ => evaluate(patch, 0., 1. - t),
        }
    }

    #[test]
    fn bezier_conversion_preserves_edited_surface() {
        fn bernstein(t: f64) -> [f64; 4] {
            let s = 1.0 - t;
            [s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t]
        }

        let mut cage = Cage::cube();
        cage.set_point(6, [1.5, 0.8, 2.0]).unwrap();
        let patches = cage.regular_patches(2).unwrap();
        assert_eq!(patches.len(), 72);
        // Four distinct parameters per axis determine a bicubic polynomial.
        for patch in patches {
            let bezier = patch.bezier_control_points();
            for u in [0., 0.25, 0.65, 1.] {
                for v in [0., 0.15, 0.85, 1.] {
                    let mut point = [0.; 3];
                    for (row, y) in bezier.iter().zip(bernstein(v)) {
                        for (&control, x) in row.iter().zip(bernstein(u)) {
                            point = crate::add(point, crate::scale(control, x * y));
                        }
                    }
                    assert!(point
                        .iter()
                        .zip(evaluate(&patch, u, v))
                        .all(|(a, b)| (a - b).abs() < 1e-12));
                }
            }
        }
    }

    #[test]
    fn regular_spans_meet_after_cage_edit() {
        let mut cage = Cage::cube();
        assert!(cage.regular_patches(0).unwrap().is_empty());
        cage.set_point(6, [1.5, 1., 2.]).unwrap();
        let mesh = cage.preview(2).unwrap();
        let patches = cage.regular_patches(2).unwrap();
        assert_eq!(patches.len(), 72);

        let mut edges = BTreeMap::new();
        let mut seams = 0;
        for patch in &patches {
            let [a, b, c, d] = mesh.quads[patch.face];
            for (edge, (from, to)) in [(a, b), (b, c), (c, d), (d, a)].into_iter().enumerate() {
                if let Some((neighbor, neighbor_edge)) = edges.remove(&(to, from)) {
                    seams += 1;
                    for t in [0., 0.2, 0.5, 0.8, 1.] {
                        let p = boundary(patch, edge, t);
                        let q = boundary(neighbor, neighbor_edge, 1. - t);
                        assert!(p.iter().zip(q).all(|(x, y)| (x - y).abs() < 1e-12));
                    }
                } else {
                    edges.insert((from, to), (patch, edge));
                }
            }
        }
        assert_eq!(seams, 120);
    }
}

/// A complete level-three surface, with exact regular patches and interpolated
/// patches at extraordinary vertices. Rows run in v and columns in u.
#[derive(Debug)]
pub struct SurfacePatch {
    pub face: usize,
    pub exact: bool,
    pub control_points: [[Point; 4]; 4],
}

impl Cage {
    pub fn surface_patches(&self) -> Result<Vec<SurfacePatch>, Error> {
        if crate::crease_neighbors(self.creases())
            .values()
            .any(|neighbors| neighbors.len() != 2)
        {
            return Err(Error::CreasedSpline);
        }
        let regular: BTreeMap<_, _> = if self.creases().is_empty() {
            self.regular_patches(3)?
                .into_iter()
                .map(|patch| (patch.face, patch.bezier_control_points()))
                .collect()
        } else {
            // Use the same interpolation on both sides of every seam so
            // neighboring patches share the same cubic boundary.
            BTreeMap::new()
        };
        let (mut mesh, mut creases) = self.refined(3)?;
        let mut grids = BTreeMap::new();
        for face in 0..mesh.quads.len() {
            if !regular.contains_key(&face) {
                grids.insert(face, vec![(face, [(0, 0), (4, 0), (4, 4), (0, 4)])]);
            }
        }

        for _ in 0..2 {
            for cells in grids.values_mut() {
                *cells = children(cells);
            }
            (mesh, creases) = crate::refine_with_creases(&mesh, &creases)?;
        }
        let limits = limit_points_with_creases(&mesh, &creases)?;

        let mut patches = Vec::new();
        for (face, cells) in grids {
            let mut samples = BTreeMap::new();
            for (child, coordinates) in cells {
                let quad = mesh.quads.get(child).ok_or(Error::InvalidTopology)?;
                for (&vertex, coordinate) in quad.iter().zip(coordinates) {
                    samples.insert(
                        coordinate,
                        *limits.get(vertex).ok_or(Error::InvalidTopology)?,
                    );
                }
            }
            let mut grid = [[[0.; 3]; 4]; 4];
            for (row, v) in grid.iter_mut().zip([0, 1, 3, 4]) {
                for (point, u) in row.iter_mut().zip([0, 1, 3, 4]) {
                    *point = *samples.get(&(u, v)).ok_or(Error::InvalidTopology)?;
                }
            }
            patches.push(SurfacePatch {
                face,
                exact: false,
                control_points: transpose(transpose(grid.map(interpolate)).map(interpolate)),
            });
        }
        patches.extend(
            regular
                .into_iter()
                .map(|(face, control_points)| SurfacePatch {
                    face,
                    exact: true,
                    control_points,
                }),
        );
        patches.sort_by_key(|patch| patch.face);
        Ok(patches)
    }
}

type Cell = (usize, [(usize, usize); 4]);

fn children(cells: &[Cell]) -> Vec<Cell> {
    let midpoint = |(x, y), (u, v)| ((x + u) / 2, (y + v) / 2);
    let mut result = Vec::new();
    for &(face, [a, b, c, d]) in cells {
        let ab = midpoint(a, b);
        let bc = midpoint(b, c);
        let cd = midpoint(c, d);
        let da = midpoint(d, a);
        let center = midpoint(a, c);
        result.extend([
            (face * 4, [a, ab, center, da]),
            (face * 4 + 1, [b, bc, center, ab]),
            (face * 4 + 2, [c, cd, center, bc]),
            (face * 4 + 3, [d, da, center, cd]),
        ]);
    }
    result
}

#[cfg(test)]
fn limit_points(mesh: &crate::Mesh) -> Result<Vec<Point>, Error> {
    limit_points_with_creases(mesh, &Default::default())
}

fn limit_points_with_creases(
    mesh: &crate::Mesh,
    creases: &std::collections::BTreeSet<crate::EdgeKey>,
) -> Result<Vec<Point>, Error> {
    use crate::{add, scale};
    let mut rings = vec![([0.; 3], [0.; 3], 0_u32); mesh.points.len()];
    let point = |id| mesh.points.get(id).copied().ok_or(Error::InvalidTopology);
    for &[a, b, c, d] in &mesh.quads {
        for (center, adjacent, opposite) in [(a, b, c), (b, c, d), (c, d, a), (d, a, b)] {
            let (neighbors, diagonals, count) =
                rings.get_mut(center).ok_or(Error::InvalidTopology)?;
            *neighbors = add(*neighbors, point(adjacent)?);
            *diagonals = add(*diagonals, point(opposite)?);
            *count += 1;
        }
    }
    let sharp = crate::crease_neighbors(creases);
    mesh.points
        .iter()
        .zip(rings)
        .enumerate()
        .map(|(vertex, (&position, (neighbors, diagonals, count)))| {
            if let Some(&[a, b]) = sharp.get(&vertex).map(Vec::as_slice) {
                return Ok(scale(
                    add(add(point(a)?, point(b)?), scale(position, 4.)),
                    1. / 6.,
                ));
            }
            if count < 3 {
                return Err(Error::InvalidTopology);
            }
            let n = f64::from(count);
            Ok(scale(
                add(add(scale(position, n * n), scale(neighbors, 4.)), diagonals),
                1. / (n * (n + 5.)),
            ))
        })
        .collect()
}

// Invert the cubic Bernstein basis at 0, 1/4, 3/4, and 1.
fn interpolate([a, b, c, d]: [Point; 4]) -> [Point; 4] {
    use crate::{add, scale};
    let left = add(scale(b, 64.), scale(add(scale(a, 27.), d), -1.));
    let right = add(scale(c, 64.), scale(add(a, scale(d, 27.)), -1.));
    [
        a,
        scale(add(scale(left, 3.), scale(right, -1.)), 1. / 72.),
        scale(add(scale(right, 3.), scale(left, -1.)), 1. / 72.),
        d,
    ]
}

#[cfg(test)]
mod fitting_tests {
    #![allow(clippy::indexing_slicing, clippy::unwrap_used)]
    use super::*;

    fn sample(net: &[[Point; 4]; 4], u: f64, v: f64) -> Point {
        fn blend(points: [Point; 4], t: f64) -> Point {
            let s = 1. - t;
            points
                .into_iter()
                .zip([s.powi(3), 3. * s * s * t, 3. * s * t * t, t.powi(3)])
                .fold([0.; 3], |sum, (p, weight)| {
                    crate::add(sum, crate::scale(p, weight))
                })
        }
        blend(net.map(|row| blend(row, u)), v)
    }

    fn distance(a: Point, b: Point) -> f64 {
        a.into_iter()
            .zip(b)
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    fn boundary(net: &[[Point; 4]; 4], edge: usize) -> [Point; 4] {
        match edge {
            0 => net[0],
            1 => net.map(|row| row[3]),
            2 => net[3]
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            _ => [net[3][0], net[2][0], net[1][0], net[0][0]],
        }
    }

    fn normal(net: &[[Point; 4]; 4], edge: usize, t: f64) -> Point {
        let (u, v) = match edge {
            0 => (t, 0.),
            1 => (1., t),
            2 => (1. - t, 1.),
            _ => (0., 1. - t),
        };
        let difference = |a, b| crate::add(a, crate::scale(b, -1.));
        // Centered difference of a cubic, including at the span endpoints.
        let [a, b, c] = difference(sample(net, u + 1e-5, v), sample(net, u - 1e-5, v));
        let [x, y, z] = difference(sample(net, u, v + 1e-5), sample(net, u, v - 1e-5));
        let n = [b * z - c * y, c * x - a * z, a * y - b * x];
        crate::scale(n, 1. / distance(n, [0.; 3]))
    }

    #[test]
    fn closed_crease_ring_keeps_shared_boundary_and_sharp_normals() {
        let mut cage = Cage::cube();
        cage.set_creases([(4, 5), (5, 6), (6, 7), (7, 4)]).unwrap();
        let (mesh, creases) = cage.refined(3).unwrap();
        let patches = cage.surface_patches().unwrap();
        let mut boundaries = BTreeMap::new();
        let mut checked = 0;
        for patch in &patches {
            let [a, b, c, d] = mesh.quads[patch.face];
            for (side, (a, b)) in [(a, b), (b, c), (c, d), (d, a)].into_iter().enumerate() {
                if !creases.contains(&crate::edge_key(a, b)) {
                    continue;
                }
                let mut curve = boundary(&patch.control_points, side);
                let n = normal(&patch.control_points, side, 0.5);
                if let Some((other, other_normal)) = boundaries.remove(&(b, a)) {
                    curve.reverse();
                    assert!(curve
                        .into_iter()
                        .zip(other)
                        .all(|(p, q)| distance(p, q) < 1e-12));
                    let dot: f64 = n.into_iter().zip(other_normal).map(|(x, y)| x * y).sum();
                    assert!(dot.abs() < 0.35, "crease normal dot product {dot}");
                    checked += 1;
                } else {
                    boundaries.insert((a, b), (curve, n));
                }
            }
        }
        assert_eq!(checked, 32);
        assert!(boundaries.is_empty());
        cage.set_creases([(4, 5)]).unwrap();
        assert_eq!(cage.surface_patches().unwrap_err(), Error::CreasedSpline);
    }

    #[test]
    fn complete_surface_has_matching_cubic_edges_and_bounded_sample_error() {
        let mut cage = Cage::cube();
        for edited in [false, true] {
            if edited {
                cage.set_point(6, [1.5, 0.8, 2.]).unwrap();
            }
            let mesh = cage.preview(3).unwrap();
            let patches = cage.surface_patches().unwrap();
            assert_eq!(patches.len(), 384);
            assert_eq!(patches.iter().filter(|patch| patch.exact).count(), 360);
            let mut edges = BTreeMap::new();
            let mut max_angle = 0_f64;
            for patch in &patches {
                let [a, b, c, d] = mesh.quads[patch.face];
                for (edge, (a, b)) in [(a, b), (b, c), (c, d), (d, a)].into_iter().enumerate() {
                    let curve = boundary(&patch.control_points, edge);
                    if let Some((other, neighbor, neighbor_edge)) = edges.remove(&(b, a)) {
                        for t in [0., 0.125, 0.25, 0.5, 0.75, 0.875, 1.] {
                            let p = normal(&patch.control_points, edge, t);
                            let q = normal(neighbor, neighbor_edge, 1. - t);
                            let dot: f64 = p.into_iter().zip(q).map(|(x, y)| x * y).sum();
                            assert!(dot.is_finite());
                            max_angle = max_angle.max(dot.clamp(-1., 1.).acos().to_degrees());
                        }
                        assert!(curve
                            .into_iter()
                            .zip(other)
                            .all(|(a, b)| distance(a, b) < 1e-12));
                    } else {
                        edges.insert(
                            (a, b),
                            (
                                [curve[3], curve[2], curve[1], curve[0]],
                                &patch.control_points,
                                edge,
                            ),
                        );
                    }
                }
            }
            assert!(edges.is_empty());

            let finer = cage.preview(6).unwrap();
            let limits = limit_points(&finer).unwrap();
            let mut max_error = 0_f64;
            for patch in patches.iter().filter(|patch| !patch.exact) {
                let mut cells = vec![(patch.face, [(0, 0), (8, 0), (8, 8), (0, 8)])];
                for _ in 0..3 {
                    cells = children(&cells);
                }
                for (child, coordinates) in cells {
                    for (&vertex, (u, v)) in finer.quads[child].iter().zip(coordinates) {
                        let fitted = sample(&patch.control_points, u as f64 / 8., v as f64 / 8.);
                        max_error = max_error.max(distance(fitted, limits[vertex]));
                    }
                }
            }
            eprintln!("edited={edited}: maximum sampled error {max_error}, seam normal angle {max_angle} degrees");
            assert!(max_error < 0.0007);
            assert!(max_angle < 2.3);
        }
    }
}
