import { readFileSync } from "node:fs";
import { expect, test } from "@playwright/test";

// Share the same hydrology fixture as the Python binding tests.
const input = readFileSync(new URL("../../python/tests/data/rain_subcatch.inp", import.meta.url), "utf8");

test.beforeEach(async ({ page }) => { await page.goto("/"); });

test("collections, topology, typed configuration and atomic validation", async ({ page }) => {
  const result = await page.evaluate(async (input) => {
    const { Simulation, ObjectNotFoundError, ValidationError } = await import("/index.js");
    const sim = await Simulation.open(input, {}, { threads: 1 });
    try {
      const node = sim.nodes.get("j1");
      const same = node === sim.nodes.at(0) && node === sim.nodes.get("J1");
      let missing = false;
      try { sim.nodes.get("absent"); } catch (error) { missing = error instanceof ObjectNotFoundError; }
      await node.configure({ fullDepth: 7, tag: "browser" });
      let invalid = false;
      try { await node.configure({ tag: "must not apply", fullDepth: -1 }); }
      catch (error) { invalid = error instanceof ValidationError && error.operation === "configureNode"; }
      let unknown = false;
      try { await node.configure({ misspelledDepth: 4 }); }
      catch (error) { unknown = error instanceof ValidationError; }
      await sim.links.get("Pipe").configure({ inletLossCoefficient: 0.5 });
      await sim.subcatchments.get("Sub-Node").configure({ area: 2, width: 120 });
      await sim.options.update({ reportStepSeconds: 30, allowPonding: true });
      return {
        same, missing, invalid, unknown,
        ids: [...sim.nodes].map((node) => node.id),
        includes: sim.nodes.has("j1"),
        info: await sim.info(),
        node: await node.configuration(),
        link: await sim.links.get("Pipe").configuration(),
        subcatchment: await sim.subcatchments.get("Sub-Node").configuration(),
        downstream: await sim.subcatchments.get("Sub-To-Sub").configuration(),
        options: await sim.options.read(),
      };
    } finally { await sim.close(); }
  }, input);
  expect(result).toMatchObject({ same: true, missing: true, invalid: true, unknown: true, includes: true });
  expect(result.ids).toEqual(["J1", "Out"]);
  expect(result.info).toMatchObject({ flowUnits: "Cfs", rainGageIds: ["Gage-A", "Gage-B"], startTime: "2020-01-01T00:00:00" });
  expect(result.node).toMatchObject({ kind: "Junction", tag: "browser", fullDepth: 7 });
  expect(result.link).toMatchObject({ kind: "Conduit", inletNode: "J1", outletNode: "Out", inletLossCoefficient: 0.5 });
  expect(result.subcatchment).toMatchObject({ area: 2, width: 120, rainGage: "Gage-A", outlet: { kind: "node", id: "J1" } });
  expect(result.downstream.outlet).toEqual({ kind: "subcatchment", id: "Sub-Node" });
  expect(result.options).toMatchObject({ reportStepSeconds: 30, allowPonding: true });
});

test("async iteration, rainfall forcing, snapshots and final statistics", async ({ page }) => {
  const result = await page.evaluate(async (input) => {
    const { Simulation, LifecycleError } = await import("/index.js");
    const sim = await Simulation.open(input, {}, { threads: 1 });
    try {
      const gage = sim.rainGages.get("Gage-A");
      await gage.setPrecipitation(0.5);
      await gage.setRainfallOverride(1);
      const before = await sim.status();
      const times: string[] = [];
      let first;
      let blocked = false;
      for await (const time of sim.steps({ seconds: 60, strict: true })) {
        times.push(time);
        if (times.length === 1) {
          first = await sim.subcatchments.snapshot(["Sub-To-Sub", "Sub-Node"]);
          try { await sim.step(); } catch (error) { blocked = error instanceof LifecycleError; }
          await gage.setRainfallOverride(null);
          await sim.nodes.get("J1").setExternalInflow(2);
        }
      }
      const status = await sim.status();
      const stats = await sim.statistics();
      const rain = await gage.results();
      const output = await sim.finish();
      return {
        times, blocked, before, status, stats, rain, first,
        frozen: Object.isFrozen(first) && Object.isFrozen(first.rainfall),
        state: await sim.getState(), outputLength: output.output.length,
        report: output.report.includes("Flow Routing Continuity"),
      };
    } finally { await sim.close(); }
  }, input);
  expect(result.blocked).toBe(true);
  expect(result.before).toMatchObject({ state: "open", currentTime: null, percentComplete: 0, durationSeconds: 1200 });
  expect(result.times[0]).toBe("2020-01-01T00:01:00");
  expect(result.times.length).toBeGreaterThan(1);
  expect(result.status).toMatchObject({ state: "complete", percentComplete: 100, elapsedSeconds: 1200 });
  expect(result.stats.routingDiagnostics.stepCount).toBeGreaterThan(0);
  expect(Number.isFinite(result.stats.routingTotals.continuityError)).toBe(true);
  expect(result.rain).toMatchObject({ externalPrecipitationRate: 0.5, rainfallOverride: null });
  expect(result.first.objectIds).toEqual(["Sub-To-Sub", "Sub-Node"]);
  expect(result.frozen).toBe(true);
  expect(result).toMatchObject({ state: "ended", report: true });
  expect(result.outputLength).toBeGreaterThan(100);
});

