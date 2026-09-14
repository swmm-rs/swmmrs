// -----------------------------------------------------------------------------
//   runswmm.rs (command-line wrapper)
//
//   Project:  EPA SWMM5 (Rust port)
//   Purpose:  Mirror the C `run` package command-line driver using
//             `SwmmSimulation`.
// -----------------------------------------------------------------------------

#![allow(non_snake_case)]

use std::env;
use std::io::{self, IsTerminal, Write};
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use rand::prelude::IndexedRandom;
use swmmrs::engine::consts::SOLVER_VERSION;
use swmmrs::engine::error::SwmmError;
use swmmrs::simulation::SwmmSimulation;
const USAGE: &str = "Usage: runswmmrs <input file> <report file> <output file>";
const HELP: &str = concat!(
    "Run a SWMM5 simulation.\n\n",
    "Usage:\n",
    "  runswmmrs <input file> <report file> <output file>\n\n",
    "Options:\n",
    "  -h, --help     Show this help\n",
    "  -v, --version  Show the SWMM engine version",
);
const SPINNER: [&str; 67] = [
    // ─────────────── Scuttle right ───────────────
    "▐🦀______________________________▌",
    "▐.🦀_____________________________▌",
    "▐.·🦀____________________________▌",
    "▐_.·°🦀__________________________▌",
    "▐__.°·o🦀________________________▌",
    "▐___.·oO°🦀______________________▌",
    "▐____·°oO○◌🦀____________________▌",
    "▐_____.o°O◌○🦀___________________▌",
    "▐______.·oO○◌🦀__________________▌",
    "▐_______.°Oo◌○🦀_________________▌",
    "▐________.·oO○◌🦀________________▌",
    "▐_________.o°O◌○🦀_______________▌",
    "▐__________.·oO○◌🦀______________▌",
    "▐___________.°Oo◌○🦀_____________▌",
    "▐____________.·oO○◌🦀____________▌",
    "▐_____________.o°O◌○🦀___________▌",
    "▐______________.·oO○◌🦀__________▌",
    "▐_______________.°Oo◌○🦀_________▌",
    "▐________________.·oO○◌🦀________▌",
    "▐_________________.o°O◌○🦀_______▌",
    "▐__________________.·oO○◌🦀______▌",
    "▐___________________.°Oo◌○🦀_____▌",
    "▐____________________.·oO○◌🦀____▌",
    "▐_____________________.o°O◌○🦀___▌",
    "▐______________________.·oO○◌🦀__▌",
    "▐_______________________.°Oo◌○🦀_▌",
    "▐________________________.·o○◌🦀>▌",
    // ─────────────── Hit the wall ───────────────
    "▐________________________o°O◌○🦀!▌",
    "▐_________________________·oO○🦀!▌",
    "▐___________________________oO🦀!▌",
    "▐_____________________________🦀!▌",
    "▐_____________________________🦀!▌",
    // ─────────────── Back up ───────────────
    "▐_____________________________🦀.▌",
    "▐____________________________🦀·.▌",
    "▐___________________________🦀°·o▌",
    "▐_________________________🦀◌○Oo·▌",
    "▐________________________🦀○◌Oo·.▌",
    // ─────────────── Scuttle left ───────────────
    "▐________________________🦀◌○oO°.▌",
    "▐_______________________🦀○◌Oo·._▌",
    "▐______________________🦀◌○O°o.__▌",
    "▐_____________________🦀○◌Oo·.___▌",
    "▐____________________🦀◌○oO°.____▌",
    "▐___________________🦀○◌Oo·._____▌",
    "▐__________________🦀◌○O°o.______▌",
    "▐_________________🦀○◌Oo·._______▌",
    "▐________________🦀◌○oO°.________▌",
    "▐_______________🦀○◌Oo·._________▌",
    "▐______________🦀◌○O°o.__________▌",
    "▐_____________🦀○◌Oo·.___________▌",
    "▐____________🦀◌○oO°.____________▌",
    "▐___________🦀○◌Oo·._____________▌",
    "▐__________🦀◌○O°o.______________▌",
    "▐_________🦀○◌Oo·._______________▌",
    "▐________🦀◌○oO°.________________▌",
    "▐_______🦀○◌Oo·._________________▌",
    "▐______🦀◌○O°o.__________________▌",
    "▐_____🦀○◌Oo·.___________________▌",
    "▐____🦀◌○oO°.____________________▌",
    "▐___🦀○◌Oo·._____________________▌",
    "▐__🦀◌○O°o.______________________▌",
    "▐_🦀○◌O◌·._______________________▌",
    "▐<🦀O○◌.·________________________▌",
    "▐!🦀○o·._________________________▌",
    "▐!🦀○◌.__________________________▌",
    "▐!🦀◌·___________________________▌",
    "▐!🦀.____________________________▌",
    "▐!🦀_____________________________▌",
];
const CRAB_VERBS: &[&str] = &[
    "Scuttling",
    "Sidling",
    "Skittering",
    "Pinching",
    "Clacking",
    "Snapping",
    "Burrowing",
    "Moulting",
    "Foraging",
    "Lurking",
    "Scrambling",
    "Shuffling",
    "Waving",
    "Clambering",
    "Crabbing",
    "Crabulating",
    "Crustaceating",
    "Shelling",
    "Shellabrating",
    "Pincering",
    "Clawing",
    "Carapacing",
    "Barnacling",
    "Beachcombing",
    "Tidepooling",
    "Kelping",
    "Scuttlemaxxing",
    "Pinchifying",
    "Crustafying",
    "Shellinating",
    "Snippity-snapping",
    "Clickclacking",
    "Sandboogeying",
    "Tidewaddling",
    "Crabonizing",
    "Crustulating",
    "Shellicating",
    "Ferrisizing",
    "Borrowing",
    "Unsafe-scuttling",
    "Macro-expanding",
    "Trait-impling",
    "Pinching bytes",
    "Owning",
    "Dereferencing",
    "Monomorphizing",
    "Cargoing",
    "Rustifying",
];

