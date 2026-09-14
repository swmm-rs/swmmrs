"""Typed Python bindings for an isolated EPA SWMM simulation owner.

The package exposes lifecycle management, live project-object views, immutable
snapshots, solver metadata, and typed public failures.
"""

import os as _os
import sys as _sys
import warnings as _warnings
from collections.abc import Iterable as _Iterable
from datetime import datetime as _datetime, timedelta as _timedelta
from importlib.metadata import version as _distribution_version
from pathlib import Path
from types import ModuleType as _ModuleType
from typing import (
    TYPE_CHECKING as _TYPE_CHECKING,
    Final as _Final,
    Literal as _Literal,
    NamedTuple as _NamedTuple,
    Never as _Never,
    Self as _Self,
    SupportsIndex as _SupportsIndex,
    cast as _cast,
    final,
)

from . import enums as _enums, snapshots as _snapshots
from ._swmmrs import (
    solver_build_id as _native_solver_build_id,
    solver_version as _native_solver_version,
)
from .enums import CustomEllipseModel, SimulationState, SolverErrorCode
from .exceptions import (
    ConfigurationDiagnostic,
    ConfigurationError,
    ConfigurationObjectIdentity,
    InternalSimulationError,
    LifecycleError,
    SolverError,
    StaleViewError,
    SwmmError,
    ValidationError,
)

if _TYPE_CHECKING:
    from ._swmmrs import NativeSimulation as _NativeSimulation


__version__: _Final[str] = _distribution_version("swmmrs")
solver_version: _Final[str] = _native_solver_version
solver_build_id: _Final[str] = _native_solver_build_id


from . import objects as _objects

_PathInput = str | _os.PathLike[str]


@final
class SimulationStatus(_NamedTuple):
    """Immutable lifecycle and progress values from one owner acquisition."""

    state: SimulationState
    is_open: bool
    is_started: bool
    configuration_dirty: bool
    save_results: bool
    current_time: _datetime | None
    elapsed_time: _timedelta
    duration: _timedelta
    percent_complete: float
    step_count: int
    report_period_count: int
    warning_count: int


