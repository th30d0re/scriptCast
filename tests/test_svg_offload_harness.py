"""E3 SVG offload harness: the dry-run must plan the full command matrix
(10 cues x 1 condition x 4 workers, condition B) with per-cell worktree
isolation, without running or writing anything."""
import importlib.util
import re
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
RUN_PY = ROOT / "experiments" / "svg_offload" / "run.py"
CUES_CSV = ROOT / "experiments" / "svg_offload" / "cues.csv"
CANDIDATES_MD = ROOT / "experiments" / "svg_offload" / "candidates.md"
BOOK_REPO = Path("/Users/emmanuel/Documents/Theory/TheOriginalPower")
LIVE_YT02 = BOOK_REPO / "Architecting_the_operation" / "video" / "yt02"
STAGE_DIR = LIVE_YT02 / "stage"

spec = importlib.util.spec_from_file_location("svg_offload_run", RUN_PY)
run = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = run  # dataclass needs the module registered
spec.loader.exec_module(run)

# Operator-approved selection (see task brief); cues.csv must match exactly.
EXPECTED_CUES = [
    "doctrine-chain",
    "practice-network",
    "debate-funnel",
    "router-tree",
    "group-boundary",
    "compressed-bus",
    "term-glyphs",
    "comparator-loop",
    "pulse-network",
    "dual-rails",
]

# brief lives at out/<cue>-<cond>-<worker>/brief.md
BRIEF_RE = re.compile(r"([a-z0-9-]+)-([AB])-([a-z0-9-]+)/brief\.md")

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


def test_cues_csv_is_the_approved_selection():
    assert CUES_CSV.is_file()
    assert run.load_cues(CUES_CSV) == EXPECTED_CUES
    assert run.DEFAULT_CUES == EXPECTED_CUES
    assert set(EXPECTED_CUES) <= {c.slug for c in run.CANDIDATES}


def test_default_matrix_shape():
    assert len(run.DEFAULT_CUES) == 10
    assert len(run.WORKERS) == 4
    assert run.CONDITIONS == ("A", "B")


def test_dry_run_dispatch_matrix(dry_run_output):
    cells = _cells(dry_run_output)
    expected = {
        (cue, "B", worker.slug)
        for cue in run.DEFAULT_CUES
        for worker in run.WORKERS
    }
    assert cells == expected
    # 10 cues x 1 condition (B) x 4 workers, each exactly once.
    assert len(_dispatch_lines(dry_run_output)) == 40


def test_dry_run_stage_dispatch_workers(dry_run_output):
    lines = _dispatch_lines(dry_run_output)
    for slug, (agent, model) in EXPECTED_AGENTS.items():
        agent_lines = [l for l in lines if f"-{slug}/brief.md" in l]
        assert len(agent_lines) == 10, slug  # 10 cues x 1 condition
        for line in agent_lines:
            assert "scriptcast.tools.stage_dispatch" in line
            assert f"--agent {agent} " in line
            assert f"--model {model} " in line
            assert "--brief " in line
            assert "--worktree " in line


def test_dry_run_muse_worker(dry_run_output):
    muse_lines = [l for l in _dispatch_lines(dry_run_output) if "-muse/brief.md" in l]
    assert len(muse_lines) == 10  # 10 cues x 1 condition
    for line in muse_lines:
        assert " muse exec " in line
        assert "scriptcast.tools.stage_dispatch" not in line
        assert "--workspace " in line
        assert "--prompt-file " in line
        assert "--approval-mode never" in line


def test_dry_run_isolation_paths(dry_run_output):
    """Workers never see the live checkout: every dispatch worktree/workspace
    is the yt02 subdir of a per-cell worktree under out/wt/."""
    lines = _dispatch_lines(dry_run_output)
    assert lines
    worktree_paths = set()
    for line in lines:
        m = re.search(r"--(?:worktree|workspace) (\S+)", line)
        assert m, line
        wt = m.group(1)
        assert str(LIVE_YT02) not in line  # never the live repo
        assert f"{run.YT02_SUBDIR}" in wt  # yt02 subdir inside the worktree
        assert "/wt/" in wt
        worktree_paths.add(wt)
    # one distinct worktree per cell
    assert len(worktree_paths) == len(lines) == 40


