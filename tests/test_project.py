from pathlib import Path

import pytest

from scriptcast.archive import load_registry
from scriptcast.engine import _resolve_reference
from scriptcast.pronunciation import load_respellings
from scriptcast.project import DEFAULTS, Project, current, find_root


def test_env_var_sets_root(tmp_path, monkeypatch) -> None:
    monkeypatch.setenv("SCRIPTCAST_PROJECT", str(tmp_path))
    assert find_root() == tmp_path.resolve()
    assert current().root == tmp_path.resolve()


def test_ancestor_discovery(tmp_path, monkeypatch) -> None:
    monkeypatch.delenv("SCRIPTCAST_PROJECT", raising=False)
    (tmp_path / "scriptcast.toml").write_text("", encoding="utf-8")
    nested = tmp_path / "a" / "b"
    nested.mkdir(parents=True)
    assert find_root(nested) == tmp_path.resolve()


def test_cwd_fallback(tmp_path, monkeypatch) -> None:
    monkeypatch.delenv("SCRIPTCAST_PROJECT", raising=False)
    monkeypatch.chdir(tmp_path)
    assert find_root() == tmp_path.resolve()


def test_defaults(tmp_path) -> None:
    project = Project(tmp_path)
    for key, default in DEFAULTS.items():
        assert project.path(key) == tmp_path / default


def test_unknown_key_names_it(tmp_path) -> None:
    (tmp_path / "scriptcast.toml").write_text(
        'voices = "voices.yaml"\nbogus_key = "x"\n', encoding="utf-8"
    )
    with pytest.raises(ValueError, match="bogus_key"):
        Project(tmp_path)


def test_absolute_path_used_as_given(tmp_path) -> None:
    elsewhere = tmp_path / "elsewhere" / "voices.yaml"
    (tmp_path / "scriptcast.toml").write_text(
        f'voices = "{elsewhere}"\n', encoding="utf-8"
    )
    assert Project(tmp_path).voices == elsewhere


def test_relative_path_anchors_at_root(tmp_path) -> None:
    (tmp_path / "scriptcast.toml").write_text(
        'voices = "configs/voices.yaml"\n', encoding="utf-8"
    )
    assert Project(tmp_path).voices == tmp_path / "configs" / "voices.yaml"


def test_missing_optional_files_behave_as_empty(tmp_path, monkeypatch) -> None:
    monkeypatch.setenv("SCRIPTCAST_PROJECT", str(tmp_path))
    assert load_registry() == {}
    assert load_respellings() == {}


def test_reference_audio_resolves_against_project_root(tmp_path, monkeypatch) -> None:
    monkeypatch.setenv("SCRIPTCAST_PROJECT", str(tmp_path))
    assert _resolve_reference("voices/candidates/ref.wav") == (
        tmp_path / "voices" / "candidates" / "ref.wav"
    )
    absolute = tmp_path / "elsewhere" / "ref.wav"
    assert _resolve_reference(str(absolute)) == absolute