const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);
const PROGRESS_PUBLISH_STEPS: usize = 64;
const PROGRESS_DONE: u64 = u64::MAX;

/// Parses command-line arguments, runs SWMM, and exits with the resulting status.
fn main() {
    let args: Vec<String> = env::args().collect();
    let exit_code = match args.as_slice() {
        [_] => {
            println!("{HELP}");
            0
        }
        [_, flag] if flag == "--help" || flag == "-h" => {
            println!("{}", HELP);
            0
        }
        [_, flag] if flag == "--version" || flag == "-v" => {
            println!("runswmm version {SOLVER_VERSION}");
            0
        }
        [_, input, report, output] => match run_simulation(input, report, output) {
            Ok(_) => 0,
            Err(err) => {
                eprintln!("error: {err}");
                1
            }
        },
        _ => {
            eprintln!("{}", USAGE);
            1
        }
    };

    process::exit(exit_code);
}

/// Runs one SWMM project through the complete command-line lifecycle.
///
/// # Arguments
/// * `input` - Path to the SWMM input file.
/// * `report` - Path to the text report file.
/// * `output` - Path to saved binary output, or an empty string for scratch output.
///
/// # Returns
/// `Ok(())` after the project and all owned resources close successfully.
///
/// # Errors
/// Returns the primary open, start, step, end, report, or close error.
fn run_simulation(input: &str, report: &str, output: &str) -> Result<(), SwmmError> {
    let start = Instant::now();
    let mut simulation = SwmmSimulation::default();
    let lifecycle_result = match simulation.open(input, report, output) {
        Ok(()) => run_opened_and_close(&mut simulation),
        Err(error) => close_preserving(&mut simulation, Err(error)),
    };
    lifecycle_result?;

    let elapsed = start.elapsed();
    println!("\n... swmmrs completed in {:.3?}\n", elapsed);
    Ok(())
}

