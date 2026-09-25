import json
import shlex
from copy import deepcopy
from pathlib import Path
from types import SimpleNamespace

import pytest

from scriptcast.parser import parse_transcript


def parse_transcript_text(text):
    import tempfile
    with tempfile.NamedTemporaryFile('w', suffix='.md', delete=False) as handle:
        handle.write(text)
    return parse_transcript(Path(handle.name))

from scriptcast.tools import video
from scriptcast.tools.video_plan import build_plan, frame_at


@pytest.fixture
def inputs(tmp_path):
    script = tmp_path / 'shapes.md'
    script.write_text('Archive (00:00)\n[clip:round] A circle.\n\nHost (00:01)\nTwo shapes.\n\nArchive (00:02)\n[clip:square] A square.\n')
    manifest = {'turns': [
        {'turn_index': 0, 'speaker_id': 'archive', 'start_ms': 0, 'end_ms': 1033},
        {'turn_index': 1, 'speaker_id': 'host', 'start_ms': 1033, 'end_ms': 2033},
        {'turn_index': 2, 'speaker_id': 'archive', 'start_ms': 2033, 'end_ms': 4033},
    ]}
    registry = {'clips': {'round': {'source': 'round.mp4', 'start': 2, 'end': 4},
                          'square': {'source': 'square.mp4', 'start': 5, 'end': 6}}}
    specs = {'shots': [{'id': 'S-1', 'anchor': {'turn_index': 1, 'start_ms': 1033}, 'hold': {}}]}
    card = {'component': 'TitleCard', 'headline': 'Shapes', 'items': [{'title': 'Circle'}], 'sources': 'Synthetic example'}
    return dict(manifest=manifest, script_turns=parse_transcript(script), specs=specs,
                registry=registry, cards={'s1': card}, audio=str(tmp_path / 'voice.mp3'), project_root=tmp_path)


def test_order_trim_hold_and_purity(inputs):
    before = deepcopy(inputs)
    plan = build_plan(**inputs)
    assert inputs == before
    assert [(c['turn_index'], c['clip_id'], c['start_ms']) for c in plan['clips']] == [(0, 'round', 0), (2, 'square', 2033)]
    assert plan['clips'][0]['in_ms'] == 2000
    assert plan['clips'][0]['out_ms'] == 3033
    assert plan['clips'][1]['out_ms'] == 6000
    assert plan['warnings'] == ['square: hold last frame for 1000 ms']
    assert plan['duration_ms'] == 4033
    assert plan['cards'][0]['end_ms'] == 2033


def test_missing_card(inputs):
    inputs['cards'] = {}
    plan = build_plan(**inputs)
    assert plan['cards'] == []
    assert 'S-1: no card file; skipped' in plan['warnings']


def test_overlap_and_case(inputs):
    inputs['cards']['S-2'] = inputs['cards'].pop('s1')
    inputs['cards']['s-1'] = inputs['cards']['S-2']
    inputs['specs']['shots'][0]['hold'] = {'script_ms': 2500}
    inputs['specs']['shots'].append({'id': 's-2', 'anchor': {'turn_index': 2, 'start_ms': 2033}})
    plan = build_plan(**inputs)
    assert [(c['start_ms'], c['end_ms']) for c in plan['cards']] == [(1033, 2033), (2033, 4033)]
    assert any('overlap shortened' in w for w in plan['warnings'])


def test_shared_frame_boundaries(inputs):
    plan = build_plan(**inputs)
    windows = [plan['clips'][0], plan['cards'][0], plan['clips'][1]]
    frames = [range(frame_at(w['start_ms']), frame_at(w['end_ms'])) for w in windows]
    assert [f for window in frames for f in window] == list(range(frame_at(plan['duration_ms'])))
    assert frame_at(50) == 2  # JS half-up, not Python bankers rounding


def test_missing_clip_names_id(inputs):
    del inputs['registry']['clips']['square']
    with pytest.raises(ValueError, match='square'):
        build_plan(**inputs)


def test_archive_count_mismatch(inputs):
    inputs['manifest']['turns'].pop()
    with pytest.raises(ValueError, match='count mismatch'):
        build_plan(**inputs)


def test_equal_start_and_subframe_cards(inputs):
    inputs['specs']['shots'] += [{'id': 'S-2', 'anchor': {'start_ms': 1033, 'turn_index': 1}},
                               {'id': 'S-3', 'anchor': {'start_ms': 2033, 'turn_index': 2}, 'hold': {'script_ms': 1}}]
    inputs['cards'].update({'s2': inputs['cards']['s1'], 's3': inputs['cards']['s1']})
    plan = build_plan(**inputs)
    assert [c['shot_id'] for c in plan['cards']] == ['S-2']


