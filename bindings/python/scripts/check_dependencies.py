"""Validate shared native owners; local development must be explicitly selected."""
import argparse
import json

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("metadata")
parser.add_argument("backend", choices=("native", "portable"))
parser.add_argument("--allow-local-fastsecdec", action="store_true")
args = parser.parse_args()
with open(args.metadata) as stream:
    metadata = json.load(stream)
backend = args.backend
nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
packages = [package for package in metadata["packages"] if package["id"] in nodes]
owners = {}
for name in (
    "fastsecdec", "fastsecdec-sectors", "fastsecdec-python", "symbolica", "graphica", "numerica",
    "feynkit-graph", "feynkit-model", "feynkit-kinematics", "feynkit-tensor",
    "feynkit-py", "spynso3", "linnet", "idenso", "spenso", "pyo3",
):
    matches = [package for package in packages if package["name"] == name]
    assert len(matches) == 1, (name, [package["id"] for package in matches])
    owners[name] = matches[0]
features = nodes[owners["fastsecdec"]["id"]]["features"]
assert set(features) & {"native", "portable"} == {backend}, features
leaf_features = nodes[owners["fastsecdec-python"]["id"]]["features"]
assert set(leaf_features) & {"native", "portable"} == {backend}, leaf_features
sector_features = nodes[owners["fastsecdec-sectors"]["id"]]["features"]
assert set(sector_features) & {"native", "portable"} == {backend}, sector_features
numeric_features = set(nodes[owners["numerica"]["id"]]["features"])
native_numeric = {"integer-gmp", "float-mpfr"}
portable_numeric = {"integer-malachite", "float-astro"}
required, forbidden = (native_numeric, portable_numeric) if backend == "native" else (portable_numeric, native_numeric)
assert required <= numeric_features, numeric_features
# Cargo metadata includes a feature union from the host's target-conditional
# native integrations even when --filter-platform removes their WASM nodes.
# This is not a report of the compiler's selected cfg features. The standalone
# leaf has no such host integrations and can enforce numerical exclusivity here.
if not any(package["name"] == "symbolica_community" for package in packages):
    assert not forbidden & numeric_features, numeric_features
if args.allow_local_fastsecdec:
    for name in ("fastsecdec", "fastsecdec-sectors", "fastsecdec-python"):
        assert owners[name]["source"] is None, (name, owners[name]["source"])
    print(f"Unique native owners; {backend} backend; explicitly local FastSecDec development")
else:
    community = next(package for package in packages if package["name"] == "symbolica_community")
    declared = next(dependency["source"] for dependency in community["dependencies"] if dependency["name"] == "fastsecdec-python")
    assert declared.startswith("git+https://github.com/alphal00p/fastSecDec?rev="), declared
    revision = declared.rsplit("=", 1)[1]
    assert len(revision) == 40 and all(character in "0123456789abcdef" for character in revision)
    for name in ("fastsecdec", "fastsecdec-sectors", "fastsecdec-python"):
        assert owners[name]["source"] == declared + "#" + revision, owners[name]["source"]
    print(f"Unique native owners; {backend} backend; published FastSecDec {revision}")
