"""Native cumulative citation records, including cold-load usage admission."""
import json
from pathlib import Path
import subprocess
import sys
import textwrap

from symbolica import Citation, get_citations
from symbolica.community.hepkit import sector_decomposition as sd

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "examples/hepkit"))
from showcase import inputs


IDS = {
    "https://github.com/alphal00p/fastSecDec",
    "doi:10.1016/j.cpc.2017.09.015",
    "doi:10.1016/j.cpc.2010.04.001",
    "doi:10.1016/S0550-3213(00)00429-6",
}


def records():
    all_records = get_citations()
    assert len({record.id for record in all_records}) == len(all_records)
    return {record.id: record for record in all_records if record.id in IDS}


def semantic_records():
    return {key: (item.reference, item.bibtex, tuple(item.reasons), item.description, item.relevance)
            for key, item in records().items()}


def test_generation_records_are_native_exportable_and_cumulative():
    integral = sd.Integral(**inputs.massive_triangle().integral_arguments())
    generated = integral.generate(progress=None)
    citations = records()
    assert set(citations) == IDS
    for identity, citation in citations.items():
        assert isinstance(citation, Citation)
        assert citation.reference and citation.description and citation.reasons
        assert citation.relevance is None
        bibtex = citation.to_bibtex()
        assert bibtex == citation.bibtex
        assert bibtex.startswith(("@article{", "@misc{", "@software{")) and bibtex.endswith("}")
        assert bibtex.count("{") == bibtex.count("}")
        assert "author =" in bibtex and "title =" in bibtex
        assert identity.removeprefix("doi:") in bibtex
        html = citation._repr_html_()
        assert 'class="symbolica-citation"' in html and "<code>" + identity + "</code>" in html
        assert "Reasons:" in html
        assert bibtex in citation.to_markdown(include_bibtex=True)
    before = semantic_records()
    assert semantic_records() == before  # Queries never consume usage.
    assert generated.snapshot().kernels == 0
    artifact = generated.compile(progress=None).to_bytes()
    sd.Kernels.from_bytes(artifact)
    integral.generate(progress=None)
    assert semantic_records() == before  # Repeated use merges by stable ID.


def test_cold_artifact_load_registers_references(tmp_path):
    artifact = sd.Integral(**inputs.massive_triangle().integral_arguments()).generate(
        progress=None).compile(progress=None).to_bytes()
    path = tmp_path / "kernels.bin"
    path.write_bytes(artifact)
    if sys.platform == "emscripten":
        # Pyodide has no subprocess. The maintained runtime runner checks the
        # import-only empty state before pytest; exercise actual load here.
        loaded = sd.Kernels.from_bytes(path.read_bytes())
        assert loaded.sector_count > 0 and set(records()) == IDS
        return
    code = textwrap.dedent("""
        import json, sys
        from pathlib import Path
        from symbolica import get_citations
        from symbolica.community.hepkit import sector_decomposition as sd
        ids = set(json.loads(sys.argv[3]))
        def used():
            return {item.id for item in get_citations()} & ids
        assert not used()
        sd.QmcSettings(points=1024, shifts=2)
        assert not used()
        sys.path.insert(0, sys.argv[2])
        from showcase import inputs
        integral = sd.Integral(**inputs.massive_triangle().integral_arguments())
        assert not used()
        try:
            integral.generate(progress=lambda _: False)
        except sd.CancelledError:
            pass
        else:
            raise AssertionError('initial callback did not cancel')
        assert not used()
        try:
            sd.Kernels.from_bytes(b'invalid artifact')
        except sd.FastSecDecError:
            pass
        else:
            raise AssertionError('invalid artifact was admitted')
        assert not used()
        loaded = sd.Kernels.from_bytes(Path(sys.argv[1]).read_bytes())
        assert loaded.sector_count > 0
        assert used() == ids
        assert used() == ids
        print(json.dumps({'ids': sorted(used()), 'sectors': loaded.sector_count}))
    """)
    process = subprocess.run(
        [sys.executable, "-c", code, str(path),
         str(Path(__file__).resolve().parents[3] / "examples/hepkit"), json.dumps(sorted(IDS))],
        text=True, capture_output=True, timeout=30, check=True,
    )
    result = json.loads(process.stdout.strip().splitlines()[-1])
    assert set(result["ids"]) == IDS and result["sectors"] > 0
