wit_bindgen::generate!({ path: "wit", world: "importer-plugin" });

use craft::plugin::source::FileInfo;
use craft::plugin::types::{FieldFailure, PluginError};
use craft::plugin::{control, design, evaluate, source};
use exports::craft::plugin::importer::Guest;
use serde_json::{json, Value};
use subd_prototype::{cage_file, step};

const MAX_CAGE_BYTES: u32 = 32 * 1024;
const TRANSFER_CHUNK_BYTES: usize = 1024 * 1024;
const LEGACY_CAGE_WIDTH_MM: f64 = 100.0;

struct Importer;

impl Guest for Importer {
    fn import_file(entry: String, file: FileInfo, _options: String) -> Result<String, PluginError> {
        if entry != "open-cage" {
            return Err(invalid("entry", "Unknown SubD Cage Importer entry"));
        }
        if file.size == 0 || file.size > u64::from(MAX_CAGE_BYTES) {
            return Err(invalid(
                "file",
                "A cage file must be between 1 byte and 32 KiB",
            ));
        }

        let data = source::read("", 0, MAX_CAGE_BYTES)?;
        if data.len() as u64 != file.size {
            return Err(invalid("file", "Could not read the complete cage"));
        }
        let (cage, width) = cage_file::read_cage(data.as_slice())
            .map_err(|error| invalid("file", error.to_string()))?;

        control::progress(0.1, "Converting the cage to a spline solid");
        let step = step::encode_splines(&cage, width.unwrap_or(LEGACY_CAGE_WIDTH_MM))
            .map_err(|error| invalid("file", error.to_string()))?;
        let document = serde_json::to_vec(&json!({
            "format": "solidcraft/65",
            "name": "",
            "units": "mm",
            "params": [],
            "features": [{
                "id": 1,
                "name": "SubD",
                "suppressed": false,
                "body_names": ["SubD"],
                "type": "import",
                "file": "subd-cage.step",
                "step": Value::String(step),
            }],
            "next_id": 2,
        }))
        .map_err(|error| PluginError::Trap(error.to_string()))?;

        control::progress(0.6, "Checking the solid in SolidCraft");
        for chunk in document.chunks(TRANSFER_CHUNK_BYTES) {
            if control::cancelled() {
                return Err(PluginError::Cancelled);
            }
            evaluate::stage(chunk)?;
        }
        let measurement = evaluate::run("inspect.measure", "{}")?;
        evaluate::reset();
        let measured: Value = serde_json::from_str(&measurement)
            .map_err(|error| PluginError::Trap(error.to_string()))?;
        let is_closed = measured
            .pointer("/bodies/0/closed")
            .and_then(Value::as_bool)
            == Some(true);
        let volume = measured
            .pointer("/total/volume_mm3")
            .and_then(Value::as_f64);
        if measured.get("body_count").and_then(Value::as_u64) != Some(1)
            || !is_closed
            || !volume.is_some_and(|value| value.is_finite() && value > 0.0)
        {
            let reason = measured
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("The cage did not produce one closed solid with positive volume");
            return Err(invalid("file", reason));
        }

        control::progress(0.9, "Opening the solid as a new design");
        for chunk in document.chunks(TRANSFER_CHUNK_BYTES) {
            if control::cancelled() {
                return Err(PluginError::Cancelled);
            }
            design::write(chunk)?;
        }
        Ok(json!({
            "summary": "Opened the SubD cage as one spline solid.",
            "items": [{
                "severity": "info",
                "title": "Keep the original cage for further SubD editing",
                "subject": file.name,
                "message": "The SolidCraft design stores the generated solid. Edit the original cage in SubD Modeller.",
                "code": "source_cage_external",
            }],
        })
        .to_string())
    }
}

fn invalid(field: &str, reason: impl Into<String>) -> PluginError {
    PluginError::InvalidInput(FieldFailure {
        field: field.into(),
        reason: reason.into(),
    })
}

export!(Importer);
