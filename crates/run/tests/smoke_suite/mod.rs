//! Explicitly selected run-crate smoke tests for curated SWMM models.
//!
//! Most tests retain reports and binary outputs under
//! `target/run-smoke-suite/<model>/`. The detailed-report smoke test uses scratch
//! binary output so the run lifecycle renders the report.
//!
//! Run exactly one model with:
//! `cargo test --manifest-path crates/run/Cargo.toml --locked --bin runswmmrs smoke_suite::<test_name> -- --ignored --exact --test-threads=1`.

use std::path::{Path, PathBuf};

use super::run_simulation;

/// Returns the checked-in path for one smoke-test model.
///
/// # Arguments
/// * `relative_input` - Model path relative to `tests/data/regression-suite`.
///
/// # Returns
/// Absolute path to the requested SWMM input model.
///
/// # Panics
/// Panics when the requested input model is absent.
fn model_path(relative_input: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/regression-suite")
        .join(relative_input);
    assert!(
        path.is_file(),
        "missing smoke-test INP at {}",
        path.display()
    );
    path
}
/// Creates the persistent artifact directory for one smoke-test model.
///
/// # Arguments
/// * `relative_input` - Model path relative to `tests/data/regression-suite`.
///
/// # Returns
/// Repository-local directory that retains the model's report and binary output.
///
/// # Panics
/// Panics when the repository root cannot be derived or the directory cannot be created.
fn artifact_directory(relative_input: &str) -> PathBuf {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("run crate should be located at <repository>/crates/run");
    let model_path = Path::new(relative_input).with_extension("");
    let path = repository_root
        .join("target/run-smoke-suite")
        .join(model_path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

/// Smoke-runs one model through the run crate lifecycle.
///
/// # Arguments
/// * `relative_input` - Model path relative to `tests/data/regression-suite`.
/// * `save_output` - Whether to retain the binary output instead of rendering the detailed report.
///
/// # Panics
/// Panics when model or artifact paths are invalid, artifacts cannot be refreshed, or the run
/// lifecycle returns an error.
fn run_model(relative_input: &str, save_output: bool) {
    let input = model_path(relative_input);
    run_input_model(relative_input, &input, save_output);
}

/// Executes one input path while storing its artifacts under a smoke model's directory.
///
/// # Arguments
/// * `relative_input` - Model path relative to `tests/data/regression-suite`.
/// * `input` - Input file to execute.
/// * `save_output` - Whether to retain the binary output instead of rendering the detailed report.
///
/// # Panics
/// Panics when artifact paths are invalid, artifacts cannot be refreshed, or the run lifecycle
/// returns an error.
fn run_input_model(relative_input: &str, input: &Path, save_output: bool) {
    let model_name = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .expect("smoke model should have a UTF-8 file stem");
    let artifacts = artifact_directory(relative_input);
    let report = artifacts.join(format!("{model_name}.rpt"));
    let output = save_output.then(|| artifacts.join(format!("{model_name}.out")));
    for artifact in std::iter::once(&report).chain(output.iter()) {
        if artifact.exists() {
            std::fs::remove_file(artifact)
                .unwrap_or_else(|error| panic!("cannot refresh {}: {error}", artifact.display()));
        }
    }
    println!("smoke artifacts: {}", artifacts.display());

    run_simulation(
        input
            .to_str()
            .expect("smoke input path should be valid UTF-8"),
        report
            .to_str()
            .expect("smoke report path should be valid UTF-8"),
        output.as_ref().and_then(|path| path.to_str()).unwrap_or(""),
    )
    .unwrap_or_else(|error| {
        panic!(
            "{} failed: {error}; artifacts: {}",
            input.display(),
            artifacts.display()
        )
    });
}
/// Copies every regular file from one fixture directory into a scratch directory.
///
/// # Arguments
/// * `source` - Checked-in fixture directory to copy.
/// * `destination` - Scratch directory that receives the files.
///
/// # Panics
/// Panics if the source directory cannot be enumerated or a regular file cannot be copied.
fn stage_regular_files(source: &Path, destination: &Path) {
    let entries = std::fs::read_dir(source)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", source.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            panic!("cannot read an entry in {}: {error}", source.display())
        });
        let source_file = entry.path();
        let file_type = entry
            .file_type()
            .unwrap_or_else(|error| panic!("cannot inspect {}: {error}", source_file.display()));
        if file_type.is_file() {
            let destination_file = destination.join(entry.file_name());
            std::fs::copy(&source_file, &destination_file).unwrap_or_else(|error| {
                panic!(
                    "cannot copy {} to {}: {error}",
                    source_file.display(),
                    destination_file.display()
                )
            });
        }
    }
}

