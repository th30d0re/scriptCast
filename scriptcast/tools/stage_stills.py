"""Render stills for Stage scenes at cue + 700 ms and scene end - 1 frame."""
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from typing import Any, Dict, List, Optional, Tuple

from PIL import Image, ImageDraw, ImageFont

VIDEO_DIR = Path(__file__).resolve().parents[2] / "video"
DEFAULT_CHROME = Path("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")


def get_browser_args() -> List[str]:
    chrome = os.environ.get("SCRIPTCAST_CHROME")
    if not chrome and DEFAULT_CHROME.is_file():
        chrome = str(DEFAULT_CHROME)
    return [f"--browser-executable={chrome}"] if chrome else []


def resolve_scene_stills(
    plan: Dict[str, Any], scenes_filter: Optional[List[str]] = None
) -> List[Dict[str, Any]]:
    """Determine the list of stills to render per (scene, cue) and scene end."""
    fps = plan.get("fps", 30)
    range_info = plan.get("range", {})
    start_ms = range_info.get("start_ms", 0)
    frame_fn = lambda ms: math.ceil((ms - start_ms) * fps / 1000)

    stills: List[Dict[str, Any]] = []
    cues_dict = plan.get("cues", {})

    for seg in plan.get("segments", []):
        scene = seg.get("scene") or seg.get("instance")
        if not scene:
            continue
        if scenes_filter and scene not in scenes_filter:
            continue

        seg_start = seg.get("start_ms", 0)
        seg_end = seg.get("end_ms", 0)
        shots = seg.get("shots", [])

        # Collect cues belonging to this scene
        scene_cues: Dict[str, int] = {}

        # 1. Look for shot-prefixed cues
        for shot in shots:
            sid = shot.get("shot_id")
            if sid:
                prefix = f"{sid}."
                for k, v in cues_dict.items():
                    if k.startswith(prefix) and isinstance(v, (int, float)):
                        cue_name = k[len(prefix):]
                        scene_cues[cue_name] = int(v)

            # Also check if shot carries cues directly
            for c_name, c_val in shot.get("cues", {}).items():
                if c_name not in scene_cues:
                    if isinstance(c_val, (int, float)):
                        scene_cues[c_name] = int(c_val)
                    elif f"{sid}.{c_name}" in cues_dict:
                        scene_cues[c_name] = int(cues_dict[f"{sid}.{c_name}"])
                    elif c_name in cues_dict:
                        scene_cues[c_name] = int(cues_dict[c_name])

        # 2. Fallback if no cues found from shots: check cues falling in time range
        if not scene_cues:
            for k, v in cues_dict.items():
                if "." not in k and isinstance(v, (int, float)) and seg_start <= v <= seg_end:
                    scene_cues[k] = int(v)

        end_frame = frame_fn(seg_end) - 1

        # Add one still per cue at cue + 700 ms
        for cue_name, cue_ms in sorted(scene_cues.items(), key=lambda x: x[1]):
            target_ms = cue_ms + 700
            frame_idx = min(frame_fn(target_ms), end_frame)
            stills.append({
                "scene": scene,
                "cue": cue_name,
                "ms": target_ms,
                "frame": frame_idx,
                "file": f"{scene}_{cue_name}.png",
            })

        # Add one still at scene end - 1 frame
        end_ms_actual = round(start_ms + end_frame * 1000 / fps)
        stills.append({
            "scene": scene,
            "cue": "end",
            "ms": end_ms_actual,
            "frame": end_frame,
            "file": f"{scene}_end.png",
        })

    return stills


