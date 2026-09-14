"""Task wiring and preview lifecycle tests; no solver build required."""

import importlib.util
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("serve_docs", ROOT / "scripts/serve-docs.py")
serve_docs = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(serve_docs)


@unittest.skipUnless(shutil.which("just"), "just is required for recipe checks")
class Recipes(unittest.TestCase):
    def dry_run(self, *recipes):
        result = subprocess.run(
            ["just", "--justfile", str(ROOT / "justfile"), "--dry-run", *recipes],
            capture_output=True, text=True, check=True,
        )
        return result.stdout + result.stderr

    def test_docs_generate_before_build_and_check_without_generating_twice(self):
        output = self.dry_run("docs-check")
        self.assertEqual(output.count("run build"), 1)
        self.assertLess(output.index("run build"), output.index("zensical build"))
        self.assertLess(output.index("zensical build"), output.index("check-site.py"))

    def test_preview_generates_before_starting_watchers(self):
        output = self.dry_run("docs-serve")
        self.assertLess(output.index("run build"), output.index("serve-docs.py"))

    def test_cli_profiles(self):
        for profile in ("debug", "release"):
            with self.subTest(profile=profile):
                output = self.dry_run(f"cli-{profile}")
                self.assertIn("cargo build --locked", output)
                self.assertIn("-p runswmmrs --bin runswmmrs", output)
                self.assertEqual("--release" in output, profile == "release")

    def test_python_profiles_produce_separate_wheels_without_installing(self):
        for profile, flag in (("debug", "--profile dev"), ("release", "--profile release")):
            with self.subTest(profile=profile):
                output = self.dry_run(f"python-{profile}")
                self.assertIn("uv build python --wheel", output)
                self.assertIn(f"python/target/wheels/{profile}", output)
                self.assertIn(f"maturin.build-args={flag}", output)
                self.assertNotIn("sync", output)
                self.assertNotIn("install", output)

    def test_js_profiles_build_wasm_before_typescript(self):
        for profile, flag in (("debug", "--dev"), ("release", "--release")):
            with self.subTest(profile=profile):
                output = self.dry_run(f"js-{profile}")
                self.assertIn(f"run build:wasm -- {flag} --locked", output)
                self.assertLess(output.index("build:wasm"), output.index("build:ts"))


class PreviewCommand(unittest.TestCase):
    def test_watch_and_preview_arguments_and_working_directories(self):
        with patch.object(serve_docs, "supervise", return_value=0) as supervise, \
                patch.object(sys, "argv", ["serve-docs.py", "--dev-addr", "127.0.0.1:8001"]):
            self.assertEqual(serve_docs.main(), 0)
        watcher, server = supervise.call_args.args[0]
        self.assertIn("--watch", watcher[0])
        self.assertIn("--preserveWatchOutput", watcher[0])
        self.assertEqual(watcher[0][-2:], ["--cleanOutputDir", "false"])
        self.assertEqual(supervise.call_args.kwargs["ready_message"], "json generated at")
        self.assertEqual(watcher[1], ROOT / "docs/typescript-docs")
        self.assertEqual(server[0], [sys.executable, "-m", "zensical", "serve",
                                     "--dev-addr", "127.0.0.1:8001"])
        self.assertEqual(server[1], ROOT)


@unittest.skipUnless(os.name == "posix", "POSIX process-group lifecycle checks")
class PreviewLifecycle(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.runner = None

    def tearDown(self):
        if self.runner is not None and self.runner.poll() is None:
            self.runner.terminate()
            self.runner.wait(timeout=10)

    def start(self, exit_code=None, ready_message=None):
        child = self.directory / "child.py"
        child.write_text('''
import os, pathlib, signal, sys, time
name = sys.argv[1]
def stopped(signum, frame):
    pathlib.Path(name + ".stopped").write_text(str(signum))
    raise SystemExit(0)
signal.signal(signal.SIGTERM, stopped)
pathlib.Path(name + ".pid").write_text(str(os.getpid()))
if name == "other" and pathlib.Path("gate").exists():
    while pathlib.Path("gate").exists():
        time.sleep(0.01)
    print("ready", flush=True)
if len(sys.argv) > 2:
    while not pathlib.Path("other.pid").exists():
        time.sleep(0.01)
    raise SystemExit(int(sys.argv[2]))
while True:
    time.sleep(0.1)
''')
        commands = [
            [sys.executable, str(child), "other"],
            [sys.executable, str(child), "first", *([] if exit_code is None else [str(exit_code)])],
        ]
        runner = self.directory / "runner.py"
        runner.write_text(f'''
import importlib.util, pathlib
spec = importlib.util.spec_from_file_location("serve_docs", {str(ROOT / "scripts/serve-docs.py")!r})
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
raise SystemExit(module.supervise([(command, pathlib.Path({str(self.directory)!r})) for command in {commands!r}], ready_message={ready_message!r}))
''')
        self.runner = subprocess.Popen([sys.executable, str(runner)])

    def wait_for(self, name):
        path = self.directory / name
        deadline = time.monotonic() + 5
        while not path.exists():
            if time.monotonic() > deadline:
                self.fail(f"Timed out waiting for {name}")
            time.sleep(0.02)
        return path

    def assert_child_stopped(self, name):
        self.assertTrue((self.directory / f"{name}.stopped").exists())
        pid = int((self.directory / f"{name}.pid").read_text())
        with self.assertRaises(ProcessLookupError):
            os.kill(pid, 0)

    def test_child_failure_stops_sibling_and_propagates_status(self):
        self.start(exit_code=7)
        self.assertEqual(self.runner.wait(timeout=10), 7)
        self.assert_child_stopped("other")

    def test_server_waits_for_watcher_readiness(self):
        gate = self.directory / "gate"
        gate.touch()
        self.start(ready_message="ready")
        self.wait_for("other.pid")
        time.sleep(0.2)
        self.assertFalse((self.directory / "first.pid").exists())
        gate.unlink()
        self.wait_for("first.pid")
        self.runner.terminate()
        self.assertEqual(self.runner.wait(timeout=10), 143)
        self.assert_child_stopped("other")
        self.assert_child_stopped("first")

    def test_interrupt_stops_both_watchers(self):
        self.start()
        self.wait_for("other.pid")
        self.wait_for("first.pid")
        self.runner.send_signal(signal.SIGINT)
        self.assertEqual(self.runner.wait(timeout=10), 130)
        self.assert_child_stopped("other")
        self.assert_child_stopped("first")

    def test_termination_stops_both_watchers(self):
        self.start()
        self.wait_for("other.pid")
        self.wait_for("first.pid")
        self.runner.terminate()
        self.assertEqual(self.runner.wait(timeout=10), 143)
        self.assert_child_stopped("other")
        self.assert_child_stopped("first")

    def test_partial_startup_failure_cleans_up_started_process(self):
        with patch.object(serve_docs.subprocess, "Popen") as popen, \
                patch.object(serve_docs, "signal_process") as stop:
            child = popen.return_value
            popen.side_effect = [child, FileNotFoundError("missing command")]
            with self.assertRaises(FileNotFoundError):
                serve_docs.supervise([(["first"], ROOT), (["missing"], ROOT)])
            stop.assert_any_call(child, signal.SIGTERM)
            child.wait.assert_called()


if __name__ == "__main__":
    unittest.main()
