use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use js_sys::{Error, Reflect};
use serde::Serialize;
use swmmrs::engine::datetime::{DateTime, datetime_decodeDate, datetime_decodeTime};
use swmmrs::engine::enums::ObjectType;
use swmmrs::engine::error::{ErrorCode, SwmmError};
use swmmrs::platform::fs;
use swmmrs::simulation::links::{LinkResultsRead, LinkSnapshotRead};
use swmmrs::simulation::nodes::{NodeResultsRead, NodeSnapshotRead};
use swmmrs::simulation::{SimulationLifecycle, SwmmSimulation};
use wasm_bindgen::prelude::*;

mod native;

use native::common::decode_value;

static NEXT_PROJECT: AtomicUsize = AtomicUsize::new(0);

/// An isolated project. Call close before freeing it to report cleanup errors.
#[wasm_bindgen]
pub struct Simulation {
    inner: SwmmSimulation,
    root: PathBuf,
}

#[wasm_bindgen]
impl Simulation {
    /// Input and external files are bytes; external names are relative to the model.
    #[wasm_bindgen(constructor)]
    pub fn new(input: &[u8], files: JsValue) -> Result<Simulation, JsValue> {
        let files: BTreeMap<String, Vec<u8>> = serde_wasm_bindgen::from_value(files)?;
        let mut owner = Self {
            inner: SwmmSimulation::default(),
            root: PathBuf::from(format!(
                "/swmm/{}",
                NEXT_PROJECT.fetch_add(1, Ordering::Relaxed)
            )),
        };
        fs::create_dir_all(&owner.root).map_err(io_error)?;
        for (name, bytes) in files {
            if matches!(name.as_str(), "model.inp" | "model.rpt" | "model.out") {
                return Err(Error::new(&format!("reserved external-file path: {name}")).into());
            }
            let path = owner.path(&name)?;
            fs::create_dir_all(path.parent().unwrap()).map_err(io_error)?;
            fs::write(path, bytes).map_err(io_error)?;
        }
        let input_path = owner.root.join("model.inp");
        let report_path = owner.root.join("model.rpt");
        let output_path = owner.root.join("model.out");
        fs::write(&input_path, input).map_err(io_error)?;
        if let Err(error) = owner.inner.open(
            input_path.to_str().unwrap(),
            report_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
        ) {
            let _ = owner.inner.close();
            return Err(owner.error("open", error));
        }
        Ok(owner)
    }

    #[wasm_bindgen(getter)]
    pub fn state(&self) -> String {
        match self.inner.lifecycle_read().state {
            SimulationLifecycle::Closed => "closed",
            SimulationLifecycle::Open => "open",
            SimulationLifecycle::Running => "running",
            SimulationLifecycle::Complete => "complete",
            SimulationLifecycle::Ended => "ended",
            SimulationLifecycle::Failed => "failed",
        }
        .into()
    }

    /// Project units, object IDs, and the effective hydraulic lane count.
    pub fn info(&self) -> Result<JsValue, JsValue> {
        self.info_metadata()
    }

    pub fn start(&mut self, save_results: Option<bool>) -> Result<(), JsValue> {
        self.inner
            .start(save_results.unwrap_or(true))
            .map_err(|e| self.error("start", e))
    }

    /// Timezone-free model timestamp, or undefined at natural completion.
    pub fn step(&mut self) -> Result<Option<String>, JsValue> {
        self.inner
            .step()
            .map(|time| time.map(timestamp))
            .map_err(|e| self.error("step", e))
    }

    /// Advances by a positive whole number of seconds.
    pub fn stride(&mut self, seconds: f64, strict: bool) -> Result<Option<String>, JsValue> {
        if !seconds.is_finite()
            || seconds.fract() != 0.0
            || !(1.0..=i32::MAX as f64).contains(&seconds)
        {
            return Err(Error::new("seconds must be a positive 32-bit integer").into());
        }
        self.inner
            .stride(seconds as i32, strict)
            .map(|time| time.map(timestamp))
            .map_err(|e| self.error("stride", e))
    }

