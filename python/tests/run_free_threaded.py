"""Build and test an isolated free-threaded wheel without changing the project venv.

Examples from the repository root:
    uv run --no-project python python/tests/run_free_threaded.py
    uv run --no-project python python/tests/run_free_threaded.py --python 3.15t
Requires uv, Rust 1.97.1, and checked-out solver submodules.
"""

import argparse
import subprocess
from pathlib import Path
from tempfile import TemporaryDirectory


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        "--python",
        choices=("3.14t", "3.15t"),
        default="3.14t",
        help="Free-threaded Python version to test (default: 3.14t).",
    )
    version = parser.parse_args().python
    project = Path(__file__).resolve().parents[1]
    fixture = project.parent / "crates/solver/tests/data/test_ex1_metric.inp"
    subprocess.run(["uv", "python", "install", version], check=True)
    with TemporaryDirectory(prefix=f"swmmrs-{version}-") as directory:
        interpreter = subprocess.check_output(
            ["uv", "python", "find", version], text=True, cwd=directory
        ).strip()
        subprocess.run(
            [
                "uv",
                "run",
                "--no-project",
                "--with",
                "maturin==1.15.0",
                "maturin",
                "build",
                "--manifest-path",
                str(project / "Cargo.toml"),
                "--locked",
                "--features",
                "test-support",
                "--interpreter",
                interpreter,
                "--out",
                directory,
            ],
            check=True,
        )
        [wheel] = Path(directory).glob("*.whl")
        command = [
            "uv",
            "run",
            "--no-project",
            "--python",
            interpreter,
            "--with",
            str(wheel),
            "--with",
            "pytest",
            "--with",
            "mypy",
            "python",
        ]
        # No -X gil=0 here: forcing it would hide a broken module declaration.
        subprocess.run(
            command
            + [
                "-I",
                "-W",
                "error::RuntimeWarning",
                str(project / "tests/wheel_smoke.py"),
                str(fixture),
            ],
            check=True,
            cwd=directory,
        )
        subprocess.run(
            command + ["-X", "gil=0", "-m", "pytest", str(project / "tests"), "-q"],
            check=True,
            cwd=directory,
        )


if __name__ == "__main__":
    main()
