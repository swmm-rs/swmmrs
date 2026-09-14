# Units and time

The API returns quantities in the model's configured project units. Read
`flowUnits` from `await simulation.info()` before choosing configuration values,
controls, or interpreting results.

## Unit families

| Quantity | US projects (`Cfs`, `Gpm`, `Mgd`) | SI projects (`Cms`, `Lps`, `Mld`) |
| --- | --- | --- |
| Flow | Configured flow unit | Configured flow unit |
| Length, elevation, hydraulic depth | ft | m |
| Hydraulic surface area, ponded area | ft² | m² |
| Hydraulic volume and routing totals | ft³ | m³ |
| Velocity | ft/s | m/s |
| Subcatchment land area | acres | hectares |
| Rainfall, infiltration, seepage rate | in/hour | mm/hour |
| Subcatchment evaporation rate | in/day | mm/day |
| Snow depth, depression storage, runoff-depth totals | in | mm |
| Durations with a `Seconds` suffix | seconds | seconds |

Fractions and slopes are ratios, not percentages. `imperviousFraction: 0.4`
means 40%, and `slope: 0.01` means 1%. Continuity errors and
`percentComplete` are percentages. Pollutant concentrations and loads use the
native model units described in the [API records](../api/index.md).

## Model timestamps

`ModelTime` is a timezone-free string in `YYYY-MM-DDTHH:mm:ss` form. It names a
point in the model's calendar, not a UTC instant. Do not append `Z` or pass it to
`Date` unless your application has chosen an explicit timezone interpretation.

`info()` provides `startTime`, `endTime`, and `reportStart`. `status()` provides
`currentTime`, elapsed and duration seconds, and completion from 0 to 100.
`currentTime` can be `null` when no result time exists. Use `elapsedSeconds` for
numeric progress.

JavaScript uses numeric seconds for observation intervals and runtime durations.
It does not accept Python `timedelta` values. Observation intervals must be
positive 32-bit whole seconds. Some model timing options still allow fractional
seconds when the native option accepts them.

## Three independent cadences

| Cadence | API | Purpose |
| --- | --- | --- |
| Routing | `options.update({ routingStepSeconds })` | Configure the solver's routing step; native routing may use smaller steps |
| Observation | `steps({ seconds, strict })` or `stride(seconds, strict)` | Decide when control returns to the application |
| Reporting | `options.update({ reportStepSeconds })` | Configure saved report periods in binary output |

`strict` defaults to `true`. An exact observation interval can shorten the final
routing step to hit its boundary and may change numerical results. Pass
`strict: false` explicitly to advance whole routing steps until the interval is
reached or exceeded. Omitting `seconds` observes every routing step, and the
`strict` flag has no effect.
