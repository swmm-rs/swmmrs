"""Check the installed wheel, GIL declaration, and concurrent solver execution."""

import argparse
import sys
import sysconfig
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from tempfile import TemporaryDirectory


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path, help="SWMM input fixture to run")
    args = parser.parse_args()
    free_threaded = bool(sysconfig.get_config_var("Py_GIL_DISABLED"))
    if free_threaded:
        assert not sys._is_gil_enabled(), "start with the GIL disabled"

    import swmmrs
    from swmmrs.output import OutputReader, OutputTimeSeries

    if free_threaded:
        assert not sys._is_gil_enabled(), "importing swmmrs enabled the GIL"

    with TemporaryDirectory(prefix="swmmrs-wheel-") as directory:
        root = Path(directory)

        def run(index: int) -> bytes:
            output = root / f"{index}.out"
            simulation = swmmrs.Simulation(args.input, root / f"{index}.rpt", output)
            simulation.execute()
            assert simulation.state is swmmrs.SimulationState.CLOSED
            OutputReader(output).system_series("rainfall")
            return output.read_bytes()

        reference = run(0)
        with ThreadPoolExecutor(max_workers=2) as pool:
            for result in pool.map(run, (1, 2)):
                assert result == reference, "concurrent solver output differs"
            reader = OutputReader(root / "0.out")
            expected = reader.system_series("rainfall")

            def query(low_memory: bool) -> OutputTimeSeries:
                return reader.system_series("rainfall", low_memory=low_memory)

            for result in pool.map(query, (True, False)):
                assert result == expected, "shared OutputReader query differs"

    if free_threaded:
        assert not sys._is_gil_enabled(), "solver execution enabled the GIL"
    print(f"swmmrs {swmmrs.__version__}: free_threaded={free_threaded}, concurrent outputs match")


if __name__ == "__main__":
    main()
