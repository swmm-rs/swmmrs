//! Browser adapters for EPA hotstarts and portable Simulation Checkpoints.
//!
//! Checkpoint bytes are staged in the per-WASM-instance filesystem. A bundle
//! keeps the core manifest untouched and records every file the manifest names;
//! resume therefore exercises the solver's real checkpoint validation and
//! restoration code instead of serializing a second, binding-specific state.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use js_sys::{Array, ArrayBuffer, Error, Object, Reflect, Uint8Array};
use serde::Deserialize;
use serde_json::Value;
use swmmrs::engine::error::ErrorCode;
use swmmrs::platform::fs;
use swmmrs::simulation::{SimulationLifecycle, SwmmSimulation};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use super::super::Simulation;
use super::common::decode_value;

const BUNDLE_FORMAT: &str = "swmmrs.checkpoint.bundle";
const BUNDLE_VERSION: u32 = 1;
const MAX_BUNDLE_FILE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_BUNDLE_TOTAL_BYTES: u64 = 2 * 1024 * 1024 * 1024;
static NEXT_PRIVATE_FILE: AtomicU64 = AtomicU64::new(0);

const HOTSTART_INPUT_FILE: &str = ".swmmrs-hotstart-input";
const HOTSTART_OUTPUT_FILE: &str = ".swmmrs-hotstart-output";
const CHECKPOINT_FILE: &str = ".swmmrs-checkpoint";
const STATE_LOAD_FILE: &str = ".swmmrs-checkpoint-state";
const RESUME_REPORT_FILE: &str = "model.rpt";
const RESUME_OUTPUT_FILE: &str = "model.out";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleHeader {
    format: String,
    version: u32,
    root: String,
    #[serde(rename = "manifestPath")]
    manifest_path: String,
}

struct Bundle {
    root: PathBuf,
    manifest_path: PathBuf,
    manifest: Vec<u8>,
    files: BTreeMap<String, Vec<u8>>,
}

#[wasm_bindgen]
impl Simulation {
    /// Configures or clears an EPA hotstart input from caller-owned bytes.
    #[wasm_bindgen(js_name = useHotstart)]
    pub fn use_hotstart(&mut self, value: JsValue) -> Result<(), JsValue> {
        let bytes = if value.is_null() {
            None
        } else if value.is_undefined() {
            return Err(invalid("useHotstart", "expected Uint8Array or null"));
        } else {
            Some(read_bytes(value, "useHotstart", "hotstart")?)
        };
        if !matches!(
            self.inner.lifecycle_read().state,
            SimulationLifecycle::Open | SimulationLifecycle::Ended
        ) {
            return Err(self.error(
                "useHotstart",
                swmmrs::engine::error::SwmmError::with_detail(
                    ErrorCode::ApiNotEnded,
                    "persistent hotstart input is configurable only while open or ended",
                ),
            ));
        }
        match bytes {
            Some(bytes) => {
                let path = private_path(&self.root, HOTSTART_INPUT_FILE, "useHotstart")?;
                fs::write(&path, bytes).map_err(|error| io_error("useHotstart", error))?;
                let path_string = path
                    .to_str()
                    .ok_or_else(|| invalid("useHotstart", "hotstart path is not valid UTF-8"))?;
                self.inner
                    .use_hotstart(Some(path_string))
                    .map_err(|error| self.error("useHotstart", error))?;
            }
            None => {
                self.inner
                    .use_hotstart(None)
                    .map_err(|error| self.error("useHotstart", error))?;
            }
        }
        Ok(())
    }

    /// Writes an EPA hotstart from the current active run and returns its bytes.
    #[wasm_bindgen(js_name = saveHotstart)]
    pub fn save_hotstart(&mut self) -> Result<Vec<u8>, JsValue> {
        let path = private_path(&self.root, HOTSTART_OUTPUT_FILE, "saveHotstart")?;
        let path_string = path
            .to_str()
            .ok_or_else(|| invalid("saveHotstart", "hotstart path is not valid UTF-8"))?;
        self.inner
            .save_hotstart(path_string)
            .map_err(|error| self.error("saveHotstart", error))?;
        let bytes = fs::read(&path).map_err(|error| io_error("saveHotstart", error));
        let cleanup = remove_if_present(&path);
        match (bytes, cleanup) {
            (Ok(bytes), Ok(())) => Ok(bytes),
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(io_error("saveHotstart", error)),
        }
    }

