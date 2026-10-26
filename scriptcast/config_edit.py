"""Comment-preserving edits for scriptCast project files.

YAML edits are located with ruamel.yaml (``preserve_quotes`` so a scalar's
quoting is visible). Inside an existing mapping, unchanged keys keep their
original lines, including quotes and inline comments. Only a changed value
or an added key is new text. New strings use the double- or single-quote
style already used in that mapping. TOML edits change the value token on
the key's line and leave every comment in place.
"""

from __future__ import annotations

import re
from io import StringIO

from ruamel.yaml import YAML
from ruamel.yaml.comments import CommentedMap, CommentedSeq
from ruamel.yaml.scalarstring import DoubleQuotedScalarString, SingleQuotedScalarString

from scriptcast.project import DEFAULTS
from scriptcast.review import _normalize_token

FILE_KINDS = ("voices", "clips", "pronunciations", "accepted_hearings", "toml")


class EditError(Exception):
    """One ops-list entry failed. ``index`` is its position in that list."""

    def __init__(self, index: int, message: str) -> None:
        super().__init__(message)
        self.index = index
        self.message = message


def apply_ops(texts: dict[str, str], ops: list) -> dict[str, str]:
    """Return updated file texts. ``texts`` is keyed by file kind."""
    if not isinstance(ops, list):
        raise EditError(0, "ops must be a list")
    working = dict(texts)
    for index, op in enumerate(ops):
        if not isinstance(op, dict):
            raise EditError(index, "op must be an object")
        kind = op.get("file")
        if kind not in FILE_KINDS:
            raise EditError(index, f"unknown file {kind!r}")
        working[kind] = _apply_one(index, str(kind), working.get(kind, ""), op)
    return working


def locate_block(text: str, parent_keys: list[str], key: object) -> tuple[int, int]:
    """Line span ``[start, end)`` of ``key`` under ``parent_keys``.

    The span covers the key line and its nested lines. It stops before a
    sibling, before a comment at the key's indent, and before blank lines
    that separate this block from the next one.
    """
    lines = _lines(text)
    node, actual = _mapping_and_key(text, parent_keys, key)
    start, col = _line_col(node, actual)
    return start, _block_end(lines, start, col)


def insert_at(text: str, parent_keys: list[str]) -> int:
    """Line index at which a new child of ``parent_keys`` should be inserted."""
    lines = _lines(text)
    data = _load(text)
    if not parent_keys:
        if not isinstance(data, dict) or not list(data):
            return len(lines)
        last = list(data)[-1]
        start, col = _line_col(data, last)
        return _block_end(lines, start, col)
    parent = _descend(data, parent_keys)
    if isinstance(parent, dict) and list(parent):
        last = list(parent)[-1]
        start, col = _line_col(parent, last)
        return _block_end(lines, start, col)
    owner = data if len(parent_keys) == 1 else _descend(data, parent_keys[:-1])
    start, _col = _line_col(owner, parent_keys[-1])
    return start + 1


def _apply_one(index: int, kind: str, text: str, op: dict) -> str:
    name = op.get("op")
    if kind == "voices":
        if name == "set_speaker":
            return _set_mapping(index, text, ["speakers"], _require_id(index, op, "id"), _require_fields(index, op), "speaker")
        if name == "delete_speaker":
            return _delete_mapping(index, text, ["speakers"], _require_id(index, op, "id"), "speaker")
        if name == "rename_speaker":
            return _rename_mapping(index, text, ["speakers"], _require_id(index, op, "id"), _require_id(index, op, "to"), "speaker")
        raise EditError(index, f"unknown op {name!r} for voices")
    if kind == "clips":
        if name == "set_clip":
            return _set_mapping(index, text, ["clips"], _require_id(index, op, "id"), _require_fields(index, op), "clip")
        if name == "delete_clip":
            return _delete_mapping(index, text, ["clips"], _require_id(index, op, "id"), "clip")
        raise EditError(index, f"unknown op {name!r} for clips")
    if kind == "pronunciations":
        if name == "set_entry":
            return _set_mapping(index, text, [], _require_id(index, op, "word"), _require_readings(index, op), "entry")
        if name == "delete_entry":
            return _delete_mapping(index, text, [], _require_id(index, op, "word"), "entry")
        raise EditError(index, f"unknown op {name!r} for pronunciations")
    if kind == "accepted_hearings":
        if name == "delete":
            return _delete_hearing(index, text, _require_id(index, op, "word"), _require_id(index, op, "heard"))
        raise EditError(index, f"unknown op {name!r} for accepted_hearings")
    if kind == "toml":
        if name == "set":
            return _set_toml(index, text, op)
        raise EditError(index, f"unknown op {name!r} for toml")
    raise EditError(index, f"unknown file {kind!r}")