def test_dry_run_plans_worktree_add_and_remove(dry_run_output):
    planned = [
        l[len("[dry-run] "):] for l in dry_run_output.splitlines()
        if l.startswith("[dry-run] ")
    ]
    adds = [l for l in planned if " worktree add --detach " in l]
    removes = [l for l in planned if " worktree remove --force " in l]
    diffs = [l for l in planned if re.search(r"git -C \S+ diff$", l)]
    assert len(adds) == len(removes) == len(diffs) == 40
    for line in adds + removes:
        assert line.startswith(f"git -C {BOOK_REPO} worktree ")
    for line in adds:
        assert line.endswith(" HEAD")
    # worktree add targets sit under out/wt/<cell>, never inside the live repo
    for line in adds:
        target = line.split("--detach ", 1)[1].rsplit(" HEAD", 1)[0]
        assert str(BOOK_REPO) not in target
        assert "/wt/" in target
    # diffs run inside the per-cell worktree, not the live repo
    for line in diffs:
        wt = line.split("git -C ", 1)[1].rsplit(" diff", 1)[0]
        assert str(BOOK_REPO) not in wt
        assert "/wt/" in wt


def test_dry_run_header_reports_concurrency_and_timeout(dry_run_output):
    header = dry_run_output.splitlines()[0]
    assert "concurrency=2" in header  # default
    assert "timeout=20" in header     # default minutes


def test_dry_run_respects_concurrency_flag(capsys):
    rc = run.main(["--dry-run", "--concurrency", "3", "--timeout-min", "5"])
    assert rc == 0
    header = capsys.readouterr().out.splitlines()[0]
    assert "concurrency=3" in header
    assert "timeout=5" in header


def test_no_svgo_plan_without_condition_a(dry_run_output):
    svgo_lines = [
        l for l in dry_run_output.splitlines()
        if l.startswith("[dry-run] npx svgo ")
    ]
    assert svgo_lines == []  # svgo belongs to condition A, gated off by default


def test_condition_a_exits_when_weights_missing(capsys):
    if run._hf_has_mlx_weights("svgen"):
        pytest.skip("svgen weights cached locally; gate is open")
    for argv in (["--conditions", "A"], ["--conditions", "A", "--dry-run"],
                 ["--conditions", "A,B"]):
        rc = run.main(argv)
        assert rc == 2, argv
        err = capsys.readouterr().err
        assert "weights missing" in err
        assert "svgen-7b" in err


def test_condition_a_gate_mentions_omnisvg_when_selected(capsys):
    if run._hf_has_mlx_weights("omnisvg"):
        pytest.skip("omnisvg weights cached locally; gate is open")
    rc = run.main(["--conditions", "A", "--generator", "omnisvg-1.1"])
    assert rc == 2
    assert "weights missing" in capsys.readouterr().err


def test_dry_run_writes_nothing(tmp_path, capsys):
    out_dir = tmp_path / "out"
    results = tmp_path / "results.csv"
    rc = run.main(["--dry-run", "--out-dir", str(out_dir), "--results", str(results)])
    assert rc == 0
    capsys.readouterr()
    assert not out_dir.exists()
    assert not results.exists()


def test_worker_commands_run_detached_from_stdin_and_live_repo():
    """The dispatch command targets the per-cell worktree; run_cell wires
    stdin=DEVNULL (codex hangs on an inherited stdin)."""
    cand = next(c for c in run.CANDIDATES if c.slug == "router-tree")
    brief = Path("/tmp/e3-cell/brief.md")
    wt = Path("/tmp/e3-wt/router-tree-B-muse") / run.YT02_SUBDIR
    muse = next(w for w in run.WORKERS if w.agent == "muse")
    cmd = run.worker_command(muse, brief, wt, cand, "B")
    assert cmd[:2] == ["muse", "exec"]
    assert cmd[cmd.index("--approval-mode") + 1] == "never"
    assert str(LIVE_YT02) not in " ".join(cmd)
    src = (RUN_PY).read_text()
    assert src.count("stdin=subprocess.DEVNULL") >= 5  # every subprocess.run


def test_generator_stubs_return_svg_text():
    for name, gen in run.GENERATORS.items():
        svg = gen("a test asset")
        assert "<svg" in svg and "</svg>" in svg
        frag = name.split("-")[0]  # svgen / omnisvg
        if not run._hf_has_mlx_weights(frag):
            assert "TODO" in svg  # no MLX weights cached: stub stays marked


def test_results_csv_columns():
    for col in ("worker_tokens", "tsx_typecheck_pass", "render_pass",
                "svg_node_count", "svg_bytes", "status", "critic_verdict"):
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
