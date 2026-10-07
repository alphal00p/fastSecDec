"""Both exports carry exactly the current portable helper bundle, without execution."""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import zipfile

import pytest


@pytest.mark.parametrize("notebook", ["gghh", "dashboard"])
def test_current_helpers_are_hash_bound_for_both_run_exports(tmp_path, monkeypatch, notebook):
    source = Path(__file__).resolve().parents[1] / "export.py"
    spec = importlib.util.spec_from_file_location("showcase_export_control", source)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    wheel = tmp_path / "symbolica-3.0.0-cp314-abi3-pyemscripten_2026_0_wasm32.whl"
    wheel.write_bytes(b"packaging fixture, not a native/browser runtime validation")
    destination = tmp_path / "site"
    commands = []
    def exporter(arguments, **kwargs):
        commands.append(arguments)
        destination.mkdir()
    monkeypatch.setattr(module.subprocess, "run", exporter)
    monkeypatch.setattr(sys, "argv", [str(source), "--notebook", notebook, "--wheel", str(wheel), "--output", str(destination)])
    module.main()
    assert "--no-execute" in commands[0]
    assert commands[0][commands[0].index("--mode") + 1] == "run"
    public = destination / "public/fastsecdec"
    manifest = json.loads((public / "manifest.json").read_text())
    archive = public / manifest["assets"]["filename"]
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == manifest["assets"]["sha256"]
    with zipfile.ZipFile(archive) as bundle:
        assert set(bundle.namelist()) == set(manifest["assets"]["files"])
        assert {"showcase/notebook.py", "showcase/science.py", "showcase/gghh.py"} <= set(bundle.namelist())
        for name, digest in manifest["assets"]["files"].items():
            assert not name.startswith("/") and ".." not in Path(name).parts
            assert "__marimo__" not in name and "fixtures/gghh" not in name
            assert hashlib.sha256(bundle.read(name)).hexdigest() == digest