    pub fn node(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Node, id)?;
        let values = self
            .inner
            .node_results_read(index)
            .map_err(|e| self.error("node", e))?;
        NodeResults::serialize(&values, &serializer()).map_err(Into::into)
    }

    pub fn link(&self, id: &str) -> Result<JsValue, JsValue> {
        let index = self.index(ObjectType::Link, id)?;
        let values = self
            .inner
            .link_results_read(index)
            .map_err(|e| self.error("link", e))?;
        LinkResults::serialize(&values, &serializer()).map_err(Into::into)
    }

    /// One coherent columnar snapshot, preserving the requested ID order.
    pub fn nodes(&self, ids: Option<Vec<String>>) -> Result<JsValue, JsValue> {
        let values = self
            .inner
            .node_snapshot_read(ids.as_deref())
            .map_err(|e| self.error("nodes", e))?;
        NodeSnapshot::serialize(&values, &serializer()).map_err(Into::into)
    }

    pub fn links(&self, ids: Option<Vec<String>>) -> Result<JsValue, JsValue> {
        let values = self
            .inner
            .link_snapshot_read(ids.as_deref())
            .map_err(|e| self.error("links", e))?;
        LinkSnapshot::serialize(&values, &serializer()).map_err(Into::into)
    }
    /// Runs to completion without a worker message for every routing step.
    pub fn run(&mut self, save_results: Option<bool>) -> Result<(), JsValue> {
        self.start(save_results)?;
        while self
            .inner
            .step()
            .map_err(|e| self.error("run", e))?
            .is_some()
        {}
        self.finish()
    }

    #[wasm_bindgen(js_name = setNodeExternalInflow)]
    pub fn set_node_external_inflow(&mut self, id: &str, flow: JsValue) -> Result<(), JsValue> {
        let operation = "setNodeExternalInflow";
        let index = self.index(ObjectType::Node, id)?;
        let flow = decode_value(flow, operation)?;
        self.inner
            .set_node_external_inflow(index, flow)
            .map_err(|e| self.error(operation, e))
    }

    #[wasm_bindgen(js_name = setLinkTargetSetting)]
    pub fn set_link_target_setting(&mut self, id: &str, setting: JsValue) -> Result<(), JsValue> {
        let operation = "setLinkTargetSetting";
        let index = self.index(ObjectType::Link, id)?;
        let setting = decode_value(setting, operation)?;
        self.inner
            .set_link_target_setting(index, setting)
            .map_err(|e| self.error(operation, e))
    }

    /// Finalizes binary output, requested detailed report tables, and the runtime footer.
    pub fn finish(&mut self) -> Result<(), JsValue> {
        self.inner.end().map_err(|e| self.error("finish", e))?;
        self.inner
            .flush_output()
            .map_err(|e| self.error("finish", e))?;
        if self.inner.lifecycle_read().save_results {
            self.inner.report().map_err(|e| self.error("finish", e))?;
        }
        self.inner
            .finalize_report()
            .map_err(|e| self.error("finish", e))
    }

    /// Ends computation and flushes output and summary statistics without detailed report tables.
    pub fn end(&mut self) -> Result<(), JsValue> {
        self.inner.end().map_err(|e| self.error("end", e))?;
        self.inner.flush_output().map_err(|e| self.error("end", e))?;
        self.inner.flush_report().map_err(|e| self.error("end", e))
    }

    /// Appends only the runtime footer to an ended run's summary report.
    #[wasm_bindgen(js_name = finalizeReport)]
    pub fn finalize_report(&mut self) -> Result<(), JsValue> {
        self.inner
            .finalize_report()
            .map_err(|e| self.error("finalizeReport", e))
    }

    /// Generates the requested report tables and footer on demand.
    pub fn report(&mut self) -> Result<(), JsValue> {
        self.inner.report().map_err(|e| self.error("report", e))?;
        self.inner
            .finalize_report()
            .map_err(|e| self.error("report", e))
    }

    #[wasm_bindgen(js_name = resetSolver)]
    pub fn reset_solver(&mut self) -> Result<(), JsValue> {
        self.inner
            .reset_solver()
            .map_err(|e| self.error("resetSolver", e))
    }

    #[wasm_bindgen(js_name = sleepWorkers)]
    pub fn sleep_workers(&mut self) -> Result<(), JsValue> {
        self.inner
            .sleep_dynamic_wave_workers()
            .map_err(|e| self.error("sleepWorkers", e))
    }

    /// Reads an input or generated file before close removes the project.
    #[wasm_bindgen(js_name = readFile)]
    pub fn read_file(&self, name: &str) -> Result<Vec<u8>, JsValue> {
        fs::read(self.path(name)?).map_err(io_error)
    }

    pub fn close(&mut self) -> Result<(), JsValue> {
        let result = self.inner.close().map_err(|e| self.error("close", e));
        let cleanup = fs::remove_dir_all(&self.root);
        result?;
        match cleanup {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(io_error(error)),
        }
    }
}

impl Simulation {
    fn path(&self, name: &str) -> Result<PathBuf, JsValue> {
        if name.contains(['\0', '\\'])
            || name
                .split('/')
                .any(|part| part.is_empty() || part == "." || part == "..")
        {
            return Err(Error::new(&format!("invalid project-file path: {name}")).into());
        }
        Ok(self.root.join(name))
    }

    fn index(&self, kind: ObjectType, id: &str) -> Result<usize, JsValue> {
        self.inner
            .object_identity_read(kind, id)
            .map_err(|e| self.error("lookup", e))?
            .map(|identity| identity.index)
            .ok_or_else(|| {
                self.error(
                    "lookup",
                    SwmmError::with_detail(
                        ErrorCode::ApiObjectIndex,
                        format!("unknown object: {id}"),
                    ),
                )
            })
    }

