# Python API reference

Start with [`Simulation`](simulation.md), then follow the references for the
objects you need. Collections expose configured model data, snapshots keep
results from one solver instant, and [`OutputReader`](output.md) reads binary
output after routing stops. The tables and signatures below provide the exact
names when memory supplies something plausible but wrong.

| Area | Reference |
| --- | --- |
| Project lifecycle and top-level access | [Simulation](simulation.md) |
| Binary metadata and result series | [Binary output](output.md) |
| Numerical and analysis policy | [Simulation options](options.md) |
| Nodes, links, subcatchments, rain gages, LIDs, and RDII | [Object collections](generics.md), object-family pages, and [RDII objects](rdii.md) |
| Curves, patterns, pollutants, controls, and related definitions | [Model definitions](definitions.md) |
| Current and cumulative owned records | [Snapshots and statistics](snapshots.md) and [Collect results and statistics](../../guides/collect-results.md) |
| Typed values and failures | [Enums](enums.md) and [exceptions](exceptions.md) |
| Package and embedded solver identity | [Package metadata](package-metadata.md) |

All hydraulic and quality values preserve the model's configured units. Lifecycle restrictions documented on properties and methods are part of the API contract. Supported top-level imports include `Simulation`, `SimulationState`, `SolverErrorCode`, and the exception/diagnostic types documented in [Exceptions](exceptions.md).
