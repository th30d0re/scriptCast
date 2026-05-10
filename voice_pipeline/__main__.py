"""Entry point for the local voice pipeline."""

from .platform_check import require_apple_silicon


def main() -> None:
    require_apple_silicon()
    print("voice_pipeline: not yet implemented.")


if __name__ == "__main__":
    main()
