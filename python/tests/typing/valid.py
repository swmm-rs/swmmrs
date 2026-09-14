from __future__ import annotations

from collections.abc import Mapping, MutableMapping
from datetime import datetime, timedelta
from pathlib import Path
from typing import assert_type

from swmmrs import Simulation, SimulationStatus, SolverError
from swmmrs.enums import (
    FlowUnits,
    OrificeKind,
    OutletHeadBasis,
    SimulationState,
    SolverErrorCode,
    StandardCrossSectionShape,
)
from swmmrs.objects import (
    CircularCrossSection,
    Conduit,
    ConduitSettings,
    Curve,
    Divider,
    DividerRule,
    DividerSettings,
    FunctionalOutletRating,
    LidControl,
    LidSurfaceLayer,
    Node,
    NodeCollection,
    NodeSettings,
    ObjectCollection,
    Orifice,
    OrificeSettings,
    Outfall,
    OutfallBoundary,
    OutfallSettings,
    Pollutant,
    Pump,
    PumpSettings,
    RainGage,
    RdiiAssignment,
    SnowmeltParameterSet,
    SnowmeltSurface,
    StandardCrossSection,
    StorageExfiltration,
    StorageNode,
    StorageSettings,
    StorageShape,
    SubcatchmentGroundwater,
    SubcatchmentInfiltration,
    SubcatchmentSettings,
    TabularOutletRating,
    TimePattern,
    UnitHydrograph,
    UnitHydrographMonth,
    UnitHydrographResponse,
)
from swmmrs.output import (
    BulkSeriesResult,
    ConcentrationUnits,
    FlowUnits as OutputFlowUnits,
    LinkKind,
    LinkResultAttribute,
    NodeKind,
    NodeResultAttribute,
    OutputMetadata,
    OutputName,
    OutputReader,
    OutputTimeSeries,
    OutputValueSeries,
    PollutantAttribute,
    ResultAttributeCode,
    ResultElementType,
    ResultSchema,
    RunStatus,
    SeriesSelection,
    SubcatchmentResultAttribute,
    SystemResultAttribute,
    UnitSystem as OutputUnitSystem,
    UnknownCode,
)
from swmmrs.snapshots import NodeSnapshot, SimulationStatistics, StorageStatistics


