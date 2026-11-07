"""Fast, deterministic per-element visual critic (System 1).

The module intentionally has no model/runtime dependency: the optional image
embedding hook is only used when supplied by an application.  This keeps the
hot path suitable for render-time use (NumPy + Pillow only).
"""
from __future__ import annotations

import json
import argparse
from pathlib import Path
from typing import Any, Callable

import numpy as np
from PIL import Image

WEIGHTS = Path(__file__).with_name("s1_weights.json")
FEATURE_NAMES = ("ssim", "mean_abs_diff", "edge_diff", "palette_distance", "embed_distance")


def _array(image: Image.Image | np.ndarray) -> np.ndarray:
    if isinstance(image, Image.Image):
        image = image.convert("RGB")
        return np.asarray(image, dtype=np.float32) / 255.0
    a = np.asarray(image, dtype=np.float32)
    if a.ndim == 2: a = np.repeat(a[..., None], 3, axis=2)
    return a[..., :3] / (255.0 if a.max(initial=0) > 1.0 else 1.0)


def _resize_pair(a: np.ndarray, b: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    h, w = min(a.shape[0], b.shape[0]), min(a.shape[1], b.shape[1])
    if h < 1 or w < 1: raise ValueError("crops must be non-empty")
    def resize(x: np.ndarray) -> np.ndarray:
        if x.shape[:2] == (h, w): return x
        return np.asarray(Image.fromarray(np.uint8(np.clip(x, 0, 1) * 255)).resize((w, h), Image.Resampling.BILINEAR), dtype=np.float32) / 255.0
    return resize(a), resize(b)


def _ssim(a: np.ndarray, b: np.ndarray) -> float:
    # Global SSIM is stable for tiny crops and avoids a scipy/skimage dependency.
    x, y = a.mean(2), b.mean(2)
    ux, uy = x.mean(), y.mean()
    vx, vy = x.var(), y.var()
    cov = ((x - ux) * (y - uy)).mean()
    return float(((2 * ux * uy + .01 ** 2) * (2 * cov + .03 ** 2)) / ((ux * ux + uy * uy + .01 ** 2) * (vx + vy + .03 ** 2)))


def _edges(a: np.ndarray) -> np.ndarray:
    gray = a @ np.array([.299, .587, .114], dtype=np.float32)
    p = np.pad(gray, 1, mode="edge")
    gx = p[:-2, 2:] + 2*p[1:-1, 2:] + p[2:, 2:] - p[:-2, :-2] - 2*p[1:-1, :-2] - p[2:, :-2]
    gy = p[2:, :-2] + 2*p[2:, 1:-1] + p[2:, 2:] - p[:-2, :-2] - 2*p[:-2, 1:-1] - p[:-2, 2:]
    return np.hypot(gx, gy) / 4.0


def _rgb_lab(a: np.ndarray) -> np.ndarray:
    # D65 sRGB -> CIE Lab.  Enough accuracy for palette-distance ranking.
    a = np.where(a <= .04045, a / 12.92, ((a + .055) / 1.055) ** 2.4)
    xyz = a @ np.array([[.4124564, .3575761, .1804375], [.2126729, .7151522, .072175], [.0193339, .119192, .9503041]], dtype=np.float32).T
    q = xyz / np.array([.95047, 1., 1.08883], dtype=np.float32)
    q = np.where(q > .008856, np.cbrt(q), 7.787*q + 16/116)
    return np.stack([116*q[..., 1]-16, 500*(q[..., 0]-q[..., 1]), 200*(q[..., 1]-q[..., 2])], -1)


def _palette(a: np.ndarray, k: int = 5) -> np.ndarray:
    # Quantisation gives a deterministic, very cheap top-colour approximation.
    flat = a.reshape(-1, 3)
    # A fixed stride bounds the expensive unique() call for large title crops.
    if len(flat) > 2048: flat = flat[::max(1, len(flat) // 2048)]
    q = np.clip((flat * 15).astype(np.uint8), 0, 15)
    vals, counts = np.unique(q, axis=0, return_counts=True)
    return (vals[np.argsort(counts)[-k:]] + .5) / 16.0


def _palette_distance(a: np.ndarray, b: np.ndarray) -> float:
    x, y = _rgb_lab(_palette(a)), _rgb_lab(_palette(b))
    distances = np.sqrt(((x[:, None] - y[None, :]) ** 2).sum(2))
    return float((distances.min(1).mean() + distances.min(0).mean()) / 200.0)


def features(ref_crop: Image.Image | np.ndarray, new_crop: Image.Image | np.ndarray, embedder: Callable[[Any], Any] | None = None) -> dict[str, float]:
    """Return normalized visual-difference features; 0 means identical."""
    ref, new = _resize_pair(_array(ref_crop), _array(new_crop))
    result = {"ssim": _ssim(ref, new), "mean_abs_diff": float(np.abs(ref-new).mean()),
              "edge_diff": float(np.abs(_edges(ref)-_edges(new)).mean()), "palette_distance": _palette_distance(ref, new)}
    if embedder is not None:
        r, n = np.asarray(embedder(ref), dtype=float).ravel(), np.asarray(embedder(new), dtype=float).ravel()
        result["embed_distance"] = float(1 - np.dot(r, n) / (np.linalg.norm(r)*np.linalg.norm(n) + 1e-12))
    return result


def _load_weights(path: Path = WEIGHTS) -> dict[str, Any]:
    if path.exists(): return json.loads(path.read_text())
    return {"intercept": -4.0, "weights": {"ssim": -5., "mean_abs_diff": 12., "edge_diff": 10., "palette_distance": 3., "embed_distance": 2.}, "threshold": .5}


def score(values: dict[str, float], weights_path: Path = WEIGHTS) -> float:
    """Score a feature dict using saved logistic-regression weights."""
    model = _load_weights(weights_path)
    z = float(model.get("intercept", 0.0))
    for name, coefficient in model.get("weights", {}).items():
        if name in values:
            mean = float(model.get("mean", {}).get(name, 0.0))
            scale = float(model.get("scale", {}).get(name, 1.0))
            z += float(coefficient) * ((float(values[name]) - mean) / max(scale, 1e-9))
    return float(1 / (1 + np.exp(-np.clip(z, -60, 60))))


def _crop(im: Image.Image, box: dict[str, Any]) -> Image.Image:
    x, y, w, h = (float(box[k]) for k in ("x", "y", "w", "h"))
    W, H = im.size
    # normalized boxes are the public API; accepting pixels aids E1 tooling.
    if max(x, y, w, h) <= 1.01: x, y, w, h = x*W, y*H, w*W, h*H
    pad = max(2, round(min(w, h) * .08))
    return im.crop((max(0, round(x-pad)), max(0, round(y-pad)), min(W, round(x+w+pad)), min(H, round(y+h+pad))))


def judge_frame(ref_png: str | Path | Image.Image, new_png: str | Path | Image.Image, boxes: list[dict[str, Any]], threshold: float | None = None) -> dict[str, Any]:
    """Judge normalized ``xywh`` element boxes and return a frame verdict."""
    ref = ref_png.convert("RGB") if isinstance(ref_png, Image.Image) else Image.open(ref_png).convert("RGB")
    new = new_png.convert("RGB") if isinstance(new_png, Image.Image) else Image.open(new_png).convert("RGB")
    model = _load_weights(); cutoff = model.get("threshold", .5) if threshold is None else threshold
    elements = []
    for i, box in enumerate(boxes):
        vals = features(_crop(ref, box), _crop(new, box))
        elements.append({"element_id": box.get("element_id", box.get("id", str(i))), "p_defect": score(vals), "features": vals})
    elements.sort(key=lambda e: e["p_defect"], reverse=True)
    return {"verdict": "fail" if any(e["p_defect"] >= cutoff for e in elements) else "pass", "elements": elements}


def download_embedding_model(repo_id: str = "mlx-community/siglip-so400m-patch14-384") -> str:
    """Opt-in model acquisition; never runs during normal judging.

    The returned snapshot can be wired into an ``embedder`` callable (for
    example one using ``mlx_vlm``).  Keeping loading outside ``features``
    prevents a missing MLX installation or model cache from blocking S1.
    """
    from huggingface_hub import snapshot_download
    return snapshot_download(repo_id=repo_id)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="S1 visual critic helpers")
    parser.add_argument("--download", action="store_true", help="download optional MLX SigLIP weights")
    parser.add_argument("--model", default="mlx-community/siglip-so400m-patch14-384")
    args = parser.parse_args()
    if args.download: print(download_embedding_model(args.model))
