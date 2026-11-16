"""MLX subprocess entry point.  Deliberately imports no project modules."""
import json
import sys


def main() -> None:
    request = json.load(sys.stdin)
    from mlx_vlm import load, generate
    from mlx_vlm.prompt_utils import apply_chat_template
    from mlx_vlm.utils import load_config

    model, processor = load(request["model"])
    config = load_config(request["model"])
    results = []
    for item in request["items"]:
        images = ([item["reference"]] if item.get("reference") else []) + [item["still"]]
        prompt = apply_chat_template(processor, config, item["prompt"], num_images=len(images))
        response = generate(model, processor, prompt, images, max_tokens=700, temperature=0.0, verbose=False)
        results.append({"file": item["file"], "raw": getattr(response, "text", response)})
    json.dump({"results": results}, sys.stdout)


if __name__ == "__main__":
    main()