test("iterator ownership, early return, input validation and close invalidate handles", async ({ page }) => {
  const result = await page.evaluate(async (input) => {
    const { Simulation, LifecycleError } = await import("/index.js");
    const sim = await Simulation.open(input, {}, { threads: 1 });
    const node = sim.nodes.get("J1");
    const iterator = sim.steps({ seconds: 60 });
    let competing = false;
    let finishing = false;
    let invalid = false;
    try {
      await iterator.next();
      try { await sim.steps().next(); } catch (error) { competing = error instanceof LifecycleError; }
      try { await sim.finish(); } catch (error) { finishing = error instanceof LifecycleError; }
      await iterator.return();
      try { await sim.stride(0.25); } catch (error) { invalid = error instanceof RangeError; }
      const next = await sim.step();
      const snapshot = await sim.nodes.snapshot(["Out", "J1"]);
      await sim.close();
      await sim.close();
      let staleRead = false;
      let staleLookup = false;
      try { await node.results(); } catch (error) { staleRead = error instanceof LifecycleError; }
      try { sim.nodes.get("J1"); } catch (error) { staleLookup = error instanceof LifecycleError; }
      return { competing, finishing, invalid, next, snapshot, staleRead, staleLookup, state: await sim.getState() };
    } finally { await iterator.return(); await sim.close(); }
  }, input);
  expect(result).toMatchObject({ competing: true, finishing: true, invalid: true, staleRead: true, staleLookup: true, state: "closed" });
  expect(result.next).toMatch(/^2020-01-01T/);
  expect(result.snapshot.objectIds).toEqual(["Out", "J1"]);
  expect(result.snapshot.depth).toHaveLength(2);
});

test("configuration phases, failed selections, and concurrent reads keep the owner usable", async ({ page }) => {
  const result = await page.evaluate(async (input) => {
    const { Simulation, LifecycleError, SwmmError } = await import("/index.js");
    const sim = await Simulation.open(input, {}, { threads: 1 });
    try {
      await sim.start();
      let phase = false;
      try { await sim.nodes.get("J1").configure({ fullDepth: 9 }); }
      catch (error) { phase = error instanceof LifecycleError; }
      const failed = await Promise.allSettled([
        sim.nodes.snapshot(["J1", "j1"]),
        sim.links.snapshot(["missing"]),
      ]);
      const [node, link, empty] = await Promise.all([
        sim.nodes.get("J1").results(), sim.links.get("Pipe").results(), sim.nodes.snapshot([]),
      ]);
      return {
        phase, failed: failed.every((entry) => entry.status === "rejected" && entry.reason instanceof SwmmError),
        depth: node.depth, flow: link.flow, empty,
      };
    } finally { await sim.close(); }
  }, input);
  expect(result).toMatchObject({ phase: true, failed: true, empty: { objectIds: [], depth: [] } });
  expect(Number.isFinite(result.depth)).toBe(true);
  expect(Number.isFinite(result.flow)).toBe(true);
});

test("pending manual advancement excludes iteration and competing manual calls", async ({ page }) => {
  const result = await page.evaluate(async (input) => {
    const { Simulation, LifecycleError } = await import("/index.js");
    const sim = await Simulation.open(input, {}, { threads: 1 });
    try {
      const running = sim.run();
      const iterator = sim.steps();
      const attempted = await Promise.allSettled([iterator.next(), sim.start()]);
      await running;
      await iterator.return();
      return {
        rejected: attempted.every((entry) => entry.status === "rejected" && entry.reason instanceof LifecycleError),
        state: await sim.getState(),
      };
    } finally { await sim.close(); }
  }, input);
  expect(result).toEqual({ rejected: true, state: "ended" });
});

