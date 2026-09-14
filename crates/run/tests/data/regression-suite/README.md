# Curated run regression model suite

This directory snapshots the sanitized, redistributable `swmm-bench` regression suite for explicitly selected `runswmmrs` smoke tests. Tests use only these checked-in fixtures and write artifacts under `tmp/run-smoke-suite/`.

Run one model from the repository root, for example:

```sh
cargo test --manifest-path crates/run/Cargo.toml --locked --bin runswmmrs smoke_suite::lid_green_roof_paired -- --ignored --exact --test-threads=1
```

## Organization

- `hydrology/`: runoff, LID, groundwater, and snowmelt coverage
- `water-quality/`: pollutant buildup, washoff, and event-based simulations
- `hydraulics/`: hydraulic structures, inlet capture, outfall behavior, and cross-section shapes
- `controls/`: dynamic-wave RTC scenarios with pumps, storage, orifices, weirs, and outlets
- `routing/`: alternate routing solvers beyond the default dynamic wave engine
- `use_interfaces/`: canonical EPA rainfall, runoff, hotstart, RDII, and routing interface inputs
- `save_interfaces/`: paired interface generators exercised in scratch space

## Selected models

Every `.inp` records its source or purpose-built status as a `Provenance:` line inside `[TITLE]`; `CITATION.txt` contains repository, commit, license, and adaptation details.