    /// Publishes the complete core checkpoint and detaches it into bytes.
    #[wasm_bindgen(js_name = exportCheckpoint)]
    pub fn export_checkpoint(&mut self) -> Result<JsValue, JsValue> {
        let manifest_path = private_path(&self.root, CHECKPOINT_FILE, "exportCheckpoint")?;
        let path = manifest_path
            .to_str()
            .ok_or_else(|| invalid("exportCheckpoint", "checkpoint path is not valid UTF-8"))?;
        if let Err(error) = self.inner.save_checkpoint(path) {
            let _ = remove_if_present(&manifest_path);
            return Err(self.error("exportCheckpoint", error));
        }

        let result = self.capture_bundle(&manifest_path);
        let mut cleanup = vec![manifest_path.clone()];
        if let Ok(manifest) = fs::read(&manifest_path) {
            if let Ok(value) = serde_json::from_slice::<Value>(&manifest) {
                cleanup.extend(sidecar_paths(&value, &manifest_path).unwrap_or_default());
            }
        }
        let cleanup_result = remove_paths(&cleanup);
        match (result, cleanup_result) {
            (Ok(bundle), Ok(())) => bundle_value(&bundle),
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(io_error("exportCheckpoint", error)),
        }
    }

    /// Imports only the core physical checkpoint state into this open owner.
    #[wasm_bindgen(js_name = loadCheckpointState)]
    pub fn load_checkpoint_state(&mut self, value: JsValue) -> Result<(), JsValue> {
        let bundle = decode_bundle(value, "loadCheckpointState")?;
        validate_bundle_manifest(&bundle, "loadCheckpointState", false)?;
        let path = private_path(&self.root, STATE_LOAD_FILE, "loadCheckpointState")?;
        fs::write(&path, &bundle.manifest)
            .map_err(|error| io_error("loadCheckpointState", error))?;
        let path_string = path
            .to_str()
            .ok_or_else(|| invalid("loadCheckpointState", "checkpoint path is not valid UTF-8"))?;
        let result = self
            .inner
            .load_checkpoint_state(path_string)
            .map_err(|error| self.error("loadCheckpointState", error));
        let cleanup = remove_if_present(&path);
        match (result, cleanup) {
            (Err(error), _) => Err(error),
            (Ok(()), Ok(())) => Ok(()),
            (Ok(()), Err(error)) => Err(io_error("loadCheckpointState", error)),
        }
    }

    #[wasm_bindgen(js_name = resumeCheckpoint)]
    pub fn resume_checkpoint(value: JsValue) -> Result<Simulation, JsValue> {
        let bundle = decode_bundle(value, "resumeCheckpoint")?;
        validate_bundle_manifest(&bundle, "resumeCheckpoint", true)?;
        if fs::metadata(&bundle.root).is_ok() {
            return Err(invalid(
                "resumeCheckpoint",
                "checkpoint root already exists in this worker",
            ));
        }
        fs::create_dir_all(&bundle.root).map_err(|error| io_error("resumeCheckpoint", error))?;
        let result = (|| {
            for (name, bytes) in &bundle.files {
                let path = bundle.root.join(name);
                let parent = path
                    .parent()
                    .ok_or_else(|| invalid("resumeCheckpoint", "bundle file has no parent"))?;
                fs::create_dir_all(parent).map_err(|error| io_error("resumeCheckpoint", error))?;
                fs::write(path, bytes).map_err(|error| io_error("resumeCheckpoint", error))?;
            }
            let manifest = bundle.root.join(&bundle.manifest_path);
            fs::write(&manifest, &bundle.manifest)
                .map_err(|error| io_error("resumeCheckpoint", error))?;
            let manifest_string = manifest
                .to_str()
                .ok_or_else(|| invalid("resumeCheckpoint", "checkpoint path is not valid UTF-8"))?;
            let prepared = SwmmSimulation::prepare_resume(manifest_string)
                .map_err(|error| checkpoint_error("resumeCheckpoint", error))?;
            let report = bundle.root.join(RESUME_REPORT_FILE);
            let output = bundle.root.join(RESUME_OUTPUT_FILE);
            let report_string = report
                .to_str()
                .ok_or_else(|| invalid("resumeCheckpoint", "report path is not valid UTF-8"))?;
            let output_string = output
                .to_str()
                .ok_or_else(|| invalid("resumeCheckpoint", "output path is not valid UTF-8"))?;
            let inner = SwmmSimulation::resume_prepared(prepared, report_string, output_string)
                .map_err(|error| checkpoint_error("resumeCheckpoint", error))?;
            Ok(Simulation {
                inner,
                root: bundle.root.clone(),
            })
        })();
        match result {
            Ok(simulation) => Ok(simulation),
            Err(error) => {
                let _ = fs::remove_dir_all(&bundle.root);
                Err(error)
            }
        }
    }
}

