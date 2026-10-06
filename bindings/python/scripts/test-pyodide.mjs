// Run the same installed scientific bridge gates in the actual Pyodide runtime.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile, readdir } from "node:fs/promises";
import { basename, dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const wheelDir = process.argv[2];
const runtimeDir = process.env.PYODIDE_DIST_DIR;
const communityRoot = process.argv[3];
assert(wheelDir && runtimeDir && communityRoot, "Pass wheel and community checkout directories; set PYODIDE_DIST_DIR");
const wheels = (await readdir(wheelDir)).filter(name => name.endsWith("-pyemscripten_2026_0_wasm32.whl"));
assert.equal(wheels.length, 1, "Expected exactly one PyEmscripten wheel");
const bytes = await readFile(join(wheelDir, wheels[0]));
const root = fileURLToPath(new URL("../../../", import.meta.url));
const { loadPyodide } = await import(pathToFileURL(join(runtimeDir, "pyodide.mjs")));
const pyodide = await loadPyodide({
  indexURL: runtimeDir,
  env: { SYMBOLICA_LICENSE_KEY: process.env.SYMBOLICA_LICENSE_KEY || "" },
});
await pyodide.loadPackage("micropip");
pyodide.FS.writeFile(`/${wheels[0]}`, bytes);
pyodide.globals.set("bridge_wheel_uri", `emfs:/${wheels[0]}`);
const tests = [
  "bindings/python/tests/test_fastsecdec.py",
  "bindings/python/tests/test_inspection.py",
  "bindings/python/tests/test_mc.py",
  "examples/hepkit/tests/test_inputs.py",
];
const fixtures = (await readdir(join(root, "examples/hepkit/fixtures/fastsecdec"))).sort();
const paths = [
  ...tests,
  "examples/hepkit/showcase/__init__.py",
  "examples/hepkit/showcase/inputs.py",
  ...fixtures.map(name => `examples/hepkit/fixtures/fastsecdec/${name}`),
];
for (const path of paths) {
  const destination = `/bridge/fastsecdec/${path}`;
  pyodide.FS.mkdirTree(dirname(destination));
  pyodide.FS.writeFile(destination, await readFile(join(root, path)));
}
const sharedTest = "/bridge/community/tests/test_hep_wavefunctions.py";
pyodide.FS.mkdirTree(dirname(sharedTest));
pyodide.FS.writeFile(sharedTest, await readFile(join(communityRoot, "tests/test_hep_wavefunctions.py")));
pyodide.globals.set("bridge_test_paths", [
  ...tests.map(path => `/bridge/fastsecdec/${path}`), sharedTest,
]);
const result = await pyodide.runPythonAsync(`
import json
import os
import sys
import time
import micropip
await micropip.install([bridge_wheel_uri, "pytest"])
from symbolica import set_license_key
if os.environ.get("SYMBOLICA_LICENSE_KEY"):
    set_license_key(os.environ["SYMBOLICA_LICENSE_KEY"])
assert sys.platform == "emscripten"
import pytest
started = time.perf_counter()
class Report:
    collected = 0
    passed = failed = skipped = 0
    nodeids = []
    files = []
    def pytest_collection_finish(self, session):
        self.collected = len(session.items)
        self.nodeids = [item.nodeid for item in session.items]
        self.files = sorted({item.path.name for item in session.items})
    def pytest_runtest_logreport(self, report):
        if report.when == "call":
            self.passed += report.passed
            self.failed += report.failed
            self.skipped += report.skipped
        elif report.failed:
            self.failed += 1
        elif report.skipped:
            self.skipped += 1
report = Report()
code = pytest.main(["-q", "-p", "no:cacheprovider", *bridge_test_paths.to_py()], plugins=[report])
json.dumps({"exit_code": int(code), "collected": report.collected,
            "nodeids": report.nodeids, "files": report.files, "passed": report.passed,
            "failed": report.failed, "skipped": report.skipped,
            "seconds": time.perf_counter() - started,
            "python": sys.version, "platform": sys.platform})
`);
const report = JSON.parse(result);
report.wheel = wheels[0];
report.wheel_sha256 = createHash("sha256").update(bytes).digest("hex");
console.log(JSON.stringify(report, null, 2));
assert.equal(report.exit_code, 0, "Installed Pyodide bridge gates failed");
assert.deepEqual(report.files, [...tests.map(path => basename(path)), "test_hep_wavefunctions.py"].sort());
assert(report.collected > 0, "Expected maintained native binding controls");
assert.equal(report.passed, report.collected, "Every collected control must pass");
assert.equal(report.failed, 0);
assert.equal(report.skipped, 0);
