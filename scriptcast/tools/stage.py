"""Plan, render and preview time-driven Stage episodes."""
import argparse
import copy
import json
import math
import os
from pathlib import Path
import shlex
import subprocess
from datetime import datetime, timezone
from scriptcast.tools import video
from scriptcast.tools.stage_plan import compile_plan


def append_rejection(episode_dir: Path, *, shot: str, still_path: Path,
                     frame: int, reason: str, element_id: str | None = None,
                     ts: str | None = None) -> Path:
    """Append one human still rejection to ``stage/review/rejections.jsonl``."""
    reason = reason.strip()
    if not reason:
        raise ValueError("rejection reason must not be empty")
    path = Path(episode_dir) / 'review' / 'rejections.jsonl'
    path.parent.mkdir(parents=True, exist_ok=True)
    record = {
        'shot': shot,
        'still_path': str(Path(still_path).resolve()),
        'frame': int(frame),
        'reason': reason,
        'ts': ts or datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ'),
    }
    if element_id:
        record['element_id'] = element_id
    with path.open('a', encoding='utf-8') as handle:
        handle.write(json.dumps(record, ensure_ascii=False) + '\n')
        handle.flush()
        os.fsync(handle.fileno())
    return path


def export_kit_review(input_path: Path, output_path: Path) -> Path:
    """Export actionable Kit Review entries as a dispatch-ready Markdown file."""
    raw = json.loads(input_path.read_text(encoding='utf-8')) if input_path.exists() else {'version': 1, 'items': {}}
    if raw.get('version') != 1 or not isinstance(raw.get('items'), dict):
        raise ValueError(f'Invalid kit review file: {input_path}')
    priority_order = {'P0': 0, 'P1': 1, 'P2': 2, 'P3': 3, None: 4}
    rows = []
    for item_id, item in raw['items'].items():
        # Variant levels are actionable even on Keep items: they say which variant becomes the default.
        if not isinstance(item, dict) or (item.get('status') not in {'change', 'cut'} and not item.get('variants')):
            continue
        priority = item.get('priority')
        if priority not in priority_order:
            raise ValueError(f'Invalid priority for {item_id}')
        rows.append((priority_order[priority], item_id, item))
    rows.sort(key=lambda row: (row[0], row[1]))
    lines = ['# Kit review export', '']
    for _, item_id, item in rows:
        priority = item.get('priority') or 'None'
        lines.extend([f'## {item_id}', '', f'- Status: {item["status"]}', f'- Priority: {priority}'])
        variants = item.get('variants') or {}
        if variants:
            ranked = sorted(variants.items(), key=lambda kv: (priority_order.get(kv[1], 4), kv[0]))
            lines.append('- Variant levels: ' + ', '.join(f'{name} {level}' for name, level in ranked))
        tags = item.get('tags', [])
        if tags:
            lines.append(f'- Tags: {", ".join(tags)}')
        lines.extend(['', item.get('notes', ''), ''])
    if not rows:
        lines.extend(['No items marked Change or Cut.', ''])
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text('\n'.join(lines), encoding='utf-8')
    return output_path


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    p = sub.add_parser('plan')
    p.add_argument('source', type=Path)
    p.add_argument('--range')
    p.add_argument('-o', type=Path)
    p = sub.add_parser('reject', help='record a rejected Stage still')
    p.add_argument('plan', type=Path)
    p.add_argument('--shot', required=True)
    p.add_argument('--reason', required=True)
    p.add_argument('--still', type=Path, help='still image path (defaults to <plan-dir>/<shot>.png)')
    p.add_argument('--frame', type=int, help='render frame (defaults to the shot midpoint)')
    p.add_argument('--element-id')
    p = sub.add_parser('kit-review', help='manage Kit Review exports')
    kit_sub = p.add_subparsers(dest='kit_review_command', required=True)
    p = kit_sub.add_parser('export', help='export Change and Cut kit reviews as Markdown')
    repo_root = Path(__file__).resolve().parents[2]
    p.add_argument('--input', type=Path, default=repo_root / 'review' / 'kit-review.json')
    p.add_argument('--output', type=Path, default=repo_root / 'review' / 'kit-review.md')
    for name in ('render', 'still', 'boxes', 'site'):
        p = sub.add_parser(name)
        p.add_argument('plan', type=Path)
        p.add_argument('--dry-run', action='store_true')
        if name != 'site':
            p.add_argument('-o', type=Path)
            p.add_argument('--concurrency', type=int, default=8)
            p.add_argument('--timeout-ms', type=int, default=120000)
            p.add_argument('--scale', type=float, default=1.0, help='Output scale; 2 renders a 1080p plan at 4K')
        if name == 'render':
            p.add_argument('--per-shot', type=Path)
            p.add_argument('--chunks', type=int, default=1)
            p.add_argument('--chunk-concurrency', type=int)
        if name in ('still', 'boxes'):
            p.add_argument('--shot', required=True)
            p.add_argument('--at', type=float, default=0.5, help='Fraction through the shot, 0..1')
        if name == 'still':
            p.add_argument('--boxes', action='store_true', help='also write <output>.boxes.json using live Stage marks')
    args = parser.parse_args(argv)
    try:
        if args.command == 'plan':
            plan = compile_plan(args.source, args.range, video.VIDEO_DIR / 'public')
            out = args.o or args.source.parent / 'stage_plan.json'
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text(json.dumps(plan, indent=2) + '\n')
            print(f'wrote {out}')
            return 0
        if args.command == 'kit-review':
            output = export_kit_review(args.input, args.output)
            print(f'wrote {output}')
            return 0
        plan_path = args.plan.resolve()
        plan = json.loads(plan_path.read_text())
        episode_dir = Path(plan.get('episode_dir', plan_path.parent)).resolve()
        if not (episode_dir / 'stage.yaml').is_file():
            raise ValueError(f'No stage.yaml in {episode_dir}')
        env = dict(os.environ, STAGE_EPISODE_DIR=str(episode_dir))
        prefix = f'STAGE_EPISODE_DIR={shlex.quote(str(episode_dir))}'
        if args.command == 'reject':
            shots = [s for seg in plan['segments'] for s in seg['shots']]
            shot = next((s for s in shots if s['shot_id'] == args.shot), None)
            if shot is None:
                raise ValueError(f'reject requires a known --shot (got {args.shot!r})')
            fps = plan['fps']
            start = plan['range']['start_ms']
            frame = args.frame
            if frame is None:
                midpoint = (shot['start_ms'] + shot['end_ms']) / 2
                frame = math.ceil((midpoint - start) * fps / 1000)
            still = args.still or plan_path.parent / f'{args.shot}.png'
            path = append_rejection(
                episode_dir, shot=args.shot, still_path=still, frame=frame,
                reason=args.reason, element_id=args.element_id,
            )
            print(f'appended rejection to {path}')
            return 0
        def run(command):
            if args.dry_run:
                print(prefix, shlex.join(command)); return 0
            return subprocess.run(command, cwd=video.VIDEO_DIR, env=env, check=False).returncode
        if args.command == 'site':
            env['STAGE_PLAN'] = str(plan_path)
            prefix += f' STAGE_PLAN={shlex.quote(str(plan_path))}'
            return run(['npx', '--no-install', 'vite', '--config', 'site/vite.config.ts'])
        shots = [s for seg in plan['segments'] for s in seg['shots']]
        fps = plan['fps']
        start = plan['range']['start_ms']
        frame = lambda ms: math.ceil((ms-start)*fps/1000)
        total = max(1, frame(plan['range']['end_ms']))
        common = [f'--props={plan_path}', f'--timeout={args.timeout_ms}']
        if args.scale != 1.0:
            common.append(f'--scale={args.scale:g}')
        if args.command in ('still', 'boxes'):
            chrome_args = video.browser_args()
            if chrome_args:
                env['SCRIPTCAST_CHROME'] = chrome_args[0].split('=', 1)[1]
                prefix += f' SCRIPTCAST_CHROME={shlex.quote(env["SCRIPTCAST_CHROME"])}'
            shot = next((s for s in shots if s['shot_id'] == args.shot), None)
            if shot is None or not 0 <= args.at <= 1 or not args.o:
                raise ValueError(f'{args.command} requires a known --shot, --at in 0..1, and -o')
            at = min(frame(shot['end_ms'])-1, frame(shot['start_ms'] + args.at*(shot['end_ms']-shot['start_ms'])))
            boxes = args.o.resolve() if args.command == 'boxes' else args.o.resolve().with_name(args.o.name + '.boxes.json')
            mark_command = ['node', 'scripts/stage-marks.mjs', '--plan', str(plan_path), '--frame', str(at), '--output', str(boxes)]
            if args.command == 'boxes':
                return run(mark_command)
            code = run(['npx', '--no-install', 'remotion', 'still', 'src/stage/index.ts', 'StageStill', str(args.o.resolve()), *common, f'--frame={at}', *chrome_args])
            return run(mark_command) if not code and args.boxes else code
        base = ['npx', '--no-install', 'remotion', 'render', 'src/stage/index.ts', 'StageEpisode']
        common += ['--codec', 'h264']
        if args.per_shot:
            jobs = [(args.per_shot / (s['shot_id']+'.mp4'), frame(s['start_ms']), frame(s['end_ms'])-1) for s in shots]
        elif args.o:
            jobs = [(args.o, 0, total-1)]
        else:
            raise ValueError('render requires -o or --per-shot')
        for out, first, last in jobs:
            out = out.resolve()
            if not args.dry_run: out.parent.mkdir(parents=True, exist_ok=True)
            ranges = video.chunk_ranges(first, last, args.chunks)
            if len(ranges) == 1:
                extra = [f'--frames={first}-{last}', '--muted'] if args.per_shot else []
                code = run([*base, str(out), *common, *extra, f'--concurrency={args.concurrency}', *video.browser_args()])
            else:
                # Reuse the existing chunk renderer for video only, then mux the exact range.
                chunk_args = copy.copy(args)
                chunk_args.frames = (first, last)
                previous = os.environ.get('STAGE_EPISODE_DIR')
                os.environ['STAGE_EPISODE_DIR'] = str(episode_dir)
                try:
                    if args.dry_run: print('export ' + prefix)
                    code = video.render_chunks(base, common, ranges, out, None, plan, chunk_args)
                finally:
                    if previous is None: os.environ.pop('STAGE_EPISODE_DIR', None)
                    else: os.environ['STAGE_EPISODE_DIR'] = previous
                if not code and not args.per_shot and plan['audio']:
                    muxed = out.with_name(out.stem+'.mux.mp4')
                    code = run(['ffmpeg', '-y', '-i', str(out), '-ss', str(start/1000), '-i', str(video.VIDEO_DIR/'public'/plan['audio']), '-t', str(total/fps), '-map', '0:v', '-map', '1:a', '-c:v', 'copy', '-c:a', 'aac', str(muxed)])
                    if not args.dry_run and not code: muxed.replace(out)
            if code: return code
        return 0
    except (OSError, ValueError, KeyError) as exc:
        parser.error(str(exc))


if __name__ == '__main__':
    raise SystemExit(main())