test("independent scenarios, binary inputs and configured worker URL", async ({ page }) => {
  const result = await page.evaluate(async (input) => {
    const { Simulation } = await import("/index.js");
    const encoded = new TextEncoder().encode(input);
    const [left, right] = await Promise.all([
      Simulation.open(new Blob([encoded]), {}, { threads: 1, workerUrl: new URL("/worker.js", location.href) }),
      Simulation.open(encoded.subarray(0), {}, { threads: 1 }),
    ]);
    try {
      await left.nodes.get("J1").configure({ fullDepth: 8 });
      await right.nodes.get("J1").configure({ fullDepth: 6 });
      const [a, b] = await Promise.all([left.run(), right.run()]);
      return {
        left: (await left.nodes.get("J1").configuration()).fullDepth,
        right: (await right.nodes.get("J1").configuration()).fullDepth,
        outputLengths: [a.output.length, b.output.length],
        sourceRetained: encoded.length > 100,
      };
    } finally { await Promise.all([left.close(), right.close()]); }
  }, input);
  expect(result).toMatchObject({ left: 8, right: 6, sourceRetained: true });
  expect(result.outputLengths.every((length) => length > 100)).toBe(true);
});

test("invalid files and thread counts reject before or during open", async ({ page }) => {
  const result = await page.evaluate(async (input) => {
    const { Simulation, SwmmError } = await import("/index.js");
    const messages = [];
    for (const files of [{ "../outside.dat": "x" }, { "model.inp": "x" }]) {
      try { await Simulation.open(input, files, { threads: 1 }); }
      catch (error) { messages.push(error instanceof SwmmError && error.message); }
    }
    let threads = false;
    try { await Simulation.open(input, {}, { threads: 0 }); } catch (error) { threads = error instanceof RangeError; }
    let solver = false;
    try { await Simulation.open("[OPTIONS]\nFLOW_UNITS INVALID", {}, { threads: 1 }); }
    catch (error) {
      // First-pass option errors have a generic native message and partial report.
      solver = error instanceof SwmmError && error.code === 200 && error.message.includes("ERROR 200") && error.report.includes("SWMMRS");
    }
    let detailedReport = false;
    try { await Simulation.open(input.replace("Pipe J1 Out 100", "Pipe J1 Out invalid"), {}, { threads: 1 }); }
    catch (error) { detailedReport = error instanceof SwmmError && error.report.includes("ERROR") && error.report.includes("invalid"); }
    return { messages, threads, solver, detailedReport };
  }, input);
  expect(result.messages[0]).toContain("invalid project-file path");
  expect(result.messages[1]).toContain("reserved external-file path");
  expect(result).toMatchObject({ threads: true, solver: true, detailedReport: true });
});

test("Node imports and worker execution use the public simulation API", async () => {
  // Exercise the Node module-loading boundary without browser globals.
  const { Simulation } = await import("../index.js");
  const simulation = await Simulation.open(input, {}, { threads: 1 });
  try {
    await simulation.start({ saveResults: false });
    expect(await simulation.stride(61)).toBe("2020-01-01T00:01:01");
  } finally { await simulation.close(); }
});

test("worker initialization errors reject instead of leaving open pending", async ({ page }) => {
  const result = await page.evaluate(async (input) => {
    const { Simulation, WorkerError } = await import("/index.js");
    const url = URL.createObjectURL(new Blob(['throw new Error("worker boot failed")'], { type: "text/javascript" }));
    try {
      await Simulation.open(input, {}, { threads: 1, workerUrl: url });
      return false;
    } catch (error) { return error instanceof WorkerError && error.message.includes("worker boot failed"); }
    finally { URL.revokeObjectURL(url); }
  }, input);
  expect(result).toBe(true);
});

test("dynamic-wave outfall stage and invalid rainfall use native control validation", async ({ page }) => {
  const result = await page.evaluate(async (input) => {
    const { Simulation, SwmmError, ValidationError } = await import("/index.js");
    const sim = await Simulation.open(input, {}, { threads: 1 });
    try {
      const gage = sim.rainGages.get("Gage-A");
      await gage.setPrecipitation(0.5);
      let invalid = false;
      try { await gage.setPrecipitation(NaN); } catch (error) { invalid = error instanceof ValidationError; }
      let wrongKind = false;
      try { await sim.nodes.get("J1").setOutfallStage(2); } catch (error) { wrongKind = error instanceof SwmmError; }
      await sim.nodes.get("Out").setOutfallStage(2);
      await sim.start();
      await sim.stride(60, true);
      return { invalid, wrongKind, rain: await gage.results(), outfall: await sim.nodes.get("Out").results() };
    } finally { await sim.close(); }
  }, input.replace("KINWAVE", "DYNWAVE"));
  expect(result).toMatchObject({ invalid: true, wrongKind: true, rain: { externalPrecipitationRate: 0.5 } });
  expect(result.outfall.depth).toBeCloseTo(2);
});
