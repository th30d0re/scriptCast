import os
import shutil
import subprocess
from pathlib import Path


def test_install_check_protocol():
    result = subprocess.run(["bash", "install/install.sh", "--check"], text=True, capture_output=True)
    assert result.returncode == 0, result.stdout + result.stderr
    lines = [line for line in result.stdout.splitlines() if line]
    assert any(line.startswith("STEP ") for line in lines)
    assert any(line.startswith("OK ") for line in lines)
    for line in lines:
        assert line.startswith(("STEP ", "OK ", "WARN ", "ERROR ", "DONE "))
    if shutil.which("shellcheck"):
        checked = subprocess.run(["shellcheck", "install/install.sh"], text=True, capture_output=True)
        assert checked.returncode == 0, checked.stdout + checked.stderr
