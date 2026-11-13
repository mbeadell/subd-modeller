# SolidCraft SubD prototype

The editable model is a fixed-topology cube cage with eight control points. The
Rust library evaluates a deterministic Catmull–Clark preview. The command-line
tool saves and loads the cage and exports the preview as a quad OBJ. Coordinates
are unitless at this stage. Vertex IDs are zero-based.
<img width="1038" height="871" alt="image" src="https://github.com/user-attachments/assets/8e380b94-c95c-47fd-8b92-62d567f0f54d" />


## Run on this Mac

The verification toolchain was installed under `/tmp`. While it is available,
paste this block into Terminal from any directory. With a regular Rust installation, use
`cargo` from your normal PATH and omit the exports.

```sh
export RUSTUP_HOME=/tmp/subd-rust/toolchains CARGO_HOME=/tmp/subd-rust/cargo
export PATH="$CARGO_HOME/bin:$PATH"
manifest="$HOME/Documents/SubD modeller/Cargo.toml"
demo=$(mktemp -d) &&
  cargo run --offline --manifest-path "$manifest" -- new "$demo/cube.cage" &&
  cargo run --offline --manifest-path "$manifest" -- set "$demo/cube.cage" 6 1.5 1 2 "$demo/edited.cage" &&
  cargo run --offline --manifest-path "$manifest" -- preview "$demo/edited.cage" 4 "$demo/edited.obj" &&
  open -R "$demo/edited.obj"
```

Open the OBJ in a mesh viewer. Use level 0 to export the control cage and levels
1 through 6 for successive preview refinements. Each command that writes a file
requires a fresh output path. Extreme cage edits can create self-intersections.

## Geometry and provenance

The subdivision rules are an original Rust implementation based on Edwin
Catmull and James Clark, *Recursively generated B-spline surfaces on arbitrary
topological meshes* (1978).
[Paper](https://doi.org/10.1016/0010-4485(78)90110-0)

No third-party geometry code, reverse-engineered implementation, proprietary
fixtures, or external crate dependencies were used. The crate declares
`MIT OR Apache-2.0`, matching SolidCraft's contribution terms. The geometry
library forbids unsafe Rust, uses deterministic traversal, and keeps filesystem
access in the CLI.
[SolidCraft contribution terms](https://github.com/tangency-labs/solidcraft/blob/main/CONTRIBUTING.md) ·
[Solidcore engineering rules](https://github.com/tangency-labs/solidcore/blob/main/AGENTS.md)

## Path to a CAD body

Keep the cage as the editable feature input and derive the CAD body from it.
The current output is a finite polygonal preview. The next geometry milestone
is to extract exact bicubic patches in regular regions, then design and verify
the approximation around the eight extraordinary vertices. Measure deviation
from the limit surface and continuity across seams before attempting to stitch
the patches through Solidcore. The integration milestone is a valid closed
body that survives a cylindrical cut and a rebuild after a cage edit.

## Graphical cage editor

Paste this block into Terminal and leave it running while you edit.

```sh
(
  cd "$HOME/Documents/SubD modeller" || exit
  export RUSTUP_HOME=/tmp/subd-rust/toolchains CARGO_HOME=/tmp/subd-rust/cargo
  export PATH="$CARGO_HOME/bin:$PATH"
  cargo build --offline && python3 editor/serve.py
)
```

Open [the local editor](http://127.0.0.1:8765). Drag a corner in the current
view plane and orbit by dragging the background. **Save cage** downloads
a file readable by the CLI's `preview` command. Press Control-C in Terminal
to stop the server. The browser draws the mesh, and the Rust CLI evaluates
every preview. This editor runs independently of SolidCraft.

### Scale in millimetres

The editor defaults to a reference cage width of 100 mm. This scales the
original two-unit cube. Moving points keeps that scale and changes the
preview dimensions. The dimensions shown are the level-3 mesh bounds.

Saved cages include the reference width. The CLI preserves it through edits.
To create a scaled cage from Terminal, use `subd-prototype new cube.cage 100`.
OBJ exports from scaled cages use millimetre coordinates. OBJ itself is
unitless, so choose millimetres in SolidCraft's import prompt. Older cage
files retain their original unitless export behaviour.

### Export a faceted STEP solid

Save a scaled cage from the browser and run the following from Terminal.
Adjust the input path if your browser saved the cage elsewhere. Use a fresh
output path. Level 2 produces 192 planar triangular faces.

```sh
"$HOME/Documents/SubD modeller/target/debug/subd-prototype" step \
  "$HOME/Downloads/edited.cage" 2 "$HOME/Downloads/edited.step"
```

The STEP file declares millimetres and uses a closed shell of planar
advanced faces with shared edges. It is an import and scale test for
SolidCraft. Its bounds come from the chosen refinement level. Use level 3
to match the editor's preview dimensions. Smooth spline conversion remains
a separate milestone. Large cage edits can cause self-intersections.

The writer uses the published STEP entity definitions and contains no
third-party geometry or STEP implementation code.
[ISO 10303-42 geometry and topology](https://ap238.org/SMRL_v8_final/data/resource_docs/geometric_and_topological_representation/sys/6_schema.htm)

### Export bicubic STEP

Use a saved cage with a reference width in millimetres and a fresh output path.

```sh
"$HOME/Documents/SubD modeller/target/debug/subd-prototype" spline-step \
  "$HOME/Downloads/edited.cage" "$HOME/Downloads/edited-spline.step"
```

The shell contains 360 exact bicubic faces and 24 fitted corner faces.
The corner fit shares the boundary curves and can show a tangent break.
The existing `step` command still writes the faceted version.

The editor's **Move** menu constrains corner dragging to X, Y, Z, or the
view plane. The selected corner stays highlighted. **Undo** or Command-Z
reverses the last cage drag or reset.
