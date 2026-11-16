# Setup

## One-click install

The macOS app runs `install/install.sh` for the selected app version. It creates the scriptCast environment in `$HOME/Library/Application Support/scriptCast/`, installs the matching GitHub release wheel with verification support, and writes `install.json` after verification.

To diagnose a machine without installing anything, run:

```sh
bash install/install.sh --check
```

## Manual install

From a checkout, use the installer with a release version:

```sh
bash install/install.sh --app-version X.Y.Z
```

For a local wheel, use `--wheel /path/to/scriptcast-*.whl` or a wheel URL. A conventional development install is also available:

```sh
python3 -m pip install '.[verify]'
```

The installer uses `uv` when present and otherwise uses Python's venv and pip. It requires arm64 macOS and Python 3.11 or newer.

## Files and tools

The install root contains `venv/`, the optional `venv-omnivoice/`, the timestamped `install.log`, and the successful-run metadata in `install.json`. Console scripts are in `venv/bin/`.

The app looks for tools in this order: its CLI folder, a project `.venv*`, `PATH`, a development checkout, and finally the scriptCast install root.

## Troubleshooting

| Code | Fix |
| --- | --- |
| E_ARCH | Use arm64 macOS. |
| E_PYTHON | Install Python 3.11 or newer, or install uv, then rerun `--check`. |
| E_NETWORK | Check connectivity and the GitHub release/tag or wheel URL. |
| E_VENV | Check write access and free disk space under the install root. |
| E_PIP | Read the tail of `install.log`; repair the wheel or dependency/network problem and rerun. |
| E_DISK | Free disk space and ensure the install root is writable. |
| W_FFMPEG | With Homebrew, rerun to allow `brew install ffmpeg`; without it, install Homebrew from `https://brew.sh`, then run `brew install ffmpeg`. |
| W_UV | This is informational: the installer falls back to `python -m venv`. Install uv to use the faster path. |

## OmniVoice

Pass `--with-omnivoice` to also create `venv-omnivoice/` with `torch==2.8.0`, `torchaudio==2.8.0`, and `omnivoice`. The main scriptCast environment remains in `venv/`.

## Uninstall

Delete the install root:

```sh
rm -rf "$HOME/Library/Application Support/scriptCast"
```

