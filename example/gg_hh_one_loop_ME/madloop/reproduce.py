#!/usr/bin/env python3
"""Generate a private MadLoop standalone process and evaluate the shared point.

Python steers the external reference only; it contains no amplitude calculation.
Requires Python >=3.10 with six, GNU Fortran/C++, GNU make, and an existing MG5.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import resource
import shutil
import signal
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
BUILD = HERE / ".build"
OUTPUT = BUILD / "gghh_standalone"
SUBPROCESS = OUTPUT / "SubProcesses" / "P0_gg_hh"
MEMORY_LIMIT = 15 * 1024**3


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def group_rss(group: int) -> int:
    """Linux resident-memory sum of this command's process group only."""
    if not Path("/proc").is_dir():
        return 0
    pages = 0
    for entry in Path("/proc").iterdir():
        if not entry.name.isdigit():
            continue
        try:
            fields = (entry / "stat").read_text().rsplit(") ", 1)[1].split()
            if int(fields[2]) == group:
                pages += int(fields[21])
        except (OSError, ValueError, IndexError):
            pass
    return pages * os.sysconf("SC_PAGE_SIZE")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mg5-root", type=Path, default=Path(
        os.environ.get("MG5_ROOT", "/common/dev/MG5_aMC_v3_7_0")))
    parser.add_argument("--reuse", action="store_true",
                        help="reuse generated private sources; rebuild the point driver")
    args = parser.parse_args()
    started = time.monotonic()
    deadline = started + 600
    BUILD.mkdir(exist_ok=True)
    commands = []
    env = os.environ.copy()
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    env["MAKEFLAGS"] = "-j2"
    env["SHELL"] = shutil.which("bash") or "/bin/sh"

    def bounded(command: list[str], cwd: Path, log_name: str) -> None:
        def limit() -> None:
            resource.setrlimit(resource.RLIMIT_AS, (MEMORY_LIMIT, MEMORY_LIMIT))
        before = time.monotonic()
        maximum = 0
        with (HERE / log_name).open("wb") as log:
            process = subprocess.Popen(command, cwd=cwd, env=env, stdout=log,
                                       stderr=subprocess.STDOUT, start_new_session=True,
                                       preexec_fn=limit)
            try:
                while process.poll() is None:
                    maximum = max(maximum, group_rss(process.pid))
                    if time.monotonic() >= deadline or maximum > MEMORY_LIMIT:
                        raise RuntimeError("MadLoop reference exceeded 10 minutes or 15 GiB")
                    time.sleep(0.1)
            except BaseException:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
                raise
            if process.returncode:
                raise RuntimeError(f"{command[0]} failed; see {HERE / log_name}")
        commands.append({"command": command, "seconds": time.monotonic() - before,
                         "peak_process_group_rss_bytes": maximum})

    private = BUILD / "MG5_aMC_v3_7_0"
    if not args.reuse:
        if not private.exists():
            # MG5 writes restriction caches and configuration; never run from
            # the shared installation. Extra HEPTools are disabled in the card.
            excluded = shutil.ignore_patterns("HEPTools", "TMPQUICK*", "ZenoTest*", ".venv",
                                              "__pycache__", "*.pyc", "*.pkl",
                                              "MG5_debug", "ME5_debug", ".git")
            shutil.copytree(args.mg5_root.resolve(), private, ignore=excluded,
                            symlinks=True)
        if OUTPUT.exists():
            shutil.rmtree(OUTPUT)
        bounded([sys.executable, str(private / "bin/mg5_aMC"), "process.mg5"],
                HERE, "generation.log")
        generation = (HERE / "generation.log").read_text()
        if "8 loops, 2 R2, 0 UV" not in generation or "Output to directory" not in generation:
            raise RuntimeError("MadLoop did not finish the expected top-loop export")
    if not (SUBPROCESS / "loop_matrix.f").is_file():
        raise RuntimeError("Missing generated MadLoop process; run without --reuse")
    if "version = 3.7.0" not in (private / "VERSION").read_text():
        raise RuntimeError("This audited amplitude-layout driver requires MG5_aMC 3.7.0")

    point = json.loads((HERE.parent / "point.json").read_text())
    if point["helicities"] != [1, 1, 0, 0]:
        raise RuntimeError("This driver is explicitly for incoming ++ helicity")
    parameters = {}
    block = None
    for line in (HERE / "param_card.dat").read_text().splitlines():
        fields = line.split("#", 1)[0].split()
        if not fields:
            continue
        if fields[0].lower() == "block":
            block = fields[1].lower()
        elif fields[0].lower() == "decay":
            if float(fields[2]) != 0.:
                raise RuntimeError("This benchmark requires every width to be zero")
            block = None
        elif block is not None:
            parameters[block, int(fields[0])] = float(fields[1])
    for name, block, code in [("MH", "mass", 25), ("MT", "mass", 6),
                              ("ymt", "yukawa", 6), ("aEWM1", "sminputs", 1),
                              ("Gf", "sminputs", 2), ("aS", "sminputs", 3),
                              ("MZ", "mass", 23)]:
        if parameters[block, code] != point["parameters"][name]:
            raise RuntimeError(f"Parameter card disagrees with the shared point: {name}")
    helicities = (OUTPUT / "SubProcesses/MadLoop5_resources/ML5_0_HelConfigs.dat").read_text()
    if list(map(int, helicities.splitlines()[3].split())) != point["helicities"]:
        raise RuntimeError("Generated helicity row differs")
    projection = OUTPUT / "SubProcesses/MadLoop5_resources/ML5_0_LoopColorFlowCoefs.dat"
    projection_lines = projection.read_text().splitlines()
    if ("Tr(1,2)" not in projection_lines[0]
            or list(map(int, projection_lines[1].split())) != [1, 1] + [-1] * 8
            or list(map(int, projection_lines[2].split())) != [1] * 10):
        raise RuntimeError("Generated native color projector differs from this audited driver")
    matrix = (SUBPROCESS / "loop_matrix.f").read_text()
    if "DATA IDEN/512/" not in matrix or "DATA HELAVGFACTOR/4/" not in matrix:
        raise RuntimeError("Generated averaging/identical-particle convention changed")

    shutil.copyfile(HERE / "check_point.f", SUBPROCESS / "check_sa.f")
    shutil.copyfile(HERE / "param_card.dat", OUTPUT / "Cards/param_card.dat")
    for destination in [OUTPUT / "Cards/MadLoopParams.dat",
                        OUTPUT / "SubProcesses/MadLoop5_resources/MadLoopParams.dat"]:
        shutil.copyfile(HERE / "MadLoopParams.dat", destination)
    (SUBPROCESS / "PS.input").write_text("".join(
        " ".join(format(x, ".17e") for x in row) + "\n"
        for row in point["physical_momenta_GeV"]))
    # The native standalone makefile links IREGI even when CutTools is the
    # selected runtime provider. Internal export omits binary archives, and
    # its default Source target omits this library. Reuse the same installation's
    # built archive, as for its compiler-specific CutTools module. If absent,
    # build it privately with one job: IREGI's module rules are not parallel safe.
    ire_archive = private / "vendor/IREGI/src/libiregi.a"
    output_archive = OUTPUT / "Source/IREGI/src/libiregi.a"
    if not output_archive.exists() and ire_archive.is_file():
        shutil.copyfile(ire_archive, output_archive)
    if not output_archive.exists():
        bounded(["make", "-j1", "libiregi", f"SHELL={env['SHELL']}"],
                OUTPUT / "Source", "compile_iregi.log")
    bounded(["make", "-j2", "check", f"SHELL={env['SHELL']}"], SUBPROCESS, "compile.log")
    bounded([str(SUBPROCESS / "check")], SUBPROCESS, "evaluation.log")

    values = {}
    polarizations = {"EPS1": {}, "EPS2": {}}
    raw = {}
    for line in (HERE / "evaluation.log").read_text().splitlines():
        fields = line.split()
        if not fields:
            continue
        name = fields[0]
        if name == "RAW_AMPL":
            raw[int(fields[1])] = list(map(float, fields[2:]))
        elif name in polarizations:
            polarizations[name][int(fields[1])] = list(map(float, fields[2:]))
        elif name in {"RET_CODE", "RELATIVE_STABILITY", "A_TOTAL", "A_TRIANGLE", "A_BOXES",
                      "A_SINGLE_POLE", "A_DOUBLE_POLE", "MADLOOP_FIXED_HELICITY",
                      "COLOR_SUM", "COLOR_SUM_FROM_A"}:
            values[name] = list(map(float, fields[1:]))
    complex_value = lambda name: complex(*values[name])
    amplitude = complex_value("A_TOTAL")
    if int(values["RET_CODE"][0]) // 100 not in (2, 3):
        raise RuntimeError("MadLoop did not certify a stable or successfully rescued point")
    if abs(amplitude - complex_value("A_TRIANGLE") - complex_value("A_BOXES")) > 1e-15:
        raise RuntimeError("Triangle and box partition does not reconstruct the native flow")
    if not math.isclose(values["COLOR_SUM"][0], 8 * abs(amplitude)**2, rel_tol=1e-11):
        raise RuntimeError("Native squared matrix element disagrees with complex color flow")
    if max(abs(complex_value(name)) for name in ("A_SINGLE_POLE", "A_DOUBLE_POLE")) > 1e-12:
        raise RuntimeError("UV/IR poles failed to cancel")
    eps = [[polarizations[name][i] for i in range(4)] for name in ("EPS1", "EPS2")]
    h = math.sqrt(0.5)
    expected = [[[0., 0.], [-h, 0.], [0., -h], [0., 0.]],
                [[0., 0.], [-h, 0.], [0., h], [0., 0.]]]
    if max(abs(a - b) for row, other in zip(eps, expected)
           for component, component_other in zip(row, other)
           for a, b in zip(component, component_other)) > 1e-15:
        raise RuntimeError("Incoming HELAS polarization convention changed")
    result = {
        "engine": "MadGraph5_aMC@NLO 3.7.0 / MadLoop / CutTools 1.9.3 / OneLOop 3.6",
        "process": "g g > h h [sqrvirt=QCD]", "model": "loop_sm-no_b_mass (1.2)",
        "flavor_scope": point["flavor_scope"], "helicities": point["helicities"],
        "loop_diagrams": 8, "r2_amplitudes": 2, "uv_counterterm_amplitudes": 0,
        "amplitude_color_coefficient": values["A_TOTAL"],
        "triangle_including_r2": values["A_TRIANGLE"],
        "boxes_including_r2": values["A_BOXES"],
        "single_pole": values["A_SINGLE_POLE"], "double_pole": values["A_DOUBLE_POLE"],
        "color_sum_fixed_helicity": values["COLOR_SUM"][0],
        "color_average_fixed_helicity": values["COLOR_SUM"][0] / 64.,
        "raw_madloop_fixed_helicity": values["MADLOOP_FIXED_HELICITY"][0],
        "return_code": int(values["RET_CODE"][0]),
        "reported_relative_stability": values["RELATIVE_STABILITY"][0],
        "stability_request": 1e-12, "incoming_polarizations": eps,
        "normalization": {
            "color": "M_ab=delta_ab*A; sole native flow Tr(Ta Tb)=delta_ab/2; A=JAMPL/2",
            "loop_factor": "1/(16*pi^2) is already included by CT_interface.f",
            "initial_helicity_average": False,
            "final_identical_particle_phase_space_factor_included": False,
            "native_squared_result_multiplier": 128,
            "complex_phase": "native incoming HELAS VXXXXX convention, no phase fitted to FastSecDec",
        },
        "input_sha256": {name: digest(HERE / name) for name in
                         ("process.mg5", "param_card.dat", "MadLoopParams.dat", "check_point.f")},
        "point_sha256": digest(HERE.parent / "point.json"),
        "generated_source_sha256": {name: digest(SUBPROCESS / name) for name in
                                    ("loop_matrix.f", "CT_interface.f", "helas_calls_ampb_1.f",
                                     "loop_CT_calls_1.f", "compute_color_flows.f")},
        "model_source_sha256": {name: digest(private / "models/loop_sm" / name) for name in
                                ("parameters.py", "couplings.py", "restrict_no_b_mass.dat")},
        "raw_finite_amplitudes": raw,
    }
    (HERE / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    (BUILD / "run_metrics.json").write_text(json.dumps({"seconds": time.monotonic() - started,
                                                       "commands": commands}, indent=2) + "\n")
    print(json.dumps({key: result[key] for key in
                     ("amplitude_color_coefficient", "color_sum_fixed_helicity", "return_code")}, indent=2))


if __name__ == "__main__":
    main()
