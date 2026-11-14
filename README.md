# SubD modeller

A standalone Catmull–Clark cage editor and STEP exporter for testing freeform
modelling workflows with SolidCraft. The Rust library evaluates previews and
builds bicubic patches. The browser editor uses a local Python server to call
the Rust CLI.

Current version: **0.3.0**.

## What's new in v0.3.0

- File menu with New, Open, Save, and Save As. Preview a cube, cylinder,
  truncated cone, torus, or thin slab at your chosen XYZ size before creating it.
- Shift-click points to move them together. Edit surface-line junctions in
  SubD view, lock them in XYZ, and right-click faces to select or subdivide them.
  Face extrusion adds protrusions.
- Binary STL export with adjustable refinement. The editor highlights
  detected preview crossings and warns before downloading a crossing STL.
- Hard creases on edges and planar rings. STL and Faceted STEP preserve them.
  Spline STEP supports closed, unbranched crease rings.

## Run the editor

Install stable Rust and Python 3, then run these commands from the checkout.

```sh
cargo build --locked
python3 editor/serve.py
```

Open [the editor](http://127.0.0.1:8765/) and leave the server running while
you edit. On the original development Mac, if the temporary Rust installation
is still available, these exports make it available to the current shell.

```sh
export RUSTUP_HOME=/tmp/subd-rust/toolchains CARGO_HOME=/tmp/subd-rust/cargo
export PATH="$CARGO_HOME/bin:$PATH"
```

- Left-drag a control point to edit, right-drag to orbit, and scroll to zoom.
  The Home button resets the Z-up view. Press Tab in the viewport to switch
  between the primitive cage and the rounded surface.
- Start with the cube and insert loops as needed. Set the reference width in millimetres.
  Click a point or edge to select it. Drag its X, Y, or Z handle, enter
  point coordinates, or resize an edge about its midpoint.
- Select an edge to preview a loop across connected quads. **Insert edge loop**
  adds it at the crossed edges' midpoints. Undo restores the previous cage.
  Toggle the cage frame, facets, and surface lines from the toolbar.
- Save and reopen the cage, or download **Spline STEP** or **Faceted STEP**.
  Both STEP variants declare millimetres. Preview dimensions come from the
  display mesh and can differ from the exported body's bounds.

The CLI also accepts saved cages. Use a fresh output path.

```sh
cargo run -- spline-step edited.cage edited-spline.step
cargo run -- step edited.cage 2 edited-faceted.step
```

## Try the creased cylinder

1. Open [creased-cylinder.cage](examples/creased-cylinder.cage) using **File → Open**.
2. Click the viewport and press **Tab** to see the sharp top rim and rounded bottom.
3. Choose **File → Save As → Spline STEP** and import the result into SolidCraft.
   Choose **STL mesh** instead for Cura and select the mesh refinement level.
   STL coordinates are in millimetres by convention.

To create a crease yourself, right-click the middle of a visible cage edge and
choose **Crease edge** or **Crease ring**. Use STL or Faceted STEP for open-ended
or branched creases. Surface junction locks currently require a smooth cage.

## Geometry and provenance

The subdivision rules are an original safe Rust implementation based on
[Catmull and Clark (1978)](https://doi.org/10.1016/0010-4485(78)90110-0).
The STEP writer uses the published [STEP entity definitions](https://ap238.org/SMRL_v8_final/data/resource_docs/geometric_and_topological_representation/sys/6_schema.htm).
The crate has no external Rust dependencies and uses no third-party geometry
implementation or reverse-engineered code. It is licensed under MIT or
Apache 2.0.

Spline exports use exact bicubic patches in regular regions and fitted patches
at extraordinary vertices. Fitted seams can show tangent breaks. Extreme cage
edits can create self-intersections. The example solids imported into SolidCraft,
and the rounded cube and edited cube passed a cylindrical through-cut. A tapered
shell with a cross-plane loop also imported as one closed spline solid.

## Next session

Explore mirror planes for quad cages and add crease weights to the geometry
library. A future native SolidCraft feature would store the cage and regenerate
the CAD body when it changes.
