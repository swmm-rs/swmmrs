"""Run the TypeDoc watcher and Zensical preview as one foreground task."""

from __future__ import annotations

import os
from pathlib import Path
import signal
import subprocess
import sys
import time
import threading


ROOT = Path(__file__).resolve().parents[1]


def signal_process(process: subprocess.Popen, signum: int) -> None:
    try:
        if os.name == "posix":
            # Zensical may spawn workers. Stop the whole group, even if its
            # original process has already exited.
            os.killpg(process.pid, signum)
        elif process.poll() is None:
            if signum == signal.SIGTERM:
                process.terminate()
            else:
                process.kill()
    except ProcessLookupError:
        pass


def supervise(commands: list[tuple[list[str], Path]], ready_message: str | None = None) -> int:
    """Stop all children when one exits, startup fails, or the user interrupts."""
    processes = []
    previous_handlers = {}

    def interrupted(signum, _frame):
        raise SystemExit(128 + signum)

    try:
        for signum in (signal.SIGINT, signal.SIGTERM):
            previous_handlers[signum] = signal.signal(signum, interrupted)
        for index, (command, directory) in enumerate(commands):
            wait_for_ready = index == 0 and ready_message is not None
            process = subprocess.Popen(
                command, cwd=directory, start_new_session=os.name == "posix",
                stdout=subprocess.PIPE if wait_for_ready else None,
                stderr=subprocess.STDOUT if wait_for_ready else None,
                text=True,
            )
            processes.append(process)
            if wait_for_ready:
                ready = threading.Event()

                def relay_output(stream):
                    with stream as output:
                        for line in output:
                            print(line, end="", flush=True)
                            if ready_message in line:
                                ready.set()

                threading.Thread(target=relay_output, args=(process.stdout,), daemon=True).start()
                deadline = time.monotonic() + 120
                while not ready.wait(0.1):
                    code = process.poll()
                    if code is not None:
                        return code or 1
                    if time.monotonic() >= deadline:
                        raise OSError("Timed out waiting for TypeDoc's initial generation")
        while True:
            for process in processes:
                code = process.poll()
                if code is not None:
                    return code if code >= 0 else 128 - code
            time.sleep(0.1)
    finally:
        # A second Ctrl-C must not interrupt cleanup and strand a watcher.
        for signum in previous_handlers:
            signal.signal(signum, signal.SIG_IGN)
        for process in processes:
            signal_process(process, signal.SIGTERM)
        deadline = time.monotonic() + 5
        for process in processes:
            try:
                process.wait(timeout=max(0, deadline - time.monotonic()))
            except subprocess.TimeoutExpired:
                pass
        for process in processes:
            # Clean up any descendants left behind by an exiting group leader.
            signal_process(process, signal.SIGKILL if os.name == "posix" else signal.SIGTERM)
            process.wait()
        for signum, handler in previous_handlers.items():
            signal.signal(signum, handler)


def main() -> int:
    typedoc = ROOT / "docs/typescript-docs"
    return supervise([
        (["node", "node_modules/typedoc/bin/typedoc", "--options", "typedoc.json",
          "--watch", "--preserveWatchOutput", "--cleanOutputDir", "false"], typedoc),
        ([sys.executable, "-m", "zensical", "serve", *sys.argv[1:]], ROOT),
    ], ready_message="json generated at")


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OSError as error:
        print(f"Could not start documentation preview: {error}", file=sys.stderr)
        raise SystemExit(1)