@pytest.fixture
def cli(inputs, tmp_path, monkeypatch):
    root = tmp_path / 'video'
    (root / 'src').mkdir(parents=True)
    (root / 'src/index.ts').touch()
    (root / 'node_modules/.bin').mkdir(parents=True)
    (root / 'node_modules/.bin/remotion').touch()
    monkeypatch.setattr(video, 'VIDEO_DIR', root)
    monkeypatch.setenv('SCRIPTCAST_CHROME', '/browser path/chrome')
    for media in ('voice.mp3', 'round.mp4', 'square.mp4'):
        (tmp_path / media).write_bytes(b'synthetic media')
    (tmp_path / 'episode_manifest.json').write_text(json.dumps(inputs['manifest']))
    (tmp_path / 'specs.json').write_text(json.dumps(inputs['specs']))
    (tmp_path / 'clips.yaml').write_text(json.dumps(inputs['registry']))
    cards = tmp_path / 'cards'
    cards.mkdir()
    (cards / 'S-1.json').write_text(json.dumps(inputs['cards']['s1']))
    args = ['render', str(tmp_path), '--script', str(tmp_path / 'shapes.md'), '--specs', str(tmp_path / 'specs.json'),
            '--clips', str(tmp_path / 'clips.yaml'), '--cards', str(cards), '--out', str(tmp_path / 'out file.mp4'),
            '--project-root', str(tmp_path)]
    return args, root, tmp_path / 'out file.mp4.plan.json'


def test_plan_only(cli, monkeypatch):
    args, root, out = cli
    monkeypatch.setattr(video.subprocess, 'run', lambda *a, **kw: pytest.fail('No Node'))
    (root / 'node_modules/.bin/remotion').unlink()
    assert video.main(args + ['--plan-only']) == 0
    plan = json.loads(out.read_text())
    for src in [plan['audio'], *(c['src'] for c in plan['clips'])]:
        assert src.startswith('episode/run-')
        assert (root / 'public' / src).read_bytes() == b'synthetic media'


def test_dry_run_and_render_argv(cli, monkeypatch, capsys):
    args, root, out = cli
    calls = []
    monkeypatch.setattr(video.subprocess, 'run', lambda cmd, **kw: calls.append((cmd, kw)) or SimpleNamespace(returncode=7))
    assert video.main(args + ['--dry-run']) == 0
    expected = ['npx', '--no-install', 'remotion', 'render', 'src/index.ts', 'Episode',
                str(out).removesuffix('.plan.json'), f'--props={out}', '--codec', 'h264', '--timeout=120000', '--concurrency=3', '--browser-executable=/browser path/chrome']
    assert shlex.split(capsys.readouterr().out) == expected
    assert calls == []
    assert video.main(args) == 7
    assert calls == [(expected, {'cwd': root, 'check': False})]


@pytest.mark.parametrize('count', [0, 2])
def test_audio_selection(cli, monkeypatch, count):
    args, root, out = cli
    voice = out.parent / 'voice.mp3'
    if count == 0:
        voice.unlink()
    else:
        (out.parent / 'other.mp3').touch()
    monkeypatch.setattr(video.subprocess, 'run', lambda *a, **kw: pytest.fail('No Node'))
    with pytest.raises(SystemExit):
        video.main(args + ['--plan-only'])
    voice.write_bytes(b'synthetic media')
    assert video.main(args + ['--plan-only', '--audio', str(voice)]) == 0


def test_persist_cards_until_next_card_or_clip(inputs):
    inputs['specs']['shots'] = [{'id': 'S-1', 'anchor': {'turn_index': 1, 'start_ms': 1033}, 'hold': {'script_ms': 100}}]
    plan = build_plan(**inputs)
    assert plan['cards'][0]['end_ms'] == 1133
    persisted = build_plan(**inputs, persist_cards=True)
    # the next archive clip starts at 2033, so the card stops there instead of covering it
    assert persisted['cards'][0]['end_ms'] == 2033
    assert plan['cards'][0]['end_ms'] == 1133


def test_persist_cards_between_cards_and_last_to_end(inputs):
    inputs['manifest']['turns'].append({'turn_index': 3, 'speaker_id': 'host', 'start_ms': 4033, 'end_ms': 6033})
    inputs['script_turns'] = parse_transcript_text('Archive (00:00)\n[clip:round] A circle.\n\nHost (00:01)\nTwo shapes.\n\nArchive (00:02)\n[clip:square] A square.\n\nHost (00:03)\nDone.\n')
    inputs['specs']['shots'] = [
        {'id': 'S-1', 'anchor': {'turn_index': 1, 'start_ms': 1033}, 'hold': {'script_ms': 100}},
        {'id': 'S-2', 'anchor': {'turn_index': 3, 'start_ms': 4033}, 'hold': {'script_ms': 100}},
    ]
    inputs['cards']['s2'] = deepcopy(inputs['cards']['s1'])
    persisted = build_plan(**inputs, persist_cards=True)
    assert [(c['shot_id'], c['start_ms'], c['end_ms']) for c in persisted['cards']] == [('S-1', 1033, 2033), ('S-2', 4033, 6033)]
