use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;

use swmmrs::simulation::SwmmSimulation;

// Inputs and all generated model artifacts live in the caller's private run directory.
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: swmmrs-native-bench INPUT RUN_DIRECTORY REPETITIONS".into());
    }
    let input = &args[0];
    let directory = PathBuf::from(&args[1]);
    let repetitions: usize = args[2].parse()?;
    if repetitions == 0 {
        return Err("REPETITIONS must be positive".into());
    }
    fs::create_dir(&directory)?;
    let report_path = directory.join("model.rpt");
    let output_path = directory.join("model.out");
    let started = Instant::now();
    let mut simulation = SwmmSimulation::default();
    simulation.open(
        input,
        report_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
    )?;
    let open_ms = started.elapsed().as_secs_f64() * 1000.0;
    let threads = simulation.effective_thread_count_read()?;
    println!("{{\"event\":\"open\",\"openMs\":{open_ms},\"effectiveThreads\":{threads}}}");
    io::stdout().flush()?;

    let result = (|| -> Result<(), Box<dyn Error>> {
        for run in 0..repetitions {
            let started = Instant::now();
            simulation.start(true)?;
            while simulation.step()?.is_some() {}
            simulation.end()?;
            simulation.flush_output()?;
            simulation.finalize_report()?;
            // The public browser run() also reads both complete files before returning.
            let report = fs::read_to_string(&report_path)?;
            let output = fs::read(&output_path)?;
            let run_ms = started.elapsed().as_secs_f64() * 1000.0;
            let state = simulation.lifecycle_read();
            let steps = state.total_step_count;
            let periods = state.report_period_count;
            let warnings = state.warning_count;
            let output_bytes = output.len();
            let report_bytes = report.len();
            println!(
                "{{\"event\":\"run\",\"run\":{run},\"runMs\":{run_ms},\"stepCount\":{steps},\"reportPeriodCount\":{periods},\"warningCount\":{warnings},\"outputBytes\":{output_bytes},\"reportBytes\":{report_bytes}}}"
            );
            io::stdout().flush()?;
        }
        Ok(())
    })();
    let started = Instant::now();
    let cleanup = simulation.close();
    result?;
    cleanup?;
    let close_ms = started.elapsed().as_secs_f64() * 1000.0;
    println!("{{\"event\":\"close\",\"closeMs\":{close_ms}}}");
    Ok(())
}
