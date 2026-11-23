# SubD Modeller roadmap

Updated 23 November 2026.

The aim is a capable single-body SubD modeller with editable cages,
millimetre dimensions, and useful mesh and CAD exports. The standalone editor
also provides a place to test workflows for future SolidCraft integration.

The order below is a proposed working sequence based on the project so far.
Completed items describe implemented capabilities. Export checks cover the
recorded fixtures and operations.

## Next

- [ ] **Finish symmetry for topology edits.** Mirror face extrusion, edge-loop
  insertion, and crease changes across the active plane. Both sides should
  update in one undo step, keep a closed quad cage, and survive save/reopen.
- [ ] **Loop selection and transforms.** Select a complete loop, move, scale,
  or rotate it with shared handles, and slide support loops along neighboring
  edges. Build the enclosure's tapered nose and tight lower rim with these tools.
- [ ] **Preserve editing state.** Save surface locks and symmetry settings with
  the cage. Reopening should restore their positions and behaviour. Add redo
  and make Save/Save As behaviour clear across supported browsers.
- [ ] **Verify the editor in a browser.** Check selection, context menus, handles,
  the left inspector, primitive previews, and narrow-window layouts together.
  Keep controls visible without shifting the viewport.
- [ ] **Strengthen crease regression tests.** Cover fractional sharpness,
  save/reopen, crease inheritance after edge splits, mixed weights, and the
  transition from sharp refinement to smooth refinement.
- [ ] **Run the importer integration suite against this revision.** Build the
  WebAssembly package and run the legacy cube, raised-cap, and malformed-file
  cases with the supported SolidCraft version.

### Working modelling milestone

Build a symmetric enclosure from a cube with a tapered nose, rounded crown,
tight lower rim, and a protrusion. Save and reopen it, export STL for Cura,
and export spline STEP for SolidCraft. Record the cage and a downstream
through-cut as repeatable fixtures.

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