impl Simulation {
    fn capture_bundle(&mut self, manifest_path: &Path) -> Result<Bundle, JsValue> {
        let root = validate_root(&self.root, "exportCheckpoint")?;
        let manifest =
            fs::read(manifest_path).map_err(|error| io_error("exportCheckpoint", error))?;
        let _ =
            SwmmSimulation::prepare_resume(manifest_path.to_str().ok_or_else(|| {
                invalid("exportCheckpoint", "checkpoint path is not valid UTF-8")
            })?)
            .map_err(|error| checkpoint_error("exportCheckpoint", error))?;
        let value = parse_manifest(&manifest, "exportCheckpoint")?;
        let mut files = BTreeMap::new();
        let mut total = checked_size(manifest.len() as u64, "manifest")?;
        for path in manifest_resource_paths(&value, &root, manifest_path, "exportCheckpoint")? {
            let key = relative_key(&root, &path, "exportCheckpoint")?;
            let bytes = fs::read(&path).map_err(|error| io_error("exportCheckpoint", error))?;
            let size = checked_size(bytes.len() as u64, &key)?;
            total = total
                .checked_add(size)
                .ok_or_else(|| invalid("exportCheckpoint", "checkpoint bundle is too large"))?;
            if total > MAX_BUNDLE_TOTAL_BYTES {
                return Err(invalid(
                    "exportCheckpoint",
                    "checkpoint bundle is too large",
                ));
            }
            if files.insert(key, bytes).is_some() {
                return Err(invalid(
                    "exportCheckpoint",
                    "checkpoint resource path collision",
                ));
            }
        }
        let manifest_path = relative_key(&root, manifest_path, "exportCheckpoint")?;
        Ok(Bundle {
            root,
            manifest_path: PathBuf::from(manifest_path),
            manifest,
            files,
        })
    }
}

