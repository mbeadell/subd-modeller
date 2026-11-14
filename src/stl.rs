use std::io::{self, Write};

use subd_prototype::Mesh;

/// Binary STL. Coordinates are expressed in millimetres by convention.
pub fn write(mesh: Mesh, scale: f64, mut output: impl Write) -> io::Result<()> {
    output.write_all(&[0; 80])?;
    let count = u32::try_from(mesh.quads.len() * 2).map_err(io::Error::other)?;
    output.write_all(&count.to_le_bytes())?;
    for [a, b, c, d] in mesh.quads {
        for indices in [[a, b, c], [a, c, d]] {
            let [a, b, c] = indices.map(|index| {
                mesh.points
                    .get(index)
                    .copied()
                    .ok_or_else(|| io::Error::other("invalid mesh index"))
            });
            let (a, b, c) = (a?, b?, c?);
            let [ax, ay, az] = a;
            let [bx, by, bz] = b;
            let [cx, cy, cz] = c;
            let [x, y, z] = [bx - ax, by - ay, bz - az];
            let [u, v, w] = [cx - ax, cy - ay, cz - az];
            let normal = [y * w - z * v, z * u - x * w, x * v - y * u];
            let length = normal.into_iter().map(|n| n * n).sum::<f64>().sqrt();
            let normal = if length > 0. {
                normal.map(|n| n / length)
            } else {
                [0.; 3]
            };
            for value in normal
                .into_iter()
                .chain([a, b, c].into_iter().flatten().map(|n| n * scale))
            {
                output.write_all(&(value as f32).to_le_bytes())?;
            }
            output.write_all(&[0; 2])?;
        }
    }
    output.flush()
}
