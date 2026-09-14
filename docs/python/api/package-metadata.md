# Package metadata

The Python package and its embedded solver have separate identities. Include
all three values below in reproducibility or compatibility reports so nobody
has to guess which solver came with your installation.

| Name | Meaning |
| --- | --- |
| `swmmrs.__version__` | Installed Python package version. |
| `swmmrs.solver_version` | Embedded solver version string. |
| `swmmrs.solver_build_id` | Embedded solver build identity string. |

```python
import swmmrs

print("package:", swmmrs.__version__)
print("solver:", swmmrs.solver_version)
print("solver build:", swmmrs.solver_build_id)
```

The Python package version alone does not identify the solver. Keep the build
ID as an opaque public string for diagnostics and provenance. Its format is
implementation-defined, so code should not depend on parsing it.

See [Compatibility and limitations](../compatibility.md) for supported runtime
requirements and [Simulation](simulation.md) for the primary owner API.