| Curated file | Source file | Primary regression target | Notable secondary coverage |
| --- | --- | --- | --- |
| `hydrology/lid-example_lid_rb.inp` | `EPA/Example_LID_RB.inp` | LID runoff controls | raingage/timeseries runoff, subcatchment hydrology |
| `hydrology/lid-controls-ghcnd.inp` | derived from `crates/solver/tests/data/lid/test_w_wo_*_2Subcatchments.inp` | all eight LID process types | detailed LID reporting, pollutant removals, GHCND climate input, and temperature-derived evaporation |
| `hydrology/lid-*-paired.inp` (seven files) | `LID/w_wo_{BC,GR,IT,PP,RB,RG,SWALE}_2Subcatchments.inp` | paired rainfall response for bioretention, green roof, infiltration trench, porous pavement, rain barrel, rain garden, and vegetative swale | nonzero rain/runoff, LID fluxes, storage, infiltration, and drain routing |
| `hydrology/groundwater-si_gw_model.inp` | `Hydrology/SI_GW_Model.inp` | groundwater and aquifer interaction | GWF, RDII hydrographs, weir, controls, storage |
| `hydrology/snowmelt-small_snowmelt_model.inp` | `SWMM5_NCIMM/small_snowmelt_model.inp` | snowpack and temperature-driven snowmelt | raingage/timeseries runoff |
| `hydrology/greenville-lid-snow-groundwater.inp` | `Greenville/Greenville_small_snowmelt_model.inp` | integrated LID, snowmelt, and groundwater behavior | seven LID process types, two snowpacks, RDII, DWF, and dynamic-wave routing |
| `hydrology/rdii-assignment_groundwater-si_gw_model.inp` | derived from `hydrology/groundwater-si_gw_model.inp` | RDII node assignment against existing unit hydrographs | groundwater + controls baseline retained |
| `hydrology/curvenumber-lid-example_lid_rb.inp` | derived from `hydrology/lid-example_lid_rb.inp` | Curve Number infiltration option coverage | LID runoff controls and KINWAVE routing baseline retained |
| `hydrology/climate-file-evaporation.inp` | purpose-built | daily climate-file evaporation | file-backed, time-varying temperature and evaporation |
| `hydrology/climate-td3200.inp` | purpose-built | NCDC TD-3200 climate records | file-backed daily temperature, evaporation, and wind |
| `hydrology/climate-dly0204.inp` | purpose-built | Canadian DLY02/DLY04 climate records | file-backed daily temperature and evaporation |
| `hydrology/ncdc-rainfall.inp` | purpose-built | NCDC rainfall-file parsing | external rain-gage input and time-varying runoff |
| `hydrology/rainfall-format-matrix.inp` | purpose-built | NWS comma, NWS tape, and Canadian CMC hourly rainfall parsing | accumulated-period handling and nonzero runoff from each source |
| `hydrology/lid-roof-disconnection-runon.inp` | purpose-built | rooftop-disconnection LID runoff | paired LID/no-LID rainfall response |
| `hydrology/modified-horton.inp` | purpose-built | `MODIFIED_HORTON` infiltration | rainfall-driven soil recovery and routed runoff |
| `hydrology/modified-green-ampt.inp` | purpose-built | `MODIFIED_GREEN_AMPT` infiltration | rainfall-driven soil recovery and routed runoff |
| `hydrology/outfall-runon.inp` | purpose-built | outfall runon returned to a subcatchment | routed outfall flow and downstream runoff |
| `water-quality/waterquality-events_example.inp` | `OWA_update_v5111/events_example.inp` | pollutant buildup/washoff water quality | event-based runs, subcatchment runoff |
| `water-quality/treatment-waterquality-events_example.inp` | derived from `water-quality/waterquality-events_example.inp` | node treatment expression evaluation | pollutant buildup/washoff/event behavior baseline retained |
| `water-quality/treatment-expression-rainfall.inp` | `SWMM5_NCIMM/treatment.INP` | treatment expressions evaluated under rainfall-runoff water quality | five pollutant loads, storage treatment, and nonzero outfall loads |
| `water-quality/treatment-variable-matrix.inp` | derived from `water-quality/treatment-expression-rainfall.inp` | treatment process-variable and removal references | HRT, time step, flow, depth, area, concentration, and prior-pollutant removal terms |
| `water-quality/landuse-function-matrix.inp` | purpose-built | land-use buildup and washoff function matrix | multiple pollutants and time-varying quality |
| `hydraulics/inlets-inlet_capture_test.inp` | `Hydraulics/inlet_capture_test.inp` | streets, inlets, and inlet capture | conduit routing and node inflows |
| `hydraulics/inlets-onsag-slotted-custom.inp` | purpose-built | on-sag grate, curb, and slotted-drain capture | custom inlet capture curves and explicit inlet placement |
| `hydraulics/weirs-rainfall-2weirs-4subs.inp` | `Weirs/2_Weirs_4Subs.inp` | rainfall-driven transverse and trapezoidal weirs | four runoff-producing subcatchments and nonzero overflow-weir flows |
| `hydraulics/inlets-rainfall-drop-matrix.inp` | purpose-built | rainfall-driven drop-grate and drop-curb capture | on-sag pollutant capture and bypass routing |
| `hydraulics/outfalls-all_outfall_types_model.inp` | `EPA/ALL_Outfall_Types_Model.inp` | outfall boundary condition types | inflow hydrograph and stage relationships |
| `hydraulics/elevation-offsets_outfalls-all_outfall_types_model.inp` | derived from `hydraulics/outfalls-all_outfall_types_model.inp` | alternate hydraulic options (`LINK_OFFSETS ELEVATION`, `FORCE_MAIN_EQUATION D-W`) | outfall boundary condition baseline retained |
| `hydraulics/shapes-swmm5_shapes.inp` | `EPA/swmm5_shapes.inp` | custom/irregular cross-sections and gated outfalls | event windows and tide-gate behavior |
| `hydraulics/culvert-roadway-exfiltration.inp` | derived from `crates/solver/tests/data/link_constantinflow.inp` | culvert inlet-control regimes and paved/gravel roadway overtopping | tabular storage exfiltration |
| `hydraulics/storage-cross-section-matrix.inp` | purpose-built | cylindrical, conical, paraboloid, pyramidal, and functional storage | parabolic, power, rectangular-round, modified-basket, and rectangular-open cross-sections plus functional outlet flow |
| `hydraulics/forcemain-darcy-weisbach.inp` | purpose-built | Darcy-Weisbach force-main losses | nonzero dynamic-wave pressurized flow |
| `hydraulics/slot-surcharge_tunnelmh.inp` | `Simon_EPA/Session46_TunnelMH.inp` | Preissmann-slot surcharge routing | actively surcharged closed conduit, storage, timed orifice control |
| `hydraulics/inertial-damping-full_exam80a-sw5.inp` | derived from `EPA/Exam80A-SW5.inp` | full inertial damping under surcharge | alternating conduit sizes and sustained external inflow; unused runoff-interface output removed |
| `hydraulics/outlet-tabular-depth_extran3-rc.inp` | `OWA_Extran/Extran3-RC.inp` | tabular depth outlets and rating curves | nonzero outlet flow, storage, bottom orifice, surcharge, and flooding |
| `controls/rtc-master_extran_rtc.inp` | `EPA/Master_Extran_RTC.inp` | real-time control rules | pumps, weirs, orifices, outlets, storage, DWF |
| `controls/rtc-many-rules.inp` | `Weirs/v14RTCManyRules.inp` | mixed RTC premises and actions | PID and time-series settings across pumps, orifices, and weirs |
| `controls/variables-expressions-rain.inp` | purpose-built | named control variables and expressions | rain-gage, simulation-time, and link-open-duration premises |
| `controls/math-operator-matrix.inp` | purpose-built | control-rule math operator matrix | time-varying premises and actuator actions |
| `routing/kinwave-routing_kinwave.inp` | `OWA_ROUTING/routing_kinwave.inp` | kinematic wave routing | inline inflow hydrograph and simple conduit chain |
| `routing/kinwave-divider-types.inp` | purpose-built | all four flow-divider types under KINWAVE | tabular diversion curve and external node inflows |
| `routing/kinwave-storage-shapes.inp` | purpose-built | non-dynamic storage volume/depth routing | all analytic storage shapes, tabular area inversion, evaporation, and conduit releases |
| `routing/steady-attenuation_steady_flow.inp` | `Hydraulics/attenuation_steady_flow.inp` | steady routing solver | long conduit chain attenuation |
| `routing/pump-storage-table-matrix.inp` | purpose-built | pump curves and tabular storage routing | time-varying pump and storage state |
| `use_interfaces/rdii-use.inp` | purpose-built | `USE RDII` against an EPA-generated binary interface | one-node unit hydrograph and nonzero RDII routing |
| `use_interfaces/rdii-use-text.inp` | purpose-built | legacy text `USE RDII` interface | time-varying RDII inflow and dynamic-wave routing |
| `use_interfaces/rainfall-use.inp` | purpose-built | `USE RAINFALL` against an EPA-generated binary interface | external-file rain gage and single-subcatchment runoff |
| `use_interfaces/runoff-use.inp` | purpose-built | `USE RUNOFF` against an EPA-generated binary interface | single-subcatchment runoff routed through one conduit |
| `use_interfaces/hotstart-use.inp` | purpose-built | `USE HOTSTART` against an EPA-generated binary state | storage and routing-state continuation |
| `use_interfaces/inflows-use-toy.inp` | purpose-built | `USE INFLOWS` against an EPA-generated routing interface | downstream dynamic-wave routing |
| `save_interfaces/*.inp` + paired `use_interfaces/*.inp` | purpose-built | scratch SAVE/USE roundtrip for outflows/inflows, rainfall, runoff, RDII, and hotstart | generated interfaces never rewrite checked-in fixtures |

