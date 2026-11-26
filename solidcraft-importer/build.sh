#!/bin/sh
set -eu

plugin_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
repo_dir=$(dirname -- "$plugin_dir")
output_dir="$repo_dir/target/solidcraft-plugin"

for tool in cargo wasm-tools solidcraft-cli zip; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf 'Missing build tool: %s\n' "$tool" >&2
        exit 1
    fi
done

cargo build --locked --manifest-path "$plugin_dir/Cargo.toml" --target-dir "$repo_dir/target" \
    --release --target wasm32-unknown-unknown

mkdir -p "$output_dir"
build_dir=$(mktemp -d "$output_dir/build.XXXXXX")
trap 'rm -rf "$build_dir"' EXIT
package_dir="$build_dir/subd-cage"
mkdir -p "$package_dir/tests"
cp "$plugin_dir/plugin.json" "$package_dir/"
cp "$plugin_dir/tests/"* "$package_dir/tests/"
cp "$repo_dir/examples/creased-cylinder-V2.cage" "$repo_dir/examples/symmetric-enclosure.cage" "$package_dir/tests/"

wasm-tools component new "$repo_dir/target/wasm32-unknown-unknown/release/subd_cage_importer.wasm" \
    -o "$package_dir/subd-cage.wasm"
solidcraft-cli plugin check "$package_dir"
solidcraft-cli plugin test "$package_dir" --require-cases

(cd "$package_dir" && zip -q "$build_dir/subd-cage.zip" plugin.json subd-cage.wasm)
mv -f "$build_dir/subd-cage.zip" "$output_dir/subd-cage.zip"
printf 'Installable plugin: %s\n' "$output_dir/subd-cage.zip"
