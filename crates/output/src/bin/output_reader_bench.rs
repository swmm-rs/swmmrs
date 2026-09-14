use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use swmm_output::{
    BulkSeriesResult, FlowUnits, LinkResultAttribute, NodeResultAttribute, OutputMetadata,
    OutputRange, OutputReader, OutputSeriesSelection, RunStatus, SubcatchmentResultAttribute,
    SystemResultAttribute,
};

const SHA256_INITIAL: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

const SHA256_ROUND: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

struct Sha256 {
    state: [u32; 8],
    buffer: [u8; 64],
    buffered: usize,
    length: u64,
}

impl Sha256 {
    fn new() -> Self {
        Self {
            state: SHA256_INITIAL,
            buffer: [0; 64],
            buffered: 0,
            length: 0,
        }
    }

    fn update(&mut self, bytes: &[u8]) {
        self.length = self.length.wrapping_add(bytes.len() as u64);
        let mut input = bytes;
        if self.buffered != 0 {
            let take = (64 - self.buffered).min(input.len());
            self.buffer[self.buffered..self.buffered + take].copy_from_slice(&input[..take]);
            self.buffered += take;
            input = &input[take..];
            if self.buffered == 64 {
                let block = self.buffer;
                self.compress(&block);
                self.buffered = 0;
            }
        }
        while input.len() >= 64 {
            self.compress(&input[..64]);
            input = &input[64..];
        }
        if !input.is_empty() {
            self.buffer[..input.len()].copy_from_slice(input);
            self.buffered = input.len();
        }
    }

    fn compress(&mut self, block: &[u8]) {
        let mut words = [0u32; 64];
        for (index, word) in words[..16].iter_mut().enumerate() {
            let offset = index * 4;
            *word = u32::from_be_bytes([
                block[offset],
                block[offset + 1],
                block[offset + 2],
                block[offset + 3],
            ]);
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }
        let mut state = self.state;
        for index in 0..64 {
            let s1 =
                state[4].rotate_right(6) ^ state[4].rotate_right(11) ^ state[4].rotate_right(25);
            let choice = (state[4] & state[5]) ^ ((!state[4]) & state[6]);
            let temp1 = state[7]
                .wrapping_add(s1)
                .wrapping_add(choice)
                .wrapping_add(SHA256_ROUND[index])
                .wrapping_add(words[index]);
            let s0 =
                state[0].rotate_right(2) ^ state[0].rotate_right(13) ^ state[0].rotate_right(22);
            let majority = (state[0] & state[1]) ^ (state[0] & state[2]) ^ (state[1] & state[2]);
            let temp2 = s0.wrapping_add(majority);
            state = [
                temp1.wrapping_add(temp2),
                state[0],
                state[1],
                state[2],
                state[3].wrapping_add(temp1),
                state[4],
                state[5],
                state[6],
            ];
        }
        for (destination, source) in self.state.iter_mut().zip(state) {
            *destination = destination.wrapping_add(source);
        }
    }

