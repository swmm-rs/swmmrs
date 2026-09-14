# Enumerations

Use these public string enums for lifecycle states, project units, object
kinds, hydraulic states, and stable solver error categories. They give each
choice a name you can look up and check.

| Category | Enums |
| --- | --- |
| Lifecycle and diagnostics | `SimulationState`, `SolverErrorCode` |
| Units and project policy | `UnitSystem`, `FlowUnits`, `InfilKind`, `InertiaDamping`, `NormalFlowLimit`, `SurchargeMethod` ,`CustomEllipseModel` |
| Object and result kinds | `NodeKind`, `LinkKind`, `FlowClass`, `OutKind` |
| Typed link configuration | `OrificeKind`, `WeirKind`, `OutletHeadBasis`, `RoadSurface`, `StandardCrossSectionShape` |

::: swmmrs.enums
    options:
        show_root_heading: false
        show_if_no_docstring: true
        show_inheritance_diagram: false
