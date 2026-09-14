import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { Simulation, OutputReader, OutputError, OutputName, ValidationError } from "../index.js";

const quality = await readFile(new URL("../../python/tests/data/quality.inp", import.meta.url), "utf8");

test("strict stepping, detailed reports, and byte-backed output queries agree", async () => {
  const simulation = await Simulation.open(quality, {}, { threads: 1 });
  try {
    await simulation.start();
    assert.equal(await simulation.stride(61), "2020-01-01T00:01:01");
    const { report, output } = await simulation.finish();
    assert.match(report, /<<< Node J1 >>>/);
    const reader = await OutputReader.open(output);
    try {
      assert.equal(reader.isFinalized, true);
      assert.deepEqual(reader.times, ["2020-01-01T00:01:00"]);
      const selection = { elementType: "node", element: "J1", attribute: "invert_depth" };
      const duplicate = await reader.readBulkSeries([selection, selection]);
      const narrow = await reader.readBulkSeries([selection, selection], { lowMemory: true });
      assert.deepEqual(duplicate, narrow);
      assert.deepEqual(duplicate.series[0].values, duplicate.series[1].values);
      assert.deepEqual((await reader.nodeSeries("J1", "invert_depth")).values, duplicate.series[0].values);
      await assert.rejects(async () => reader.nodeSeries("J1", "invert_depth", { start: "2020-02-31T00:00:00" }));
      await assert.rejects(() => OutputReader.open(output.subarray(0, 8)), OutputError);
    } finally { await reader.close(); }
    const repeated = await simulation.finish();
    assert.equal(repeated.report, report);
    await simulation.start({ saveResults: false });
    await simulation.step();
    assert.equal((await simulation.finish()).output.byteLength, 0);
  } finally { await simulation.close(); }
});

test("nested patches reject unknown fields atomically and quality overrides expire", async () => {
  const simulation = await Simulation.open(quality, {}, { threads: 1 });
  try {
    const link = simulation.links.get("C1");
    const before = await link.configuration();
    await assert.rejects(() => link.configure({ subtype: { kind: "conduit", length: 200, misspelled: 1 } }), ValidationError);
    assert.deepEqual(await link.configuration(), before);
    for (const patch of [
      { initialFlow: null },
      { subtype: { kind: "conduit", initialSetting: 0.5 } },
      { crossSection: { kind: "circular", diameter: 1, geometry: [1, 2, 3, 4] } },
    ]) {
      await assert.rejects(() => link.configure({ tag: "must-not-apply", ...patch }), ValidationError);
      assert.deepEqual(await link.configuration(), before);
    }
    const outfall = simulation.nodes.get("O1");
    const originalOutfall = await outfall.configuration();
    await assert.rejects(() => outfall.configure({ tag: "must-not-apply", boundary: { kind: "free", stage: 1 } }), ValidationError);
    assert.deepEqual(await outfall.configuration(), originalOutfall);
    await link.configure({ subtype: { kind: "conduit", length: 200 } });
    assert.equal((await link.configuration()).subtype.length, 200);
    const node = simulation.nodes.get("J1");
    await node.updateExternalPollutantMassFlux({ TSS: 2.5, Count: 125 });
    await assert.rejects(() => node.updateExternalPollutantMassFlux({ TSS: 8, tss: 9 }), ValidationError);
    assert.deepEqual(await node.externalPollutantMassFlux(), { TSS: 2.5, Count: 125 });
    await node.clearExternalPollutantMassFlux();
    await simulation.start();
    await node.overridePollutantConcentrations({ TSS: 123, Count: 456 });
    await simulation.step();
    assert.deepEqual((await node.quality()).concentrations, [123, 456]);
    const snapshot = await simulation.nodes.qualitySnapshot(["O1", "J1"]);
    assert.deepEqual(snapshot.objectIds, ["O1", "J1"]);
    assert.deepEqual(snapshot.pollutantIds, ["TSS", "Count"]);
    assert.equal(snapshot.concentrations[0][1], 123);
    await simulation.step();
    assert.notDeepEqual((await node.quality()).concentrations, [123, 456]);
    assert.equal(snapshot.concentrations[0][1], 123);
    const statistics = await simulation.nodes.statisticsSnapshot(["J1"]);
    assert.equal(statistics.maximumDepth[0], (await node.statistics()).maximumDepth);
  } finally { await simulation.close(); }
});

