"""Platform checks for MLX-backed voice pipeline commands."""

import platform
import sys


def apple_silicon_error_message(arch: str) -> str:
    return f"MLX requires Apple Silicon (arm64). This machine reports: {arch}. Exiting."


def require_apple_silicon() -> None:
    arch = platform.machine()
    if arch != "arm64":
        print(apple_silicon_error_message(arch))
        sys.exit(1)


def main() -> None:
    require_apple_silicon()


if __name__ == "__main__":
    main()