def _require_id(index: int, op: dict, field: str) -> str:
    value = op.get(field)
    if not isinstance(value, str) or value == "":
        raise EditError(index, f"{field} must be a non-empty string")
    return value


def _require_fields(index: int, op: dict) -> dict:
    fields = op.get("fields")
    if not isinstance(fields, dict):
        raise EditError(index, "fields must be an object")
    return fields


def _require_readings(index: int, op: dict) -> dict:
    readings = op.get("readings")
    if not isinstance(readings, dict):
        raise EditError(index, "readings must be an object")
    return readings


def _set_mapping(index: int, text: str, parent_keys: list[str], key: str, value: dict, _noun: str) -> str:
    text = _ensure_parent(text, parent_keys)
    text = _expand_flow_parent(text, parent_keys)
    data = _load(text)
    if parent_keys:
        try:
            parent = _descend(data, parent_keys)
        except KeyError:
            raise EditError(index, f"missing {'.'.join(parent_keys)}") from None
    else:
        parent = data if isinstance(data, dict) else None
    if parent is not None and not isinstance(parent, dict):
        label = parent_keys[-1] if parent_keys else "file"
        raise EditError(index, f"{label} must be a mapping")
    existing = _match_key(parent, key) if isinstance(parent, dict) else None
    if existing is not None and _is_block_mapping(text, parent_keys, existing):
        return _merge_into(text, [*parent_keys, str(existing)], value)
    quote = _quote_style(parent) or '"'
    indent = _key_column(parent, existing) if existing is not None else _child_column(text, parent_keys)
    block = _render_block(key, value, indent, _newline(text), quote)
    if existing is not None:
        start, end = locate_block(text, parent_keys, existing)
        return _splice(text, start, end, block)
    at = insert_at(text, parent_keys)
    lines = _lines(text)
    if at > 0 and at <= len(lines) and lines[at - 1].strip():
        block = _newline(text) + block
    return _splice(text, at, at, block)


def _delete_mapping(index: int, text: str, parent_keys: list[str], key: str, noun: str) -> str:
    data = _load(text)
    try:
        parent = _descend(data, parent_keys) if parent_keys else data
    except KeyError:
        parent = None
    actual = _match_key(parent, key) if isinstance(parent, dict) else None
    if actual is None:
        raise EditError(index, f"{noun} {key!r} not found")
    start, end = locate_block(text, parent_keys, actual)
    return _splice(text, start, end, "")


def _rename_mapping(index: int, text: str, parent_keys: list[str], old: str, new: str, noun: str) -> str:
    data = _load(text)
    try:
        parent = _descend(data, parent_keys) if parent_keys else data
    except KeyError:
        parent = None
    if not isinstance(parent, dict):
        raise EditError(index, f"{noun} {old!r} not found")
    actual = _match_key(parent, old)
    if actual is None:
        raise EditError(index, f"{noun} {old!r} not found")
    if _match_key(parent, new) is not None and str(actual) != new:
        raise EditError(index, f"{noun} {new!r} already exists")
    if str(actual) == new:
        return text
    start, col = _line_col(parent, actual)
    lines = _lines(text)
    try:
        lines[start] = _replace_key_token(lines[start], col, new)
    except ValueError as exc:
        raise EditError(index, str(exc)) from exc
    return "".join(lines)


def _delete_hearing(index: int, text: str, word: str, heard: str) -> str:
    data = _load(text)
    if not isinstance(data, dict):
        raise EditError(index, f"hearing {word!r} / {heard!r} not found")
    actual = None
    wanted = _normalize_token(word)
    for key in data:
        if _normalize_token(str(key)) == wanted and wanted:
            actual = key
            break
    if actual is None:
        raise EditError(index, f"hearing {word!r} / {heard!r} not found")
    start, end = locate_block(text, [], actual)
    lines = _lines(text)
    block = lines[start:end]
    updated, found = _remove_hearing_item(block, heard)
    if not found:
        raise EditError(index, f"hearing {word!r} / {heard!r} not found")
    if _list_items(updated) == 0:
        return _splice(text, start, end, "")
    return _splice(text, start, end, "".join(updated))