/// Runs and closes a simulation whose project has already been opened.
///
/// # Arguments
/// * `simulation` - Mutable open SWMM simulation.
///
/// # Returns
/// `Ok(())` after the run and close both succeed.
///
/// # Errors
/// Returns the primary start, step, end, report, or close error.
fn run_opened_and_close(simulation: &mut SwmmSimulation) -> Result<(), SwmmError> {
    let operation_result = run_opened_simulation(simulation);
    close_preserving(simulation, operation_result)
}

/// Starts and executes an open simulation, then ends and reports it.
///
/// # Arguments
/// * `simulation` - Mutable open SWMM simulation.
///
/// # Returns
/// `Ok(())` after the simulation ends and any required report is written.
///
/// # Errors
/// Returns the primary start, step, end, or detailed-report error.
fn run_opened_simulation(simulation: &mut SwmmSimulation) -> Result<(), SwmmError> {
    // A failed start rolls back its partial ownership internally; only a
    // successful start requires the matching end attempt below.

    let mut rng = rand::rng();
    let verb = CRAB_VERBS.choose(&mut rng).unwrap();

    let simulation_time = simulation.simulation_time_read()?;
    simulation.start(true)?;

    let progress = Arc::new(AtomicU64::new(simulation_time.start_time.0.to_bits()));
    let progress_thread = if io::stdout().is_terminal() {
        let progress = Arc::clone(&progress);
        let start_time = simulation_time.start_time.0;
        let end_time = simulation_time.end_time.0;
        thread::Builder::new()
            .name("swmm-progress".into())
            .spawn(move || {
                let mut terminal = io::stdout().lock();
                let mut frame = 0usize;
                loop {
                    let routing_time = progress.load(Ordering::Relaxed);
                    if routing_time == PROGRESS_DONE {
                        break;
                    }
                    let duration = end_time - start_time;
                    let percent = if duration > 0.0 {
                        (100.0 * (f64::from_bits(routing_time) - start_time) / duration)
                            .clamp(0.0, 100.0) as u8
                    } else {
                        0
                    };
                    if write!(
                        terminal,
                        "\r\x1b[2K  {}  {} {:>3}%",
                        SPINNER[frame], verb, percent
                    )
                    .and_then(|_| terminal.flush())
                    .is_err()
                    {
                        return;
                    }
                    thread::park_timeout(PROGRESS_INTERVAL);
                    frame = (frame + 1) % SPINNER.len();
                }
                let _ = write!(terminal, "\r\x1b[2K").and_then(|_| terminal.flush());
            })
            .ok()
    } else {
        None
    };

    let mut step_count = 0usize;

    let step_result = loop {
        match simulation.step() {
            Ok(Some(routing_time)) => {
                if progress_thread.is_some() {
                    step_count += 1;
                    if step_count.is_multiple_of(PROGRESS_PUBLISH_STEPS) {
                        progress.store(routing_time.0.to_bits(), Ordering::Relaxed);
                    }
                }
            }
            Ok(None) => break Ok(()),
            Err(error) => break Err(error),
        }
    };
    progress.store(PROGRESS_DONE, Ordering::Relaxed);
    if let Some(progress_thread) = progress_thread {
        progress_thread.thread().unpark();
        let _ = progress_thread.join();
    }
    finish_started_run(simulation, step_result)
}

/// Ends a started run and writes its detailed report when appropriate.
///
/// # Arguments
/// * `simulation` - Mutable started SWMM simulation.
/// * `step_result` - Result of stepping the simulation to completion.
///
/// # Returns
/// `Ok(())` after end and any required detailed report both succeed.
///
/// # Errors
/// Returns the step error before any end error, otherwise an end or report error.
fn finish_started_run(
    simulation: &mut SwmmSimulation,
    step_result: Result<(), SwmmError>,
) -> Result<(), SwmmError> {
    let end_result = simulation.end();
    step_result?;
    end_result?;

    if simulation.output_file_read().is_scratch {
        simulation.report()?;
    }
    Ok(())
}

