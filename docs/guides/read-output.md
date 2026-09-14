# Read binary output

`OutputReader` reads SWMM `.out` files independently of a live `Simulation`. It detects a valid final trailer automatically. Use the reader for post-run time-series analysis; use live properties and snapshots when Python must inspect or control a simulation while it advances.

```python
from swmmrs.output import OutputReader

reader = OutputReader("model.out")
print(reader.metadata.run_status)
print(reader.metadata.flow_units)
print(reader.metadata.report_timing.period_count)
```

Opening always validates immutable metadata and the result schema. A valid final
trailer enables strict stored-period, run-status, and exact-extent validation.
Without one, the reader exposes every complete result record already visible in
the file and ignores a partial trailing record.

## Recover complete records after a failed run

The normal constructor handles output left incomplete by a failed run:

```python
reader = OutputReader("failed-run.out")
assert not reader.is_finalized
assert reader.metadata.run_status.code is None
print(reader.metadata.report_timing.period_count)
```

`reader.is_finalized` is `True` when a valid final trailer was present at open
time. Otherwise it is `False`, and `period_count` is
`floor(available result bytes / bytes per period)`. Zero complete records is
valid. Metadata must still be complete and valid.

The reader snapshots the visible file length when it opens. It does not tail a
running file and cannot recover bytes that the failed process never flushed to
the filesystem.

## Inspect identities and schemas

Metadata preserves physical order, duplicate names, unknown signed codes, and exact stored name bytes:

```python
for node in reader.metadata.nodes:
    print(node.index, node.name.raw, node.name.text, node.kind)

print(reader.metadata.result_schema.node)
print(reader.metadata.pollutants)
```

`OutputName.raw` is authoritative. `OutputName.text` is available only when those bytes are valid UTF-8. Name selectors match stored bytes exactly and case-sensitively; there is no trimming, normalization, abbreviation, or lossy decoding.

## Read one family series

Family methods return an immutable `OutputTimeSeries` containing one canonical selection, one nominal datetime tuple, and one aligned value tuple:

```python
from swmmrs.output import PollutantAttribute

node_depth = reader.node_series("J1", "depth")
link_flow = reader.link_series("C1", "flow")
runoff = reader.subcatchment_series("S1", "runoff_flow")
rainfall = reader.system_series("rainfall")
tss = reader.node_series("J1", PollutantAttribute("TSS"))

assert len(node_depth.times) == len(node_depth.values)
```

A non-system element can be selected by:

- zero-based physical index;
- exact UTF-8 `str`;
- exact `bytes`; or
- `OutputName` from metadata.

Known attributes accept their family enum or exact canonical lower-snake-case string. Use `PollutantAttribute` for pollutant columns, even when a pollutant name collides with a static attribute such as `depth`. `ResultAttributeCode` selects a stored unknown result code explicitly.

## Select periods and dates

Every result method accepts independently optional `start` and `end` bounds. The resolved range is half-open: `start` is inclusive and `end` is exclusive.

```python
from datetime import datetime

first_day = reader.node_series("J1", "depth", start=0, end=12)
calendar_slice = reader.node_series(
    "J1",
    "depth",
    start=datetime(2020, 1, 1),
    end=datetime(2020, 1, 2),
)
```

Integer bounds are exact zero-based boundaries and must satisfy `0 <= start <= end <= period_count`. Naive datetime bounds use lower-bound lookup against the rounded Nominal Report Date axis and clip naturally outside that axis. Integer and datetime bounds can be mixed; each resolves independently. Booleans, negative integers, timezone-aware datetimes, and inverted ranges are rejected. A valid empty range succeeds.

## Read several series together

`SeriesSelection` describes one element family, element, and attribute. Bulk results preserve request order and duplicates:

```python
from swmmrs.output import SeriesSelection

selections = (
    SeriesSelection("node", "J1", "depth"),
    SeriesSelection("node", "J2", "depth"),
    SeriesSelection("link", "C1", "flow"),
    SeriesSelection("node", "J1", PollutantAttribute("TSS")),
)

result = reader.read_bulk_series(selections, start=0, end=24)
print(result)  # BulkSeriesResult(periods=24, series=4)

for series in result.series:
    print(series.selection, series.values[0])

one_value = result.value(period_offset=0, selection_offset=2)
```

`BulkSeriesResult.times` is shared by every `OutputValueSeries`. No flat period-major buffer or synthetic system element is exposed. Empty periods, empty selections, and both dimensions empty remain valid structured results.

## Choose the I/O strategy

`read_bulk_series` and all four family methods accept the keyword-only `low_memory` flag.

| Setting | Physical read plan | Scratch memory | Prefer when |
| --- | --- | --- | --- |
| `low_memory=False` (default) | One complete result payload per selected period | One period payload | The request is wide or reducing read calls matters most |
| `low_memory=True` | Deduplicated exact adjacent runs containing selected cells | Largest selected run | The request is narrow or the period payload is large |

```python
wide = reader.read_bulk_series(selections)
narrow = reader.read_bulk_series(selections, low_memory=True)
assert narrow == wide

single = reader.node_series("J1", "depth", low_memory=True)
```

Both strategies return the same canonical selections, nominal dates, dimensions, and exact promoted `f32` values. `low_memory` changes only physical I/O and scratch allocation; it does not change result shape or range semantics.

## Nominal and Stored Report Dates

`reader.times` is a lazy, identity-reused tuple of naive Python datetimes derived from exact Nominal Report Dates. Conversion uses the SWMM epoch of 1899-12-30 and whole-second round-half-to-even.

Stored Report Dates are separate exact file facts:

```python
nominal = reader.times[:2]
stored_serial_days = reader.read_stored_dates(start=0, end=2)
```

Stored dates never select, snap, reorder, or replace the nominal axis. An integer-only `read_stored_dates` call does not populate the lazy `times` cache.

## Convert a bulk result to pandas

Pandas is not a `swmmrs` runtime dependency. A caller that already uses pandas can preserve ordering and duplicate selections with temporary integer columns, then install a labelled `MultiIndex`:

```python
import pandas as pd

from swmmrs.output import BulkSeriesResult


def to_dataframe(result: BulkSeriesResult) -> pd.DataFrame:
    index = pd.DatetimeIndex(result.times, name="time")
    frame = pd.DataFrame(
        {
            offset: series.values
            for offset, series in enumerate(result.series)
        },
        index=index,
    )

    labels = [
        (
            series.selection.element_type.value,
            series.selection.element,
            series.selection.attribute,
        )
        for series in result.series
    ]
    frame.columns = (
        pd.MultiIndex.from_tuples(
            labels,
            names=("element_type", "element", "attribute"),
        )
        if labels
        else pd.MultiIndex.from_arrays(
            ([], [], []),
            names=("element_type", "element", "attribute"),
        )
    )
    return frame


frame = to_dataframe(result)
```

Do not construct the dataframe from a mapping keyed directly by `SeriesSelection`: duplicate selections would overwrite one another before pandas sees them.

## Handle failures

Native reader failures use `OutputError.category` as a stable high-level discriminator:

```python
from swmmrs.output import OutputError

try:
    reader.node_series("missing", "depth")
except OutputError as error:
    print(error.category, str(error))
```

Missing and duplicate exact names, missing and duplicate attributes, invalid file layouts, date overflow, and I/O failures remain distinct categories. Python type and value mistakes raise `TypeError` or `ValueError` before native work begins.

One reader serializes its physical file operations while releasing the GIL during native reads. Use separate `OutputReader` instances when independent threads need concurrent file reads.

See the [binary-output API reference](../python/api/output.md) for exact signatures and exported record types.