def _remove_hearing_item(block: list[str], heard: str) -> tuple[list[str], bool]:
    import yaml

    wanted = _normalize_token(heard)
    if not wanted:
        return block, False
    for index in range(1, len(block)):
        stripped = block[index].strip()
        if not stripped.startswith("-"):
            continue
        payload = stripped[1:].strip()
        if not payload or payload.startswith("#"):
            continue
        try:
            value = yaml.safe_load(payload)
        except yaml.YAMLError:
            value = payload
        if _normalize_token("" if value is None else str(value)) != wanted:
            continue
        remove_from = index
        cursor = index - 1
        while cursor > 0 and not block[cursor].strip():
            cursor -= 1
        if cursor > 0 and re.match(r"#\s*review\b", block[cursor].strip()):
            remove_from = cursor
        return block[:remove_from] + block[index + 1 :], True
    return block, False


def _list_items(block: list[str]) -> int:
    count = 0
    for line in block[1:]:
        stripped = line.strip()
        if stripped.startswith("-") and not stripped.startswith("- #"):
            payload = stripped[1:].strip()
            if payload and not payload.startswith("#"):
                count += 1
    return count


def _set_toml(index: int, text: str, op: dict) -> str:
    key = op.get("key")
    if not isinstance(key, str) or key == "":
        raise EditError(index, "key must be a non-empty string")
    if key not in DEFAULTS:
        known = ", ".join(sorted(DEFAULTS))
        raise EditError(index, f"unknown key {key!r}. Known keys: {known}")
    if "value" not in op:
        raise EditError(index, "missing value")
    try:
        encoded = _encode_toml(op["value"])
    except ValueError as exc:
        raise EditError(index, str(exc)) from exc
    newline = _newline(text) if text else "\n"
    lines = _lines(text)
    matcher = re.compile(rf"^(\s*{re.escape(key)}\s*=\s*)(.*)$")
    for line_index, line in enumerate(lines):
        body, ending = _split_ending(line)
        match = matcher.match(body)
        if not match:
            continue
        _value, suffix = _split_value_comment(match.group(2))
        lines[line_index] = match.group(1) + encoded + suffix + ending
        return "".join(lines)
    if text and not text.endswith(("\n", "\r\n")):
        text += newline
    return text + f"{key} = {encoded}{newline}"


def _encode_toml(value: object) -> str:
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, float):
        if value != value or value in (float("inf"), float("-inf")):
            raise ValueError("value must be a finite number")
        return repr(value)
    if isinstance(value, str):
        escaped = value.replace("\\", "\\\\").replace('"', '\\"')
        return f'"{escaped}"'
    raise ValueError("value must be a string, number, or bool")


def _split_value_comment(rest: str) -> tuple[str, str]:
    in_string = False
    quote = ""
    escaped = False
    for index, char in enumerate(rest):
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\" and quote == '"':
                escaped = True
            elif char == quote:
                in_string = False
            continue
        if char in ('"', "'"):
            in_string = True
            quote = char
            continue
        if char == "#":
            value = rest[:index].rstrip(" ")
            return value, rest[len(value) : index] + rest[index:]
    return rest, ""


def _ensure_parent(text: str, parent_keys: list[str]) -> str:
    if not parent_keys:
        return text
    data = _load(text)
    node = data if isinstance(data, dict) else None
    missing_at = 0
    if isinstance(data, dict):
        missing_at = len(parent_keys)
        node = data
        for index, key in enumerate(parent_keys):
            if not isinstance(node, dict) or key not in node:
                missing_at = index
                break
            if index < len(parent_keys) - 1:
                node = node[key]
    if missing_at == len(parent_keys):
        return text
    newline = _newline(text) if text else "\n"
    addition = ""
    if text and not text.endswith(("\n", "\r\n")):
        addition += newline
    indent = 0
    for depth, key in enumerate(parent_keys[missing_at:]):
        addition += f"{' ' * (indent + 2 * depth)}{key}:{newline}"
    return text + addition


def _expand_flow_parent(text: str, parent_keys: list[str]) -> str:
    if not parent_keys:
        return text
    data = _load(text)
    try:
        owner = data if len(parent_keys) == 1 else _descend(data, parent_keys[:-1])
        start, col = _line_col(owner, parent_keys[-1])
    except (KeyError, TypeError, AttributeError):
        return text
    lines = _lines(text)
    body, ending = _split_ending(lines[start])
    rest = body[col:]
    match = re.match(rf"^{re.escape(str(parent_keys[-1]))}\s*:\s*\{{\s*\}}\s*$", rest)
    if not match:
        return text
    prefix = body[:col]
    lines[start] = f"{prefix}{parent_keys[-1]}:{ending or _newline(text)}"
    return "".join(lines)