fn decode_bundle(value: JsValue, operation: &str) -> Result<Bundle, JsValue> {
    if value.is_null() || value.is_undefined() || !value.is_object() || Array::is_array(&value) {
        return Err(invalid(
            operation,
            "checkpoint bundle must be a non-null record",
        ));
    }
    let object = value.unchecked_ref::<Object>();
    let prototype = Object::get_prototype_of(&Object::new());
    let object_prototype = Object::get_prototype_of(object);
    if !object_prototype.is_null() && !Object::is(&object_prototype, &prototype) {
        return Err(invalid(
            operation,
            "checkpoint bundle must be a plain record",
        ));
    }
    let allowed = [
        "format",
        "version",
        "root",
        "manifestPath",
        "manifest",
        "files",
    ];
    for key in Object::keys(object).iter() {
        let key = key
            .as_string()
            .ok_or_else(|| invalid(operation, "checkpoint bundle has an invalid key"))?;
        if !allowed.contains(&key.as_str()) {
            return Err(invalid(
                operation,
                &format!("unknown checkpoint bundle field: {key}"),
            ));
        }
    }
    let header_object = Object::new();
    for key in ["format", "version", "root", "manifestPath"] {
        let property = Reflect::get(object, &JsValue::from_str(key))
            .map_err(|_| invalid(operation, "cannot read checkpoint bundle header"))?;
        Reflect::set(&header_object, &JsValue::from_str(key), &property)
            .map_err(|_| invalid(operation, "cannot read checkpoint bundle header"))?;
    }
    let header: BundleHeader = decode_value(header_object.into(), operation)?;
    if header.format != BUNDLE_FORMAT || header.version != BUNDLE_VERSION {
        return Err(invalid(operation, "unsupported checkpoint bundle format"));
    }
    let root = validate_root(Path::new(&header.root), operation)?;
    let manifest_path = safe_relative(&header.manifest_path, operation, "manifestPath")?;
    let manifest_value = Reflect::get(object, &JsValue::from_str("manifest"))
        .map_err(|_| invalid(operation, "cannot read checkpoint manifest bytes"))?;
    let manifest = read_bytes(manifest_value, operation, "manifest")?;
    if manifest.is_empty() {
        return Err(invalid(operation, "checkpoint manifest is empty"));
    }
    let files_value = Reflect::get(object, &JsValue::from_str("files"))
        .map_err(|_| invalid(operation, "cannot read checkpoint resources"))?;
    if files_value.is_null()
        || files_value.is_undefined()
        || !files_value.is_object()
        || Array::is_array(&files_value)
    {
        return Err(invalid(
            operation,
            "checkpoint files must be a plain record",
        ));
    }
    let files_object = files_value.unchecked_ref::<Object>();
    let prototype = Object::get_prototype_of(&Object::new());
    let files_prototype = Object::get_prototype_of(files_object);
    if !files_prototype.is_null() && !Object::is(&files_prototype, &prototype) {
        return Err(invalid(
            operation,
            "checkpoint files must be a plain record",
        ));
    }
    let mut total = checked_size(manifest.len() as u64, "manifest")?;
    let mut files = BTreeMap::new();
    for entry in Object::entries(files_object).iter() {
        let entry = entry.unchecked_into::<Array>();
        let name = entry
            .get(0)
            .as_string()
            .ok_or_else(|| invalid(operation, "checkpoint resource has an invalid name"))?;
        let name = safe_relative(&name, operation, "file name")?;
        if name == manifest_path {
            return Err(invalid(
                operation,
                "bundle files must not replace the manifest",
            ));
        }
        let key = name.to_string_lossy().into_owned();
        let bytes = read_bytes(entry.get(1), operation, &format!("file {key}"))?;
        let size = checked_size(bytes.len() as u64, &key)?;
        total = total
            .checked_add(size)
            .ok_or_else(|| invalid(operation, "checkpoint bundle is too large"))?;
        if total > MAX_BUNDLE_TOTAL_BYTES {
            return Err(invalid(operation, "checkpoint bundle is too large"));
        }
        if files.insert(key, bytes).is_some() {
            return Err(invalid(operation, "duplicate checkpoint resource path"));
        }
    }
    Ok(Bundle {
        root,
        manifest_path,
        manifest,
        files,
    })
}

fn validate_bundle_manifest(
    bundle: &Bundle,
    operation: &str,
    require_resources: bool,
) -> Result<(), JsValue> {
    let manifest_path = bundle.root.join(&bundle.manifest_path);
    if manifest_path == bundle.root || !manifest_path.starts_with(&bundle.root) {
        return Err(invalid(operation, "checkpoint manifest escapes its root"));
    }
    let value = parse_manifest(&bundle.manifest, operation)?;
    for path in manifest_resource_paths(&value, &bundle.root, &manifest_path, operation)? {
        let key = relative_key(&bundle.root, &path, operation)?;
        if require_resources && !bundle.files.contains_key(&key) {
            return Err(invalid(
                operation,
                &format!("checkpoint resource is missing: {key}"),
            ));
        }
    }
    Ok(())
}

fn parse_manifest(bytes: &[u8], operation: &str) -> Result<Value, JsValue> {
    serde_json::from_slice(bytes)
        .map_err(|error| invalid(operation, &format!("invalid checkpoint manifest: {error}")))
}