/// Closes a simulation without replacing an earlier lifecycle error.
///
/// # Arguments
/// * `simulation` - Mutable simulation whose owned resources are released.
/// * `operation_result` - Primary lifecycle result produced before close.
///
/// # Returns
/// `Ok(())` when both the lifecycle and close succeed.
///
/// # Errors
/// Returns the original primary lifecycle error when present. If close also fails,
/// appends its diagnostic as secondary detail; otherwise returns the close error.
fn close_preserving(
    simulation: &mut SwmmSimulation,
    operation_result: Result<(), SwmmError>,
) -> Result<(), SwmmError> {
    preserve_close_result(operation_result, simulation.close())
}

/// Preserves a primary lifecycle error while incorporating a close result.
///
/// # Arguments
/// * `operation_result` - Primary lifecycle result produced before close.
/// * `close_result` - Result of releasing the owner's resources.
///
/// # Returns
/// `Ok(())` when both operations succeed.
///
/// # Errors
/// Returns the primary error with any distinct cleanup failure retained as detail,
/// or the close error when no primary operation failed.
fn preserve_close_result(
    operation_result: Result<(), SwmmError>,
    close_result: Result<(), SwmmError>,
) -> Result<(), SwmmError> {
    match (operation_result, close_result) {
        (Ok(()), close_result) => close_result,
        (Err(mut primary_error), Err(close_error)) => {
            // Cleanup can surface a distinct lifecycle failure. Attach it only when
            // it adds information beyond the primary operation failure.
            let secondary = close_error.detail.clone().or_else(|| {
                (close_error.code != primary_error.code).then(|| close_error.message())
            });
            if let Some(secondary) = secondary {
                primary_error.detail = Some(match primary_error.detail.take() {
                    Some(detail) => format!("{detail}; secondary cleanup failure: {secondary}"),
                    None => format!("secondary cleanup failure: {secondary}"),
                });
            }
            Err(primary_error)
        }
        (Err(primary_error), Ok(())) => Err(primary_error),
    }
}

