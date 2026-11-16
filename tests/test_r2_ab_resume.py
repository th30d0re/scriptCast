import csv
import json
from pathlib import Path
import subprocess

import pytest

from experiments.router_eval import r2_ab


SESSION = "01a126f5-da05-7222-8a0c-a170a5ff6081"


def fixtures(runs):
    for fix in r2_ab.load_fixes():
        dest = runs / f"{fix['id']}-A"
        dest.mkdir(parents=True)
        (dest / "worker.stdout.tail").write_text(
            f"worker output\nDispatched codex (gpt-5.6-luna). Session ID: {SESSION}\n")
        (dest / "git.diff").write_text("original fix patch")


def test_resume_followup_dry_run_writes_resume_and_fresh_rows(tmp_path, capsys, monkeypatch):
    runs = tmp_path / "runs"
    fixtures(runs)
    def forbidden(*args, **kwargs):
        pytest.fail("dry run launched subprocess")
    monkeypatch.setattr(r2_ab.subprocess, "run", forbidden)

    output = tmp_path / "resume.csv"
    assert r2_ab.main(["--resume-followup", "--runs", str(runs),
                       "--output", str(output)]) == 0
    rows = list(csv.DictReader(output.open()))
    assert len(rows) == 6
    assert {row["condition"] for row in rows} == {"resume", "fresh"}
    assert all(row["tokens_kind"] == "dry-run" for row in rows)
    printed = capsys.readouterr().out
    assert f"resume --session {SESSION} --agent codex" in printed
    assert " start --agent codex" in printed


def test_session_lookup_reads_stdout_tail(tmp_path):
    runs = tmp_path / "runs"
    fixtures(runs)
    assert r2_ab._session_id_from_run("minute-two-figure-centering", runs) == SESSION
    (runs / "minute-two-figure-centering-A" / "worker.stdout.tail").write_text(
        "Dispatched codex (model). Session ID: None\n")
    with pytest.raises(ValueError):
        r2_ab._session_id_from_run("minute-two-figure-centering", runs)


@pytest.mark.parametrize("apply_fails", [False, True])
def test_execute_isolates_arms_and_cleans_up(tmp_path, monkeypatch, apply_fails):
    runs = tmp_path / "runs"
    fixtures(runs)
    book = tmp_path / "book"
    book.mkdir()
    active = {}
    removed = []
    applied = set()
    dispatched = []

    def fake_run(cmd, **kwargs):
        if cmd[:3] == ["git", "-C", str(book)]:
            if cmd[4] == "add":
                worktree = Path(cmd[6])
                active[worktree] = cmd[7]
            else:
                worktree = Path(cmd[-1])
                assert worktree in active
                removed.append(worktree)
                del active[worktree]
                for log in worktree.glob("*.jsonl"):
                    log.unlink()
                worktree.rmdir()
        elif cmd[:2] == ["git", "apply"]:
            worktree = kwargs["cwd"]
            assert worktree in active and worktree != book
            fix = next(f for f in r2_ab.load_fixes() if f"{f['id']}-A" == Path(cmd[2]).parent.name)
            assert active[worktree] == fix["base"]
            assert Path(cmd[2]).read_text() == "original fix patch"
            if apply_fails:
                raise subprocess.CalledProcessError(1, cmd)
            applied.add(worktree)
        elif cmd[:2] == ["git", "diff"]:
            assert kwargs["cwd"] in active
            return subprocess.CompletedProcess(cmd, 0, "saved diff", "")
        else:
            worktree = kwargs["cwd"]
            assert worktree in applied
            assert cmd[cmd.index("--worktree") + 1] == str(worktree)
            assert kwargs["stdin"] == subprocess.DEVNULL
            assert kwargs["env"]["PYTHONPATH"] == str(r2_ab.ROOT.resolve())
            assert cmd[cmd.index("--agent") + 1] == "codex"
            flag = "--message" if cmd[2] == "resume" else "--brief"
            assert Path(cmd[cmd.index(flag) + 1]).is_file()
            dispatched.append(worktree)
            Path(cmd[cmd.index("--log") + 1]).write_text(json.dumps({"tokens": 42, "tokens_kind": "reported"}))
            return subprocess.CompletedProcess(cmd, 0, "worker done", "")
        return subprocess.CompletedProcess(cmd, 0, "", "")

    monkeypatch.setattr(r2_ab.subprocess, "run", fake_run)
    argv = ["--resume-followup", "--execute", "--book-repo", str(book),
            "--runs", str(runs), "--output", str(tmp_path / "out.csv")]
    if apply_fails:
        with pytest.raises(subprocess.CalledProcessError):
            r2_ab.main(argv)
        assert len(removed) == 1 and not dispatched
    else:
        assert r2_ab.main(argv) == 0
        assert len(set(dispatched)) == len(removed) == 6
        with (tmp_path / "out.csv").open() as handle:
            rows = list(csv.DictReader(handle))
        assert all(row["tokens"] == "42" and row["tokens_kind"] == "reported" for row in rows)
        for fix in r2_ab.load_fixes():
            for arm in ("resume", "fresh"):
                dest = runs / f"{fix['id']}-followup-{arm}"
                assert (dest / "git.diff").read_text() == "saved diff"
                assert "worker done" in (dest / "worker.stdout.tail").read_text()
    assert not active
