import { Simulation, runSwmm, LifecycleError, type Node, type NodeCollection, type SimulationOptionsView, type NodeResults, type NodeSnapshot, type SimulationStatistics } from "../../index.js";

// @ts-expect-error Live objects can only be obtained from their simulation.
new Node();
// @ts-expect-error Collection construction is private.
new NodeCollection();
// @ts-expect-error RPC factories are not part of the public declaration.
Node.create;
// @ts-expect-error Options belong to an existing owner.
new SimulationOptionsView();

async function consumer(input: Blob): Promise<void> {
  const simulation = await Simulation.open(input, {}, { threads: 1 });
  try {
    const node = simulation.nodes.get("J1");
    await node.configure({ fullDepth: 4, includedInReport: true });
    await simulation.options.update({ reportStepSeconds: 10 });
    for await (const time of simulation.steps({ seconds: 10, strict: true })) {
      const value: NodeResults = await node.results();
      const snapshot: NodeSnapshot = await simulation.nodes.snapshot([node.id]);
      console.log(time, value.depth, snapshot.depth[0]);
    }
    const totals: SimulationStatistics = await simulation.statistics();
    console.log(totals.routingTotals.continuityError);
    const results = await simulation.finish();
    await simulation.finalizeReport();
    const output: Uint8Array = results.output;
    console.log(output.byteLength);

    // @ts-expect-error Live object results require an explicit async acquisition.
    console.log(node.depth);
    // @ts-expect-error Unsupported configuration fields must not compile.
    await node.configure({ roughness: 0.02 });
    // @ts-expect-error Runtime controls are separate from stable declarations.
    await simulation.links.get("C1").configure({ targetSetting: 1 });
    // @ts-expect-error Option values are concrete types.
    await simulation.options.update({ allowPonding: "yes" });
    // @ts-expect-error Nullable rainfall override is numeric, not text.
    await simulation.rainGages.get("RG").setRainfallOverride("rain");
    // @ts-expect-error Snapshot columns are immutable.
    (await simulation.nodes.snapshot()).depth.push(1);
  } catch (error) {
    if (error instanceof LifecycleError) console.log(error.operation);
    throw error;
  } finally { await simulation.close(); }
  await runSwmm(input);
}
void consumer;