def _merge_into(text: str, container: list[str], new_fields: dict) -> str:
    """Update the mapping at ``container`` one key at a time."""
    data = _load(text)
    node = _descend(data, container)
    if not isinstance(node, dict):
        return _replace_container(text, container, new_fields)
    quote = _quote_style(node) or _quote_style(_parent_of(data, container)) or '"'
    lines = _lines(text)
    existing = {str(key): key for key in node}
    incoming = {str(key): key for key in new_fields}
    edits: list[tuple] = []
    for key_text, actual in existing.items():
        start, col = _line_col(node, actual)
        end = _block_end(lines, start, col)
        if key_text not in incoming:
            edits.append((start, "del", end))
            continue
        new_value = new_fields[incoming[key_text]]
        old_value = node[actual]
        if _same(old_value, new_value):
            continue
        if isinstance(old_value, dict) and isinstance(new_value, dict):
            edits.append((start, "recur", actual, new_value))
            continue
        edits.append((start, "set", end, new_value, col))
    for edit in sorted(edits, key=lambda item: item[0], reverse=True):
        if edit[1] == "del":
            text = _splice(text, edit[0], edit[2], "")
        elif edit[1] == "recur":
            text = _merge_into(text, [*container, str(edit[2])], edit[3])
        else:
            text = _replace_value(text, edit[0], edit[2], edit[3], edit[4], quote)
    present = _key_names(text, container)
    child_indent = _indent_under(text, container)
    newline = _newline(text)
    for key, new_value in new_fields.items():
        if str(key) in present:
            continue
        end = _span(text, container)[1]
        text = _splice(text, end, end, _render_added_key(str(key), new_value, child_indent, quote, newline))
        present.add(str(key))
    return text


def _replace_container(text: str, container: list[str], value: dict) -> str:
    data = _load(text)
    quote = _quote_style(_parent_of(data, container)) or '"'
    parent = _parent_of(data, container)
    actual = _match_key(parent, container[-1]) if isinstance(parent, dict) else container[-1]
    indent = _key_column(parent, actual) if isinstance(parent, dict) and actual is not None else 0
    block = _render_block(str(container[-1]), value, indent, _newline(text), quote)
    start, end = _span(text, container)
    return _splice(text, start, end, block)


def _replace_value(text: str, start: int, end: int, value: object, indent: int, quote: str) -> str:
    lines = _lines(text)
    if end == start + 1 and not isinstance(value, (dict, list)):
        lines[start] = _replace_scalar_line(lines[start], value, quote)
        return "".join(lines)
    return _splice(text, start, end, _render_added_key(_key_on_line(lines[start], indent), value, indent, quote, _newline(text)))


def _replace_scalar_line(line: str, value: object, quote: str) -> str:
    body, ending = _split_ending(line)
    split = _split_key_prefix(body)
    if split is None:
        return line
    prefix, rest = split
    _old, suffix = _split_value_comment(rest)
    return prefix + _encode_scalar(value, quote) + suffix + ending


def _split_key_prefix(body: str) -> tuple[str, str] | None:
    in_string = False
    quote = ""
    escaped = False
    for index, char in enumerate(body):
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\" and quote == '"':
                escaped = True
            elif char == quote:
                in_string = False
            continue
        if char in ('"', "'"):
            in_string = True
            quote = char
            continue
        if char == ":":
            cursor = index + 1
            while cursor < len(body) and body[cursor] == " ":
                cursor += 1
            return body[:cursor], body[cursor:]
    return None


def _key_on_line(line: str, col: int) -> str:
    body, _ending = _split_ending(line)
    token = body[col:].split(":", 1)[0].strip()
    if len(token) >= 2 and token[0] == token[-1] and token[0] in {'"', "'"}:
        return token[1:-1]
    return token


def _render_added_key(key: str, value: object, indent: int, quote: str, newline: str) -> str:
    if isinstance(value, dict):
        return _render_block(key, value, indent, newline, quote)
    pad = " " * indent
    if isinstance(value, list):
        lines = [f"{pad}{_format_yaml_key(key)}:"]
        for item in value:
            lines.append(f"{pad}  - {_encode_scalar(item, quote)}")
        return newline.join(lines) + newline
    return f"{pad}{_format_yaml_key(key)}: {_encode_scalar(value, quote)}{newline}"