def inspect_public_contract(simulation: Simulation, error: SolverError) -> None:
    assert_type(simulation.state, SimulationState)
    assert_type(simulation.status, SimulationStatus)
    assert_type(simulation.flow_units, FlowUnits)
    assert_type(simulation.current_time, datetime)
    assert_type(simulation.percent_complete, float)
    assert_type(simulation.nodes, NodeCollection)
    assert_type(simulation.pollutants, ObjectCollection[Pollutant])
    aquifer = simulation.aquifers["Aquifer-A"]
    assert_type(aquifer.porosity, float)
    aquifer.porosity = 0.5
    aquifer.update(porosity=0.5, field_capacity=0.35)
    assert_type(aquifer.upper_evaporation_pattern, TimePattern | None)
    aquifer.upper_evaporation_pattern = None
    snowmelt: SnowmeltParameterSet = simulation.snowmelt_sets["Snow-A"]
    assert_type(snowmelt.plowable_fraction, float)
    snowmelt.update(plowable_fraction=0.5)
    plowable: SnowmeltSurface = snowmelt.plowable
    assert_type(plowable.minimum_melt_coefficient, float)
    plowable.update(minimum_melt_coefficient=0.01)
    unit_hydrograph: UnitHydrograph = simulation.unit_hydrographs["RTK"]
    assert_type(unit_hydrograph.rain_gage, RainGage)
    responses = unit_hydrograph.monthly_responses
    assert_type(responses, tuple[UnitHydrographMonth, ...])
    assert_type(responses[0].short, UnitHydrographResponse)
    assert_type(responses[0].medium, UnitHydrographResponse)
    assert_type(responses[0].long, UnitHydrographResponse)
    responses[0].short.rainfall_fraction = 0.1
    unit_hydrograph.monthly_responses = responses
    assert_type(simulation.rdii_assignments, tuple[RdiiAssignment, ...])
    assignment = RdiiAssignment(simulation.nodes["J1"], unit_hydrograph, 1.0)
    simulation.rdii_assignments = (assignment,)
    subcatchment = simulation.subcatchments["S-A"]
    settings = subcatchment.settings
    assert_type(settings, SubcatchmentSettings)
    assert_type(settings.width, float)
    settings.width = 125.0
    settings.initial_buildup["TSS"] = 2.5
    settings.initial_buildup.update({"TSS": 3.0})
    infiltration = settings.infiltration
    assert_type(infiltration, SubcatchmentInfiltration | None)
    assert infiltration is not None
    infiltration.initial_rate = 3.5
    assert_type(settings.groundwater, SubcatchmentGroundwater | None)
    assert_type(settings.snowpack, SnowmeltParameterSet | None)
    assert_type(subcatchment.external_pollutant_buildup_increment, MutableMapping[str, float])
    subcatchment.external_pollutant_buildup_increment["TSS"] = 1.0
    subcatchment.external_pollutant_buildup_increment.update({"TSS": 2.0})
    subcatchment.set_precipitation_scale_factors(rainfall=1.0, snowfall=1.0)
    gage = simulation.rain_gages["RG1"]
    gage.use_external_precipitation(rate=0.0)
    assert_type(simulation.nodes.snapshot(), NodeSnapshot)
    assert_type(simulation.statistics, SimulationStatistics)

    node = simulation.nodes["J1"]
    assert_type(node, Node)
    node_settings = node.settings
    assert_type(node_settings, NodeSettings)
    node_settings.update(invert_elevation=10.0, full_depth=5.0)
    assert_type(node_settings.invert_elevation, float)
    assert_type(node.pollut_quality, Mapping[str, float])
    assert_type(node.external_pollutant_mass_flux, MutableMapping[str, float])
    node.external_pollutant_mass_flux["TSS"] = 1.0
    node.external_pollutant_mass_flux.update({"Count": 2.0})
    node.override_pollutant_concentrations({"TSS": 3.0})
    if isinstance(node, StorageNode):
        storage_settings = node.settings
        assert_type(storage_settings, StorageSettings)
        assert_type(storage_settings.shape, StorageShape)
        assert_type(storage_settings.exfiltration, StorageExfiltration | None)
        assert_type(node.storage_statistics, StorageStatistics)
        storage_settings.update(evaporation_fraction=0.5)
    if isinstance(node, Outfall):
        outfall_settings = node.settings
        assert_type(outfall_settings, OutfallSettings)
        assert_type(outfall_settings.boundary, OutfallBoundary)
        outfall_settings.update(has_flap_gate=True)
    if isinstance(node, Divider):
        divider_settings = node.settings
        assert_type(divider_settings, DividerSettings)
        assert_type(divider_settings.rule, DividerRule)
        divider_settings.update(rule=DividerRule("overflow"))

    link = simulation.links["L1"]
    assert_type(link.pollut_quality, Mapping[str, float])
    assert_type(link.external_pollutant_mass_flux, MutableMapping[str, float])
    link.external_pollutant_mass_flux["TSS"] = 1.0
    link.override_pollutant_concentrations({"TSS": 2.0})
    if isinstance(link, Conduit):
        conduit_settings = link.settings
        assert_type(conduit_settings, ConduitSettings)
        assert_type(conduit_settings.length, float)
        conduit_settings.length = 100.0
        conduit_settings.cross_section = CircularCrossSection(3.0)
        conduit_settings.update(
            roughness=0.013,
            cross_section=StandardCrossSection(
                StandardCrossSectionShape.CIRCULAR,
                (3.0, 0.0, 0.0, 0.0),
            ),
        )
    if isinstance(link, Pump):
        pump_settings = link.settings
        assert_type(pump_settings, PumpSettings)
        assert_type(pump_settings.curve, Curve | None)
        pump_settings.use_curve(simulation.curves["P1"])
        pump_settings.use_ideal()
    if isinstance(link, Orifice):
        orifice_settings = link.settings
        assert_type(orifice_settings, OrificeSettings)
        orifice_settings.kind = OrificeKind.SIDE
    outlet = FunctionalOutletRating(2.0, 1.5, OutletHeadBasis.HEAD)
    assert_type(outlet, FunctionalOutletRating)
    tabular = TabularOutletRating(simulation.curves["Rating"], OutletHeadBasis.DEPTH)
    assert_type(tabular, TabularOutletRating)

    control: LidControl = simulation.lid_controls["BIO"]
    surface = control.surface
    if surface is not None:
        assert_type(surface, LidSurfaceLayer)
        surface.update(thickness=6.0, roughness=0.1)
    lid_unit = simulation.subcatchments["S-A"].lid_units[0]
    assert_type(lid_unit.area, float)
    lid_unit.update(area=50.0, control=control, drain_destination=None)

    simulation.start_time = datetime(2026, 1, 1)
    simulation.update_schedule(
        start_time=datetime(2026, 1, 1),
        report_start=datetime(2026, 1, 1),
        end_time=datetime(2026, 1, 2),
    )
    simulation.options.routing_step = timedelta(seconds=30)
    simulation.options.requested_threads = 2
    simulation.options.update(
        routing_step=timedelta(seconds=20),
        requested_threads=2,
    )
    simulation.save_checkpoint(Path("state.json"))
    assert_type(
        Simulation.resume("state.json", "resumed.rpt", "resumed.out"),
        Simulation,
    )
    simulation.load_checkpoint_state("state.json")
    assert_type(simulation.fork("child.rpt", "child.out"), Simulation)
    assert_type(error.operation, str)
    assert_type(error.code, SolverErrorCode)
    assert_type(error.native_code, int | None)
    assert_type(error.detail, str | None)
    assert_type(error.semantic_code, str)


