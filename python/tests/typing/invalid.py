# This fixture intentionally fails mypy; suppress duplicate editor diagnostics.
# pyright: reportGeneralTypeIssues=false, reportAttributeAccessIssue=false, reportAssignmentType=false, reportIndexIssue=false

from __future__ import annotations

from datetime import timedelta

from swmmrs import Simulation, SolverError
from swmmrs.objects import Conduit, Junction, LidControl, LidSurfaceLayer, Pump, StorageNode
from swmmrs.snapshots import NodeSnapshot


class UnsupportedSimulationSubclass(Simulation):
    pass


class UnsupportedJunctionSubclass(Junction):
    pass


def misuse_public_contract(
    simulation: Simulation,
    snapshot: NodeSnapshot,
    error: SolverError,
) -> None:
    simulation.start_time = timedelta(seconds=1)
    simulation.options.requested_threads = 1.5
    snapshot.depth = ()
    storage: StorageNode = simulation.nodes["J1"]
    storage.shape = None
    storage.update(shape=None)
    junction: Junction = simulation.nodes["J1"]
    junction.full_depth = 5.0
    junction.update(full_depth=5.0)
    junction.pollut_quality = ("TSS", 1.0)
    junction.external_pollutant_mass_flux = ("TSS", 1.0)
    junction.external_pollutant_mass_flux[1] = 1.0  # pyright: ignore[reportArgumentType]
    junction.override_pollutant_concentrations({1.5: 1.0})  # pyright: ignore[reportArgumentType]
    control: LidControl = simulation.lid_controls["BIO"]
    surface: LidSurfaceLayer = control.surface
    error.code = "memory"
    subcatchment = simulation.subcatchments["S-A"]
    settings = subcatchment.settings
    subcatchment.rain_scale_factor = 1.0
    subcatchment.external_pollutant_buildup_increment = ("TSS", 1.0)
    simulation.rain_gages["RG1"].external_precipitation_rate = 1.0
    settings.infiltration = None
    settings.initial_buildup[1] = 2.5  # pyright: ignore[reportArgumentType]
    settings.initial_buildup.copy()
    conduit: Conduit = simulation.links["L1"]
    conduit.settings.length = "100"
    conduit.length = 100.0
    pump: Pump = simulation.links["P1"]
    pump.settings.curve = simulation.curves["P1"]
    pump.settings.use_curve(simulation.nodes["J1"])  # pyright: ignore[reportArgumentType]