fn manifest_resource_paths(
    manifest: &Value,
    root: &Path,
    manifest_path: &Path,
    operation: &str,
) -> Result<Vec<PathBuf>, JsValue> {
    let mut resources = Vec::new();
    if let Some(input) = manifest
        .get("state")
        .and_then(|state| state.get("payload"))
        .and_then(|payload| payload.get("owner"))
        .and_then(|owner| owner.get("input_path"))
        .and_then(Value::as_str)
    {
        resources.push(checked_root_path(input, root, operation, "input path")?);
    }
    let sidecars = manifest
        .get("sidecars")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid(operation, "checkpoint sidecars are missing"))?;
    for sidecar in sidecars {
        if sidecar.get("presence").and_then(Value::as_str) != Some("present") {
            continue;
        }
        let relative = sidecar
            .get("relative_path")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid(operation, "present checkpoint sidecar has no path"))?;
        let relative = safe_relative(relative, operation, "sidecar path")?;
        let parent = manifest_path
            .parent()
            .ok_or_else(|| invalid(operation, "checkpoint manifest has no parent"))?;
        let path = parent.join(relative);
        if !path.starts_with(root) {
            return Err(invalid(operation, "checkpoint sidecar escapes its root"));
        }
        resources.push(path);
    }
    let dependencies = manifest
        .get("paths")
        .and_then(|paths| paths.get("external_dependencies"))
        .and_then(Value::as_object)
        .ok_or_else(|| invalid(operation, "checkpoint dependencies are missing"))?;
    for dependency in dependencies.values() {
        let path = dependency
            .get("resolved_path")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid(operation, "checkpoint dependency has no resolved path"))?;
        resources.push(checked_root_path(path, root, operation, "dependency path")?);
    }
    resources.sort();
    resources.dedup();
    Ok(resources)
}

fn sidecar_paths(manifest: &Value, manifest_path: &Path) -> Result<Vec<PathBuf>, JsValue> {
    let mut paths = Vec::new();
    let sidecars = manifest
        .get("sidecars")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("exportCheckpoint", "checkpoint sidecars are missing"))?;
    let parent = manifest_path
        .parent()
        .ok_or_else(|| invalid("exportCheckpoint", "checkpoint manifest has no parent"))?;
    for sidecar in sidecars {
        if sidecar.get("presence").and_then(Value::as_str) != Some("present") {
            continue;
        }
        let relative = sidecar
            .get("relative_path")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("exportCheckpoint", "present checkpoint sidecar has no path"))?;
        let relative = safe_relative(relative, "exportCheckpoint", "sidecar path")?;
        paths.push(parent.join(relative));
    }
    Ok(paths)
}

fn checked_root_path(
    raw: &str,
    root: &Path,
    operation: &str,
    label: &str,
) -> Result<PathBuf, JsValue> {
    let path = Path::new(raw);
    if !swmmrs::platform::is_absolute_path(path) || !path.starts_with(root) {
        return Err(invalid(
            operation,
            &format!("{label} {raw:?} escapes checkpoint root {}", root.display()),
        ));
    }
    let relative = path
        .strip_prefix(root)
        .map_err(|_| invalid(operation, &format!("{label} escapes the checkpoint root")))?;
    safe_relative(
        relative
            .to_str()
            .ok_or_else(|| invalid(operation, &format!("{label} is not valid UTF-8")))?,
        operation,
        label,
    )?;
    Ok(path.to_path_buf())
}

fn validate_root(path: &Path, operation: &str) -> Result<PathBuf, JsValue> {
    let mut components = path.components();
    if components.next() != Some(Component::RootDir)
        || components.next() != Some(Component::Normal(std::ffi::OsStr::new("swmm")))
    {
        return Err(invalid(
            operation,
            "checkpoint root must be an absolute /swmm/<id> path",
        ));
    }
    let Some(Component::Normal(name)) = components.next() else {
        return Err(invalid(
            operation,
            "checkpoint root must have one owner component",
        ));
    };
    if components.next().is_some()
        || name.is_empty()
        || !name
            .as_encoded_bytes()
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(invalid(
            operation,
            "checkpoint root contains an unsafe owner component",
        ));
    }
    Ok(path.to_path_buf())
}

fn safe_relative(raw: &str, operation: &str, label: &str) -> Result<PathBuf, JsValue> {
    if raw.is_empty() || raw.contains(['\0', '\\']) || raw.starts_with('/') {
        return Err(invalid(
            operation,
            &format!("{label} is not a safe relative path"),
        ));
    }
    if raw
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(invalid(
            operation,
            &format!("{label} is not a safe relative path"),
        ));
    }
    let path = PathBuf::from(raw);
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(invalid(
            operation,
            &format!("{label} is not a safe relative path"),
        ));
    }
    Ok(path)
}

