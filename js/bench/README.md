# Compare native and browser execution

This benchmark runs a self-contained INP through the native Rust owner and the
public browser `Simulation.run()` API. It does not change the original input or
solver. Temporary input copies differ only in the `THREADS` setting and newline
normalization.

## Build

From `js/`:

```sh
npm ci
npm run build
cargo build --release --locked --manifest-path bench/native/Cargo.toml --target-dir target/native-bench
```

Native and browser builds use the pinned toolchain in `js/rust-toolchain.toml`.
The native benchmark matches the native CLI's release profile, including fat LTO and one
codegen unit. `npm run build` produces both browser artifacts using the
`js/Cargo.toml` profile. A browser run with `threads: 1` loads the serial artifact
in `dist/serial/`; counts above `1` load the threaded artifact in `dist/`.
The baseline therefore measures the current release configurations, not identical
optimization flags.

## Run

Choose a new output directory outside the repository. It contains copies of the
model and output files, which can include private identifiers.

```sh
CHROMIUM_PATH=/path/to/chromium node bench/compare.mjs \
	--input /path/to/model.inp \
	--output /tmp/swmm-benchmark-run \
	--threads 1,2,4,8 \
	--repeats 4
```

Omit `CHROMIUM_PATH` to use Playwright's installed Chromium. Use `--engine native`
or `--engine wasm` to run one side. `--wasm-dist` selects a separate generated
WASM directory for controlled build experiments. Include its threaded artifacts,
`snippets/`, and `serial/` subtree. Run `--help` for all options.

The runner binds its asset server to loopback only. It runs the engines
sequentially and alternates which engine goes first across thread settings.
Do not run builds, tests, or other CPU-intensive work during measurement.

## Read the measurements

`environment.json` records the input and executable hashes, runtime versions,
CPU information, and requested parameters. `timings.jsonl` is append-only and
retains completed measurements if a later run fails. `results.json` contains
the complete successful run.

For each thread setting, each engine opens one owner and executes it repeatedly.
Run zero is the first execution on that owner. Later runs reuse it and should
be analyzed separately from the first run. Browser code caching may persist
between configurations, so run zero is not necessarily a cold browser cache.

| Measurement | Included work |
| --- | --- |
| Native `openMs` | Owner construction and model parsing, excluding process launch |
| Native `launchToOpenMs` | Process launch through receipt of the open event |
| Browser `importMs` | Public JavaScript module loading, excluding page/browser launch |
| Browser `openMs` | Input staging, worker/module/pool initialization, model parsing |
| `runMs` | Start, batch routing, end, output flush, report finalization, and complete report/output reads |
| `closeMs` | Explicit owner cleanup |

Browser `runMs` also includes its one worker request/response and returned-record
handling. Neither engine collects live snapshots or progress during routing.
Statistics/status logging and saving browser artifacts happen outside the timer.
Native report and output files use the filesystem at the chosen output path;
using a RAM-backed `/tmp` avoids measuring physical-disk throughput against the
browser's memory filesystem. Those filesystem implementations still differ.

The text report appends to the same file on each execution. Each timed run reads
the complete accumulated report, so later repetitions read more report bytes.
Compare matching run ordinals and retain the reported byte counts.

The output comparator checks header/metadata equality, report layout, finalized
status, exact report timestamps, and every saved float32 result. It reports
byte equality, differing-value count, maximum absolute difference, and maximum
scaled difference, with scale `max(1, abs(native), abs(wasm))`. These differences
span result fields with different units; they are not a domain-specific acceptance
threshold. Text reports contain runtime-dependent footers and are not compared
byte-for-byte. The native directory retains its last binary output, compared
with the final browser run at the same thread count. All browser run outputs
are retained for additional checks.

This is a local batch-lifecycle benchmark. It is not an isolated arithmetic-kernel
benchmark or a cross-browser qualification. Three repeat samples give a useful
local estimate, not a general performance guarantee. Thread settings run in the
requested order, which can confound thread scaling with changing machine load
or temperature. Repeat in a different order when that distinction matters.
