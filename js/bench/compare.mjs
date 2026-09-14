import { spawn, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createServer } from "node:http";
import { appendFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { availableParallelism, cpus, release, totalmem } from "node:os";
import { dirname, join, resolve } from "node:path";
import { parseArgs } from "node:util";
import { fileURLToPath } from "node:url";
import { chromium } from "@playwright/test";

const packageRoot = fileURLToPath(new URL("../", import.meta.url));
const { values } = parseArgs({ options: {
  input: { type: "string" }, output: { type: "string" },
  threads: { type: "string", default: "1,2,4,8" },
  repeats: { type: "string", default: "4" },
  native: { type: "string", default: join(packageRoot, "target/native-bench/release/swmmrs-native-bench") },
  "wasm-dist": { type: "string", default: join(packageRoot, "dist") },
  chromium: { type: "string" },
  engine: { type: "string", default: "both" },
  help: { type: "boolean" },
} });

if (values.help) {
  console.log(`Usage: node bench/compare.mjs --input MODEL.inp --output NEW_DIRECTORY [options]

Runs matched native and browser batch lifecycles, saving raw timing and output files.
The first run on each owner is reported separately; subsequent runs reuse it.
The original INP is never modified. Only self-contained models are supported.

Options:
  --threads LIST      Requested thread counts, default 1,2,4,8
  --repeats N         Total runs per owner including the first run, default 4
  --engine NAME       both, native, or wasm; default both
  --native PATH       Release native benchmark executable
  --wasm-dist PATH    Built WASM directory, default js/dist
  --chromium PATH     Chromium executable; also accepts CHROMIUM_PATH

Example, from js/ after building both artifacts:
  node bench/compare.mjs --input /path/to/model.inp --output /tmp/swmm-bench-run --threads 1 --repeats 4`);
  process.exit(0);
}
if (!values.input || !values.output) throw new Error("--input and --output are required; run with --help for an example");
const repeats = Number(values.repeats);
const threadCounts = values.threads.split(",").map(Number);
if (!Number.isInteger(repeats) || repeats < 1) throw new Error("--repeats must be a positive integer");
if (!threadCounts.length || threadCounts.some(n => !Number.isInteger(n) || n < 1 || n > availableParallelism())) {
  throw new Error(`--threads must contain integers from 1 to ${availableParallelism()}`);
}
if (new Set(threadCounts).size !== threadCounts.length) throw new Error("--threads must not contain duplicates");
if (!["both", "native", "wasm"].includes(values.engine)) throw new Error("--engine must be both, native, or wasm");
const outputRoot = resolve(values.output);
if (existsSync(outputRoot)) throw new Error(`Output directory already exists: ${outputRoot}; choose a new directory`);
const nativeBinary = resolve(values.native);
const wasmDist = resolve(values["wasm-dist"]);
const original = readFileSync(resolve(values.input));
const input = original.toString("utf8");
let section = "";
for (const line of input.split(/\r?\n/)) {
  const text = line.replace(/;.*/, "").trim();
  if (text.startsWith("[")) { section = text.toUpperCase(); continue; }
  if (text && (section === "[FILES]" || /\bFILE\b/i.test(text))) {
    throw new Error("This benchmark supports self-contained models only; resolve external file references first");
  }
}
const hash = bytes => createHash("sha256").update(bytes).digest("hex");
const artifacts = {};
if (values.engine !== "wasm") artifacts.nativeSha256 = hash(readFileSync(nativeBinary));
if (values.engine !== "native") {
  artifacts.wasmBuilds = {
    serial: hash(readFileSync(join(wasmDist, "serial/swmmrs_bg.wasm"))),
    threaded: hash(readFileSync(join(wasmDist, "swmmrs_bg.wasm"))),
  };
}
mkdirSync(outputRoot, { recursive: true, mode: 0o700 });
const records = [];
const record = value => {
  const item = { recordedAt: new Date().toISOString(), ...value };
  records.push(item);
  appendFileSync(join(outputRoot, "timings.jsonl"), JSON.stringify(item) + "\n");
  console.log(JSON.stringify(item));
};
writeFileSync(join(outputRoot, "environment.json"), JSON.stringify({
  capturedAt: new Date().toISOString(), inputSha256: hash(original), inputBytes: original.length,
  threadCounts, repeats, engine: values.engine, ...artifacts,
  cpu: cpus()[0].model, availableParallelism: availableParallelism(), logicalCpus: cpus().length,
  totalMemoryBytes: totalmem(), kernel: release(), node: process.version,
  toolchain: execFileSync("rustc", ["--version"], { cwd: packageRoot, encoding: "utf8" }).trim(),
}, null, 2));

function withThreads(source, threads) {
  let section = "", found = false, options = false;
  const lines = source.split(/\r?\n/).map(line => {
    const text = line.replace(/;.*/, "").trim();
    if (text.startsWith("[")) section = text.toUpperCase();
    if (section === "[OPTIONS]") options = true;
    if (section === "[OPTIONS]" && /^THREADS\s/i.test(text)) {
      if (found) throw new Error("Duplicate THREADS options");
      found = true;
      return `THREADS              ${threads}`;
    }
    return line;
  });
  if (!options) throw new Error("Model has no [OPTIONS] section");
  if (!found) lines.splice(lines.findIndex(line => line.trim().toUpperCase() === "[OPTIONS]") + 1, 0, `THREADS ${threads}`);
  return lines.join("\n");
}

async function runNative(threads, path) {
  const start = performance.now();
  const child = spawn(nativeBinary, [path, join(outputRoot, `native-t${threads}`), String(repeats)], {
    cwd: outputRoot, stdio: ["ignore", "pipe", "pipe"],
  });
  let pending = "";
  child.stdout.setEncoding("utf8");
  child.stdout.on("data", chunk => {
    pending += chunk;
    let newline;
    while ((newline = pending.indexOf("\n")) !== -1) {
      const line = pending.slice(0, newline); pending = pending.slice(newline + 1);
      if (!line) continue;
      const event = JSON.parse(line);
      record({ engine: "native", threads, ...event,
        ...(event.event === "open" ? { launchToOpenMs: performance.now() - start } : {}),
      });
    }
  });
  child.stderr.on("data", chunk => process.stderr.write(chunk));
  await new Promise((resolve, reject) => {
    child.on("error", reject);
    child.on("close", (code, signal) => code === 0 ? resolve() : reject(new Error(`Native benchmark failed: ${code ?? signal}`)));
  });
}

let currentCapture;
const headers = {
  "Cross-Origin-Opener-Policy": "same-origin",
  "Cross-Origin-Embedder-Policy": "require-corp",
  "Cross-Origin-Resource-Policy": "same-origin",
  "Cache-Control": "no-store",
};
const server = createServer(async (request, response) => {
  try {
    const url = new URL(request.url, "http://localhost");
    if (request.method === "POST" && url.pathname === "/capture" && currentCapture) {
      const name = url.searchParams.get("name");
      if (!/^run-\d+\.(out|rpt)$/.test(name ?? "")) throw new Error("Invalid artifact name");
      const chunks = [];
      for await (const chunk of request) chunks.push(chunk);
      writeFileSync(join(currentCapture, name), Buffer.concat(chunks));
      response.writeHead(204, headers).end(); return;
    }
    if (url.pathname === "/") {
      response.writeHead(200, { ...headers, "Content-Type": "text/html" }).end("<!doctype html><title>SWMM benchmark</title>"); return;
    }
    if (url.pathname === "/favicon.ico") { response.writeHead(204, headers).end(); return; }
    const path = decodeURIComponent(url.pathname).slice(1);
    if (path.includes("..") || path.includes("\\") || !/^(index\.js|worker\.js|lib\/.*\.js|dist\/.*\.(js|wasm))$/.test(path)) {
      response.writeHead(404, headers).end(); return;
    }
    const file = path.startsWith("dist/") ? join(wasmDist, path.slice(5)) : join(packageRoot, path);
    response.writeHead(200, { ...headers, "Content-Type": path.endsWith(".wasm") ? "application/wasm" : "text/javascript" });
    response.end(await readFile(file));
  } catch (error) {
    response.writeHead(500, headers).end(String(error));
  }
});
let browser;

async function runWasm(threads, model) {
  const page = await browser.newPage();
  page.on("pageerror", error => console.error(error));
  await page.goto(`http://127.0.0.1:${server.address().port}/`);
  await page.exposeFunction("recordBenchmark", event => record({
    engine: "wasm", threads, wasmBuild: threads === 1 ? "serial" : "threaded", ...event,
  }));
  currentCapture = join(outputRoot, `wasm-t${threads}`);
  mkdirSync(currentCapture);
  try {
    await page.evaluate(async ({ model, threads, repeats }) => {
      const importStart = performance.now();
      const { Simulation } = await import("/index.js");
      const importMs = performance.now() - importStart;
      const start = performance.now();
      const simulation = await Simulation.open(model, {}, { threads });
      const openMs = performance.now() - start;
      const info = await simulation.info();
      await window.recordBenchmark({ event: "open", openMs, importMs, effectiveThreads: info.effectiveThreads,
        hardwareConcurrency: navigator.hardwareConcurrency, crossOriginIsolated });
      try {
        for (let run = 0; run < repeats; run++) {
          const start = performance.now();
          const result = await simulation.run();
          const runMs = performance.now() - start;
          const status = await simulation.status();
          await window.recordBenchmark({ event: "run", run, runMs,
            stepCount: status.stepCount, reportPeriodCount: status.reportPeriodCount,
            warningCount: status.warningCount, outputBytes: result.output.length,
            reportBytes: new TextEncoder().encode(result.report).length,
          });
          // Save artifacts outside the measured interval. Never serialize result arrays into JSON.
          for (const [extension, body] of [["out", result.output], ["rpt", result.report]]) {
            const response = await fetch(`/capture?name=run-${run}.${extension}`, { method: "POST", body });
            if (!response.ok) throw new Error("Could not save benchmark output");
          }
        }
      } finally {
        const start = performance.now();
        await simulation.close();
        await window.recordBenchmark({ event: "close", closeMs: performance.now() - start });
      }
    }, { model, threads, repeats });
  } finally { await page.close(); }
}

function compareOutput(reference, candidate) {
  const left = readFileSync(reference), right = readFileSync(candidate);
  const trailer = bytes => {
    if (bytes.length < 52 || bytes.readInt32LE(0) !== 516114522 || bytes.readInt32LE(bytes.length - 4) !== 516114522) {
      throw new Error("Invalid SWMM output magic");
    }
    const start = bytes.readInt32LE(bytes.length - 16);
    const periods = bytes.readInt32LE(bytes.length - 12);
    const code = bytes.readInt32LE(bytes.length - 8);
    const width = (bytes.length - 24 - start) / periods;
    if (code || start < 28 || periods < 1 || !Number.isInteger(width) || width < 8 || width % 4) throw new Error("Invalid finalized SWMM output");
    return { start, periods, width };
  };
  const a = trailer(left), b = trailer(right);
  if (a.periods !== b.periods || a.width !== b.width || !left.subarray(0, a.start).equals(right.subarray(0, b.start))) {
    throw new Error("Output metadata or report layout differs between compared runs");
  }
  let differentValues = 0, maxAbsoluteDifference = 0, maxScaledDifference = 0;
  for (let period = 0; period < a.periods; period++) {
    const x = a.start + period * a.width, y = b.start + period * b.width;
    if (left.readDoubleLE(x) !== right.readDoubleLE(y)) throw new Error("Report timestamps differ");
    for (let offset = 8; offset < a.width; offset += 4) {
      const l = left.readFloatLE(x + offset), r = right.readFloatLE(y + offset);
      if (!Number.isFinite(l) || !Number.isFinite(r)) throw new Error("Non-finite output result");
      if (l !== r) {
        differentValues++;
        maxAbsoluteDifference = Math.max(maxAbsoluteDifference, Math.abs(l - r));
        maxScaledDifference = Math.max(maxScaledDifference, Math.abs(l - r) / Math.max(1, Math.abs(l), Math.abs(r)));
      }
    }
  }
  return { referenceSha256: hash(left), candidateSha256: hash(right), byteIdentical: left.equals(right),
    periods: a.periods, comparedValues: a.periods * (a.width - 8) / 4,
    differentValues, maxAbsoluteDifference, maxScaledDifference };
}

let active = "setup";
const heartbeat = setInterval(() => console.error(`Benchmark active: ${active}`), 30000);
try {
  if (values.engine !== "native") {
    await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
    browser = await chromium.launch({ headless: true,
      executablePath: values.chromium ?? process.env.CHROMIUM_PATH,
      args: ["--disable-background-timer-throttling", "--disable-renderer-backgrounding"],
    });
    record({ event: "browser", version: browser.version() });
  }
  for (const [index, threads] of threadCounts.entries()) {
    const model = withThreads(input, threads);
    const path = join(outputRoot, `threads-${threads}.inp`);
    writeFileSync(path, model, { mode: 0o600 });
    const order = values.engine === "both" ? (index % 2 ? ["wasm", "native"] : ["native", "wasm"]) : [values.engine];
    for (const engine of order) {
      active = `${engine}, ${threads} thread(s)`;
      if (engine === "native") await runNative(threads, path);
      else await runWasm(threads, model);
    }
    if (values.engine === "both") {
      record({ event: "comparison", threads, ...compareOutput(
        join(outputRoot, `native-t${threads}`, "model.out"),
        join(outputRoot, `wasm-t${threads}`, `run-${repeats - 1}.out`),
      ) });
    }
  }
  writeFileSync(join(outputRoot, "results.json"), JSON.stringify(records, null, 2));
} finally {
  clearInterval(heartbeat);
  await browser?.close();
  if (server.listening) await new Promise(resolve => server.close(resolve));
}
