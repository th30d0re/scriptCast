"""Plan, render and preview time-driven Stage episodes."""
import argparse
import copy
import json
import math
import os
from pathlib import Path
import shlex
import subprocess
from scriptcast.tools import video
from scriptcast.tools.stage_plan import compile_plan


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    p = sub.add_parser('plan')
    p.add_argument('source', type=Path)
    p.add_argument('--range')
    p.add_argument('-o', type=Path)
    for name in ('render', 'still', 'site'):
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
        if name == 'still':
            p.add_argument('--shot', required=True)
            p.add_argument('--at', type=float, default=0.5, help='Fraction through the shot, 0..1')
    args = parser.parse_args(argv)
    try:
        if args.command == 'plan':
            plan = compile_plan(args.source, args.range, video.VIDEO_DIR / 'public')
            out = args.o or args.source.parent / 'stage_plan.json'
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text(json.dumps(plan, indent=2) + '\n')
            print(f'wrote {out}')
            return 0
        plan_path = args.plan.resolve()
        plan = json.loads(plan_path.read_text())
        episode_dir = Path(plan.get('episode_dir', plan_path.parent)).resolve()
        if not (episode_dir / 'stage.yaml').is_file():
            raise ValueError(f'No stage.yaml in {episode_dir}')
        env = dict(os.environ, STAGE_EPISODE_DIR=str(episode_dir))
        prefix = f'STAGE_EPISODE_DIR={shlex.quote(str(episode_dir))}'
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
        if args.command == 'still':
            shot = next((s for s in shots if s['shot_id'] == args.shot), None)
            if shot is None or not 0 <= args.at <= 1 or not args.o:
                raise ValueError('still requires a known --shot, --at in 0..1, and -o')
            at = min(frame(shot['end_ms'])-1, frame(shot['start_ms'] + args.at*(shot['end_ms']-shot['start_ms'])))
            return run(['npx', '--no-install', 'remotion', 'still', 'src/stage/index.ts', 'StageStill', str(args.o.resolve()), *common, f'--frame={at}', *video.browser_args()])
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
