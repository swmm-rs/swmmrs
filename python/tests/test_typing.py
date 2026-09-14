from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TYPING = ROOT / "tests" / "typing"


def mypy(fixture: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        (
            sys.executable,
            "-m",
            "mypy",
            "--strict",
            "--follow-imports=silent",
            str(TYPING / fixture),
        ),
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )


def test_representative_supported_uses_type_check() -> None:
    result = mypy("valid.py")
    assert result.returncode == 0, result.stdout


def test_representative_unsupported_uses_are_rejected() -> None:
    result = mypy("invalid.py")
    assert result.returncode != 0, result.stdout
    expected = (
        'Cannot inherit from final class "Simulation"',
        'Cannot inherit from final class "Junction"',
        'expression has type "timedelta", variable has type "datetime"',
        'expression has type "float", variable has type "int"',
        'Property "depth" defined in "NodeSnapshot" is read-only',
        'expression has type "Node", variable has type "StorageNode"',
        'expression has type "LidSurfaceLayer | None", variable has type "LidSurfaceLayer"',
        'expression has type "str", variable has type "SolverErrorCode"',
        'expression has type "None", variable has type "HortonInfiltrationSettings',
        'Invalid index type "int" for "MutableMapping[str, float]"; expected type "str"',
        '"MutableMapping[str, float]" has no attribute "copy"',
    )
    for diagnostic in expected:
        assert diagnostic in result.stdout
