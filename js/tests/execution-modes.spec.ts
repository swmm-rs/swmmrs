import { readFileSync } from "node:fs";
import { expect, test } from "@playwright/test";

const input = readFileSync(new URL("../../python/tests/data/rain_subcatch.inp", import.meta.url), "utf8");
const ordinaryOrigin = "http://127.0.0.1:8087";

test("ordinary pages run, control, and rerun the serial solver without shared memory", async ({ page }) => {
  const assets: string[] = [];
  page.on("request", request => { if (request.url().includes("/dist/")) assets.push(request.url()); });
  const response = await page.goto(ordinaryOrigin);
  expect(response?.headers()["cross-origin-embedder-policy"]).toBeUndefined();
  const result = await page.evaluate(async (input) => {
    const { Simulation } = await import("/index.js");
    const isolated = crossOriginIsolated;
    const sharedAvailable = typeof SharedArrayBuffer !== "undefined";
    const simulation = await Simulation.open(input);
    try {
      const info = await simulation.info();
      await simulation.nodes.get("J1").configure({ fullDepth: 7 });
      await simulation.start();
      await simulation.nodes.get("J1").setExternalInflow(2);
      await simulation.stride(60, true);
      const snapshot = await simulation.nodes.snapshot(["J1"]);
      await simulation.finish();
      const first = await simulation.run();
      const second = await simulation.run();
      return {
        isolated, sharedAvailable, threads: info.effectiveThreads,
        depth: snapshot.depth[0], inflow: snapshot.lateralInflow[0],
        outputBytes: first.output.length,
        identical: first.output.every((value, index) => value === second.output[index]),
        report: second.report.includes("Flow Routing Continuity"),
      };
    } finally { await simulation.close(); await simulation.close(); }
  }, input);
  expect(result).toMatchObject({ isolated: false, sharedAvailable: false, threads: 1, identical: true, report: true });
  expect(result.inflow).toBeGreaterThanOrEqual(2);
  expect(Number.isFinite(result.depth)).toBe(true);
  expect(result.outputBytes).toBeGreaterThan(100);
  expect(assets.some(url => url.endsWith("/dist/serial/swmmrs_bg.wasm"))).toBe(true);
  expect(assets.every(url => url.includes("/dist/serial/"))).toBe(true);
});

test("ordinary pages reject explicit parallel requests before loading WASM", async ({ page }) => {
  await page.addInitScript(() => Object.defineProperty(navigator, "hardwareConcurrency", { value: 1 }));
  const assets: string[] = [];
  page.on("request", request => { if (request.url().includes("/dist/")) assets.push(request.url()); });
  await page.goto(ordinaryOrigin);
  const result = await page.evaluate(async (input) => {
    const { Simulation, WorkerError } = await import("/index.js");
    try { const simulation = await Simulation.open(input, {}, { threads: 2 }); await simulation.close(); }
    catch (error) { return { typed: error instanceof WorkerError, message: error.message, operation: error.operation }; }
    return null;
  }, input);
  expect(result).toMatchObject({ typed: true, operation: "open" });
  expect(result?.message).toMatch(/multiple threads|multithread|parallel/i);
  expect(assets).toEqual([]);
});

test("explicit one-thread execution loads only the serial build on isolated pages", async ({ page }) => {
  const assets: string[] = [];
  page.on("request", request => { if (request.url().includes("/dist/")) assets.push(request.url()); });
  await page.goto("/");
  const result = await page.evaluate(async (input) => {
    const { runSwmm } = await import("/index.js");
    const result = await runSwmm(input, {}, { threads: 1 });
    return { isolated: crossOriginIsolated, bytes: result.output.length };
  }, input);
  expect(result.isolated).toBe(true);
  expect(result.bytes).toBeGreaterThan(100);
  expect(assets.some(url => url.endsWith("/dist/serial/swmmrs_bg.wasm"))).toBe(true);
  expect(assets.every(url => url.includes("/dist/serial/"))).toBe(true);
});

test("serial execution clamps an INP thread hint to its one available lane", async ({ page }) => {
  await page.goto(ordinaryOrigin);
  const result = await page.evaluate(async (input) => {
    const { Simulation } = await import("/index.js");
    const simulation = await Simulation.open(input);
    try {
      const threads = (await simulation.info()).effectiveThreads;
      const output = await simulation.run();
      return { threads, bytes: output.output.length };
    } finally { await simulation.close(); }
  }, input.replace("[OPTIONS]", "[OPTIONS]\nTHREADS 4"));
  expect(result.threads).toBe(1);
  expect(result.bytes).toBeGreaterThan(100);
});

test("ordinary pages can run independent serial scenarios concurrently", async ({ page }) => {
  await page.goto(ordinaryOrigin);
  const result = await page.evaluate(async (input) => {
    const { Simulation } = await import("/index.js");
    const left = await Simulation.open(input);
    try {
      const right = await Simulation.open(input);
      try {
        await left.nodes.get("J1").configure({ fullDepth: 9 });
        const [a, b] = await Promise.all([left.run(), right.run()]);
        await left.close();
        return {
          rightDepth: (await right.nodes.get("J1").configuration()).fullDepth,
          rightState: await right.getState(),
          sizes: [a.output.length, b.output.length],
        };
      } finally { await right.close(); }
    } finally { await left.close(); }
  }, input);
  expect(result.rightDepth).toBe(5);
  expect(result.rightState).toBe("ended");
  expect(result.sizes.every(bytes => bytes > 100)).toBe(true);
});

test("the serial binary has unshared memory and rejects a parallel pool", async ({ page }) => {
  await page.goto(ordinaryOrigin);
  const result = await page.evaluate(async () => {
    const bindings = await import("/dist/serial/swmmrs.js");
    const wasm = await bindings.default();
    let rejected = false;
    try { await bindings.initThreadPool(2, "unused"); } catch { rejected = true; }
    await bindings.initThreadPool(1, "unused");
    bindings.terminateThreadPool();
    bindings.terminateThreadPool();
    return { unshared: wasm.memory.buffer instanceof ArrayBuffer, rejected };
  });
  expect(result).toEqual({ unshared: true, rejected: true });
});

test("isolated pages automatically select the threaded build when CPUs are available", async ({ page }) => {
  await page.goto("/");
  test.skip(await page.evaluate(() => navigator.hardwareConcurrency < 2), "Needs two browser CPU lanes");
  const assets: string[] = [];
  page.on("request", request => { if (request.url().includes("/dist/")) assets.push(request.url()); });
  const bytes = await page.evaluate(async (input) => {
    const { runSwmm } = await import("/index.js");
    return (await runSwmm(input)).output.length;
  }, input);
  expect(bytes).toBeGreaterThan(100);
  expect(assets.some(url => url.endsWith("/dist/swmmrs_bg.wasm"))).toBe(true);
  expect(assets.some(url => url.includes("/dist/snippets/"))).toBe(true);
  expect(assets.every(url => !url.includes("/dist/serial/"))).toBe(true);
});
