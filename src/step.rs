// Original STEP writer based on ISO 10303-21 and ISO 10303-42 entity definitions.
use std::collections::BTreeMap;
use std::fmt::Write;
use subd_prototype::{Mesh, Point, SurfacePatch};

#[derive(Default)]
struct Step {
    data: String,
    last: usize,
}

impl Step {
    fn entity(&mut self, entity: std::fmt::Arguments<'_>) -> Result<usize, std::fmt::Error> {
        self.last += 1;
        writeln!(self.data, "#{}={entity};", self.last)?;
        Ok(self.last)
    }

    fn coordinates(&mut self, kind: &str, [x, y, z]: Point) -> Result<usize, std::fmt::Error> {
        self.entity(format_args!("{kind}('',({x:.16E},{y:.16E},{z:.16E}))"))
    }
}

fn subtract([a, b, c]: Point, [x, y, z]: Point) -> Point {
    [a - x, b - y, c - z]
}

fn cross([a, b, c]: Point, [x, y, z]: Point) -> Point {
    [b * z - c * y, c * x - a * z, a * y - b * x]
}

fn unit([x, y, z]: Point) -> Result<Point, &'static str> {
    let length = x.hypot(y).hypot(z);
    if length <= 1e-12 || !length.is_finite() {
        return Err("degenerate STEP facet");
    }
    Ok([x / length, y / length, z / length])
}

fn references(ids: &[usize]) -> String {
    ids.iter()
        .map(|id| format!("#{id}"))
        .collect::<Vec<_>>()
        .join(",")
}

pub fn encode(mesh: Mesh, width_mm: f64) -> Result<String, Box<dyn std::error::Error>> {
    encode_body(mesh, width_mm, None)
}

pub fn encode_splines(
    mesh: Mesh,
    width_mm: f64,
    patches: Vec<SurfacePatch>,
) -> Result<String, Box<dyn std::error::Error>> {
    encode_body(mesh, width_mm, Some(patches))
}

fn encode_body(
    mesh: Mesh,
    width_mm: f64,
    patches: Option<Vec<SurfacePatch>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut step = Step::default();
    let application = step.entity(format_args!("APPLICATION_CONTEXT('automotive design')"))?;
    step.entity(format_args!("APPLICATION_PROTOCOL_DEFINITION('international standard','automotive_design',2000,#{application})"))?;
    let product_context = step.entity(format_args!(
        "PRODUCT_CONTEXT('',#{application},'mechanical')"
    ))?;
    let product = step.entity(format_args!(
        "PRODUCT('SubD','SubD','',(#{product_context}))"
    ))?;
    let formation = step.entity(format_args!(
        "PRODUCT_DEFINITION_FORMATION('','',#{product})"
    ))?;
    let definition_context = step.entity(format_args!(
        "PRODUCT_DEFINITION_CONTEXT('part definition',#{application},'design')"
    ))?;
    let definition = step.entity(format_args!(
        "PRODUCT_DEFINITION('design','',#{formation},#{definition_context})"
    ))?;
    let shape = step.entity(format_args!(
        "PRODUCT_DEFINITION_SHAPE('','',#{definition})"
    ))?;
    let mm = step.entity(format_args!(
        "(LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.))"
    ))?;
    let angle = step.entity(format_args!(
        "(NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.))"
    ))?;
    let solid_angle = step.entity(format_args!(
        "(NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT())"
    ))?;
    let tolerance = step.entity(format_args!(
        "UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.E-7),#{mm},'distance_accuracy_value','')"
    ))?;
    let context = step.entity(format_args!("(GEOMETRIC_REPRESENTATION_CONTEXT(3) GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#{tolerance})) GLOBAL_UNIT_ASSIGNED_CONTEXT((#{mm},#{angle},#{solid_angle})) REPRESENTATION_CONTEXT('',''))"))?;

    let faces = if let Some(patches) = patches {
        spline_faces(&mut step, &mesh, width_mm, patches)?
    } else {
        let mut vertices = Vec::new();
        for point in mesh.points {
            let point = point.map(|value| value * width_mm * 0.5);
            let cartesian = step.coordinates("CARTESIAN_POINT", point)?;
            let vertex = step.entity(format_args!("VERTEX_POINT('',#{cartesian})"))?;
            vertices.push((point, cartesian, vertex));
        }
        let vertex = |index| vertices.get(index).copied().ok_or("invalid mesh vertex");
        let mut edges = BTreeMap::new();
        let mut faces = Vec::new();
        for [a, b, c, d] in mesh.quads {
            for [a, b, c] in [[a, b, c], [a, c, d]] {
                let (origin, origin_id, _) = vertex(a)?;
                let (second, _, _) = vertex(b)?;
                let (third, _, _) = vertex(c)?;
                let tangent = unit(subtract(second, origin))?;
                let normal = unit(cross(tangent, unit(subtract(third, origin))?))?;
                let axis = step.coordinates("DIRECTION", normal)?;
                let reference = step.coordinates("DIRECTION", tangent)?;
                let placement = step.entity(format_args!(
                    "AXIS2_PLACEMENT_3D('',#{origin_id},#{axis},#{reference})"
                ))?;
                let plane = step.entity(format_args!("PLANE('',#{placement})"))?;
                let mut boundary = Vec::new();
                for (from, to) in [(a, b), (b, c), (c, a)] {
                    let key = (from.min(to), from.max(to));
                    let forward = from < to;
                    let edge = match edges.entry(key) {
                        std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
                        std::collections::btree_map::Entry::Vacant(entry) => {
                            let (start, start_point, start_vertex) = vertex(key.0)?;
                            let (end, _, end_vertex) = vertex(key.1)?;
                            let direction =
                                step.coordinates("DIRECTION", unit(subtract(end, start))?)?;
                            let vector = step.entity(format_args!("VECTOR('',#{direction},1.)"))?;
                            let line =
                                step.entity(format_args!("LINE('',#{start_point},#{vector})"))?;
                            *entry.insert(step.entity(format_args!(
                                "EDGE_CURVE('',#{start_vertex},#{end_vertex},#{line},.T.)"
                            ))?)
                        }
                    };
                    let orientation = if forward { ".T." } else { ".F." };
                    boundary.push(
                        step.entity(format_args!("ORIENTED_EDGE('',*,*,#{edge},{orientation})"))?,
                    );
                }
                let edge_loop =
                    step.entity(format_args!("EDGE_LOOP('',({}))", references(&boundary)))?;
                let bound = step.entity(format_args!("FACE_OUTER_BOUND('',#{edge_loop},.T.)"))?;
                faces.push(step.entity(format_args!("ADVANCED_FACE('',(#{bound}),#{plane},.T.)"))?);
            }
        }
        faces
    };
    let shell = step.entity(format_args!("CLOSED_SHELL('',({}))", references(&faces)))?;
    let body = step.entity(format_args!("MANIFOLD_SOLID_BREP('SubD',#{shell})"))?;
    let representation = step.entity(format_args!(
        "ADVANCED_BREP_SHAPE_REPRESENTATION('',(#{body}),#{context})"
    ))?;
    step.entity(format_args!(
        "SHAPE_DEFINITION_REPRESENTATION(#{shape},#{representation})"
    ))?;
    Ok(format!("ISO-10303-21;\nHEADER;\nFILE_DESCRIPTION(('SubD solid'),'2;1');\nFILE_NAME('subd.step','2026-11-13T00:00:00',(''),(''),'SubD prototype','SubD prototype','');\nFILE_SCHEMA(('AUTOMOTIVE_DESIGN'));\nENDSEC;\nDATA;\n{}ENDSEC;\nEND-ISO-10303-21;\n", step.data))
}