def _encode_scalar(value: object, quote: str) -> str:
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, float):
        return repr(value)
    if value is None:
        return "null"
    if isinstance(value, str):
        return _encode_yaml_string(value, quote)
    return _encode_yaml_string(str(value), quote)


def _encode_yaml_string(value: str, quote: str) -> str:
    if quote == "'" and "\n" not in value and "\r" not in value:
        return "'" + value.replace("'", "''") + "'"
    if quote == "" and _plain_scalar_ok(value):
        return value
    escaped = (
        value.replace("\\", "\\\\")
        .replace('"', '\\"')
        .replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace("\t", "\\t")
    )
    return f'"{escaped}"'


def _plain_scalar_ok(value: str) -> bool:
    if value == "" or value != value.strip():
        return False
    if value.lower() in {"true", "false", "null", "yes", "no", "~"}:
        return False
    if re.match(r"[-?:]", value):
        return False
    return re.fullmatch(r"[A-Za-z0-9_./\- ]+", value) is not None and ":" not in value


def _same(old: object, new: object) -> bool:
    if isinstance(old, dict) and isinstance(new, dict):
        old_map = {str(key): value for key, value in old.items()}
        new_map = {str(key): value for key, value in new.items()}
        return set(old_map) == set(new_map) and all(_same(old_map[key], new_map[key]) for key in old_map)
    if isinstance(old, list) and isinstance(new, list):
        return len(old) == len(new) and all(_same(left, right) for left, right in zip(old, new))
    if isinstance(old, bool) or isinstance(new, bool):
        return isinstance(old, bool) and isinstance(new, bool) and old == new
    if isinstance(old, (int, float)) and isinstance(new, (int, float)):
        return float(old) == float(new)
    if isinstance(old, str) or isinstance(new, str):
        return isinstance(old, str) and isinstance(new, str) and str(old) == str(new)
    return old is None and new is None


def _quote_style(node: object) -> str | None:
    found: list[str] = []

    def walk(value: object) -> None:
        if isinstance(value, DoubleQuotedScalarString):
            found.append('"')
        elif isinstance(value, SingleQuotedScalarString):
            found.append("'")
        elif isinstance(value, dict):
            for item in value.values():
                walk(item)
        elif isinstance(value, list):
            for item in value:
                walk(item)

    walk(node)
    if not found:
        return None
    if found.count('"') >= found.count("'"):
        return '"'
    return "'"


def _is_block_mapping(text: str, parent_keys: list[str], key: object) -> bool:
    data = _load(text)
    parent = _descend(data, parent_keys) if parent_keys else data
    if not isinstance(parent, dict) or not isinstance(parent.get(key), dict):
        return False
    start, _col = _line_col(parent, key)
    body, _ending = _split_ending(_lines(text)[start])
    return "{" not in body


def _parent_of(data: object, container: list[str]) -> object:
    if len(container) <= 1:
        return data
    return _descend(data, container[:-1])


def _key_names(text: str, container: list[str]) -> set[str]:
    data = _load(text)
    try:
        node = _descend(data, container)
    except KeyError:
        return set()
    if not isinstance(node, dict):
        return set()
    return {str(key) for key in node}


def _indent_under(text: str, container: list[str]) -> int:
    data = _load(text)
    node = _descend(data, container)
    if isinstance(node, dict) and list(node):
        return _key_column(node, next(iter(node)))
    parent = _parent_of(data, container)
    actual = _match_key(parent, container[-1]) if isinstance(parent, dict) else None
    if isinstance(parent, dict) and actual is not None:
        return _key_column(parent, actual) + 2
    return 0


def _span(text: str, container: list[str]) -> tuple[int, int]:
    return locate_block(text, container[:-1], container[-1])


def _render_block(key: str, value: dict, indent: int, newline: str, quote: str = '"') -> str:
    pad = " " * indent
    key_text = _format_yaml_key(key)
    dumped = _dump_mapping(value, quote)
    if not value:
        return f"{pad}{key_text}: {{}}{newline}"
    child = " " * (indent + 2)
    lines = [f"{pad}{key_text}:"]
    for raw in dumped.split("\n"):
        lines.append(f"{child}{raw}" if raw else raw)
    return newline.join(lines) + newline


def _dump_mapping(value: dict, quote: str) -> str:
    yaml = YAML()
    yaml.preserve_quotes = True
    yaml.default_flow_style = False
    yaml.width = 10**6
    yaml.allow_unicode = True
    buffer = StringIO()
    yaml.dump(_styled(value, quote), buffer)
    text = buffer.getvalue()
    if text.endswith("\n"):
        text = text[:-1]
    return text


