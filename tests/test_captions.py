from pathlib import Path

from scriptcast.parser import parse_transcript
from scriptcast.tools.captions import build_captions, chunk_words, normalize
from scriptcast.tools.video_plan import build_plan


def _script(tmp_path):
    path = tmp_path / 's.md'
    path.write_text('Archive (00:00)\n[clip:x] Hello there.\n\nHost (00:01)\nOne two three four, five six seven eight nine ten eleven twelve.\n\nHost (00:02)\nGoodbye now.\n')
    return parse_transcript(path)


def _asr(words, start, step=0.4):
    return [{'word': w, 'start': start + i * step, 'end': start + i * step + 0.3} for i, w in enumerate(words)]


def test_normalize_strips_punctuation():
    assert normalize('Yes-on-9,') == 'yeson9'


def test_captions_skip_clips_and_follow_audio(tmp_path):
    turns = _script(tmp_path)
    manifest = {'turns': [
        {'turn_index': 0, 'start_ms': 0, 'end_ms': 1000},
        {'turn_index': 1, 'start_ms': 1000, 'end_ms': 7000},
        {'turn_index': 2, 'start_ms': 7000, 'end_ms': 8000},
    ]}
    heard = _asr(['hello', 'there'], 0.0) + _asr('one two three four five six seven eight nine ten eleven twelve'.split(), 1.2) \
        + _asr(['goodbye', 'now'], 7.1)
    captions = build_captions(heard, turns, manifest, max_chars=20)
    assert all(c['turn_index'] != 0 for c in captions)
    assert captions[0]['text'] == 'One two three four,'
    assert captions[0]['start_ms'] == 1200
    assert captions[-1]['text'] == 'Goodbye now.'
    assert all(len(c['text']) <= 20 for c in captions)
    assert all(a['end_ms'] <= b['start_ms'] for a, b in zip(captions, captions[1:]))


def test_unmatched_words_are_interpolated_inside_the_turn(tmp_path):
    turns = _script(tmp_path)
    manifest = {'turns': [
        {'turn_index': 0, 'start_ms': 0, 'end_ms': 1000},
        {'turn_index': 1, 'start_ms': 1000, 'end_ms': 7000},
        {'turn_index': 2, 'start_ms': 7000, 'end_ms': 8000},
    ]}
    heard = _asr(['hello', 'there'], 0.0) + _asr(['one', 'twelve'], 1.2) + _asr(['goodbye', 'now'], 7.1)
    captions = build_captions(heard, turns, manifest)
    middle = [c for c in captions if c['turn_index'] == 1]
    assert middle and all(1000 <= c['start_ms'] <= 7200 for c in middle)


def test_chunks_break_at_sentences():
    words = [{'text': t} for t in ['Hi.', 'Then', 'more', 'words', 'here.']]
    assert [[w['text'] for w in c] for c in chunk_words(words, 40)] == [['Hi.'], ['Then', 'more', 'words', 'here.']]


def test_plan_carries_validated_captions(tmp_path):
    turns = _script(tmp_path)
    manifest = {'turns': [
        {'turn_index': 0, 'speaker_id': 'archive', 'start_ms': 0, 'end_ms': 1000},
        {'turn_index': 1, 'speaker_id': 'host', 'start_ms': 1000, 'end_ms': 2000},
        {'turn_index': 2, 'speaker_id': 'host', 'start_ms': 2000, 'end_ms': 3000},
    ]}
    plan = build_plan(manifest, turns, {'shots': []}, {'clips': {'x': {'source': 'x.mp4', 'start': 0, 'end': 1}}}, {},
                      audio=str(tmp_path / 'a.mp3'), project_root=tmp_path,
                      captions=[{'start_ms': 1500, 'end_ms': 9000, 'text': ' Hi '}, {'start_ms': 100, 'end_ms': 900, 'text': 'First'}])
    assert [c['text'] for c in plan['captions']] == ['First', 'Hi']
    assert plan['captions'][1]['end_ms'] == 3000