fn spline_faces(
    step: &mut Step,
    mesh: &Mesh,
    width: f64,
    patches: Vec<SurfacePatch>,
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let mut vertices = BTreeMap::new();
    let mut edges = BTreeMap::new();
    let mut faces = Vec::new();
    for patch in patches {
        let &[a, b, c, d] = mesh.quads.get(patch.face).ok_or("invalid patch face")?;
        let mut grid = [[0; 4]; 4];
        for (ids, row) in grid.iter_mut().zip(patch.control_points) {
            for (id, point) in ids.iter_mut().zip(row) {
                *id = step.coordinates("CARTESIAN_POINT", point.map(|x| x * width * 0.5))?;
            }
        }
        let [south, row1, row2, north] = grid;
        let [sw, s1, s2, se] = south;
        let [nw, n1, n2, ne] = north;
        let [w1, i11, i12, e1] = row1;
        let [w2, i21, i22, e2] = row2;
        let columns = [
            [sw, w1, w2, nw],
            [s1, i11, i21, n1],
            [s2, i12, i22, n2],
            [se, e1, e2, ne],
        ];
        let grid_refs = columns
            .map(|column| format!("({})", references(&column)))
            .join(",");
        let surface = step.entity(format_args!("B_SPLINE_SURFACE_WITH_KNOTS('',3,3,({grid_refs}),.UNSPECIFIED.,.F.,.F.,.F.,(4,4),(4,4),(0.,1.),(0.,1.),.PIECEWISE_BEZIER_KNOTS.)"))?;

        for (vertex, point) in [(a, sw), (b, se), (c, ne), (d, nw)] {
            if let std::collections::btree_map::Entry::Vacant(entry) = vertices.entry(vertex) {
                entry.insert(step.entity(format_args!("VERTEX_POINT('',#{point})"))?);
            }
        }
        let mut boundary = Vec::new();
        for (from, to, mut controls) in [
            (a, b, south),
            (b, c, [se, e1, e2, ne]),
            (c, d, [ne, n2, n1, nw]),
            (d, a, [nw, w2, w1, sw]),
        ] {
            let forward = from < to;
            let key = (from.min(to), from.max(to));
            let edge = match edges.entry(key) {
                std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
                std::collections::btree_map::Entry::Vacant(entry) => {
                    if !forward {
                        controls.reverse();
                    }
                    let start = vertices.get(&key.0).ok_or("missing edge start")?;
                    let end = vertices.get(&key.1).ok_or("missing edge end")?;
                    let curve = step.entity(format_args!("B_SPLINE_CURVE_WITH_KNOTS('',3,({}),.UNSPECIFIED.,.F.,.F.,(4,4),(0.,1.),.PIECEWISE_BEZIER_KNOTS.)", references(&controls)))?;
                    *entry.insert(
                        step.entity(format_args!("EDGE_CURVE('',#{start},#{end},#{curve},.T.)"))?,
                    )
                }
            };
            let orientation = if forward { ".T." } else { ".F." };
            boundary
                .push(step.entity(format_args!("ORIENTED_EDGE('',*,*,#{edge},{orientation})"))?);
        }
        let edge_loop = step.entity(format_args!("EDGE_LOOP('',({}))", references(&boundary)))?;
        let bound = step.entity(format_args!("FACE_OUTER_BOUND('',#{edge_loop},.T.)"))?;
        faces.push(step.entity(format_args!("ADVANCED_FACE('',(#{bound}),#{surface},.T.)"))?);
    }
    Ok(faces)
}
