"""Compile spoken-word cues into an episode-absolute Stage plan."""
from collections import Counter
import json
from pathlib import Path
import re
import shutil
import yaml


def token(text):
    return re.sub(r'^[\W_]+|[\W_]+$', '', text.lower())


def normalize_words(manifest, timings):
    turns = {t['turn_id']: t for t in manifest['turns']}
    votes = set()
    for tid, words in timings['turns'].items():
        if tid not in turns or not words or not turns[tid]['start_ms']:
            continue
        t = turns[tid]
        lo, hi = min(w['start_ms'] for w in words), max(w['end_ms'] for w in words)
        relative = 0 <= lo and hi <= t['end_ms'] - t['start_ms'] + 250
        absolute = t['start_ms'] <= lo and hi <= t['end_ms'] + 250
        if relative != absolute:
            votes.add('relative' if relative else 'absolute')
    if len(votes) > 1:
        raise ValueError('Mixed relative and absolute word timings')
    mode = next(iter(votes), 'relative')
    normalized = {}
    for tid, words in timings['turns'].items():
        if tid not in turns:
            continue
        offset = turns[tid]['start_ms'] if mode == 'relative' else 0
        normalized[tid] = [dict(text=w['text'], start_ms=w['start_ms'] + offset,
                                end_ms=w['end_ms'] + offset) for w in words]
    return normalized, mode


def resolve_cue(cue, turn, words):
    if 'ms' in cue:
        at = turn['start_ms'] + cue['ms']
    else:
        wanted = [token(w) for w in cue['word'].split()]
        tokens = [token(w['text']) for w in words]
        matches = [i for i in range(len(tokens) - len(wanted) + 1)
                   if tokens[i:i + len(wanted)] == wanted]
        occurrence = cue.get('occurrence', 1)
        if not isinstance(occurrence, int) or occurrence < 1 or occurrence > len(matches):
            raise ValueError(f"Missing word {cue['word']!r} occurrence {occurrence} in turn {turn['turn_id']}; tokens: {' '.join(tokens)}")
        edge = cue.get('at', 'start')
        if edge not in ('start', 'end'):
            raise ValueError('Cue at must be start or end')
        index = matches[occurrence - 1] + (len(wanted) - 1 if edge == 'end' else 0)
        at = words[index][edge + '_ms']
    return at + cue.get('offset_ms', 0)


def build_plan(spec, manifest, timings, shot_range=None):
    words, mode = normalize_words(manifest, timings)
    turns = {t['turn_id']: t for t in manifest['turns']}
    source = spec['shots']
    if not source or len({s['shot_id'] for s in source}) != len(source):
        raise ValueError('Shots must be nonempty with unique shot_id values')
    for key in ('fps', 'width', 'height'):
        value = spec.get(key, {'fps': 30, 'width': 1920, 'height': 1080}[key])
        if not isinstance(value, (int, float)) or value <= 0:
            raise ValueError(f'{key} must be positive')
    shots = []
    canonical, aliases = {}, {}
    counts = Counter(name for s in source for name in s.get('cues', {}))
    for i, s in enumerate(source):
        if not re.fullmatch(r'[A-Za-z0-9_-]+', s['shot_id']):
            raise ValueError('shot_id must contain only letters, digits, underscore or hyphen')
        t = turns[s['turn_id']]
        end = turns[source[i+1]['turn_id']]['start_ms'] if i+1 < len(source) else t['end_ms'] + 800
        if end <= t['start_ms']:
            raise ValueError('Shots must have strictly increasing turn start times')
        shots.append(dict(shot_id=s['shot_id'], turn_id=t['turn_id'], turn_index=t['turn_index'],
                          start_ms=t['start_ms'], end_ms=end, props=s.get('props', {})))
        for name, cue in s.get('cues', {}).items():
            value = resolve_cue(cue, t, words.get(t['turn_id'], []))
            canonical[f"{s['shot_id']}.{name}"] = value
            if counts[name] == 1:
                aliases[name] = value
    duration = shots[-1]['end_ms']
    if shot_range:
        ids = [s['shot_id'] for s in shots]
        try:
            first, last = shot_range.split('..')
            a, b = ids.index(first), ids.index(last)
            if a > b: raise ValueError()
        except ValueError:
            raise ValueError(f'Invalid shot range: {shot_range}') from None
        shots, source = shots[a:b+1], source[a:b+1]
    segments = []
    for s, original in zip(shots, source):
        instance = original.get('instance', s['shot_id'])
        if segments and segments[-1]['instance'] == instance:
            if segments[-1]['scene'] != original['scene']:
                raise ValueError('Consecutive instance shots must use the same scene')
            segments[-1]['shots'].append(s)
            segments[-1]['end_ms'] = s['end_ms']
        else:
            segments.append(dict(instance=instance, scene=original['scene'], start_ms=s['start_ms'],
                                 end_ms=s['end_ms'], shots=[s]))
    return dict(episode=spec['episode'], fps=spec.get('fps', 30), width=spec.get('width', 1920),
                height=spec.get('height', 1080), duration_ms=duration,
                range=dict(start_ms=shots[0]['start_ms'], end_ms=shots[-1]['end_ms']), audio='',
                cues={**aliases, **canonical}, segments=segments, words=words,
                warnings=[f'Word timings normalized from {mode} milliseconds'])


def compile_plan(path, shot_range=None, public_dir=None):
    path = Path(path).resolve()
    spec = yaml.safe_load(path.read_text())
    outputs = (path.parent / spec.get('outputs', '.')).resolve()
    manifest = json.loads((outputs / 'episode_manifest.json').read_text())
    timings = json.loads((outputs / 'word_timings.json').read_text())
    plan = build_plan(spec, manifest, timings, shot_range)
    plan['episode_dir'] = str(path.parent)
    episode = spec['episode']
    if Path(episode).name != episode or episode in ('.', '..'):
        raise ValueError('episode must be a simple name')
    candidates = [outputs / f'{episode}.mp3', outputs.parent / f'{episode}.mp3']
    audio = next((p for p in candidates if p.is_file()), None)
    if audio is not None and public_dir is not None:
        target = Path(public_dir) / 'stage' / episode / audio.name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(audio, target)
        plan['audio'] = target.relative_to(public_dir).as_posix()
    elif audio is None:
        plan['warnings'].append('No episode MP3 found; silent episode')
    return plan
