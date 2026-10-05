#!/usr/bin/env bash
# Run the maintained binding/example controls against the installed host wheel.
set -euo pipefail
[[ $# == 1 ]] || { printf 'Usage: %s COMMUNITY_CHECKOUT\n' "$0" >&2; exit 2; }
community_root=$(cd "$1" && pwd -P)
delivery_root=$(cd "$(dirname "$0")/../../.." && pwd -P)
report=$(mktemp "${TMPDIR:-/tmp}/fastsecdec-native-tests.XXXXXX.xml")
python -m pytest -q "$delivery_root/bindings/python/tests" \
  "$delivery_root/examples/hepkit/tests" \
  "$community_root/tests/test_hep_wavefunctions.py" --junitxml="$report"
python - "$report" <<'PY'
import sys
import xml.etree.ElementTree as ET
suites = ET.parse(sys.argv[1]).getroot().findall("testsuite")
assert sum(int(suite.attrib["tests"]) for suite in suites) >= 61
assert all(int(suite.attrib[key]) == 0 for suite in suites for key in ("failures", "errors", "skipped"))
PY
printf 'Focused native control report: %s\n' "$report"