#[cfg(test)]
#[path = "../tests/smoke_suite/mod.rs"]
mod smoke_suite;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::sync::{
        Mutex, MutexGuard,
        atomic::{AtomicU64, Ordering},
    };
    use swmmrs::engine::error::ErrorCode;

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);
    static SCRATCH_OUTPUT: Mutex<()> = Mutex::new(());

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

    /// Creates a unique temporary directory for one test.
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
        let path =
            env::temp_dir().join(format!("swmm-run-{name}-{}-{sequence}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    /// Opens the real fixture with paths owned by a test directory.
    ///
    /// # Arguments
    /// * `dir` - Directory that receives report and output files.
    /// * `output` - Output path passed to the solver, or an empty path for scratch output.
    ///
    /// # Returns
    /// An open SWMM simulation.
    ///
    /// # Panics
    /// Panics when the real fixture cannot be opened.
    fn open_fixture(dir: &Path, output: &str) -> SwmmSimulation {
        let mut simulation = SwmmSimulation::default();
        let report = dir.join("fixture.rpt");
        simulation
            .open(
                fixture_path().to_str().unwrap(),
                report.to_str().unwrap(),
                output,
            )
            .unwrap();
        simulation
    }

    /// Verifies a distinct cleanup failure is retained behind primary error detail.
    ///
    /// # Panics
    /// Panics when precedence or diagnostic preservation changes.
    #[test]
    fn close_result_retains_primary_detail_and_secondary_cleanup_failure() {
        let error = preserve_close_result(
            Err(SwmmError::with_detail(
                ErrorCode::Timestep,
                "primary operation context",
            )),
            Err(SwmmError::with_detail(
                ErrorCode::RptFile,
                "report cleanup failed",
            )),
        )
        .unwrap_err();

        assert_eq!(error.code, ErrorCode::Timestep);
        let detail = error.detail.unwrap();
        assert!(detail.contains("primary operation context"));
        assert!(detail.contains("secondary cleanup failure"));
        assert!(detail.contains("report cleanup failed"));
    }

    /// Verifies close does not duplicate the active solver error.
    ///
    /// # Panics
    /// Panics when a repeated active error is appended as cleanup detail.
    #[test]
    fn close_result_does_not_duplicate_repeated_primary_error() {
        let error = preserve_close_result(
            Err(SwmmError::new(ErrorCode::Timestep)),
            Err(SwmmError::new(ErrorCode::Timestep)),
        )
        .unwrap_err();

        assert_eq!(error.code, ErrorCode::Timestep);
        assert_eq!(error.detail, None);
    }

    /// Verifies close errors remain primary when the lifecycle succeeds.
    ///
    /// # Panics
    /// Panics when a standalone close error is replaced.
    #[test]
    fn close_result_returns_close_error_without_primary_failure() {
        let close_error = SwmmError::new(ErrorCode::RptFile);
        assert_eq!(
            preserve_close_result(Ok(()), Err(close_error.clone())).unwrap_err(),
            close_error
        );
    }

    /// Verifies real step errors still end and close without detailed reporting.
    ///
    /// # Panics
    /// Panics when the CLI lifecycle stops stepping, ending, or closing the owner.
    #[test]
    fn step_failure_returns_original_error_and_attempts_end_and_close() {
        let _scratch_output = lock_scratch_output();
        let dir = test_dir("step-failure");
        let mut simulation = open_fixture(&dir, "");
        simulation.force_timestep_error_for_test();

        let operation_result = run_opened_simulation(&mut simulation);
        let lifecycle = simulation.lifecycle_read();
        assert_eq!(
            operation_result.as_ref().unwrap_err().code,
            ErrorCode::Timestep
        );
        assert_eq!(lifecycle.total_step_count, 1);
        assert!(lifecycle.is_open);
        assert!(!lifecycle.is_started);
        let output = simulation.output_file_read();
        assert!(output.is_scratch);
        assert!(output.is_open);

        let error = close_preserving(&mut simulation, operation_result).unwrap_err();

        assert_eq!(error.code, ErrorCode::Timestep);
        assert_eq!(error.detail, None);
        assert!(!simulation.lifecycle_read().is_open);
        assert!(!simulation.output_file_read().is_open);
        let report_contents = std::fs::read_to_string(dir.join("fixture.rpt")).unwrap();
        assert!(!report_contents.contains("Time Series Results"));
        assert!(!report_contents.contains("Runoff Quantity Continuity"));
        std::fs::remove_file(dir.join("fixture.rpt")).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    /// Serializes access to the solver's process-wide scratch output path.
    ///
    /// # Returns
    /// A guard that holds exclusive scratch-output access until dropped.
    fn lock_scratch_output() -> MutexGuard<'static, ()> {
        match SCRATCH_OUTPUT.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// Verifies a complete scratch run writes detailed continuity results.
    ///
    /// # Panics
    /// Panics when the lifecycle, report contents, or cleanup contract changes.
    #[test]
    fn valid_fixture_writes_continuity_statistics_and_detailed_scratch_report() {
        let _scratch_output = lock_scratch_output();
        let dir = test_dir("valid-scratch");
        let report = dir.join("fixture.rpt");

        run_simulation(
            fixture_path().to_str().unwrap(),
            report.to_str().unwrap(),
            "",
        )
        .unwrap();

        let contents = std::fs::read_to_string(&report).unwrap();
        assert!(contents.contains("Runoff Quantity Continuity"));
        assert!(contents.contains("Flow Routing Continuity"));
        assert!(contents.contains("Subcatchment Runoff Summary"));
        assert!(contents.contains("Node Depth Summary"));
        assert!(contents.contains("Link Flow Summary"));
        assert!(contents.contains("Subcatchment Time Series Results"));
        assert!(contents.contains("Node Time Series Results"));
        assert!(contents.contains("Link Time Series Results"));
        assert!(report.metadata().unwrap().len() > 0);

        std::fs::remove_file(report).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    /// Verifies a saved binary lifecycle writes output and releases its handle.
    ///
    /// # Panics
    /// Panics when output is empty or remains open after close.
    #[test]
    fn valid_fixture_keeps_nonempty_saved_binary_and_releases_its_handle() {
        let dir = test_dir("valid-saved");
        let report = dir.join("fixture.rpt");
        let output = dir.join("fixture.out");

        run_simulation(
            fixture_path().to_str().unwrap(),
            report.to_str().unwrap(),
            output.to_str().unwrap(),
        )
        .unwrap();

        assert!(output.metadata().unwrap().len() > 0);
        std::fs::remove_file(report).unwrap();
        std::fs::remove_file(output).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    /// Verifies startup failure preserves its error and releases owned resources.
    ///
    /// # Panics
    /// Panics when startup error precedence or lifecycle cleanup changes.
    #[test]
    fn startup_failure_returns_original_error_closes_and_never_steps() {
        let dir = test_dir("start-failure");
        let output_directory = dir.join("not-an-output-file");
        std::fs::create_dir(&output_directory).unwrap();
        let mut simulation = open_fixture(&dir, output_directory.to_str().unwrap());

        let error = run_opened_and_close(&mut simulation).unwrap_err();

        assert_eq!(error.code, ErrorCode::OutFile);
        let lifecycle = simulation.lifecycle_read();
        assert_eq!(lifecycle.total_step_count, 0);
        assert!(!lifecycle.is_open);
        assert!(!lifecycle.is_started);
        let files = simulation.output_file_read();
        assert!(!files.is_open);
        assert!(!files.report_is_open);
        std::fs::remove_dir(output_directory).unwrap();
        let retry_report = dir.join("retry.rpt");
        let retry_output = dir.join("retry.out");
        simulation
            .open(
                fixture_path().to_str().unwrap(),
                retry_report.to_str().unwrap(),
                retry_output.to_str().unwrap(),
            )
            .unwrap();
        run_opened_and_close(&mut simulation).unwrap();
        std::fs::remove_file(retry_report).unwrap();
        std::fs::remove_file(retry_output).unwrap();
        std::fs::remove_file(dir.join("fixture.rpt")).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    /// Verifies project-open failure survives owner cleanup.
    ///
    /// # Panics
    /// Panics when open error precedence or cleanup changes.
    #[test]
    fn open_failure_is_preserved_by_close() {
        let dir = test_dir("open-failure");
        let input = dir.join("invalid.inp");
        let report = dir.join("invalid.rpt");
        std::fs::write(&input, "[OPTIONS]\nFLOW_UNITS INVALID\n").unwrap();
        let mut simulation = SwmmSimulation::default();
        let open_result = simulation.open(input.to_str().unwrap(), report.to_str().unwrap(), "");
        let expected_code = open_result.as_ref().unwrap_err().code;

        let error = close_preserving(&mut simulation, open_result).unwrap_err();

        assert_eq!(error.code, expected_code);
        assert!(!simulation.lifecycle_read().is_open);
        std::fs::remove_file(input).unwrap();
        std::fs::remove_file(report).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    /// Verifies detailed reporting remains limited to scratch binary output.
    ///
    /// # Panics
    /// Panics when saved or scratch reporting behavior changes.
    #[test]
    fn saved_output_skips_detailed_report_but_scratch_output_reports() {
        let saved_dir = test_dir("saved-report-mode");
        let saved_output = saved_dir.join("fixture.out");
        let mut saved = open_fixture(&saved_dir, saved_output.to_str().unwrap());
        run_opened_and_close(&mut saved).unwrap();
        let saved_report = std::fs::read_to_string(saved_dir.join("fixture.rpt")).unwrap();
        assert!(!saved_report.contains("Time Series Results"));

        let scratch_dir = test_dir("scratch-report-mode");
        let _scratch_output = lock_scratch_output();
        let mut scratch = open_fixture(&scratch_dir, "");
        run_opened_and_close(&mut scratch).unwrap();
        let scratch_report = std::fs::read_to_string(scratch_dir.join("fixture.rpt")).unwrap();
        assert!(scratch_report.contains("Time Series Results"));

        std::fs::remove_file(saved_dir.join("fixture.rpt")).unwrap();
        std::fs::remove_file(saved_output).unwrap();
        std::fs::remove_dir(saved_dir).unwrap();
        std::fs::remove_file(scratch_dir.join("fixture.rpt")).unwrap();
        std::fs::remove_dir(scratch_dir).unwrap();
    }
}
