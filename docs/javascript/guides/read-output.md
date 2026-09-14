# Read binary output

Use `OutputReader` for report-period series, not for live solver observations.
It owns an independent worker and can read output after the simulation has closed.
Pass file contents, not a host path; in Node, use bytes from `fs.readFile()`.
See [Load files and download output](files.md) for file handling.

## Query selected columns

This example assumes the output contains node `J1`:

```typescript
import { OutputReader, type FileContents } from "@swmmrs/swmmrs";

export async function readHead(input: FileContents) {
	const reader = await OutputReader.open(input);
	try {
		console.log(reader.metadata.runStatus, reader.metadata.flowUnits);
		return await reader.readBulkSeries([
			{ elementType: "node", element: "J1", attribute: "hydraulic_head" },
			{ elementType: "system", element: null, attribute: "volume_stored" },
		], { lowMemory: true });
	} finally {
		await reader.close();
	}
}
```

The columns share a nominal report-date axis. Keep their selection labels with
the values rather than relying on column positions alone. Returned arrays and
metadata are immutable and remain usable after closing the reader.

## Inspect before selecting

Check `metadata.runStatus` when processing interrupted runs: a readable file can
contain complete periods without a finalized trailer, and finalized output can
record an unsuccessful solver status. Inspect `metadata.resultSchema` to learn
which attributes are actually present.

Use metadata indices or exact stored names. Output-name matching is case-sensitive,
unlike simulation collection lookup. For names that are not valid UTF-8, retain
`OutputName` or its raw bytes instead of replacing invalid characters. Unknown
categorical codes are retained so applications can inspect newer output formats.

Pollutant columns use `{ selector: pollutant }` rather than a built-in attribute
name. The selector can be a pollutant index, exact name, raw bytes, or metadata
name. System results do not have pollutant columns.

## Choose a range and memory strategy

Use `start` and `end` to select a half-open report-period range. Integer bounds are
period offsets; calendar bounds locate nominal report dates, not simulation
routing steps. Omitted bounds select all available complete periods.

For large files, select only the columns and periods needed. `lowMemory: true`
changes the read strategy without changing values. Duplicate selections remain
separate columns, so de-duplicate them yourself when repetition is unintended.

The normal time axis is rounded and follows the nominal reporting schedule. Use
`readStoredDates()` when you need the exact SWMM serial-day values stored in the
file. Neither axis changes the observation cadence of `Simulation.steps()`.

See the [Binary output reference](../api/output.md) for selectors, units, bounds,
errors, and every metadata and result field.