def inspect_output_types() -> None:
    output_flow_units: OutputFlowUnits = OutputFlowUnits.CFS
    assert_type(OutputFlowUnits(output_flow_units), OutputFlowUnits)
    output_unit_system: OutputUnitSystem = OutputUnitSystem.US
    assert_type(OutputUnitSystem(output_unit_system), OutputUnitSystem)
    concentration_units: ConcentrationUnits = ConcentrationUnits.COUNTS_PER_LITER
    assert_type(ConcentrationUnits(concentration_units), ConcentrationUnits)
    node_kind: NodeKind = NodeKind.JUNCTION
    assert_type(NodeKind(node_kind), NodeKind)
    link_kind: LinkKind = LinkKind.CONDUIT
    assert_type(LinkKind(link_kind), LinkKind)
    result_element_type: ResultElementType = ResultElementType.NODE
    assert_type(ResultElementType(result_element_type), ResultElementType)
    subcatchment_attribute: SubcatchmentResultAttribute = SubcatchmentResultAttribute.RAINFALL
    assert_type(SubcatchmentResultAttribute(subcatchment_attribute), SubcatchmentResultAttribute)
    node_attribute: NodeResultAttribute = NodeResultAttribute.INVERT_DEPTH
    assert_type(NodeResultAttribute(node_attribute), NodeResultAttribute)
    link_attribute: LinkResultAttribute = LinkResultAttribute.FLOW_RATE
    assert_type(LinkResultAttribute(link_attribute), LinkResultAttribute)
    system_attribute: SystemResultAttribute = SystemResultAttribute.RAINFALL
    assert_type(SystemResultAttribute(system_attribute), SystemResultAttribute)
    assert_type(UnknownCode(-1).code, int)
    assert_type(ResultAttributeCode(-1).code, int)
    assert_type(PollutantAttribute(0).selector, int | str | bytes | OutputName)
    assert_type(OutputTimeSeries, type[OutputTimeSeries])
    assert_type(BulkSeriesResult, type[BulkSeriesResult])
    reader = OutputReader("model.out")
    selection = SeriesSelection("system", None, system_attribute)
    result = reader.read_bulk_series(
        (selection,),
        start=0,
        end=0,
        low_memory=True,
    )
    assert_type(result, BulkSeriesResult)
    assert_type(
        reader.subcatchment_series(
            0,
            SubcatchmentResultAttribute.RAINFALL,
            low_memory=True,
        ),
        OutputTimeSeries,
    )
    assert_type(
        reader.node_series("N1", NodeResultAttribute.INVERT_DEPTH, low_memory=True),
        OutputTimeSeries,
    )
    assert_type(
        reader.link_series(b"L1", LinkResultAttribute.FLOW_RATE, low_memory=True),
        OutputTimeSeries,
    )
    assert_type(reader.system_series("rainfall", low_memory=True), OutputTimeSeries)
    assert_type(result.times, tuple[datetime, ...])
    assert_type(result.series, tuple[OutputValueSeries, ...])
    assert_type(result.value(0, 0), float)
    assert_type(reader.read_stored_dates(start=0, end=0), list[float])
    assert_type(reader.times, tuple[datetime, ...])
    assert_type(reader.metadata.report_timing.nominal_date(0), datetime | None)
    assert_type(OutputName(b"name").text, str | None)
    assert_type(ResultSchema, type[ResultSchema])
    assert_type(OutputMetadata, type[OutputMetadata])
    assert_type(RunStatus(0).is_success, bool)
