# scriptCast

scriptCast turns marked-up scripts into speech and production assets.
It provides Python command-line tools for rendering, verification, review, and export.
The macOS app uses the installer in `install/install.sh` to provision its tools.

Setup instructions are in [docs/setup.md](docs/setup.md). Diagnose an install with:

```sh
bash install/install.sh --check
```

Build and test commands:

```sh
cargo build --release --manifest-path ui/Cargo.toml
cargo test --manifest-path ui/Cargo.toml
.venv/bin/pytest
```
