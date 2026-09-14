from __future__ import annotations

import importlib.machinery
import importlib.metadata
import importlib.resources
import sys
from pathlib import Path

import pytest  # type: ignore[import-not-found]

import swmmrs
import swmmrs.enums as public_enums
import swmmrs.exceptions as public_exceptions
import swmmrs.objects as public_objects
import swmmrs.objects._base as object_base


def test_public_facade_loads_private_native_identity() -> None:
    assert swmmrs.__version__ == importlib.metadata.version("swmmrs")
    assert str(swmmrs.__file__).endswith(".py")

    native = sys.modules["swmmrs._swmmrs"]
    assert any(
        str(native.__file__).endswith(suffix) for suffix in importlib.machinery.EXTENSION_SUFFIXES
    )
    assert "_swmmrs" not in swmmrs.__all__
    assert not hasattr(swmmrs, "_swmmrs")
    assert not hasattr(native, "NativeError")
    for name in (
        "_poison_for_test",
        "_force_timestep_error_for_test",
        "_panic_for_test",
        "_arm_owner_pause_for_test",
        "_wait_owner_paused_for_test",
        "_release_owner_pause_for_test",
        "_arm_owner_probe_for_test",
        "_wait_owner_probe_waiting_for_test",
        "_owner_probe_acquired_for_test",
    ):
        assert not hasattr(swmmrs.Simulation, name)


def test_custom_ellipse_model_is_explicit_mutable_and_reported(tmp_path: Path) -> None:
    fixture = (
        Path(__file__).resolve().parents[2]
        / "crates"
        / "solver"
        / "tests"
        / "data"
        / "test_ex1_metric.inp"
    )
    legacy_report = tmp_path / "legacy.rpt"
    legacy = swmmrs.Simulation(fixture, legacy_report)
    assert legacy.options.custom_ellipse_model is swmmrs.CustomEllipseModel.EPA_LEGACY
    assert not hasattr(legacy, "custom_ellipse_model")
    legacy.options.custom_ellipse_model = swmmrs.CustomEllipseModel.TRUE_ELLIPSE
    assert legacy.options.custom_ellipse_model is swmmrs.CustomEllipseModel.TRUE_ELLIPSE
    assert legacy.configuration_dirty
    legacy.execute()
    assert "Custom Ellipse Hydraulics  TRUE ELLIPSE (NON-EPA)" in legacy_report.read_text()
    legacy.close()
    true_fixture = tmp_path / "true-ellipse.inp"
    true_fixture.write_text(
        fixture.read_text().replace(
            "[OPTIONS]",
            "[OPTIONS]\nCUSTOM_ELLIPSE_MODEL TRUE_ELLIPSE",
            1,
        )
    )

    report = tmp_path / "exact.rpt"
    exact = swmmrs.Simulation(true_fixture, report)
    assert exact.options.custom_ellipse_model is swmmrs.CustomEllipseModel.TRUE_ELLIPSE
    exact.execute()
    assert "Custom Ellipse Hydraulics  TRUE ELLIPSE (NON-EPA)" in report.read_text()
    with pytest.raises(TypeError):
        swmmrs.Simulation(
            fixture,
            tmp_path / "invalid.rpt",
            custom_ellipse_model="true_ellipse",  # type: ignore[call-arg]
        )


@pytest.mark.parametrize(
    "name",
    ["__version__", "solver_version", "solver_build_id"],
)
def test_public_identity_is_immutable(name: str) -> None:
    with pytest.raises(AttributeError):
        setattr(swmmrs, name, "replacement")


def test_placeholder_api_is_absent() -> None:
    assert not hasattr(swmmrs, "hello")
    assert not hasattr(swmmrs, "Output")


def test_public_failure_types_are_stable_and_distinct() -> None:
    assert issubclass(swmmrs.LifecycleError, swmmrs.SwmmError)
    assert issubclass(swmmrs.StaleViewError, swmmrs.LifecycleError)
    assert issubclass(swmmrs.ValidationError, swmmrs.SwmmError)
    assert issubclass(swmmrs.SolverError, swmmrs.SwmmError)
    assert issubclass(swmmrs.InternalSimulationError, swmmrs.SwmmError)
    assert swmmrs.SolverErrorCode.TIMESTEP.value == "timestep"
    assert swmmrs.SolverErrorCode.HOTSTART_FILE_FORMAT.message == "hotstart file format"
    assert "API_NOT_OPEN" not in swmmrs.SolverErrorCode.__members__
    assert "EXCEPTION" not in swmmrs.SolverErrorCode.__members__


def test_native_public_type_bindings_ignore_later_module_mutation(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    fixture = Path(__file__).resolve().parent / "data" / "collections.inp"
    original_junction = public_objects.Junction
    original_solver_error = public_exceptions.SolverError
    original_solver_code = public_enums.SolverErrorCode
    original_stale_error = public_exceptions.StaleViewError

    simulation = swmmrs.Simulation(fixture, tmp_path / "bindings.rpt")
    subcatchment = simulation.subcatchments["Sub-A"]

    monkeypatch.setattr(public_objects, "Junction", tuple)
    monkeypatch.setattr(object_base, "_VIEW_TOKEN", object())
    monkeypatch.setattr(public_exceptions, "SolverError", RuntimeError)
    monkeypatch.setattr(public_exceptions, "StaleViewError", RuntimeError)
    monkeypatch.setattr(public_enums, "SolverErrorCode", str)

    outlet = subcatchment.settings.outlet
    node = simulation.nodes["J1"]
    assert type(outlet) is original_junction
    assert type(node) is original_junction
    simulation.close()
    with pytest.raises(original_stale_error) as stale:
        _ = outlet.id
    assert stale.value.__cause__ is None

    with pytest.raises(original_solver_error) as solver:
        swmmrs.Simulation(tmp_path / "missing.inp", tmp_path / "missing.rpt")
    assert type(solver.value.code) is original_solver_code
    assert solver.value.__cause__ is None


def test_typing_markers_ship_with_the_installed_package() -> None:
    package_files = importlib.resources.files(swmmrs)
    assert package_files.joinpath("py.typed").is_file()
    assert package_files.joinpath("_swmmrs.pyi").is_file()