test("checkpoints preserve continuation and forks do not share controls", async () => {
  const source = await Simulation.open(quality, {}, { threads: 1 });
  let resumed;
  let fork;
  try {
    await source.start();
    await source.stride(60);
    const saved = await source.saveCheckpoint();
    const boundary = await source.nodes.snapshot();
    resumed = await Simulation.resume(saved, { threads: 1 });
    assert.deepEqual(await resumed.nodes.snapshot(), boundary);
    await source.step();
    await resumed.step();
    assert.deepEqual(await resumed.nodes.snapshot(), await source.nodes.snapshot());
    fork = await source.fork({ threads: 1 });
    await source.nodes.get("J1").setExternalInflow(5);
    assert.equal(await fork.nodes.get("J1").externalInflow(), 0);
    assert.equal(await source.nodes.get("J1").externalInflow(), 5);
    const corrupt = { ...saved, manifest: saved.manifest.slice() };
    corrupt.manifest[0] = 0;
    await assert.rejects(() => Simulation.resume(corrupt, { threads: 1 }));
    assert.equal(await source.getState(), "running");
    await source.close();
    assert.equal(await resumed.step(), "2020-01-01T00:02:00");
  } finally {
    await fork?.close();
    await resumed?.close();
    await source.close();
  }
});

test("tagged configuration records round-trip through their public field names", async () => {
  const input = await readFile(new URL("../../crates/solver/tests/data/amm-us.inp", import.meta.url), "utf8");
  const amm = await Simulation.open(input, {}, { threads: 1 });
  try {
    const model = amm.ammModels.get("M1");
    const original = await model.configuration();
    assert.equal(original.components[0].dryCapture, 0.1);
    const components = original.components.map(component =>
      component.kind === "standard" ? { ...component, hotShcf: 0.06 } : component);
    await model.configure({ components });
    assert.deepEqual((await model.configuration()).components, components);
    const conduit = amm.links.get("C1");
    await conduit.configure({ crossSection: { kind: "standard", shape: "rect_closed", geometry: [2, 2, 0, 0], culvertCode: 0 } });
    assert.equal((await conduit.configuration()).crossSection.culvertCode, 0);
  } finally { await amm.close(); }
  const regulators = await readFile(new URL("../../python/tests/data/regulator_configurations.inp", import.meta.url), "utf8");
  const simulation = await Simulation.open(regulators, {}, { threads: 1 });
  try {
    const pump = simulation.links.get("Pump-A");
    await pump.configure({ subtype: { kind: "pump", initialSetting: 0.5 } });
    assert.equal((await pump.configuration()).subtype.initialSetting, 0.5);
    assert.equal((await simulation.links.get("Outlet-A").configuration()).subtype.rating.headBasis, "depth");
  } finally { await simulation.close(); }
});

test("RTK definitions remain accessible after checkpoint continuation", async () => {
  const input = (await readFile(new URL("../../crates/run/tests/data/regression-suite/save_interfaces/save_rdii.inp", import.meta.url), "utf8"))
    .replace('SAVE RDII "rdii-interface.bin"', "");
  const source = await Simulation.open(input, {}, { threads: 1 });
  let fork;
  try {
    const configuration = await source.unitHydrographs.get("UH1").configuration();
    await source.start({ saveResults: false });
    await source.stride(60);
    fork = await source.fork({ threads: 1 });
    assert.deepEqual(await fork.unitHydrographs.get("uh1").configuration(), configuration);
    await source.stride(300);
    await fork.stride(300);
    assert.deepEqual(await fork.nodes.snapshot(), await source.nodes.snapshot());
  } finally {
    await fork?.close();
    await source.close();
  }
});