fn relative_key(root: &Path, path: &Path, operation: &str) -> Result<String, JsValue> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| invalid(operation, "checkpoint resource escapes its root"))?;
    safe_relative(
        relative
            .to_str()
            .ok_or_else(|| invalid(operation, "checkpoint resource path is not valid UTF-8"))?,
        operation,
        "checkpoint resource path",
    )
    .map(|path| path.to_string_lossy().into_owned())
}

fn checked_size(size: u64, label: &str) -> Result<u64, JsValue> {
    if size > MAX_BUNDLE_FILE_BYTES {
        return Err(invalid(
            "checkpoint",
            &format!("{label} exceeds the checkpoint file-size limit"),
        ));
    }
    Ok(size)
}

fn read_bytes(value: JsValue, operation: &str, label: &str) -> Result<Vec<u8>, JsValue> {
    let value = if value.is_instance_of::<Uint8Array>() {
        value.unchecked_into::<Uint8Array>().to_vec()
    } else if value.is_instance_of::<ArrayBuffer>() {
        Uint8Array::new(&value).to_vec()
    } else {
        return Err(invalid(operation, &format!("{label} must be a Uint8Array")));
    };
    if value.len() as u64 > MAX_BUNDLE_FILE_BYTES {
        return Err(invalid(
            operation,
            &format!("{label} exceeds the file-size limit"),
        ));
    }
    Ok(value)
}

fn bundle_value(bundle: &Bundle) -> Result<JsValue, JsValue> {
    let result = Object::new();
    Reflect::set(
        &result,
        &JsValue::from_str("format"),
        &JsValue::from_str(BUNDLE_FORMAT),
    )?;
    Reflect::set(
        &result,
        &JsValue::from_str("version"),
        &JsValue::from_f64(BUNDLE_VERSION as f64),
    )?;
    let root = bundle.root.to_string_lossy().into_owned();
    Reflect::set(
        &result,
        &JsValue::from_str("root"),
        &JsValue::from_str(&root),
    )?;
    let manifest_path = bundle.manifest_path.to_string_lossy().into_owned();
    Reflect::set(
        &result,
        &JsValue::from_str("manifestPath"),
        &JsValue::from_str(&manifest_path),
    )?;
    let manifest = Uint8Array::from(bundle.manifest.as_slice());
    let manifest_value: JsValue = manifest.into();
    Reflect::set(&result, &JsValue::from_str("manifest"), &manifest_value)?;
    let files = Object::new();
    for (name, bytes) in &bundle.files {
        let bytes = Uint8Array::from(bytes.as_slice());
        let bytes_value: JsValue = bytes.into();
        Reflect::set(&files, &JsValue::from_str(name), &bytes_value)?;
    }
    Reflect::set(&result, &JsValue::from_str("files"), &files)?;
    Ok(result.into())
}

fn private_path(root: &Path, prefix: &str, operation: &str) -> Result<PathBuf, JsValue> {
    for _ in 0..1024 {
        let sequence = NEXT_PRIVATE_FILE.fetch_add(1, Ordering::Relaxed);
        let path = root.join(format!("{prefix}-{sequence}"));
        if fs::metadata(&path).is_err() {
            return Ok(path);
        }
    }
    Err(invalid(operation, "exhausted private project-file names"))
}

fn remove_if_present(path: &Path) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn remove_paths(paths: &[PathBuf]) -> std::io::Result<()> {
    for path in paths {
        remove_if_present(path)?;
    }
    Ok(())
}

fn invalid(operation: &str, detail: &str) -> JsValue {
    let error = Error::new(&format!("{operation}: invalid checkpoint: {detail}"));
    let _ = Reflect::set(
        &error,
        &"code".into(),
        &ErrorCode::ApiPropertyValue.as_i32().into(),
    );
    let _ = Reflect::set(&error, &"operation".into(), &operation.into());
    error.into()
}

fn io_error(operation: &str, error: std::io::Error) -> JsValue {
    let value = Error::new(&format!("{operation}: {error}"));
    let _ = Reflect::set(&value, &"operation".into(), &operation.into());
    value.into()
}

fn checkpoint_error(operation: &str, error: swmmrs::engine::error::SwmmError) -> JsValue {
    let value = Error::new(&error.to_string());
    let _ = Reflect::set(&value, &"code".into(), &error.code.as_i32().into());
    let _ = Reflect::set(&value, &"operation".into(), &operation.into());
    value.into()
}
