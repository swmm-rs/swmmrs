"""Simulation option view."""

from __future__ import annotations as _annotations

from datetime import date as _date, timedelta as _timedelta
from typing import (
    TYPE_CHECKING as _TYPE_CHECKING,
    Never as _Never,
    SupportsIndex as _SupportsIndex,
    cast as _cast,
    final as _final,
)

from ..enums import (
    CustomEllipseModel as _CustomEllipseModel,
    InertiaDamping as _InertiaDamping,
    NormalFlowLimit as _NormalFlowLimit,
    SurchargeMethod as _SurchargeMethod,
)
from ._base import _OPTIONS_TOKEN

if _TYPE_CHECKING:
    from .. import Simulation


@_final
class SimulationOptions:
    """Expose typed numerical and analysis policy for one Project Generation.

    Notes
    -----
    Duration options use `datetime.timedelta`. Native Rust owns extraction,
    local validation, declaration commits, deferred preparation, lifecycle, and
    atomic effective-state commit. Requested values remain readable while the
    owner is dirty; effective normalization occurs at `Simulation.start()`.
    """

    __slots__ = ("_generation", "_simulation")

    def __init__(self, simulation: Simulation, generation: int, token: object = None) -> None:
        if token is not _OPTIONS_TOKEN:
            raise TypeError("SimulationOptions cannot be constructed directly")
        self._simulation = simulation
        self._generation = generation

    def _value(self, name: str) -> object:
        return self._simulation._owner.option_value(self._generation, name)

    @property
    def custom_ellipse_model(self) -> _CustomEllipseModel:
        """Hydraulic model for custom ellipses. Setter lifecycle: `OPEN`, `ENDED`."""
        return _CustomEllipseModel(str(self._value("custom_ellipse_model")))

    @custom_ellipse_model.setter
    def custom_ellipse_model(self, value: _CustomEllipseModel | str) -> None:
        self.update(custom_ellipse_model=value)

    def update(self, **changes: object) -> None:
        """Atomically update mutable simulation policy fields."""
        self._simulation._owner.update_options(self._generation, changes)

    @staticmethod
    def _sweep_day(day_of_year: int) -> tuple[int, int]:
        value = _date(2001, 1, 1) + _timedelta(days=day_of_year - 1)
        return value.month, value.day

    @property
    def routing_step(self) -> _timedelta:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _timedelta(seconds=float(_cast(int | float, self._value("routing_step"))))

    @routing_step.setter
    def routing_step(self, value: _timedelta) -> None:
        self.update(routing_step=value)

    @property
    def maximum_routing_step(self) -> _timedelta:
        """Return the effective maximum routing step as a duration."""
        return _timedelta(seconds=self._simulation._owner.maximum_routing_step(self._generation))

    @property
    def report_step(self) -> _timedelta:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _timedelta(seconds=_cast(int, self._value("report_step")))

    @report_step.setter
    def report_step(self, value: _timedelta) -> None:
        self.update(report_step=value)

    @property
    def detailed_reporting_enabled(self) -> bool:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return bool(self._value("detailed_reporting_enabled"))

    @detailed_reporting_enabled.setter
    def detailed_reporting_enabled(self, value: bool) -> None:
        self.update(detailed_reporting_enabled=value)

    @property
    def surcharge_method(self) -> _SurchargeMethod:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _SurchargeMethod(str(self._value("surcharge_method")))

    @surcharge_method.setter
    def surcharge_method(self, value: _SurchargeMethod | str) -> None:
        self.update(surcharge_method=value)

    @property
    def allow_ponding(self) -> bool:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return bool(self._value("allow_ponding"))

    @allow_ponding.setter
    def allow_ponding(self, value: bool) -> None:
        self.update(allow_ponding=value)

    @property
    def inertia_damping(self) -> _InertiaDamping:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _InertiaDamping(str(self._value("inertia_damping")))

    @inertia_damping.setter
    def inertia_damping(self, value: _InertiaDamping | str) -> None:
        self.update(inertia_damping=value)

    @property
    def normal_flow_limit(self) -> _NormalFlowLimit:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _NormalFlowLimit(str(self._value("normal_flow_limit")))

    @normal_flow_limit.setter
    def normal_flow_limit(self, value: _NormalFlowLimit | str) -> None:
        self.update(normal_flow_limit=value)

    @property
    def skip_steady_state(self) -> bool:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return bool(self._value("skip_steady_state"))

    @skip_steady_state.setter
    def skip_steady_state(self, value: bool) -> None:
        self.update(skip_steady_state=value)

    @property
    def rainfall_enabled(self) -> bool:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return bool(self._value("rainfall_enabled"))

    @rainfall_enabled.setter
    def rainfall_enabled(self, value: bool) -> None:
        self.update(rainfall_enabled=value)

    @property
    def rdii_enabled(self) -> bool:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return bool(self._value("rdii_enabled"))

    @rdii_enabled.setter
    def rdii_enabled(self, value: bool) -> None:
        self.update(rdii_enabled=value)

    @property
    def snowmelt_enabled(self) -> bool:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return bool(self._value("snowmelt_enabled"))

    @snowmelt_enabled.setter
    def snowmelt_enabled(self, value: bool) -> None:
        self.update(snowmelt_enabled=value)

    @property
    def groundwater_enabled(self) -> bool:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return bool(self._value("groundwater_enabled"))

    @groundwater_enabled.setter
    def groundwater_enabled(self, value: bool) -> None:
        self.update(groundwater_enabled=value)

    @property
    def routing_enabled(self) -> bool:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return bool(self._value("routing_enabled"))

    @routing_enabled.setter
    def routing_enabled(self, value: bool) -> None:
        self.update(routing_enabled=value)

    @property
    def quality_enabled(self) -> bool:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return bool(self._value("quality_enabled"))

    @quality_enabled.setter
    def quality_enabled(self, value: bool) -> None:
        self.update(quality_enabled=value)

    @property
    def rule_step(self) -> _timedelta:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _timedelta(seconds=_cast(int, self._value("rule_step")))

    @rule_step.setter
    def rule_step(self, value: _timedelta) -> None:
        self.update(rule_step=value)

    @property
    def sweep_start(self) -> tuple[int, int]:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return self._sweep_day(_cast(int, self._value("sweep_start")))

    @sweep_start.setter
    def sweep_start(self, value: tuple[int, int]) -> None:
        self.update(sweep_start=value)

    @property
    def sweep_end(self) -> tuple[int, int]:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return self._sweep_day(_cast(int, self._value("sweep_end")))

    @sweep_end.setter
    def sweep_end(self, value: tuple[int, int]) -> None:
        self.update(sweep_end=value)

    @property
    def maximum_trials(self) -> int:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(int, self._value("maximum_trials"))

    @maximum_trials.setter
    def maximum_trials(self, value: int) -> None:
        self.update(maximum_trials=value)

    @property
    def requested_threads(self) -> int:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _cast(int, self._value("requested_threads"))

    @requested_threads.setter
    def requested_threads(self, value: int) -> None:
        self.update(requested_threads=value)

    @property
    def minimum_routing_step(self) -> _timedelta:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _timedelta(seconds=float(_cast(int | float, self._value("minimum_routing_step"))))

    @minimum_routing_step.setter
    def minimum_routing_step(self, value: _timedelta) -> None:
        self.update(minimum_routing_step=value)

    @property
    def lengthening_step(self) -> _timedelta:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _timedelta(seconds=float(_cast(int | float, self._value("lengthening_step"))))

    @lengthening_step.setter
    def lengthening_step(self, value: _timedelta) -> None:
        self.update(lengthening_step=value)

    @property
    def antecedent_dry_duration(self) -> _timedelta:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return _timedelta(seconds=float(_cast(int | float, self._value("antecedent_dry_duration"))))

    @antecedent_dry_duration.setter
    def antecedent_dry_duration(self, value: _timedelta) -> None:
        self.update(antecedent_dry_duration=value)

    @property
    def courant_factor(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return float(_cast(int | float, self._value("courant_factor")))

    @courant_factor.setter
    def courant_factor(self, value: float) -> None:
        self.update(courant_factor=value)

    @property
    def minimum_surface_area(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return float(_cast(int | float, self._value("minimum_surface_area")))

    @minimum_surface_area.setter
    def minimum_surface_area(self, value: float) -> None:
        self.update(minimum_surface_area=value)

    @property
    def minimum_conduit_slope(self) -> float:
        """Setter lifecycle: `OPEN`, `ENDED`."""
        return float(_cast(int | float, self._value("minimum_conduit_slope")))

    @minimum_conduit_slope.setter
    def minimum_conduit_slope(self, value: float) -> None:
        self.update(minimum_conduit_slope=value)

    @property
    def head_tolerance(self) -> float:
        """Return the dynamic-wave head-convergence tolerance in length units."""
        return float(_cast(int | float, self._value("head_tolerance")))

    @property
    def system_flow_tolerance(self) -> float:
        """Return the system-flow continuity tolerance as a percent."""
        return float(_cast(int | float, self._value("system_flow_tolerance")))

    @property
    def lateral_flow_tolerance(self) -> float:
        """Return the lateral-flow continuity tolerance as a percent."""
        return float(_cast(int | float, self._value("lateral_flow_tolerance")))

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, SimulationOptions):
            return NotImplemented
        return self._simulation is other._simulation and self._generation == other._generation

    def __hash__(self) -> int:
        return hash((id(self._simulation), self._generation, SimulationOptions))

    def __repr__(self) -> str:
        values = ", ".join(
            f"{name}={value!r}"
            for name, value in self._simulation._owner.options_read(self._generation).items()
        )
        return f"{type(self).__name__}({values})"

    def __copy__(self) -> _Never:
        raise TypeError("SimulationOptions cannot be copied")

    def __deepcopy__(self, memo: object) -> _Never:
        raise TypeError("SimulationOptions cannot be deep-copied")

    def __reduce__(self) -> _Never:
        raise TypeError("SimulationOptions cannot be pickled")

    def __reduce_ex__(self, protocol: _SupportsIndex) -> _Never:
        raise TypeError("SimulationOptions cannot be pickled")


def _new_options(simulation: Simulation, generation: int) -> SimulationOptions:
    return SimulationOptions(simulation, generation, _OPTIONS_TOKEN)