test("JavaScript boundaries reject coercion and preserve declared zero layers", async () => {
  assert.equal(new OutputName([0xef, 0xbb, 0xbf, 0x41]).text, "\ufeffA");
  assert.equal(new OutputName([0xff]).text, null);

  const simulation = await Simulation.open(quality, {}, { threads: 1 });
  try {
    const node = simulation.nodes.get("J1");
    const link = simulation.links.get("C1");
    const outfall = simulation.nodes.get("O1");
    const subcatchment = simulation.subcatchments.get("S-A");
    const rainGage = simulation.rainGages.at(0);
    const before = {
      inflow: await node.externalInflow(),
      scaleFactors: await subcatchment.precipitationScaleFactors(),
      forcing: await subcatchment.externalForcing(),
      externalPrecipitation: await rainGage.externalPrecipitationRate(),
    };
    const invalidCalls = [
      () => node.setExternalInflow("2.5"),
      () => node.setExternalInflow(true),
      () => node.setExternalInflow(null),
      () => link.setTargetSetting("0.5"),
      () => link.setFlowLimit(false),
      () => outfall.setOutfallStage(null),
      () => subcatchment.setPrecipitationScaleFactors("1", 1),
      () => subcatchment.setPrecipitationScaleFactors(1, true),
      () => subcatchment.setExternalRainfall("0.1"),
      () => subcatchment.setExternalSnowfall(false),
      () => rainGage.useExternalPrecipitation("0.1"),
      () => rainGage.setPrecipitation(true),
      () => rainGage.setRainfallOverride("0.1"),
    ];
    for (const call of invalidCalls) await assert.rejects(call, ValidationError);
    assert.equal(await node.externalInflow(), before.inflow);
    assert.deepEqual(await subcatchment.precipitationScaleFactors(), before.scaleFactors);
    assert.deepEqual(await subcatchment.externalForcing(), before.forcing);
    assert.equal(await rainGage.externalPrecipitationRate(), before.externalPrecipitation);
    await rainGage.setRainfallOverride(null);
    assert.equal(await rainGage.rainfallOverride(), null);
  } finally { await simulation.close(); }

  const lidsInput = (await readFile(new URL("../../python/tests/data/lids.inp", import.meta.url), "utf8"))
    .replace("PP      SOIL        12", "PP      SOIL        0");
  const lids = await Simulation.open(lidsInput, {}, { threads: 1 });
  try {
    const control = lids.lidControls.get("PP");
    assert.equal((await control.configuration()).soil.thickness, 0);
    await control.configure("soil", { thickness: 1 });
    assert.equal((await control.configuration()).soil.thickness, 1);
  } finally { await lids.close(); }
});

test("finalizeReport appends the runtime footer without detailed report tables", async () => {
  const simulation = await Simulation.open(quality, {}, { threads: 1 });
  try {
    await simulation.start();
    await simulation.stride(61);
    await simulation.end();
    await simulation.finalizeReport();
    const report = new TextDecoder().decode(await simulation.readFile("model.rpt"));
    assert.match(report, /Analysis begun on:/);
    assert.match(report, /Analysis ended on:/);
    assert.match(report, /Total elapsed time:/);
    assert.doesNotMatch(report, /<<< Node J1 >>>/);
    await simulation.finalizeReport();
    assert.equal(new TextDecoder().decode(await simulation.readFile("model.rpt")), report);
  } finally { await simulation.close(); }
});

test("end exposes finalized binary output before generating detailed report tables", async () => {
  const simulation = await Simulation.open(quality, {}, { threads: 1 });
  try {
    await simulation.start();
    await simulation.stride(61);
    await simulation.end();
    const output = await simulation.readFile("model.out");
    const reader = await OutputReader.open(output);
    try {
      assert.equal(reader.isFinalized, true);
      assert.equal(reader.times.length, 1);
    } finally { await reader.close(); }
    const before = new TextDecoder().decode(await simulation.readFile("model.rpt"));
    assert.match(before, /Flow Routing Continuity/);
    assert.match(before, /Node Depth Summary/);
    assert.doesNotMatch(before, /<<< Node J1 >>>/);
    await simulation.report();
    const report = new TextDecoder().decode(await simulation.readFile("model.rpt"));
    assert.match(report, /<<< Node J1 >>>/);
    assert.match(report, /Analysis ended/);
    await simulation.report();
    assert.equal(new TextDecoder().decode(await simulation.readFile("model.rpt")), report);
  } finally { await simulation.close(); }
});