def make_contact_sheet(scene: str, stills: List[Dict[str, Any]], out_path: Path) -> None:
    """Generate a contact sheet with 960 px wide tiles, labelled scene · cue · ms."""
    if not stills:
        return

    tile_w = 960
    banner_h = 36
    cols = 2 if len(stills) > 1 else 1
    rows = math.ceil(len(stills) / cols)

    # Try finding Arial font; fallback to default
    font: ImageFont.ImageFont
    font_paths = [
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/Library/Fonts/Arial.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
    ]
    for fp in font_paths:
        if os.path.exists(fp):
            try:
                font = ImageFont.truetype(fp, 18)
                break
            except Exception:
                pass
    else:
        font = ImageFont.load_default()

    loaded_tiles = []
    tile_h = 540  # Default 16:9

    for item in stills:
        img_path = item.get("path") or (out_path.parent / item["file"])
        if not Path(img_path).is_file():
            continue
        try:
            with Image.open(img_path) as im:
                im = im.convert("RGB")
                orig_w, orig_h = im.size
                tile_h = round(tile_w * orig_h / orig_w)
                resized = im.resize((tile_w, tile_h), Image.Resampling.LANCZOS)

                # Create tile with banner on top
                tile = Image.new("RGB", (tile_w, tile_h + banner_h), color=(22, 22, 28))
                draw = ImageDraw.Draw(tile)

                label = f"{item['scene']} · {item['cue']} · {item['ms']} ms"
                draw.text((16, (banner_h - 18) // 2), label, fill=(240, 240, 245), font=font)
                tile.paste(resized, (0, banner_h))
                loaded_tiles.append(tile)
        except Exception as e:
            print(f"Warning: could not process image {img_path}: {e}", file=sys.stderr)

    if not loaded_tiles:
        return

    full_tile_h = tile_h + banner_h
    sheet_w = cols * tile_w
    sheet_h = rows * full_tile_h

    sheet = Image.new("RGB", (sheet_w, sheet_h), color=(14, 14, 18))
    for idx, tile in enumerate(loaded_tiles):
        c = idx % cols
        r = idx // cols
        sheet.paste(tile, (c * tile_w, r * full_tile_h))

    out_path.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out_path, format="PNG")


def render_stills(
    plan_path: Path,
    episode_dir: Path,
    out_dir: Path,
    scenes: Optional[List[str]] = None,
    concurrency: int = 4,
    sheet: bool = False,
    timeout_ms: int = 120000,
    dry_run: bool = False,
) -> int:
    plan_path = plan_path.resolve()
    episode_dir = episode_dir.resolve()
    out_dir = out_dir.resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    plan = json.loads(plan_path.read_text())
    stills = resolve_scene_stills(plan, scenes)
    if not stills:
        print("No stills matching criteria.", file=sys.stderr)
        return 0

    print(f"Planning {len(stills)} still(s) across {len(set(s['scene'] for s in stills))} scene(s)...")

    # Reuse one Remotion bundle
    bundle_dir = out_dir / ".bundle"
    env = dict(os.environ, STAGE_EPISODE_DIR=str(episode_dir))

    t_start = time.monotonic()
    t_bundle = 0.0

    if not dry_run:
        # Check if bundle already exists or create new bundle
        index_html = bundle_dir / "index.html"
        if not index_html.is_file():
            print(f"Creating reusable Remotion bundle at {bundle_dir}...")
            t0 = time.monotonic()
            bundle_cmd = [
                "npx", "--no-install", "remotion", "bundle",
                "src/stage/index.ts",
                f"--out-dir={bundle_dir.resolve()}",
            ]
            res = subprocess.run(bundle_cmd, cwd=VIDEO_DIR, env=env, capture_output=True, text=True)
            if res.returncode != 0:
                print(f"Bundle error (fallback to entry point): {res.stderr}", file=sys.stderr)
                bundle_target = "src/stage/index.ts"
            else:
                t_bundle = time.monotonic() - t0
                print(f"Bundle created in {t_bundle:.2f}s.")
                bundle_target = str(bundle_dir.resolve())
        else:
            print(f"Reusing existing Remotion bundle at {bundle_dir}.")
            bundle_target = str(bundle_dir.resolve())
    else:
        bundle_target = str(bundle_dir.resolve())

    # Build commands for each still
    common_args = [
        f"--props={plan_path}",
        f"--timeout={timeout_ms}",
        *get_browser_args(),
    ]

    tasks = []
    for item in stills:
        out_file = out_dir / item["file"]
        item["path"] = str(out_file)
        cmd = [
            "npx", "--no-install", "remotion", "still",
            bundle_target,
            "StageStill",
            str(out_file),
            f"--frame={item['frame']}",
            *common_args,
        ]
        tasks.append((item, cmd, out_file))

    if dry_run:
        for item, cmd, _ in tasks:
            print(" ".join(cmd))
        return 0

    t_render_start = time.monotonic()

    def run_one(task):
        item, cmd, out_file = task
        res = subprocess.run(cmd, cwd=VIDEO_DIR, env=env, capture_output=True, text=True)
        return item, res.returncode, res.stdout, res.stderr

    failures = 0
    with ThreadPoolExecutor(max_workers=max(1, concurrency)) as executor:
        futures = [executor.submit(run_one, t) for t in tasks]
        for fut in as_completed(futures):
            item, code, stdout, stderr = fut.result()
            if code != 0:
                print(f"Failed still {item['file']}: {stderr}", file=sys.stderr)
                failures += 1
            else:
                print(f"  + {item['file']} (frame {item['frame']}, {item['ms']} ms)")

    t_render = time.monotonic() - t_render_start
    t_total = time.monotonic() - t_start

    # Report speed-up from bundle reuse
    est_rebundle_time = max(0.8, t_bundle if t_bundle > 0 else 0.8)
    est_no_reuse = (est_rebundle_time * len(tasks)) + t_render
    speedup = est_no_reuse / max(0.1, t_total)
    print(f"\nRendered {len(tasks) - failures}/{len(tasks)} stills in {t_total:.2f}s (bundle: {t_bundle:.2f}s, render: {t_render:.2f}s).")
    print(f"Bundle reuse saved ~{est_rebundle_time * (len(tasks) - 1):.1f}s ({speedup:.1f}x speed-up vs rebundling each still).")

    # Write stills.json listing every file with scene, cue, ms
    stills_records = [
        {
            "file": item["file"],
            "path": item["path"],
            "scene": item["scene"],
            "cue": item["cue"],
            "ms": item["ms"],
            "frame": item["frame"],
        }
        for item in stills
    ]
    stills_json = out_dir / "stills.json"
    stills_json.write_text(json.dumps(stills_records, indent=2) + "\n")
    print(f"Wrote {stills_json}")

    # Write contact sheets if requested
    if sheet:
        by_scene: Dict[str, List[Dict[str, Any]]] = {}
        for item in stills:
            by_scene.setdefault(item["scene"], []).append(item)

        for sc_name, sc_stills in by_scene.items():
            sheet_path = out_dir / f"{sc_name}_sheet.png"
            make_contact_sheet(sc_name, sc_stills, sheet_path)
            print(f"Wrote contact sheet {sheet_path}")

            # Ensure alternate naming patterns exist
            alt_path = out_dir / f"{sc_name}-sheet.png"
            if not alt_path.exists():
                shutil.copyfile(sheet_path, alt_path)

        if len(by_scene) == 1:
            single_sheet = out_dir / "sheet.png"
            first_sheet = out_dir / f"{list(by_scene.keys())[0]}_sheet.png"
            if first_sheet.is_file() and not single_sheet.exists():
                shutil.copyfile(first_sheet, single_sheet)

    return 1 if failures > 0 else 0


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", type=Path, required=True, help="Path to Stage plan JSON")
    parser.add_argument("--episode-dir", type=Path, help="Path to STAGE_EPISODE_DIR")
    parser.add_argument("--scenes", type=str, help="Comma-separated list of scene names")
    parser.add_argument("--out", type=Path, required=True, help="Output directory")
    parser.add_argument("--concurrency", type=int, default=4, help="Parallel process count")
    parser.add_argument("--sheet", action="store_true", help="Generate Pillow contact sheet per scene")
    parser.add_argument("--timeout-ms", type=int, default=120000, help="Remotion timeout ms")
    parser.add_argument("--dry-run", action="store_true", help="Print commands without executing")

    args = parser.parse_args(argv)

    episode_dir = args.episode_dir
    if episode_dir is None:
        try:
            plan_obj = json.loads(args.plan.read_text())
            episode_dir = Path(plan_obj.get("episode_dir", args.plan.parent))
        except Exception:
            episode_dir = args.plan.parent

    scenes_list = [s.strip() for s in args.scenes.split(",")] if args.scenes else None

    return render_stills(
        plan_path=args.plan,
        episode_dir=episode_dir,
        out_dir=args.out,
        scenes=scenes_list,
        concurrency=args.concurrency,
        sheet=args.sheet,
        timeout_ms=args.timeout_ms,
        dry_run=args.dry_run,
    )


if __name__ == "__main__":
    sys.exit(main())
