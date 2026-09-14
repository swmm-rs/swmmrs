# Inspect and configure a model

Open the model before looking up its objects. Collections expose canonical IDs in
configured order and use case-insensitive lookup. Configuration reads return
asynchronous, detached records.

This example accepts INP contents for a model with a node named `J1`:

```typescript
import { Simulation, type FileContents } from "@swmmrs/swmmrs";

export async function configureAndInspect(input: FileContents) {
	const simulation = await Simulation.open(input, {}, { threads: 1 });
	try {
		console.log(await simulation.info());
		console.log(simulation.nodes.ids, simulation.links.size);

		const node = simulation.nodes.get("J1");
		const before = await node.configuration();
		await node.configure({ fullDepth: before.fullDepth + 1 });
		await simulation.options.update({ reportStepSeconds: 60 });

		return await node.configuration();
	} finally {
		await simulation.close();
	}
}
```

The depth increment uses one project length unit. Read `flowUnits` and the unit
system from `await simulation.info()` before choosing values.

## Apply sparse patches

`configure(patch)` updates only supplied fields. It validates the complete patch
before committing it to one object. Omitted fields retain their values, and a
rejected patch does not partially apply. Options updates follow the same rule.
There is no transaction spanning several objects or calls.

Make edits while the owner is `open` or `ended`. An edit in `ended` prepares a
fresh run and invalidates the prior live run state. Do not assign to a returned
configuration record or spread a complete record into a patch. Records include
read-only and derived fields that patches do not accept.

## Supported configuration families

| Family | Examples |
| --- | --- |
| Node | Common fields plus storage shape/exfiltration, outfall boundary, divider rule, and report selection |
| Link | Common fields plus kind-tagged conduit, pump, orifice, weir, or outlet subtype; cross-section and inlet settings |
| Subcatchment | Surface properties, infiltration, groundwater, snowpack ID, pollutant buildup, and land-use coverage |
| Model | Timing, routing, process switches, reporting, and solver settings through `simulation.options` |
| Aquifer and snowmelt | `simulation.aquifers` and `simulation.snowmeltSets` configuration records and sparse patches |
| AMM and RTK | `simulation.ammModels` and `simulation.unitHydrographs`, plus assignment lists on `Simulation` |
| LID | `simulation.lidControls` layers and indexed units under `subcatchment.lidUnits` |

Node and link handles do not turn into public subtype classes. Inspect
`configuration().kind`, and for links inspect the kind-tagged `subtype` record.
See the [Link](../api/link.md) and [Subcatchment](../api/subcatchment.md)
references for field and patch shapes. Definition contracts are on
[Model definitions](../api/definitions.md), [RDII](../api/rdii.md), and [LID](../api/lid.md).

## Configure node kinds

All nodes use the same handle class. Inspect the configuration's `kind` before
editing a storage node, outfall, or divider; handles do not become subtype
classes. Configuration reads include common declarations and nullable
kind-specific records. A `null` subtype record means it does not apply or is
not configured.

Common and kind-specific fields can be changed together in one atomic patch.
This example assumes a storage node `STORAGE-1`, an outfall `OUT-1`, and a
boundary time series `OUTFALL-STAGE` already exist in the model:

```typescript
import type { Simulation } from "@swmmrs/swmmrs";

export async function configureNodes(simulation: Simulation) {
	const storage = simulation.nodes.get("STORAGE-1");
	if ((await storage.configuration()).kind === "Storage") {
		await storage.configure({
			fullDepth: 4.5,
			evaporationFraction: 0.75,
		});
	}

	const outfall = simulation.nodes.get("OUT-1");
	if ((await outfall.configuration()).kind === "Outfall") {
		await outfall.configure({
			boundary: { kind: "timeseries", reference: "OUTFALL-STAGE" },
			hasFlapGate: true,
		});
	}
}
```

Relationships use configured IDs, not handles or object indexes. Nested shape,
exfiltration, boundary, and divider-rule records replace that declaration;
they are not recursive sparse patches. Read the declaration first when you
want to preserve its existing nested values.

Storage shapes expose canonical coefficients. Keep those coefficients when
round-tripping a declaration rather than trying to reconstruct geometry the
parser no longer retains. The [Node reference](../api/node.md) documents each
field's units, defaults, and applicable node kinds.

A declared outfall boundary and a runtime fixed-stage override serve different
purposes. Use configuration to change the model declaration; use the
[runtime forcing methods](runtime-forcings.md) to control an existing owner.

## Configure links and subcatchments

A link subtype patch selects the existing kind; it cannot convert a conduit to
a pump. Read the subtype first and change only the fields you intend:

```typescript
import type { Simulation } from "@swmmrs/swmmrs";

export async function configureRunoffPath(simulation: Simulation) {
	const link = simulation.links.get("C1");
	if ((await link.configuration()).kind === "Conduit") {
		await link.configure({ subtype: { kind: "conduit", roughness: 0.015 } });
	}

	const catchment = simulation.subcatchments.get("S1");
	await catchment.configure({
		outlet: { kind: "node", id: "J1" },
		imperviousFraction: 0.6,
	});
}
```

Dedicated buildup and coverage update methods merge maps by default. In contrast,
putting those maps in a configuration patch replaces the complete map. Be explicit
about replacement when editing only one pollutant or land use.

## Configure specialized definitions

Specialized relationships use canonical string IDs. They do not accept handles
from another collection:

```typescript
import { type Simulation } from "@swmmrs/swmmrs";

export async function configureSpecialized(simulation: Simulation) {
	await simulation.aquifers.get("A1").configure({ waterTableElevation: 10 });
	await simulation.snowmeltSets.get("Snow").configureSurface("pervious", {
		baseTemperature: 0,
	});

	await simulation.ammModels.get("M1").configure({ hotTemperature: 30 });
	await simulation.unitHydrographs.get("UH1").configure({ rainGage: "Rain" });
	await simulation.replaceAmmAssignments([
		{ node: "J1", model: "M1", area: 1 },
	]);
	await simulation.replaceRdiiAssignments([
		{ node: "J1", unitHydrograph: "UH1", area: 1 },
	]);

	await simulation.lidControls.get("Bio").configure("surface", {
		roughness: 0.1,
	});
	const unit = await simulation.subcatchments.get("S1").lidUnits.at(0);
	await unit.configure({ count: 2 });
}
```

AMM/RTK component and monthly-response arrays replace complete declarations;
assignment replacements also replace the complete list. Read and retain existing
entries first when adding one assignment. For LID, edit shared control layers
separately from a subcatchment's indexed usages. Only existing layers are patchable,
and LID snapshots must be collected during a run, before ending it.

`simulation.pollutants`, `timePatterns`, `curves`, `timeSeries`, `controls`,
`transects`, `shapes`, `streets`, and `inletDesigns` expose identity handles and
IDs. They do not provide editable time-series definitions or object creation.
Edit those definitions in the INP and open a new simulation when needed.

## References and run state

A configuration record can expose related IDs such as a node's `routeToSubcatchment`,
a link's `inletNode`, a subcatchment's `rainGage`, or a LID unit's `control`.
Use the corresponding collection to obtain a handle when you need to read or
edit the related object.

Runtime forcings such as node inflow, link settings, pollutant overrides, and
rainfall are separate from declaration patches. See [runtime forcings](runtime-forcings.md).
Checkpoint State Load also requires an `open` owner. Use `resetSolver()` after an
ended run when you need to return to that state.
