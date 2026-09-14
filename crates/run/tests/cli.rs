use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use swmmrs::engine::consts::SOLVER_VERSION;

static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

/// Returns the checked-in integration fixture.
///
/// # Returns
/// Absolute path to the real SWMM input fixture.
///
/// # Panics
/// Panics when the checked-in fixture is absent.
fn fixture_path() -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/test_example1.inp");
    assert!(path.is_file(), "missing test INP at {}", path.display());
    path
}

/// Creates a unique temporary directory for one process test.
///
/// # Arguments
/// * `name` - Human-readable test directory prefix.
///
/// # Returns
/// Path to the created directory.
///
/// # Panics
/// Panics when the directory cannot be created.
fn test_dir(name: &str) -> PathBuf {
    let sequence = NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "swmm-run-process-{name}-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

/// Verifies help is available without arguments and through either help flag.
#[test]
fn help_is_available_without_arguments_and_through_both_flags() {
    for flag in [None, Some("-h"), Some("--help")] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_runswmmrs"));
        if let Some(flag) = flag {
            command.arg(flag);
        }
        let output = command.output().unwrap();

        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("Usage:\n  runswmmrs")
        );
        assert!(output.stderr.is_empty());
    }
}

/// Verifies `--version` prints the exact solver engine version.
#[test]
fn version_is_exact_swmm_engine_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_runswmmrs"))
        .arg("--version")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("runswmm version {SOLVER_VERSION}\n")
    );
    assert!(output.stderr.is_empty());
}

/// Verifies the executable exits successfully and releases nonempty outputs.
#[test]
fn valid_cli_run_succeeds_and_writes_nonempty_files() {
    let dir = test_dir("success");
    let report = dir.join("fixture.rpt");
    let binary = dir.join("fixture.out");

    let output = Command::new(env!("CARGO_BIN_EXE_runswmmrs"))
        .arg(fixture_path())
        .arg(&report)
        .arg(&binary)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("swmmrs completed")
    );
    assert!(output.stderr.is_empty());
    assert!(report.metadata().unwrap().len() > 0);
    assert!(binary.metadata().unwrap().len() > 0);
    let report_contents = std::fs::read_to_string(&report).unwrap();
    assert!(report_contents.contains("Runoff Quantity Continuity"));
    assert!(report_contents.contains("Node Depth Summary"));

    std::fs::remove_file(report).unwrap();
    std::fs::remove_file(binary).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

/// Verifies startup failure has canonical stderr and a nonzero exit status.
#[test]
fn startup_failure_uses_canonical_error_and_nonzero_exit() {
    let dir = test_dir("start-failure");
    let report = dir.join("fixture.rpt");
    let output_directory = dir.join("not-an-output-file");
    std::fs::create_dir(&output_directory).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_runswmmrs"))
        .arg(fixture_path())
        .arg(&report)
        .arg(&output_directory)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: \n  ERROR 307: cannot open binary results file.\n"
    );

    std::fs::remove_file(report).unwrap();
    std::fs::remove_dir(output_directory).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

/// Verifies a missing input path preserves the solver's canonical file error.
#[test]
fn missing_input_returns_canonical_input_file_error() {
    let dir = test_dir("missing-input");
    let input = dir.join("missing.inp");
    let report = dir.join("missing.rpt");
    let binary = dir.join("missing.out");

    let output = Command::new(env!("CARGO_BIN_EXE_runswmmrs"))
        .arg(&input)
        .arg(&report)
        .arg(&binary)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: \n  ERROR 303: cannot open input file.\n"
    );
    assert!(!report.exists());
    assert!(!binary.exists());

    std::fs::remove_dir(dir).unwrap();
}

/// Verifies the CLI does not create a missing report parent directory.
#[test]
fn missing_report_parent_returns_canonical_report_file_error() {
    let dir = test_dir("missing-report-parent");
    let report_parent = dir.join("missing");
    let report = report_parent.join("fixture.rpt");
    let binary = dir.join("fixture.out");

    let output = Command::new(env!("CARGO_BIN_EXE_runswmmrs"))
        .arg(fixture_path())
        .arg(&report)
        .arg(&binary)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: \n  ERROR 305: cannot open report file.\n"
    );
    assert!(!report_parent.exists());
    assert!(!binary.exists());

    std::fs::remove_dir(dir).unwrap();
}

/// Verifies the CLI does not create a missing binary-output parent directory.
#[test]
fn missing_output_parent_returns_canonical_binary_file_error() {
    let dir = test_dir("missing-output-parent");
    let report = dir.join("fixture.rpt");
    let output_parent = dir.join("missing");
    let binary = output_parent.join("fixture.out");

    let output = Command::new(env!("CARGO_BIN_EXE_runswmmrs"))
        .arg(fixture_path())
        .arg(&report)
        .arg(&binary)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: \n  ERROR 307: cannot open binary results file.\n"
    );
    assert!(!output_parent.exists());
    assert!(!binary.exists());

    std::fs::remove_file(report).unwrap();
    std::fs::remove_dir(dir).unwrap();
}
