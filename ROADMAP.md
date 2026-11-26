# SubD Modeller roadmap

Updated 27 November 2026.

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

- [x] Mirror face extrusion, including faces crossing the symmetry plane.
- [x] Mirror loop insertion, splitting a shared loop once.
- [x] Apply and remove hard and semi-sharp creases on both sides.
- [x] Preserve crease weights when edges split.

Completion checks cover XY, XZ, and YZ planes. Each operation must preserve
paired vertices, centreline positions, and closed oriented quads. Undo reverses
both sides together. A rejected edit leaves the cage intact. Include cases
where the selection mirrors onto itself and where a partner is locked.

### 2. Shape and finish with loops

- [x] Select a complete edge loop and show its selected edges clearly.
- [x] Move, scale, and rotate a loop using a shared pivot.
- [x] Place an inserted loop at an adjustable position within its quad strip.
- [x] Slide an existing support loop along neighboring edges.

Use these controls to shape the enclosure's nose and lower rim. The preview
must show the proposed edit, and Undo must restore geometry and crease weights.
Test mirrored and centreline loops, irregular quad strips, and the cage limits.

### 3. Make sessions recoverable

- [x] Add redo for geometry, topology, crease, and lock edits.
- [x] Save symmetry settings and cage/surface locks with the editable cage.
  Enclosure save/reopen and mirrored surface locks verified in the browser.
- [ ] Clarify Save and Save As behaviour.
- [x] Add browser-local session recovery and indicate unsaved edits
  ([issue #2](https://github.com/mbeadell/subd-modeller/issues/2)).
  User verified recovery in the live browser.
- [x] Keep existing cage formats readable.

Save, close, and reopen the enclosure. Confirm its dimensions, crease weights,
locks, and symmetry match the saved session. Cancelled file actions preserve
the working model. New edits after Undo clear the redo branch.

### 4. Verify the complete workflow

- [ ] Exercise the controls in a real browser, including the context menu,
  left inspector, face transform panel, primitive preview, and small windows.
- [ ] Check fractional crease weights, mixed weights, and edge-split inheritance
  against the Rust evaluator and saved cage round trips.
- [x] Inspect the enclosure STL in Cura. Binary structure verified with 2,304 triangles.
- [x] Export spline STEP, import it into SolidCraft, and perform a Ø10 mm cut.
- [x] Build and run the importer integration suite against the same revision.
- [x] Retain the enclosure cage and cut script as regression fixtures.
- [x] Update usage instructions and publish a tagged checkpoint.

The phase is complete when the saved enclosure reopens correctly, its STL has
the expected millimetre dimensions, and both STEP and importer routes produce
one closed SolidCraft body that survives the cut. Continue to run the existing
raised-cap V2 test alongside the enclosure.

## Requested primitives

[Issue #3](https://github.com/mbeadell/subd-modeller/issues/3) requests primitives
defined by the resulting SubD shape, rather than just the starting cage.
Add an ellipsoid, a cube-like SubD, and equal tees with square and cylindrical
sections. Verify the rounded result, XYZ sizing, closed topology, and exports.
Keep primitive naming explicit about what the user will see in SubD view.

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


### Enclosure verification

[Symmetric enclosure](examples/symmetric-enclosure.cage) includes a tapered
nose, paired side protrusions, a semi-sharp lower rim, and saved YZ symmetry.
The [cut script](examples/symmetric-enclosure-cut.json) leaves one closed body.
The direct STEP route measured 380,143.9833 mm³ before the cut and
376,250.0223 mm³ afterwards. The STL has 2,304 triangles. The user verified opening the cage in the editor, importing with SolidCraft
0.50.0, cutting a Ø10 mm hole, and sending SolidCraft STL/3MF to Cura.
The modeller’s native STL was also verified in Cura.