## Coverage notes

Together these models keep the suite small-to-medium while covering a wide span of SWMM engine behavior:

- hydrology: subcatchments, raingages, GHCND, TD-3200, DLY02/DLY04, NCDC online/comma/tape, and Canadian CMC rainfall files, infiltration (HORTON, MODIFIED_HORTON, GREEN_AMPT, MODIFIED_GREEN_AMPT, CURVE_NUMBER), all eight LID processes, detailed LID reports and pollutant removals, groundwater, snowmelt, RDII assignment, and outfall runon
- hydraulics: conduits, Preissmann-slot surcharge routing, all three inertial damping modes, Darcy-Weisbach force mains, pump curves, analytic and tabular storage, functional and tabular-depth outlets, rating curves, exfiltration, all four divider types, culverts, roadway weirs, orifices, outfalls, uncommon/custom/irregular cross-sections, and on-grade and on-sag street inlets
- operations: control variables, math operators and expressions, rain-gage and time-based premises, time series inflows, DWF, event windows, and detailed result reporting
- interfaces: purpose-built rainfall, runoff, hotstart, RDII, and routing interface readers and writers, legacy text RDII input, plus scratch SAVE/USE roundtrips
- routing modes: DYNWAVE, KINWAVE, and STEADY
- quality: pollutants, land uses, coverages, buildup and washoff function matrices, and treatment expressions

The suite intentionally avoids the largest stress-test models in the source catalog so it stays practical for routine regression runs.