@final
class Simulation:
    """Own an isolated SWMM project lifecycle.

    Parameters
    ----------
    input_path : str or os.PathLike[str]
        Path to an EPA SWMM input file.
    report_path : str or os.PathLike[str], optional
        Destination for the generated text report. When omitted, the binding
        derives a report path from `input_path`.
    output_path : str or os.PathLike[str], optional
        Destination for the binary output file. When omitted, SWMM uses a
        scratch output artifact.

    Notes
    -----
    A `Simulation` owns exactly one Project Generation at a time. Collections
    and object views obtained from it become stale after `close()` or `open()`
    starts a replacement generation.

    Examples
    --------
    Run a project to completion with deterministic cleanup::

        from swmmrs import Simulation

        with Simulation("model.inp") as simulation:
            simulation.execute()
    """

    __slots__ = ("_context_owned", "_owner")

    def __init__(
        self,
        input_path: _PathInput,
        report_path: _PathInput | None = None,
        output_path: _PathInput | None = None,
    ) -> None:
        """Create and open a simulation.

        Parameters
        ----------
        input_path : str or os.PathLike[str]
            Path to the input file to open.
        report_path : str or os.PathLike[str], optional
            Text-report destination.
        output_path : str or os.PathLike[str], optional
            Binary-output destination.

        Raises
        ------
        SolverError
            If the native solver cannot open or validate the project.
        ValidationError
            If paths are invalid or collide.
        """
        from ._swmmrs import NativeSimulation

        self._owner = NativeSimulation(input_path, report_path, output_path)
        self._context_owned = False

    @classmethod
    def _from_owner(cls, owner: "_NativeSimulation") -> _Self:
        """Wrap one native owner without replaying project input."""

        simulation = object.__new__(cls)
        simulation._owner = owner
        simulation._context_owned = False
        return simulation

    def open(
        self,
        input_path: _PathInput,
        report_path: _PathInput | None = None,
        output_path: _PathInput | None = None,
    ) -> None:
        """Open a fresh Project Generation on this closed facade.

        Parameters
        ----------
        input_path : str or os.PathLike[str]
            Path to the input file to open.
        report_path : str or os.PathLike[str], optional
            Text-report destination.
        output_path : str or os.PathLike[str], optional
            Binary-output destination.

        Raises
        ------
        LifecycleError
            If the simulation is not closed or is owned by a context manager.
        SolverError
            If the native solver rejects the new project.
        """

        if self._context_owned:
            raise LifecycleError("open is invalid while a context owns the simulation")
        self._owner.open(input_path, report_path, output_path)

    @property
    def input_path(self) -> Path:
        """Return the absolute input path from the last successful open."""

        return Path(self._owner.input_path)

    @property
    def report_path(self) -> Path:
        """Return the absolute report destination from the last successful open."""

        return Path(self._owner.report_path)

    @property
    def output_path(self) -> Path | None:
        """Return the last retained output path, or ``None`` for scratch output."""

        value = self._owner.output_path
        return None if value is None else Path(value)

    @property
    def state(self) -> SimulationState:
        """Return the authoritative typed owner lifecycle state."""

        return SimulationState(getattr(self._owner, "state"))

    @property
    def status(self) -> SimulationStatus:
        """Return an atomic lifecycle and progress snapshot."""

        values = self._owner.status()
        current = values["current_time"]
        return SimulationStatus(
            state=SimulationState(values["state"]),
            is_open=values["is_open"],
            is_started=values["is_started"],
            configuration_dirty=values["configuration_dirty"],
            save_results=values["save_results"],
            current_time=None if current is None else _datetime(*current),
            elapsed_time=_timedelta(seconds=values["elapsed_seconds"]),
            duration=_timedelta(seconds=values["duration_seconds"]),
            percent_complete=values["percent_complete"],
            step_count=values["step_count"],
            report_period_count=values["report_period_count"],
            warning_count=values["warning_count"],
        )

    @property
    def is_open(self) -> bool:
        """Return whether a Project Generation remains open."""

        return getattr(self._owner, "is_open")

    @property
    def is_started(self) -> bool:
        """Return whether the current run is running or complete."""

        return getattr(self._owner, "is_started")

    @property
    def configuration_dirty(self) -> bool:
        """Return whether accepted configuration needs pre-start preparation."""

        return getattr(self._owner, "configuration_dirty")

    @property
    def warning_count(self) -> int:
        """Return the accumulated solver warning count."""

        return getattr(self._owner, "warning_count")

    @property
    def report_period_count(self) -> int:
        """Return the number of saved binary report periods."""

        return getattr(self._owner, "report_period_count")

    @property
    def start_time(self) -> _datetime:
        """Timezone-naive start time. Setter lifecycle: `OPEN`, `ENDED`."""
        return _datetime(*self._owner.simulation_time("start_time"))

    @start_time.setter
    def start_time(self, value: _datetime) -> None:
        self.update_schedule(start_time=value)

    @property
    def report_start(self) -> _datetime:
        """Timezone-naive report start. Setter lifecycle: `OPEN`, `ENDED`."""
        return _datetime(*self._owner.simulation_time("report_start"))

    @report_start.setter
    def report_start(self, value: _datetime) -> None:
        self.update_schedule(report_start=value)

    @property
    def end_time(self) -> _datetime:
        """Timezone-naive end time. Setter lifecycle: `OPEN`, `ENDED`."""
        return _datetime(*self._owner.simulation_time("end_time"))

    @end_time.setter
    def end_time(self, value: _datetime) -> None:
        self.update_schedule(end_time=value)

    def update_schedule(self, **changes: object) -> None:
        """Atomically update coupled start, report-start, and end dates."""
        self._owner.update_schedule(self._generation(), changes)

    @property
    def current_time(self) -> _datetime:
        """Return the current model wall-clock time."""
        return _datetime(*self._owner.current_time())

    @property
    def elapsed_time(self) -> _timedelta:
        """Return elapsed model time from the configured start."""
        current, start = self._owner.elapsed_times()
        return _datetime(*current) - _datetime(*start)

    @property
    def percent_complete(self) -> float:
        """Return elapsed model duration as a percentage of the configured run."""
        elapsed = self.elapsed_time.total_seconds()
        duration = (self.end_time - self.start_time).total_seconds()
        return min(max(100.0 * elapsed / duration, 0.0), 100.0)

    @property
    def flow_units(self) -> _enums.FlowUnits:
        """Return configured project flow units."""
        return _enums.FlowUnits(self._owner.unit_label("flow_units"))

    @property
    def unit_system(self) -> _enums.UnitSystem:
        """Return configured project unit system."""
        return _enums.UnitSystem(self._owner.unit_label("unit_system"))

    @property
    def options(self) -> _objects.SimulationOptions:
        """Return a generation-bound typed simulation policy view."""
        return _objects._new_options(self, self._generation())

    @property
    def effective_threads(self) -> int:
        """Return the actual caller-inclusive solver team size."""
        return getattr(self._owner, "effective_threads")

    def _generation(self) -> int:
        return getattr(self._owner, "generation")

    @property
    def statistics(self) -> _snapshots.SimulationStatistics:
        """Return one coherent immutable system statistics acquisition."""
        return self._owner.simulation_statistics(self._generation())

    @property
    def rain_gages(self) -> _objects.ObjectCollection[_objects.RainGage]:
        """Return configured rain gages in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.RAIN_GAGE,
            _objects.ObjectCollection,
        )

    @property
    def subcatchments(self) -> _objects.SubcatchmentCollection:
        """Return configured subcatchments in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.SUBCATCHMENT,
            _objects.SubcatchmentCollection,
        )

    @property
    def nodes(self) -> _objects.NodeCollection:
        """Return configured nodes in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.NODE,
            _objects.NodeCollection,
        )

    @property
    def links(self) -> _objects.LinkCollection:
        """Return configured links in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.LINK,
            _objects.LinkCollection,
        )

    @property
    def pollutants(self) -> _objects.ObjectCollection[_objects.Pollutant]:
        """Return configured pollutants in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.POLLUTANT,
            _objects.ObjectCollection,
        )

    @property
    def land_uses(self) -> _objects.ObjectCollection[_objects.LandUse]:
        """Return configured land uses in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.LAND_USE,
            _objects.ObjectCollection,
        )

    @property
    def time_patterns(self) -> _objects.ObjectCollection[_objects.TimePattern]:
        """Return configured time patterns in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.TIME_PATTERN,
            _objects.ObjectCollection,
        )

    @property
    def curves(self) -> _objects.ObjectCollection[_objects.Curve]:
        """Return configured curves in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.CURVE,
            _objects.ObjectCollection,
        )

    @property
    def time_series(self) -> _objects.ObjectCollection[_objects.TimeSeries]:
        """Return configured time series in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.TIME_SERIES,
            _objects.ObjectCollection,
        )

    @property
    def controls(self) -> _objects.ObjectCollection[_objects.ControlRule]:
        """Return configured control rules in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.CONTROL,
            _objects.ObjectCollection,
        )

    @property
    def transects(self) -> _objects.ObjectCollection[_objects.Transect]:
        """Return configured transects in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.TRANSECT,
            _objects.ObjectCollection,
        )

    @property
    def aquifers(self) -> _objects.ObjectCollection[_objects.Aquifer]:
        """Return configured aquifers in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.AQUIFER,
            _objects.ObjectCollection,
        )

    @property
    def amm_models(self) -> _objects.ObjectCollection[_objects.AmmModel]:
        """Return configured AMM models in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.AMM_MODEL,
            _objects.ObjectCollection,
        )

    @property
    def amm_assignments(self) -> tuple[_objects.AmmAssignment, ...]:
        """Return detached AMM node assignments in configured order."""
        values = self._owner.amm_assignments(self, self._generation())
        return tuple(
            _objects.AmmAssignment(
                _cast(_objects.Node, node),
                _cast(_objects.AmmModel, model),
                area,
            )
            for node, model, area in values
        )

    @amm_assignments.setter
    def amm_assignments(self, value: _Iterable[_objects.AmmAssignment]) -> None:
        self.replace_amm_assignments(value)

    def replace_amm_assignments(self, assignments: _Iterable[_objects.AmmAssignment]) -> None:
        """Atomically replace all AMM node assignments in project units."""
        try:
            assignments = tuple(assignments)
        except TypeError:
            raise TypeError("assignments must be iterable") from None
        if not all(isinstance(assignment, _objects.AmmAssignment) for assignment in assignments):
            raise TypeError("assignments must contain AmmAssignment values")
        self._owner.replace_amm_assignments(
            self._generation(),
            tuple(
                (assignment.node, assignment.model, assignment.area) for assignment in assignments
            ),
        )

    @property
    def unit_hydrographs(
        self,
    ) -> _objects.ObjectCollection[_objects.UnitHydrograph]:
        """Return configured unit hydrographs in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.UNIT_HYDROGRAPH,
            _objects.ObjectCollection,
        )

    @property
    def rdii_assignments(self) -> tuple[_objects.RdiiAssignment, ...]:
        """Return detached RTK RDII node assignments in node order."""
        values = self._owner.rdii_assignments(self, self._generation())
        return tuple(
            _objects.RdiiAssignment(
                _cast(_objects.Node, node),
                _cast(_objects.UnitHydrograph, unit_hydrograph),
                area,
            )
            for node, unit_hydrograph, area in values
        )

    @rdii_assignments.setter
    def rdii_assignments(self, value: _Iterable[_objects.RdiiAssignment]) -> None:
        self.replace_rdii_assignments(value)

    def replace_rdii_assignments(self, assignments: _Iterable[_objects.RdiiAssignment]) -> None:
        """Atomically replace all RTK RDII node assignments in project units."""
        try:
            assignments = tuple(assignments)
        except TypeError:
            raise TypeError("assignments must be iterable") from None
        if not all(isinstance(assignment, _objects.RdiiAssignment) for assignment in assignments):
            raise TypeError("assignments must contain RdiiAssignment values")
        self._owner.replace_rdii_assignments(
            self._generation(),
            tuple(
                (assignment.node, assignment.unit_hydrograph, assignment.area)
                for assignment in assignments
            ),
        )

    @property
    def snowmelt_sets(
        self,
    ) -> _objects.ObjectCollection[_objects.SnowmeltParameterSet]:
        """Return configured snowmelt parameter sets in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.SNOWMELT_SET,
            _objects.ObjectCollection,
        )

    @property
    def shapes(self) -> _objects.ObjectCollection[_objects.CustomShape]:
        """Return configured custom shapes in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.SHAPE,
            _objects.ObjectCollection,
        )

    @property
    def lid_controls(self) -> _objects.ObjectCollection[_objects.LidControl]:
        """Return configured LID controls in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.LID_CONTROL,
            _objects.ObjectCollection,
        )

    @property
    def streets(self) -> _objects.ObjectCollection[_objects.Street]:
        """Return configured streets in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.STREET,
            _objects.ObjectCollection,
        )

    @property
    def inlet_designs(self) -> _objects.ObjectCollection[_objects.InletDesign]:
        """Return configured inlet designs in project order."""
        return _objects._new_collection(
            self,
            self._generation(),
            _objects._ObjectFamily.INLET_DESIGN,
            _objects.ObjectCollection,
        )

    def use_hotstart(self, path: _PathInput | None) -> None:
        """Set or clear persistent hotstart input for this Project Generation.

        Parameters
        ----------
        path : str or os.PathLike[str] or None
            EPA hotstart file to load before the next `start()`, or `None` to
            clear the configured hotstart input.

        Raises
        ------
        LifecycleError
            If called outside the open or ended state.
        ValidationError
            If `path` collides with a simulation-owned artifact.
        """

        self._owner.use_hotstart(path)

    def save_hotstart(self, path: _PathInput) -> None:
        """Write current state in EPA hotstart format and wait until it is usable.

        Parameters
        ----------
        path : str or os.PathLike[str]
            Destination for the EPA hotstart file.

        Raises
        ------
        LifecycleError
            If the simulation is not running or complete.
        ValidationError
            If `path` collides with a simulation-owned artifact.
        SolverError
            If SWMM cannot write the checkpoint.
        """

        self._owner.save_hotstart(path)

    def save_checkpoint(self, path: _PathInput) -> None:
        """Save a deterministic Simulation Checkpoint at the current boundary.

        Parameters
        ----------
        path : str or os.PathLike[str]
            Checkpoint manifest destination. Saving creates or atomically
            replaces the manifest; immutable sidecars are published beside it.

        Raises
        ------
        LifecycleError
            If the simulation is not running or complete.
        ValidationError
            If `path` collides with a simulation-owned artifact.
        SolverError
            If checkpoint capture or atomic publication fails.
        """

        self._owner.save_checkpoint(path)

    @classmethod
    def resume(
        cls,
        checkpoint_path: _PathInput,
        report_path: _PathInput,
        output_path: _PathInput,
    ) -> _Self:
        """Resume an independent owner from an immutable Simulation Checkpoint.

        Parameters
        ----------
        checkpoint_path : str or os.PathLike[str]
            Existing checkpoint manifest.
        report_path : str or os.PathLike[str]
            Fresh report destination.
        output_path : str or os.PathLike[str]
            Fresh binary-output destination.

        Returns
        -------
        Simulation
            Independent owner restored at the saved lifecycle boundary.

        Notes
        -----
        Checkpoints restore prepared state, runtime continuation, and editable
        declarations, so the resumed owner can later be ended, edited, and
        rerun.
        """

        from ._swmmrs import NativeSimulation

        owner = NativeSimulation.resume(checkpoint_path, report_path, output_path)
        return cls._from_owner(owner)

    def load_checkpoint_state(self, checkpoint_path: _PathInput) -> None:
        """Import checkpoint physical continuation state into this open owner.

        The receiver retains its dates, inputs, declarations, statistics,
        accounting, iterator cadence, and output artifacts. Compatible physical
        state and persistent forcing are committed immediately and retained so
        the next `start()` can reapply them after processor initialization.
        """

        self._owner.load_checkpoint_state(checkpoint_path)

    def fork(
        self,
        report_path: _PathInput,
        output_path: _PathInput,
    ) -> _Self:
        """Create an independent child owner at the current lifecycle boundary.

        The child receives fresh report/output destinations and a clone of the
        source's retained editable declarations.
        """

        owner = self._owner.fork(report_path, output_path)
        return type(self)._from_owner(owner)

    def start(self, *, save_results: bool = True) -> None:
        """Initialize an open or ended project and enter the running state.

        Parameters
        ----------
        save_results : bool, default=True
            Whether to save report-period results to the binary output.

        Raises
        ------
        LifecycleError
            If the project is neither open nor ended, or iteration owns
            advancement.
        ValidationError
            If `save_results` is not a bool.
        ConfigurationError
            If deferred Configuration Preparation reports ordered relational
            diagnostics. Requested declarations remain available for repair.
        SolverError
            If SWMM cannot initialize the run.
        """

        self._owner.start(save_results)

    def reset_solver(self) -> None:
        """Discard stale run state without reopening or validating the project.

        Raises
        ------
        LifecycleError
            If the simulation is not open or ended, or iteration owns advancement.
        SolverError
            If stale output state cannot be released.
        """

        self._owner.reset_solver()

    def sleep_workers(self) -> None:
        """
        Park Dynamic Wave workers until the next routing operation,
        including iterator advancement.
        """

        self._owner.sleep_workers()

    def step(self) -> _datetime | None:
        """Advance one routing step.

        Returns
        -------
        datetime.datetime or None
            The new model time, or `None` after the final step.

        Raises
        ------
        LifecycleError
            If the simulation is not running.
        """

        result = self._owner.step()
        return None if result is None else _datetime(*result)

    def stride(
        self,
        duration: _timedelta | float,
        *,
        strict: bool = True,
    ) -> _datetime | None:
        """Advance toward a positive whole-second interval.

        Parameters
        ----------
        duration : datetime.timedelta or float
            Target advance in seconds.
        strict : bool, default=True
            End exactly at the requested interval when true. When false,
            advance with ordinary routing steps until reaching or passing the
            target.

        Returns
        -------
        datetime.datetime or None
            The new model time, or `None` after the final step.

        Raises
        ------
        ValidationError
            If `duration` is not a positive whole-second duration or `strict`
            is not a bool.
        """

        result = self._owner.stride(duration, strict)
        return None if result is None else _datetime(*result)

    def step_advance(
        self,
        duration: _timedelta | float | None,
        *,
        strict: bool = True,
    ) -> None:
        """Set or clear the cadence used by ordinary iteration.

        Parameters
        ----------
        duration : datetime.timedelta or float or None
            Positive whole-second cadence, or `None` to iterate at SWMM's
            normal routing step.
        strict : bool, default=True
            End exactly at each interval when true, potentially shortening the
            final routing step. When false, advance with ordinary routing steps
            until reaching or passing the target. Ignored when `duration` is
            `None`.

        Raises
        ------
        LifecycleError
            If the project is closed.
        ValidationError
            If `duration` is invalid or `strict` is not a bool.
        """

        self._owner.step_advance(duration, strict)

    def __iter__(self) -> _Self:
        """Return this simulation's single lifecycle iterator."""

        return self

    def __next__(self) -> _datetime:
        """Advance the exclusively owned iterator or stop at its checkpoint."""

        result = self._owner.iterator_next()
        if result is None:
            raise StopIteration
        return _datetime(*result)

    def terminate(self) -> None:
        """Request orderly termination at the next iterator checkpoint."""

        self._owner.terminate()

    def execute(self, *, save_results: bool = True) -> None:
        """Run the complete non-interactive lifecycle and close the project.

        Parameters
        ----------
        save_results : bool, default=True
            Whether to save report-period results to the binary output.

        Raises
        ------
        SolverError
            If the solver rejects any lifecycle stage.
        ValidationError
            If `save_results` is not a bool or deferred Configuration
            Preparation rejects the requested declarations. Batch cleanup closes
            the owner; use explicit `start()` for structured diagnostic
            repair/retry workflows.
        """

        self._owner.execute(save_results)

    def end(self) -> None:
        """Finalize the current run while retaining its project generation.

        Raises
        ------
        LifecycleError
            If the simulation has not been started.
        """

        self._owner.end()

    def report(self) -> None:
        """Generate and flush the detailed report for an ended run.

        Raises
        ------
        LifecycleError
            If the simulation is not ended.
        SolverError
            If SWMM cannot write the report.
        """

        self._owner.report()

    def close(self) -> None:
        """Close the project and finalize owned artifacts.

        Notes
        -----
        Closing invalidates every collection and live view from the current
        Project Generation.
        """

        self._owner.close()

    def __del__(self) -> None:
        """Warn before the native owner performs best-effort destruction."""

        try:
            owner = self._owner
        except BaseException:
            return
        try:
            needs_cleanup = owner.is_open
        except BaseException:
            needs_cleanup = True
        if not needs_cleanup:
            return
        try:
            _warnings.warn(
                "unclosed Simulation; use close() or a context manager to finalize artifacts",
                ResourceWarning,
                stacklevel=2,
            )
        except BaseException:
            pass

    def __enter__(self) -> _Self:
        """Enter one cleanup scope without starting the open project."""

        if self._context_owned or self.state not in (
            SimulationState.OPEN,
            SimulationState.ENDED,
        ):
            raise LifecycleError(f"context entry is invalid while simulation is {self.state}")
        self._context_owned = True
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: object,
    ) -> _Literal[False]:
        """Finalize a started run and close without replacing a body exception."""

        cleanup_error: SwmmError | None = None
        try:
            try:
                self.close()
            except SwmmError as error:
                cleanup_error = error
        finally:
            self._context_owned = False

        if cleanup_error is not None:
            if exc_value is None:
                raise cleanup_error
            exc_value.add_note(
                f"Simulation cleanup failed with {type(cleanup_error).__name__}: {cleanup_error}"
            )
        return False

    def __copy__(self) -> _Never:
        """Reject copying Simulation Owner identity."""

        raise TypeError("Simulation cannot be copied")

    def __deepcopy__(self, memo: object) -> _Never:
        """Reject deep-copying Simulation Owner identity."""

        raise TypeError("Simulation cannot be deep-copied")

    def __reduce__(self) -> _Never:
        """Reject pickling Simulation Owner identity."""

        raise TypeError("Simulation cannot be pickled")

    def __reduce_ex__(self, protocol: _SupportsIndex) -> _Never:
        """Reject protocol-specific pickling of Simulation Owner identity."""

        raise TypeError("Simulation cannot be pickled")


from ._swmmrs import _bind_public_types as _native_bind_public_types

_native_bind_public_types()
del _native_bind_public_types


__all__ = (
    "ConfigurationDiagnostic",
    "CustomEllipseModel",
    "ConfigurationError",
    "ConfigurationObjectIdentity",
    "InternalSimulationError",
    "LifecycleError",
    "Simulation",
    "SimulationState",
    "SimulationStatus",
    "SolverError",
    "SolverErrorCode",
    "StaleViewError",
    "SwmmError",
    "ValidationError",
    "__version__",
    "solver_build_id",
    "solver_version",
)

_IMMUTABLE_METADATA = frozenset(("__version__", "solver_version", "solver_build_id"))
globals().pop("_swmmrs", None)


class _PackageModule(_ModuleType):
    """Protect immutable package and embedded-solver identity."""

    def __setattr__(self, name: str, value: object) -> None:
        if name in _IMMUTABLE_METADATA:
            raise AttributeError(f"{name} is immutable")
        super().__setattr__(name, value)

    def __delattr__(self, name: str) -> None:
        if name in _IMMUTABLE_METADATA:
            raise AttributeError(f"{name} is immutable")
        super().__delattr__(name)


_sys.modules[__name__].__class__ = _PackageModule
