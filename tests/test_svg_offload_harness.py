"""E3 SVG offload harness: the dry-run must plan the full command matrix
(10 cues x 2 conditions x 4 workers) without running or writing anything."""
import importlib.util
import re
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
RUN_PY = ROOT / "experiments" / "svg_offload" / "run.py"
CANDIDATES_MD = ROOT / "experiments" / "svg_offload" / "candidates.md"
STAGE_DIR = Path(
    "/Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation"
    "/video/yt02/stage"
)

spec = importlib.util.spec_from_file_location("svg_offload_run", RUN_PY)
run = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = run  # dataclass needs the module registered
spec.loader.exec_module(run)

BRIEF_RE = re.compile(r"briefs/(.+)-([AB])-([a-z0-9-]+)\.md")

EXPECTED_AGENTS = {
    "agy-gemini-flash-low": ("agy", "gemini-3.8-flash-low"),
    "codex-gpt-luna": ("codex", "gpt-5.6-luna"),
    "codex-gpt-terra": ("codex", "gpt-5.6-terra"),
}


@pytest.fixture()
def dry_run_output(capsys):
    rc = run.main(["--dry-run"])
    assert rc == 0
    return capsys.readouterr().out


def _dispatch_lines(output: str) -> list[str]:
    lines = []
    for line in output.splitlines():
        if not line.startswith("[dry-run] "):
            continue
        if "scriptcast.tools.stage_dispatch" in line or " muse exec " in line:
            lines.append(line)
    return lines


def _cells(output: str) -> set[tuple[str, str, str]]:
    cells = set()
    for line in _dispatch_lines(output):
        m = BRIEF_RE.search(line)
        assert m, f"dispatch line carries no brief path: {line}"
        cells.add((m.group(1), m.group(2), m.group(3)))
    return cells


def test_default_matrix_shape():
    assert len(run.DEFAULT_CUES) == 10
    assert len(run.WORKERS) == 4
    assert len(run.CONDITIONS) == 2
    assert set(run.DEFAULT_CUES) <= {c.slug for c in run.CANDIDATES}


def test_dry_run_dispatch_matrix(dry_run_output):
    cells = _cells(dry_run_output)
    expected = {
        (cue, cond, worker.slug)
        for cue in run.DEFAULT_CUES
        for cond in ("A", "B")
        for worker in run.WORKERS
    }
    assert cells == expected
    # 10 cues x 2 conditions x 4 workers, each exactly once.
    assert len(_dispatch_lines(dry_run_output)) == 80


def test_dry_run_stage_dispatch_workers(dry_run_output):
    lines = _dispatch_lines(dry_run_output)
    for slug, (agent, model) in EXPECTED_AGENTS.items():
        agent_lines = [l for l in lines if f"-{slug}.md" in l]
        assert len(agent_lines) == 20, slug  # 10 cues x 2 conditions
        for line in agent_lines:
            assert "scriptcast.tools.stage_dispatch" in line
            assert f"--agent {agent} " in line
            assert f"--model {model} " in line
            assert "--brief " in line
            assert "--worktree " in line


def test_dry_run_muse_worker(dry_run_output):
    muse_lines = [l for l in _dispatch_lines(dry_run_output) if "-muse.md" in l]
    assert len(muse_lines) == 20  # 10 cues x 2 conditions
    for line in muse_lines:
        assert " muse exec " in line
        assert "scriptcast.tools.stage_dispatch" not in line
        assert "--workspace " in line
        assert "--prompt-file " in line


def test_dry_run_svgo_plan(dry_run_output):
    svgo_lines = [
        l for l in dry_run_output.splitlines()
        if l.startswith("[dry-run] npx svgo ")
    ]
    assert len(svgo_lines) == len(run.DEFAULT_CUES)  # one per cue, condition A
    cues = {Path(l.split()[3]).name.split(".")[0] for l in svgo_lines}
    assert cues == set(run.DEFAULT_CUES)
    for line in svgo_lines:
        assert " -o " in line
        assert ".raw.svg" in line


def test_dry_run_writes_nothing(tmp_path, capsys):
    out_dir = tmp_path / "out"
    results = tmp_path / "results.csv"
    rc = run.main(["--dry-run", "--out-dir", str(out_dir), "--results", str(results)])
    assert rc == 0
    capsys.readouterr()
    assert not out_dir.exists()
    assert not results.exists()


def test_generator_stubs_return_svg_text():
    for name, gen in run.GENERATORS.items():
        svg = gen("a test asset")
        assert "<svg" in svg and "</svg>" in svg
        frag = name.split("-")[0]  # svgen / omnisvg
        if not run._hf_has_mlx_weights(frag):
            assert "TODO" in svg  # no MLX weights cached: stub stays marked


def test_results_csv_columns():
    for col in ("worker_tokens", "tsx_typecheck_pass", "render_pass",
                "svg_node_count", "svg_bytes", "critic_verdict"):
        assert col in run.RESULTS_HEADER
    assert run.CRITIC_PLACEHOLDER


def test_svg_stats_counts_nodes_and_bytes():
    svg = '<svg viewBox="0 0 10 10"><g><path d="M0 0h10"/><circle r="3"/></g></svg>'
    nodes, nbytes = run.svg_stats(svg)
    assert nodes == 4  # svg, g, path, circle
    assert nbytes == len(svg.encode())


def test_parse_tokens():
    assert run.parse_tokens('{"usage": {"totalTokens": 1234}}') == 1234
    assert run.parse_tokens("tokens used: 987") == 987
    assert run.parse_tokens("no usage here") is None


def test_candidates_md_lists_fifteen():
    text = CANDIDATES_MD.read_text()
    entries = re.findall(r"^## \d+\. ([a-z0-9-]+)$", text, re.MULTILINE)
    assert len(entries) == 15
    assert set(entries) == {c.slug for c in run.CANDIDATES}
    for cand in run.CANDIDATES:
        assert f"{cand.file}:{cand.line}" in text
        assert f"`{cand.cue}`" in text


@pytest.mark.skipif(not STAGE_DIR.is_dir(), reason="YT02 stage checkout not present")
def test_candidates_resolve_against_stage():
    ov = (STAGE_DIR / "stage_ov.yaml").read_text()
    for cand in run.CANDIDATES:
        assert (STAGE_DIR / cand.file).is_file(), cand.slug
        assert f"{cand.cue}:" in ov, cand.slug
