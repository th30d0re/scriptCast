import json
import pytest
from unittest.mock import patch

from scriptcast.tools.stage import main


def test_boxes_dry_run_uses_same_shot_frame_math(tmp_path, capsys):
    episode = tmp_path / "episode"; episode.mkdir()
    (episode / "stage.yaml").write_text("episode: demo\n")
    plan = episode / "stage_plan.json"
    plan.write_text(json.dumps({
        "episode_dir": str(episode), "fps": 30, "range": {"start_ms": 1000, "end_ms": 5000},
        "segments": [{"shots": [{"shot_id": "S00-13", "start_ms": 2000, "end_ms": 3000}]}],
        "audio": "",
    }))
    output = tmp_path / "boxes.json"
    with patch("scriptcast.tools.stage.video.browser_args", return_value=[]):
        assert main(["boxes", str(plan), "--shot", "S00-13", "--at", ".8", "-o", str(output), "--dry-run"]) == 0
    line = capsys.readouterr().out
    # ceil((2800 - 1000) * 30 / 1000), exactly as ``still`` computes it.
    assert "scripts/stage-marks.mjs" in line and "--frame 54" in line
    assert str(output) in line


@pytest.mark.parametrize('custom_chrome', [None, '/custom/Chrome'])
def test_boxes_passes_plan_episode_dir_to_node(tmp_path, monkeypatch, custom_chrome):
    episode = tmp_path / 'external-episode'
    episode.mkdir()
    (episode / 'stage.yaml').write_text('episode: demo\n')
    plan = tmp_path / 'stage_plan.json'
    plan.write_text(json.dumps({
        'episode_dir': str(episode), 'fps': 30,
        'range': {'start_ms': 0, 'end_ms': 1000},
        'segments': [{'shots': [{'shot_id': 'S01', 'start_ms': 0, 'end_ms': 1000}]}],
    }))
    monkeypatch.setenv('STAGE_EPISODE_DIR', '/stale/episode')
    chrome = tmp_path / 'Google Chrome'
    chrome.touch()
    monkeypatch.setattr('scriptcast.tools.stage.video.DEFAULT_CHROME', chrome)
    if custom_chrome:
        monkeypatch.setenv('SCRIPTCAST_CHROME', custom_chrome)
    else:
        monkeypatch.delenv('SCRIPTCAST_CHROME', raising=False)
    with patch('scriptcast.tools.stage.subprocess.run') as run:
        run.return_value.returncode = 0
        assert main(['boxes', str(plan), '--shot', 'S01', '-o', str(tmp_path / 'boxes.json')]) == 0
    assert run.call_args.kwargs['env']['STAGE_EPISODE_DIR'] == str(episode.resolve())
    assert run.call_args.kwargs['env']['SCRIPTCAST_CHROME'] == (custom_chrome or str(chrome))
    assert run.call_args.args[0][:2] == ['node', 'scripts/stage-marks.mjs']