def _styled(value: object, quote: str) -> object:
    if isinstance(value, str):
        if quote == "'":
            return SingleQuotedScalarString(value)
        if quote == '"':
            return DoubleQuotedScalarString(value)
        return value
    if isinstance(value, dict):
        node = CommentedMap()
        for key, item in value.items():
            node[key] = _styled(item, quote)
        return node
    if isinstance(value, list):
        node = CommentedSeq()
        for item in value:
            node.append(_styled(item, quote))
        return node
    return value


def _format_yaml_key(key: str) -> str:
    if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_\-]*", key):
        return key
    escaped = key.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def _mapping_and_key(text: str, parent_keys: list[str], key: object) -> tuple[dict, object]:
    data = _load(text)
    parent = _descend(data, parent_keys) if parent_keys else data
    if not isinstance(parent, dict):
        raise KeyError(key)
    actual = _match_key(parent, key)
    if actual is None:
        raise KeyError(key)
    return parent, actual


def _match_key(node: dict | None, key: object) -> object | None:
    if not isinstance(node, dict):
        return None
    for candidate in node:
        if candidate == key or str(candidate) == str(key):
            return candidate
    return None


def _descend(data: object, keys: list[str]) -> object:
    node = data
    for key in keys:
        if not isinstance(node, dict) or key not in node:
            raise KeyError(key)
        node = node[key]
    return node


def _line_col(node: dict, key: object) -> tuple[int, int]:
    info = node.lc.item(key)
    if info is None:
        raise KeyError(key)
    return int(info[0]), int(info[1])


def _key_column(node: dict, key: object) -> int:
    _line, col = _line_col(node, key)
    return col


def _child_column(text: str, parent_keys: list[str]) -> int:
    if not parent_keys:
        return 0
    data = _load(text)
    try:
        parent = _descend(data, parent_keys)
    except KeyError:
        parent = None
    if isinstance(parent, dict) and list(parent):
        _line, col = _line_col(parent, next(iter(parent)))
        return col
    try:
        owner = data if len(parent_keys) == 1 else _descend(data, parent_keys[:-1])
        _line, col = _line_col(owner, parent_keys[-1])
    except (KeyError, TypeError, AttributeError):
        return 0
    return col + 2


def _block_end(lines: list[str], start: int, key_col: int) -> int:
    index = start + 1
    while index < len(lines):
        line = lines[index]
        if not line.strip():
            index += 1
            continue
        indent = len(line) - len(line.lstrip(" "))
        if indent > key_col:
            index += 1
            continue
        break
    while index > start + 1 and not lines[index - 1].strip():
        index -= 1
    return index


def _load(text: str) -> object:
    if text.strip() == "":
        return None
    yaml = YAML(typ="rt")
    yaml.preserve_quotes = True
    return yaml.load(text)


def _lines(text: str) -> list[str]:
    return text.splitlines(keepends=True)


def _newline(text: str) -> str:
    return "\r\n" if "\r\n" in text else "\n"


def _split_ending(line: str) -> tuple[str, str]:
    if line.endswith("\r\n"):
        return line[:-2], "\r\n"
    if line.endswith("\n"):
        return line[:-1], "\n"
    return line, ""


def _splice(text: str, start: int, end: int, block: str) -> str:
    lines = _lines(text)
    newline = _newline(text) if text else "\n"
    if block:
        if newline == "\r\n":
            block = block.replace("\r\n", "\n").replace("\n", "\r\n")
        else:
            block = block.replace("\r\n", "\n")
        if not block.endswith(("\n", "\r\n")):
            block += newline
    replacement = _lines(block) if block else []
    return "".join(lines[:start] + replacement + lines[end:])


def _replace_key_token(line: str, col: int, new_key: str) -> str:
    body, ending = _split_ending(line)
    if col > len(body):
        raise ValueError("key column is past the end of the line")
    rest = body[col:]
    if rest.startswith('"') or rest.startswith("'"):
        quote = rest[0]
        cursor = 1
        while cursor < len(rest):
            if rest[cursor] == "\\" and quote == '"':
                cursor += 2
                continue
            if rest[cursor] == quote:
                cursor += 1
                break
            cursor += 1
        token_len = cursor
    else:
        match = re.match(r"[^:#\s]+", rest)
        if not match:
            raise ValueError("could not read the key token")
        token_len = match.end()
    return body[:col] + _format_yaml_key(new_key) + body[col + token_len :] + ending