    fn error(&self, operation: &str, error: SwmmError) -> JsValue {
        let mut message = error.to_string();
        if let Some(detail) = &error.detail {
            if !message.contains(detail) {
                message.push_str(": ");
                message.push_str(detail);
            }
        }
        let value = Error::new(&message);
        let _ = Reflect::set(&value, &"code".into(), &error.code.as_i32().into());
        let _ = Reflect::set(&value, &"operation".into(), &operation.into());
        if let Some(detail) = &error.detail {
            let _ = Reflect::set(&value, &"detail".into(), &detail.as_str().into());
        }
        if let Some(code) = error.semantic_code() {
            let _ = Reflect::set(&value, &"semanticCode".into(), &code.into());
        }
        if let Some(diagnostics) = &error.configuration_diagnostics {
            let records: Vec<_> = diagnostics
                .iter()
                .map(|diagnostic| Diagnostic {
                    object: DiagnosticIdentity::from(&diagnostic.object),
                    property_path: &diagnostic.property_path,
                    rule_code: &diagnostic.rule_code,
                    message: &diagnostic.message,
                    conflicting_object: diagnostic
                        .conflicting_object
                        .as_ref()
                        .map(DiagnosticIdentity::from),
                })
                .collect();
            if let Ok(records) = records.serialize(&serializer()) {
                let _ = Reflect::set(&value, &"diagnostics".into(), &records);
            }
        }
        if let Ok(report) = fs::read_to_string(self.root.join("model.rpt")) {
            let _ = Reflect::set(&value, &"report".into(), &report.into());
        }
        value.into()
    }
}

impl Drop for Simulation {
    fn drop(&mut self) {
        let _ = self.inner.close();
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn io_error(error: std::io::Error) -> JsValue {
    Error::new(&error.to_string()).into()
}

fn serializer() -> serde_wasm_bindgen::Serializer {
    serde_wasm_bindgen::Serializer::json_compatible()
}

fn timestamp(time: DateTime) -> String {
    let (mut year, mut month, mut day, mut hour, mut minute, mut second) = (0, 0, 0, 0, 0, 0);
    datetime_decodeDate(time, &mut year, &mut month, &mut day);
    datetime_decodeTime(time, &mut hour, &mut minute, &mut second);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}")
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticIdentity<'a> {
    object_type: &'a str,
    id: &'a str,
    index: usize,
}

impl<'a> From<&'a swmmrs::engine::error::ConfigurationObjectIdentity> for DiagnosticIdentity<'a> {
    fn from(value: &'a swmmrs::engine::error::ConfigurationObjectIdentity) -> Self {
        Self {
            object_type: &value.object_type,
            id: &value.id,
            index: value.index,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Diagnostic<'a> {
    object: DiagnosticIdentity<'a>,
    property_path: &'a str,
    rule_code: &'a str,
    message: &'a str,
    conflicting_object: Option<DiagnosticIdentity<'a>>,
}

#[derive(Serialize)]
#[serde(remote = "NodeResultsRead", rename_all = "camelCase")]
struct NodeResults {
    depth: f64,
    head: f64,
    volume: f64,
    lateral_inflow: f64,
    total_inflow: f64,
    total_outflow: f64,
    losses: f64,
    flooding: f64,
    hydraulic_retention_seconds: Option<f64>,
}

#[derive(Serialize)]
#[serde(remote = "NodeSnapshotRead", rename_all = "camelCase")]
struct NodeSnapshot {
    object_ids: Vec<String>,
    depth: Vec<f64>,
    head: Vec<f64>,
    volume: Vec<f64>,
    lateral_inflow: Vec<f64>,
    total_inflow: Vec<f64>,
    total_outflow: Vec<f64>,
    losses: Vec<f64>,
    flooding: Vec<f64>,
    hydraulic_retention_seconds: Vec<Option<f64>>,
}

#[derive(Serialize)]
#[serde(remote = "LinkResultsRead", rename_all = "camelCase")]
struct LinkResults {
    setting: f64,
    target_setting: f64,
    time_open_seconds: f64,
    time_closed_seconds: f64,
    flow: f64,
    depth: f64,
    velocity: f64,
    top_width: Option<f64>,
    volume: f64,
    capacity: f64,
    upstream_surface_area: f64,
    downstream_surface_area: f64,
    froude_number: f64,
}

#[derive(Serialize)]
#[serde(remote = "LinkSnapshotRead", rename_all = "camelCase")]
struct LinkSnapshot {
    object_ids: Vec<String>,
    setting: Vec<f64>,
    target_setting: Vec<f64>,
    time_open_seconds: Vec<f64>,
    time_closed_seconds: Vec<f64>,
    flow: Vec<f64>,
    depth: Vec<f64>,
    velocity: Vec<f64>,
    top_width: Vec<Option<f64>>,
    volume: Vec<f64>,
    capacity: Vec<f64>,
    upstream_surface_area: Vec<f64>,
    downstream_surface_area: Vec<f64>,
    froude_number: Vec<f64>,
}