macro_rules! smoke_model_test {
    ($name:ident, $relative_input:literal) => {
        #[doc = concat!("Smoke-runs `", $relative_input, "` through the run crate lifecycle.")]
        #[test]
        #[ignore = "explicit smoke test; select one case with --ignored --exact --test-threads=1"]
        fn $name() {
            run_model($relative_input, true);
        }
    };
}

/// Renders a detailed report by running a representative model with scratch binary output.
///
/// # Panics
/// Panics if the model cannot complete or its detailed report cannot be rendered.
#[test]
#[ignore = "explicit smoke test; select one case with --ignored --exact --test-threads=1"]
fn variables_expressions_rain_detailed_report() {
    let input = model_path("controls/variables-expressions-rain.inp");
    run_input_model(
        "controls/variables-expressions-rain-detailed-report.inp",
        &input,
        false,
    );
}

smoke_model_test!(
    waterquality_events_example,
    "water-quality/waterquality-events_example.inp"
);
smoke_model_test!(
    treatment_waterquality_events_example,
    "water-quality/treatment-waterquality-events_example.inp"
);
smoke_model_test!(
    landuse_function_matrix,
    "water-quality/landuse-function-matrix.inp"
);
smoke_model_test!(runoff_use, "use_interfaces/runoff-use.inp");
smoke_model_test!(rdii_use, "use_interfaces/rdii-use.inp");
smoke_model_test!(rainfall_use, "use_interfaces/rainfall-use.inp");
smoke_model_test!(hotstart_use, "use_interfaces/hotstart-use.inp");
smoke_model_test!(inflows_use_toy, "use_interfaces/inflows-use-toy.inp");
/// Generates each supported interface in scratch space, then consumes it with its paired model.
///
/// # Panics
/// Panics if scratch staging or cleanup fails, an interface is not generated, or a model run fails.
#[test]
#[ignore = "explicit smoke test; select one case with --ignored --exact --test-threads=1"]
fn interface_roundtrips() {
    let work = artifact_directory("interface-roundtrips.inp").join("work");
    if work
        .try_exists()
        .unwrap_or_else(|error| panic!("cannot inspect {}: {error}", work.display()))
    {
        std::fs::remove_dir_all(&work)
            .unwrap_or_else(|error| panic!("cannot clean {}: {error}", work.display()));
    }
    std::fs::create_dir_all(&work)
        .unwrap_or_else(|error| panic!("cannot create {}: {error}", work.display()));

    let save_directory = model_path("save_interfaces/save_outflows.inp")
        .parent()
        .expect("SAVE fixture should have a parent directory")
        .to_path_buf();
    let use_directory = model_path("use_interfaces/inflows-use-toy.inp")
        .parent()
        .expect("USE fixture should have a parent directory")
        .to_path_buf();
    stage_regular_files(&save_directory, &work);
    stage_regular_files(&use_directory, &work);

    let roundtrips = [
        (
            "save_outflows.inp",
            "inflows-interface.dat",
            "inflows-use-toy.inp",
        ),
        (
            "save_rainfall.inp",
            "rainfall-interface.bin",
            "rainfall-use.inp",
        ),
        ("save_runoff.inp", "runoff-interface.bin", "runoff-use.inp"),
        ("save_rdii.inp", "rdii-interface.bin", "rdii-use.inp"),
        (
            "save_hotstart.inp",
            "hotstart-interface.bin",
            "hotstart-use.inp",
        ),
    ];

    for &(_, interface_file, _) in &roundtrips {
        let interface = work.join(interface_file);
        if interface
            .try_exists()
            .unwrap_or_else(|error| panic!("cannot inspect {}: {error}", interface.display()))
        {
            std::fs::remove_file(&interface)
                .unwrap_or_else(|error| panic!("cannot remove {}: {error}", interface.display()));
        }
    }

    for &(save_input, interface_file, _) in &roundtrips {
        let input = work.join(save_input);
        run_input_model(&format!("interface-roundtrips/{save_input}"), &input, true);
        let interface = work.join(interface_file);
        assert!(
            interface.is_file(),
            "{} did not generate {}",
            input.display(),
            interface.display()
        );
    }
    for &(_, _, use_input) in &roundtrips {
        let input = work.join(use_input);
        run_input_model(&format!("interface-roundtrips/{use_input}"), &input, true);
    }
}
smoke_model_test!(rdii_use_text, "use_interfaces/rdii-use-text.inp");
smoke_model_test!(
    kinwave_routing_kinwave,
    "routing/kinwave-routing_kinwave.inp"
);
smoke_model_test!(kinwave_storage_shapes, "routing/kinwave-storage-shapes.inp");
smoke_model_test!(
    steady_attenuation_steady_flow,
    "routing/steady-attenuation_steady_flow.inp"
);
smoke_model_test!(kinwave_divider_types, "routing/kinwave-divider-types.inp");
smoke_model_test!(
    pump_storage_table_matrix,
    "routing/pump-storage-table-matrix.inp"
);
smoke_model_test!(
    elevation_offsets_outfalls_all_outfall_types_model,
    "hydraulics/elevation-offsets_outfalls-all_outfall_types_model.inp"
);
smoke_model_test!(
    inlets_inlet_capture_test,
    "hydraulics/inlets-inlet_capture_test.inp"
);
smoke_model_test!(
    inlets_onsag_slotted_custom,
    "hydraulics/inlets-onsag-slotted-custom.inp"
);
smoke_model_test!(
    culvert_roadway_exfiltration,
    "hydraulics/culvert-roadway-exfiltration.inp"
);
smoke_model_test!(
    storage_cross_section_matrix,
    "hydraulics/storage-cross-section-matrix.inp"
);
smoke_model_test!(
    outfalls_all_outfall_types_model,
    "hydraulics/outfalls-all_outfall_types_model.inp"
);
smoke_model_test!(shapes_swmm5_shapes, "hydraulics/shapes-swmm5_shapes.inp");
smoke_model_test!(
    forcemain_darcy_weisbach,
    "hydraulics/forcemain-darcy-weisbach.inp"
);
smoke_model_test!(
    slot_surcharge_tunnelmh,
    "hydraulics/slot-surcharge_tunnelmh.inp"
);
smoke_model_test!(
    inertial_damping_full_exam80a_sw5,
    "hydraulics/inertial-damping-full_exam80a-sw5.inp"
);
smoke_model_test!(
    outlet_tabular_depth_extran3_rc,
    "hydraulics/outlet-tabular-depth_extran3-rc.inp"
);
smoke_model_test!(
    curvenumber_lid_example_lid_rb,
    "hydrology/curvenumber-lid-example_lid_rb.inp"
);
smoke_model_test!(
    groundwater_si_gw_model,
    "hydrology/groundwater-si_gw_model.inp"
);
smoke_model_test!(lid_example_lid_rb, "hydrology/lid-example_lid_rb.inp");
smoke_model_test!(
    climate_file_evaporation,
    "hydrology/climate-file-evaporation.inp"
);
/// Stages the green-roof model so its detailed LID report is retained with the other artifacts.
///
/// The checked-in model intentionally keeps a simple report name for provenance. This test
/// rewrites that destination into the model artifact directory so a regression run never writes
/// `crates/run/groof.txt` or another checkout-root file.
///
/// # Panics
/// Panics if model staging, artifact cleanup, or the simulation lifecycle fails.
#[test]
#[ignore = "explicit smoke test; select one case with --ignored --exact --test-threads=1"]
fn lid_green_roof_paired() {
    let relative_input = "hydrology/lid-green-roof-paired.inp";
    let source_input = model_path(relative_input);
    let artifacts = artifact_directory(relative_input);
    let detail_report = artifacts.join("groof.txt");
    if detail_report
        .try_exists()
        .unwrap_or_else(|error| panic!("cannot inspect {}: {error}", detail_report.display()))
    {
        std::fs::remove_file(&detail_report)
            .unwrap_or_else(|error| panic!("cannot refresh {}: {error}", detail_report.display()));
    }

    let source =
        std::fs::read_to_string(&source_input).expect("green-roof model should be readable");
    assert!(
        source.contains("\"groof.txt\""),
        "green-roof model should name its detailed report"
    );
    let staged_input = artifacts.join("lid-green-roof-paired.inp");
    let report_path = detail_report
        .to_str()
        .expect("green-roof detail report path should be valid UTF-8");
    std::fs::write(
        &staged_input,
        source.replace("\"groof.txt\"", &format!("\"{report_path}\"")),
    )
    .expect("staged green-roof model should be writable");

    run_input_model(relative_input, &staged_input, true);
    assert!(
        detail_report.is_file(),
        "green-roof detailed report was not written to the artifact directory"
    );
    let checkout_root_report = Path::new(env!("CARGO_MANIFEST_DIR")).join("groof.txt");
    assert!(
        !checkout_root_report.exists(),
        "green-roof regression polluted the run crate: {}",
        checkout_root_report.display()
    );
}
smoke_model_test!(climate_td3200, "hydrology/climate-td3200.inp");
smoke_model_test!(climate_dly0204, "hydrology/climate-dly0204.inp");
smoke_model_test!(ncdc_rainfall, "hydrology/ncdc-rainfall.inp");
smoke_model_test!(
    rainfall_format_matrix,
    "hydrology/rainfall-format-matrix.inp"
);
smoke_model_test!(modified_horton, "hydrology/modified-horton.inp");
smoke_model_test!(modified_green_ampt, "hydrology/modified-green-ampt.inp");
smoke_model_test!(outfall_runon, "hydrology/outfall-runon.inp");
/// Stages the LID model so its detailed unit report is retained with other test artifacts.
///
/// # Panics
/// Panics if model staging, dependency copying, or the simulation lifecycle fails.
#[test]
#[ignore = "explicit smoke test; select one case with --ignored --exact --test-threads=1"]
fn lid_controls_ghcnd() {
    let relative_input = "hydrology/lid-controls-ghcnd.inp";
    let source_input = model_path(relative_input);
    let artifacts = artifact_directory(relative_input);
    let detail_report = artifacts.join("lid-detail.txt");
    let source = std::fs::read_to_string(&source_input).expect("LID model should be readable");
    assert!(
        source.contains("lid-detail.txt"),
        "LID model should name its detailed report"
    );
    let staged_input = artifacts.join("lid-controls-ghcnd.inp");
    std::fs::write(
        &staged_input,
        source.replace(
            "lid-detail.txt",
            detail_report
                .to_str()
                .expect("LID detail report path should be valid UTF-8"),
        ),
    )
    .expect("staged LID model should be writable");
    std::fs::copy(
        source_input.with_file_name("ghcnd-temperature.dat"),
        artifacts.join("ghcnd-temperature.dat"),
    )
    .expect("GHCND dependency should be copied");

    run_input_model(relative_input, &staged_input, true);
}
smoke_model_test!(
    rdii_assignment_groundwater_si_gw_model,
    "hydrology/rdii-assignment_groundwater-si_gw_model.inp"
);
smoke_model_test!(
    snowmelt_small_snowmelt_model,
    "hydrology/snowmelt-small_snowmelt_model.inp"
);
smoke_model_test!(
    greenville_lid_snow_groundwater,
    "hydrology/greenville-lid-snow-groundwater.inp"
);
smoke_model_test!(rtc_master_extran_rtc, "controls/rtc-master_extran_rtc.inp");
smoke_model_test!(rtc_many_rules, "controls/rtc-many-rules.inp");
smoke_model_test!(
    variables_expressions_rain,
    "controls/variables-expressions-rain.inp"
);
smoke_model_test!(math_operator_matrix, "controls/math-operator-matrix.inp");