    fn finish(mut self) -> [u8; 32] {
        let bit_length = self.length.wrapping_mul(8);
        self.update(&[0x80]);
        let padding = [0u8; 64];
        if self.buffered > 56 {
            self.update(&padding[..64 - self.buffered]);
        }
        self.update(&padding[..56 - self.buffered]);
        self.update(&bit_length.to_be_bytes());
        let mut digest = [0u8; 32];
        for (index, word) in self.state.iter().enumerate() {
            digest[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
        }
        digest
    }
}

fn hex_digest(digest: [u8; 32]) -> String {
    let mut text = String::with_capacity(64);
    for byte in digest {
        text.push(char::from(b"0123456789abcdef"[(byte >> 4) as usize]));
        text.push(char::from(b"0123456789abcdef"[(byte & 0x0f) as usize]));
    }
    text
}

#[derive(Clone, Copy)]
struct Cell {
    selection: OutputSeriesSelection,
    physical_index: usize,
}

struct Options {
    file: PathBuf,
    workload: String,
    strategy: String,
    mode: String,
}

fn parse_options(arguments: impl Iterator<Item = String>) -> Result<Options, String> {
    let mut file = None;
    let mut workload = None;
    let mut strategy = None;
    let mut mode = None;
    let mut arguments = arguments.peekable();
    while let Some(argument) = arguments.next() {
        let mut value = || {
            arguments
                .next()
                .ok_or_else(|| format!("missing value after {argument}"))
        };
        match argument.as_str() {
            "--file" => file = Some(PathBuf::from(value()?)),
            "--workload" => workload = Some(value()?),
            "--strategy" => strategy = Some(value()?),
            "--mode" => mode = Some(value()?),
            "--bench" => {}
            "--help" | "-h" => return Err(HELP.to_string()),
            unknown => return Err(format!("unknown option {unknown}")),
        }
    }
    Ok(Options {
        file: file.ok_or_else(|| "missing required --file".to_string())?,
        workload: workload.ok_or_else(|| "missing required --workload".to_string())?,
        strategy: strategy.ok_or_else(|| "missing required --strategy".to_string())?,
        mode: mode.ok_or_else(|| "missing required --mode".to_string())?,
    })
}

const HELP: &str = "Standalone swmm-output benchmark\n\nUsage: output-reader-bench --file PATH --workload OP --strategy STRATEGY --mode verify|time\n";

fn flow_code(units: FlowUnits) -> i32 {
    match units {
        FlowUnits::Cfs => 0,
        FlowUnits::Gpm => 1,
        FlowUnits::Mgd => 2,
        FlowUnits::Cms => 3,
        FlowUnits::Lps => 4,
        FlowUnits::Mld => 5,
        FlowUnits::Unknown(value) => value,
    }
}

fn run_code(status: RunStatus) -> i32 {
    match status {
        RunStatus::Unfinalized => i32::MIN,
        RunStatus::Success => 0,
        RunStatus::Warning(value) => value,
    }
}

fn subcatchment_code(attribute: SubcatchmentResultAttribute) -> i32 {
    match attribute {
        SubcatchmentResultAttribute::Rainfall => 0,
        SubcatchmentResultAttribute::SnowDepth => 1,
        SubcatchmentResultAttribute::EvaporationLoss => 2,
        SubcatchmentResultAttribute::InfiltrationLoss => 3,
        SubcatchmentResultAttribute::RunoffFlow => 4,
        SubcatchmentResultAttribute::GroundwaterFlow => 5,
        SubcatchmentResultAttribute::GroundwaterElevation => 6,
        SubcatchmentResultAttribute::SoilMoisture => 7,
        SubcatchmentResultAttribute::Pollutant(id) => 8 + id.index() as i32,
        SubcatchmentResultAttribute::Unknown(value) => value,
    }
}

fn node_code(attribute: NodeResultAttribute) -> i32 {
    match attribute {
        NodeResultAttribute::Depth => 0,
        NodeResultAttribute::HydraulicHead => 1,
        NodeResultAttribute::StoredVolume => 2,
        NodeResultAttribute::LateralInflow => 3,
        NodeResultAttribute::TotalInflow => 4,
        NodeResultAttribute::Overflow => 5,
        NodeResultAttribute::Pollutant(id) => 6 + id.index() as i32,
        NodeResultAttribute::Unknown(value) => value,
    }
}

fn link_code(attribute: LinkResultAttribute) -> i32 {
    match attribute {
        LinkResultAttribute::Flow => 0,
        LinkResultAttribute::Depth => 1,
        LinkResultAttribute::Velocity => 2,
        LinkResultAttribute::Volume => 3,
        LinkResultAttribute::Capacity => 4,
        LinkResultAttribute::Pollutant(id) => 5 + id.index() as i32,
        LinkResultAttribute::Unknown(value) => value,
    }
}

fn system_code(attribute: SystemResultAttribute) -> i32 {
    match attribute {
        SystemResultAttribute::AirTemperature => 0,
        SystemResultAttribute::Rainfall => 1,
        SystemResultAttribute::SnowDepth => 2,
        SystemResultAttribute::InfiltrationLoss => 3,
        SystemResultAttribute::RunoffFlow => 4,
        SystemResultAttribute::DryWeatherInflow => 5,
        SystemResultAttribute::GroundwaterInflow => 6,
        SystemResultAttribute::RdiiInflow => 7,
        SystemResultAttribute::ExternalInflow => 8,
        SystemResultAttribute::TotalLateralInflow => 9,
        SystemResultAttribute::FloodingOutflow => 10,
        SystemResultAttribute::OutfallFlow => 11,
        SystemResultAttribute::StorageVolume => 12,
        SystemResultAttribute::Evaporation => 13,
        SystemResultAttribute::PotentialEvapotranspiration => 14,
        SystemResultAttribute::Unknown(value) => value,
    }
}

fn put_i32(hash: &mut Sha256, value: i32) {
    hash.update(&value.to_le_bytes());
}
fn put_u32(hash: &mut Sha256, value: u32) {
    hash.update(&value.to_le_bytes());
}
fn put_u64(hash: &mut Sha256, value: u64) {
    hash.update(&value.to_le_bytes());
}
fn put_name(hash: &mut Sha256, bytes: &[u8]) {
    put_u32(hash, bytes.len() as u32);
    hash.update(bytes);
}

fn put_selection(hash: &mut Sha256, selection: OutputSeriesSelection) {
    match selection {
        OutputSeriesSelection::Subcatchment { id, attribute } => {
            put_i32(hash, 0);
            put_u64(hash, id.index() as u64);
            put_i32(hash, subcatchment_code(attribute));
        }
        OutputSeriesSelection::Node { id, attribute } => {
            put_i32(hash, 1);
            put_u64(hash, id.index() as u64);
            put_i32(hash, node_code(attribute));
        }
        OutputSeriesSelection::Link { id, attribute } => {
            put_i32(hash, 2);
            put_u64(hash, id.index() as u64);
            put_i32(hash, link_code(attribute));
        }
        OutputSeriesSelection::System { attribute } => {
            put_i32(hash, 3);
            put_i32(hash, system_code(attribute));
        }
    }
}

fn structured_fingerprint(result: &BulkSeriesResult) -> String {
    let mut hash = Sha256::new();
    put_u64(&mut hash, result.times().len() as u64);
    for time in result.times() {
        put_u64(&mut hash, time.serial_days().to_bits());
    }
    put_u64(&mut hash, result.series().len() as u64);
    for series in result.series() {
        put_selection(&mut hash, series.selection());
        put_u64(&mut hash, series.values().len() as u64);
        for value in series.values() {
            put_u32(&mut hash, value.to_bits());
        }
    }
    hex_digest(hash.finish())
}

fn value_fingerprint(result: &BulkSeriesResult) -> String {
    let mut hash = Sha256::new();
    for period in 0..result.times().len() {
        for series in result.series() {
            if let Some(value) = series.values().get(period) {
                put_u32(&mut hash, value.to_bits());
            }
        }
    }
    hex_digest(hash.finish())
}

fn metadata_fingerprint(metadata: &OutputMetadata) -> String {
    let mut hash = Sha256::new();
    put_i32(&mut hash, metadata.solver_release());
    put_i32(&mut hash, flow_code(metadata.flow_units()));
    put_i32(&mut hash, run_code(metadata.run_status()));
    put_u64(
        &mut hash,
        metadata
            .report_timing()
            .report_schedule_origin()
            .serial_days()
            .to_bits(),
    );
    put_i32(
        &mut hash,
        metadata.report_timing().report_step().as_secs() as i32,
    );
    put_i32(&mut hash, metadata.report_timing().period_count() as i32);
    for names in [
        metadata
            .subcatchments()
            .iter()
            .map(|item| item.name().as_bytes())
            .collect::<Vec<_>>(),
        metadata
            .nodes()
            .iter()
            .map(|item| item.name().as_bytes())
            .collect::<Vec<_>>(),
        metadata
            .links()
            .iter()
            .map(|item| item.name().as_bytes())
            .collect::<Vec<_>>(),
        metadata
            .pollutants()
            .iter()
            .map(|item| item.name().as_bytes())
            .collect::<Vec<_>>(),
    ] {
        put_u32(&mut hash, names.len() as u32);
        for name in names {
            put_name(&mut hash, name);
        }
    }
    for pollutant in metadata.pollutants() {
        put_i32(
            &mut hash,
            match pollutant.concentration_units() {
                swmm_output::ConcentrationUnits::MilligramsPerLiter => 0,
                swmm_output::ConcentrationUnits::MicrogramsPerLiter => 1,
                swmm_output::ConcentrationUnits::CountsPerLiter => 2,
                swmm_output::ConcentrationUnits::Unknown(value) => value,
            },
        );
    }
    for item in metadata.subcatchments() {
        put_u32(&mut hash, item.area().to_bits());
    }
    for item in metadata.nodes() {
        put_i32(
            &mut hash,
            match item.kind() {
                swmm_output::NodeKind::Junction => 0,
                swmm_output::NodeKind::Outfall => 1,
                swmm_output::NodeKind::Storage => 2,
                swmm_output::NodeKind::Divider => 3,
                swmm_output::NodeKind::Unknown(value) => value,
            },
        );
        put_u32(&mut hash, item.invert_elevation().to_bits());
        put_u32(&mut hash, item.maximum_depth().to_bits());
    }
    for item in metadata.links() {
        put_i32(
            &mut hash,
            match item.kind() {
                swmm_output::LinkKind::Conduit => 0,
                swmm_output::LinkKind::Pump => 1,
                swmm_output::LinkKind::Orifice => 2,
                swmm_output::LinkKind::Weir => 3,
                swmm_output::LinkKind::Outlet => 4,
                swmm_output::LinkKind::Unknown(value) => value,
            },
        );
        put_u32(&mut hash, item.inlet_offset().to_bits());
        put_u32(&mut hash, item.outlet_offset().to_bits());
        put_u32(&mut hash, item.maximum_depth().to_bits());
        put_u32(&mut hash, item.length().to_bits());
    }
    for attributes in [
        metadata
            .result_schema()
            .subcatchment()
            .iter()
            .map(|value| subcatchment_code(*value))
            .collect::<Vec<_>>(),
        metadata
            .result_schema()
            .node()
            .iter()
            .map(|value| node_code(*value))
            .collect::<Vec<_>>(),
        metadata
            .result_schema()
            .link()
            .iter()
            .map(|value| link_code(*value))
            .collect::<Vec<_>>(),
        metadata
            .result_schema()
            .system()
            .iter()
            .map(|value| system_code(*value))
            .collect::<Vec<_>>(),
    ] {
        put_u32(&mut hash, attributes.len() as u32);
        for attribute in attributes {
            put_i32(&mut hash, attribute);
        }
    }
    hex_digest(hash.finish())
}

fn all_cells(metadata: &OutputMetadata) -> Vec<Cell> {
    let schema = metadata.result_schema();
    let mut cells = Vec::new();
    let mut physical_index = 0;
    for item in metadata.subcatchments() {
        for attribute in schema.subcatchment() {
            cells.push(Cell {
                selection: OutputSeriesSelection::Subcatchment {
                    id: item.id(),
                    attribute: *attribute,
                },
                physical_index,
            });
            physical_index += 1;
        }
    }
    for item in metadata.nodes() {
        for attribute in schema.node() {
            cells.push(Cell {
                selection: OutputSeriesSelection::Node {
                    id: item.id(),
                    attribute: *attribute,
                },
                physical_index,
            });
            physical_index += 1;
        }
    }
    for item in metadata.links() {
        for attribute in schema.link() {
            cells.push(Cell {
                selection: OutputSeriesSelection::Link {
                    id: item.id(),
                    attribute: *attribute,
                },
                physical_index,
            });
            physical_index += 1;
        }
    }
    for attribute in schema.system() {
        cells.push(Cell {
            selection: OutputSeriesSelection::System {
                attribute: *attribute,
            },
            physical_index,
        });
        physical_index += 1;
    }
    cells
}
fn payload_cell_count(metadata: &OutputMetadata) -> usize {
    metadata.subcatchments().len() * metadata.result_schema().subcatchment().len()
        + metadata.nodes().len() * metadata.result_schema().node().len()
        + metadata.links().len() * metadata.result_schema().link().len()
        + metadata.result_schema().system().len()
}

fn workload_cells(metadata: &OutputMetadata, operation: &str) -> Result<Vec<Cell>, String> {
    if matches!(operation, "open-metadata" | "stored-dates") {
        return Ok(Vec::new());
    }
    let cells = all_cells(metadata);
    if cells.is_empty() {
        return Ok(Vec::new());
    }
    match operation {
        "singleton" => Ok(vec![cells[0]]),
        "sparse-8" => Ok((0..cells.len().min(8))
            .map(|index| cells[(index * (cells.len() - 1)) / 7])
            .collect()),
        "clustered-32-duplicates" => Ok((0..32)
            .map(|index| cells[index % cells.len().min(16)])
            .collect()),
        "spread-1pct" => {
            let step = (cells.len() / 100).max(1);
            Ok((0..cells.len())
                .step_by(step)
                .map(|index| cells[index])
                .collect())
        }
        "all-values" => Ok(cells),
        _ => Err(format!("unknown workload operation {operation}")),
    }
}

fn workload_periods(metadata: &OutputMetadata, operation: &str) -> Vec<usize> {
    let count = metadata.report_timing().period_count();
    match operation {
        "open-metadata" => Vec::new(),
        "stored-dates" => {
            let start = usize::from(count > 1);
            (start..count.min(start + 8)).collect()
        }
        "all-values" => (0..count).collect(),
        _ => (0..count.min(8)).collect(),
    }
}

fn metrics(
    cells: &[Cell],
    periods: &[usize],
    strategy: &str,
    payload_bytes: usize,
    operation: &str,
) -> (usize, usize, usize) {
    if operation == "open-metadata" {
        return (0, 0, 0);
    }
    if operation == "stored-dates" {
        return (
            periods.len() * 8,
            periods.len(),
            if periods.is_empty() { 0 } else { 8 },
        );
    }
    let mut unique: Vec<usize> = cells.iter().map(|cell| cell.physical_index).collect();
    unique.sort_unstable();
    unique.dedup();
    if strategy == "by-period" {
        return (
            periods.len() * payload_bytes,
            periods.len(),
            if periods.is_empty() { 0 } else { payload_bytes },
        );
    }
    let mut runs = 0;
    let mut largest = 0;
    let mut current = 0;
    let mut previous = None;
    for cell in unique.iter().copied() {
        if previous != Some(cell.wrapping_sub(1)) {
            runs += 1;
            current = 1;
        } else {
            current += 1;
        }
        largest = largest.max(current);
        previous = Some(cell);
    }
    (
        periods.len() * unique.len() * 4,
        periods.len() * runs,
        largest * 4,
    )
}

fn run(options: Options) -> Result<(), String> {
    if options.mode != "verify" && options.mode != "time" {
        return Err(format!(
            "unknown benchmark mode {}; expected verify or time",
            options.mode
        ));
    }
    let mut reader = OutputReader::open(&options.file).map_err(|error| error.to_string())?;
    let metadata = reader.metadata();
    let cells = workload_cells(metadata, &options.workload)?;
    let periods = workload_periods(metadata, &options.workload);
    let payload_bytes = payload_cell_count(metadata) * 4;
    let (read_bytes, read_calls, scratch_bytes) = metrics(
        &cells,
        &periods,
        &options.strategy,
        payload_bytes,
        &options.workload,
    );
    let mut fingerprint = String::new();
    let mut structured_digest = String::new();
    let value_count;
    if options.workload == "open-metadata" {
        if options.mode == "verify" {
            fingerprint = metadata_fingerprint(metadata);
        } else {
            std::hint::black_box(metadata);
        }
        value_count = 0;
    } else if options.workload == "stored-dates" {
        let start = periods.first().copied().unwrap_or(0);
        let end = periods.last().map_or(start, |period| period + 1);
        let range = OutputRange::Periods { start, end };
        let dates = reader
            .read_stored_dates(range)
            .map_err(|error| error.to_string())?;
        value_count = dates.len();
        if options.mode == "verify" {
            let mut hash = Sha256::new();
            for date in &dates {
                hash.update(&date.serial_days().to_bits().to_le_bytes());
            }
            fingerprint = hex_digest(hash.finish());
        } else {
            std::hint::black_box(&dates);
        }
    } else {
        let selections: Vec<OutputSeriesSelection> =
            cells.iter().map(|cell| cell.selection).collect();
        let start = periods.first().copied().unwrap_or(0);
        let end = periods.last().map_or(start, |period| period + 1);
        let range = OutputRange::Periods { start, end };
        let result = if options.strategy == "by-period" {
            reader
                .read_bulk_series_by_period(&selections, range)
                .map_err(|error| error.to_string())?
        } else if options.strategy == "selective" {
            reader
                .read_bulk_series(&selections, range)
                .map_err(|error| error.to_string())?
        } else {
            return Err(format!("unknown result strategy {}", options.strategy));
        };
        value_count = result
            .times()
            .len()
            .checked_mul(result.series().len())
            .ok_or_else(|| "result dimension count overflow".to_string())?;
        if options.mode == "verify" {
            fingerprint = value_fingerprint(&result);
            structured_digest = structured_fingerprint(&result);
        } else {
            std::hint::black_box(&result);
        }
    }
    println!(
        "{{\"schema_version\":1,\"workload\":\"{}\",\"strategy\":\"{}\",\"fingerprint\":\"{}\",\"structured_fingerprint\":\"{}\",\"value_count\":{},\"read_bytes\":{},\"read_calls\":{},\"scratch_bytes\":{}}}",
        options.workload,
        options.strategy,
        fingerprint,
        structured_digest,
        value_count,
        read_bytes,
        read_calls,
        scratch_bytes
    );
    Ok(())
}

fn main() -> ExitCode {
    let arguments = env::args().skip(1);
    match parse_options(arguments).and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if error == HELP => {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
