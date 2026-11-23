# SubD Modeller roadmap

Updated 23 November 2026.

The aim is a capable single-body SubD modeller with editable cages,
millimetre dimensions, and useful mesh and CAD exports. The standalone editor
also provides a place to test workflows for future SolidCraft integration.

The order below is a proposed working sequence based on the project so far.
Completed items describe implemented capabilities. Export checks cover the
recorded fixtures and operations.

## Next phase — symmetric enclosure workflow

Build and save a symmetric enclosure from a cube, with a tapered nose, rounded
crown, tight lower rim, and a mirrored protrusion. The sequence below puts
topology editing before finishing and export verification.

### 1. Complete mirrored topology editing

- [ ] Mirror face extrusion, including faces crossing the symmetry plane.
- [ ] Mirror loop insertion, splitting a shared loop once.
- [ ] Apply and remove hard and semi-sharp creases on both sides.
- [ ] Preserve crease weights when edges split.

Completion checks cover XY, XZ, and YZ planes. Each operation must preserve
paired vertices, centreline positions, and closed oriented quads. Undo reverses
both sides together. A rejected edit leaves the cage intact. Include cases
where the selection mirrors onto itself and where a partner is locked.

### 2. Shape and finish with loops

- [ ] Select a complete edge loop and show its selected edges clearly.
- [ ] Move, scale, and rotate a loop using a shared pivot.
- [ ] Place an inserted loop at an adjustable position within its quad strip.
- [ ] Slide an existing support loop along neighboring edges.

Use these controls to shape the enclosure's nose and lower rim. The preview
must show the proposed edit, and Undo must restore geometry and crease weights.
Test mirrored and centreline loops, irregular quad strips, and the cage limits.

### 3. Make sessions recoverable

- [ ] Add redo for geometry, topology, crease, and lock edits.
- [ ] Save symmetry settings and cage/surface locks with the editable cage.
- [ ] Clarify Save and Save As behaviour and indicate unsaved edits.
- [ ] Keep existing cage formats readable.

Save, close, and reopen the enclosure. Confirm its dimensions, crease weights,
locks, and symmetry match the saved session. Cancelled file actions preserve
the working model. New edits after Undo clear the redo branch.

### 4. Verify the complete workflow

- [ ] Exercise the controls in a real browser, including the context menu,
  left inspector, face transform panel, primitive preview, and small windows.
- [ ] Check fractional crease weights, mixed weights, and edge-split inheritance
  against the Rust evaluator and saved cage round trips.
- [ ] Export the enclosure as STL and inspect its size and mesh in Cura.
- [ ] Export spline STEP, import it into SolidCraft, and perform a Ø10 mm cut.
- [ ] Build and run the importer integration suite against the same revision.
- [ ] Retain the enclosure cage and cut script as regression fixtures.
- [ ] Update usage instructions and publish a tagged checkpoint.

The phase is complete when the saved enclosure reopens correctly, its STL has
the expected millimetre dimensions, and both STEP and importer routes produce
one closed SolidCraft body that survives the cut. Continue to run the existing
raised-cap V2 test alongside the enclosure.

## Later

- [ ] **Improve spline conversion at crease endpoints and junctions.** Support
  open-ended and branched hard creases. Measure shared-edge gaps and surface
  deviation, then check imported bodies and downstream cuts.
- [ ] **Broaden spline diagnostics.** Add seam and fit-error measurements,
  degenerate-patch checks, and clearer links from warnings to affected geometry.
  The current near-parallel-corner warning is an inspection aid.
- [ ] **Improve self-intersection detection.** Cover local folds between
  triangles sharing vertices and assess intersections on fitted spline surfaces.
- [ ] **Raise practical cage complexity.** Review current point/face limits,
  preview responsiveness, and export cost using real saved models.
- [ ] **Add open surfaces.** Support a true plane and boundary subdivision rules,
  followed by suitable surface exports.
- [ ] **Add 3MF export.** Preserve declared units and provide mesh-density controls.
- [ ] **Develop SolidCraft editing integration.** Discuss native feature and
  extension options with maintainers. The desired result stores an editable
  cage and rebuilds the solid and downstream features after a cage edit.

## Implemented

### Modelling and viewport

- [x] Cube, cylinder, truncated cone, torus, and thin-slab primitives.
- [x] Primitive axis, cone direction, XYZ cage dimensions, and live creation preview.
- [x] Z-up orientation marker, Home view, right-drag orbit, and wheel zoom.
- [x] Cage/SubD switching, cage frame, facets, and surface-line display controls.
- [x] Automatic point/edge selection, Shift multi-point selection, and axis handles.
- [x] Point coordinates and edge lengths in millimetres.
- [x] Face selection, translation, numeric scale, and axis rotation.
- [x] Cross-plane edge-loop insertion, face subdivision, and face extrusion.
- [x] XY, XZ, and YZ symmetry with visible planes for already symmetric cages.
- [x] Mirrored point/edge movement and paired face subdivision.
- [x] Cage-point locks and XYZ surface-junction locks on smooth cages.
- [x] Undo, right-click actions, File menu, and a left inspector.

### Geometry and exchange

- [x] Safe Rust Catmull–Clark evaluation with deterministic topology traversal.
- [x] Hard creases and adjustable semi-sharp edge/ring weights from 0 to 3.
- [x] Editable cage persistence, including crease weights and legacy cage input.
- [x] Binary STL with adjustable mesh refinement and millimetre coordinates.
- [x] Faceted STEP with explicit millimetre units.
- [x] Spline STEP using exact regular patches and fitted extraordinary regions.
- [x] Closed, unbranched hard-crease rings in spline STEP.
- [x] Spline preview and near-parallel patch-corner warnings.
- [x] Preview crossing highlights and intersection warnings for exported STL.
- [x] Raised-cap V2 fixture and repeatable SolidCraft through-cut.
- [x] SolidCraft cage importer source, shared Rust cage reader, and packaging script.

## Regression fixtures

- [Creased cylinder](examples/creased-cylinder.cage) checks a sharp rim and smooth bottom.
- [Raised-cap V2](examples/creased-cylinder-V2.cage) exercises the spline rim-corner fix.
- [V2 cut script](examples/creased-cylinder-V2-cut.json) checks one closed body after
  a Ø10 mm cut with reduced volume.
- [Importer instructions and fixtures](solidcraft-importer/README.md) cover packaging
  and import behaviour.

Use GitHub issues for individual implementation tasks and bug reproductions.
Link them here as they are opened.
