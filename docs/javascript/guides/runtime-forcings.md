# Runtime forcings

Use explicit methods to change runtime inputs. Assigning a field on a result or
configuration record never changes the solver. Values use the model's project
units unless noted.

## Choose a forcing method

| Method | Meaning | Writable phases |
| --- | --- | --- |
| `node.setExternalInflow(flow)` | Persistent additive node inflow | `open`, `running`, `ended` |
| `node.setOutfallStage(stage)` | Persistent fixed-stage outfall boundary | `open`, `running`, `ended`; outfalls only |
| `link.setTargetSetting(setting)` | Active regulator setting or pump speed factor | `running` |
| `link.setFlowLimit(flow)` | Persistent link flow limit | `open`, `running`, `ended` |
| `link.updateInlet(patch)` | Persistent inlet clogging or flow limit | `open`, `running`, `ended` |
| `gage.setPrecipitation(rate)` | Select persistent API precipitation source | `open`, `running`, `ended` |
| `gage.setRainfallOverride(rate)` | Override the current rain-gage source, or clear with `null` | `open`, `running`, `ended` |
| `subcatchment.setExternalRainfall(value)` | Persistent additive catchment rainfall | `open`, `running`, `ended` |
| `subcatchment.setExternalSnowfall(value)` | Persistent additive catchment snowfall | `open`, `running`, `ended` |
| `subcatchment.setPrecipitationScaleFactors(rainfall, snowfall)` | Persistent rain and snow multipliers | `open`, `running`, `ended` |
| `node.updateExternalPollutantMassFlux(values)` | Persistent node pollutant mass-flux map | `open`, `running`, `ended` |
| `link.updateExternalPollutantMassFlux(values)` | Persistent link pollutant mass-flux map | `open`, `running`, `ended` |
| `subcatchment.updateExternalPollutantBuildupIncrement(values)` | Persistent pollutant buildup increment map | `open`, `running`, `ended` |
| `node.overridePollutantConcentrations(values)` | Queue one node's concentrations for the next quality step | `running` |
| `link.overridePollutantConcentrations(values)` | Queue one link's concentrations for the next quality step | `running` |

Persistent forcings survive solver initialization and reruns on the same owner.
They are cleared by `close()`. Checkpoint Resume and `fork()` copy them; State
Load keeps the receiver's existing forcings.

## Control a link from node depth

```typescript
import { Simulation, type FileContents } from "@swmmrs/swmmrs";

export async function controlledRun(input: FileContents) {
	const simulation = await Simulation.open(input, {}, { threads: 1 });
	try {
		const node = simulation.nodes.get("J1");
		const regulator = simulation.links.get("OR1");
		await node.setExternalInflow(2);

		for await (const time of simulation.steps({ seconds: 60, strict: true })) {
			const { depth } = await node.results();
			await regulator.setTargetSetting(depth > 2 ? 1 : 0.25);
			console.log(time, depth);
		}
		return await simulation.finish();
	} finally {
		await simulation.close();
	}
}
```

Node API inflow is additive to other sources. Set it to zero to remove that API
contribution. A link setting must be finite. Non-pump settings are clamped to
0 to 1; pump speed factors are nonnegative and may exceed one. INP control rules can
later change a link's target setting.

## Set rainfall and catchment inputs

```typescript
import { Simulation, type FileContents } from "@swmmrs/swmmrs";

export async function setRainAndRun(input: FileContents) {
	const simulation = await Simulation.open(input, {}, { threads: 1 });
	try {
		const gage = simulation.rainGages.get("Rain");
		const catchment = simulation.subcatchments.get("S1");
		await gage.setPrecipitation(0.5);
		await gage.setRainfallOverride(1);
		await catchment.setExternalRainfall(0.25);
		await catchment.setExternalSnowfall(0);
		await catchment.setPrecipitationScaleFactors(1, 1);
		return await simulation.run({ saveResults: false });
	} finally {
		await simulation.close();
	}
}
```

Rates must be finite and nonnegative. They are inches/hour for US projects or
millimetres/hour for SI projects. Zero and `null` differ: zero forces dry
rainfall, while `null` clears only the override. Clearing an override exposes
the selected API precipitation source again. Selecting API precipitation is
persistent; there is no method to restore the original shared-gage source on the
same owner.

## Apply quality forcings

Quality reads and overrides use pollutant IDs as keys. A node or link concentration
override applies to the next quality step and then expires. Persistent mass-flux
and buildup-increment maps remain until changed or replaced. Pass `true` as the
second argument to replace a complete mapping, or omit it to merge supplied IDs:

Nonzero link mass flux requires a non-dummy conduit; wrong-kind calls reject.

```typescript
import { Simulation, type FileContents } from "@swmmrs/swmmrs";

export async function applyQualityForcing(input: FileContents) {
	const simulation = await Simulation.open(input, {}, { threads: 1 });
	try {
		const node = simulation.nodes.get("J1");
		const link = simulation.links.get("C1");
		const catchment = simulation.subcatchments.get("S1");
		await node.updateExternalPollutantMassFlux({ TSS: 2.5 });
		await link.updateExternalPollutantMassFlux({ TSS: 1.5 });
		await catchment.updateExternalPollutantBuildupIncrement({ TSS: 0.5 });
		await simulation.start();
		await node.overridePollutantConcentrations({ TSS: 123 });
		await simulation.step();
		return await node.quality();
	} finally {
		await simulation.close();
	}
}
```

For a node, you can merge changes, replace the complete map, or clear all API
mass fluxes without changing other inflow sources:

```typescript
import type { Node } from "@swmmrs/swmmrs";
declare const node: Node;

await node.updateExternalPollutantMassFlux({ TSS: 2.5 });
await node.updateExternalPollutantMassFlux({ TSS: 1 }, true);
await node.clearExternalPollutantMassFlux();
```

The replacement resets omitted pollutants to zero. A previously returned map
is a detached read; mutating it would not update the model. See the
[Node reference](../api/node.md) for argument details and units.

Use [quality and statistics reads](collect-results.md) for current concentrations,
quality snapshots, cumulative loads, and continuity balances. Use [lifecycle and
ownership](../concepts/lifecycle.md) for phase rules and scenario-copy behavior.
