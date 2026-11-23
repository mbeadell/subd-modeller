# SubD Cage Importer

The importer opens `.cage` files in SolidCraft as new designs containing one
closed spline solid. It shares the cage reader and spline STEP writer with the
SubD Modeller CLI. Cage editing stays in the browser editor. Keep the original
`.cage` file for further edits.

## Build and test

Use stable Rust with the `wasm32-unknown-unknown` target, `wasm-tools`, `zip`,
and SolidCraft CLI 0.47.0. The emitted design format is `solidcraft/65`.
The vendored [plugin interface](https://github.com/tangency-labs/solidcraft/blob/v0.47.0/crates/craft-plugin/wit/plugin.wit)
comes from SolidCraft v0.47.0 under MIT or Apache 2.0.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-tools --locked
sh solidcraft-importer/build.sh
```

The script checks the package and tests a legacy cube, a weighted-crease cube,
the raised-cap creased cylinder, and a malformed cage before it creates
`target/solidcraft-plugin/subd-cage.zip`. Legacy cages without a width use the
browser editor's 100 mm default. The importer uses no folder grants.

Install the ZIP from **Preferences → Plugins → Install plugin…**, then open a
`.cage` file from **File → Open**. The SolidCraft design embeds the spline STEP
and can reopen independently of the plugin and the source cage.

For a downstream check, open `examples/creased-cylinder-V2.cage` with the plugin
and save it as `raised-cap.sc`. From the project root run:

```sh
solidcraft-cli eval examples/creased-cylinder-V2-cut.json --in raised-cap.sc
```

The cut should leave one closed body with about 765,277 mm³ of volume.
