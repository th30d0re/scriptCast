"""Cue card: the short, structured task description the router classifies.

Cards (never briefs) go to laya; the English checkpoint context is 512 tokens,
so a card is capped at 400 (tech-plan D4).
"""
from dataclasses import asdict, dataclass, field
from typing import Callable, List, Optional

MAX_TOKENS = 400


class CardTooLong(ValueError):
    def __init__(self, tokens: int, limit: int = MAX_TOKENS):
        super().__init__(f"cue card is {tokens} tokens (limit {limit})")
        self.tokens = tokens
        self.limit = limit


@dataclass
class CueCard:
    scene: str = ""
    note: str = ""
    kind: str = "fix"  # fix | check | implement | research | review
    turn: Optional[int] = None
    cue: str = ""
    element_id: str = ""
    defect_category: str = ""
    owned_files: List[str] = field(default_factory=list)
    has_live_session: bool = False
    episode: Optional[int] = None
    episode_kind: str = ""  # premiere | series | special | ""

    def to_dict(self) -> dict:
        return asdict(self)


def estimate_tokens(text: str) -> int:
    return max(1, (len(text) + 3) // 4)


def _text(card: CueCard, note: str) -> str:
    where = " / ".join(x for x in (card.scene, card.element_id, card.cue) if x)
    head = f"[{card.kind}]" + (f" [{where}]" if where else "")
    if card.turn is not None:
        head += f" turn {card.turn}"
    if card.defect_category:
        head += f" ({card.defect_category})"
    lines = [f"{head} {note}".strip()]
    if card.owned_files:
        lines.append("files: " + ", ".join(card.owned_files))
    return "\n".join(lines)


def render(card: CueCard, count: Optional[Callable[[str], int]] = None) -> str:
    """Render the card; raise CardTooLong above MAX_TOKENS."""
    count = count or estimate_tokens
    text = _text(card, card.note)
    n = count(text)
    if n > MAX_TOKENS:
        raise CardTooLong(n)
    return text


def render_fit(card: CueCard, count: Optional[Callable[[str], int]] = None):
    """Render, truncating the note to fit. Returns (text, tokens, truncated).

    Raises CardTooLong if the card is over budget even with an empty note.
    """
    count = count or estimate_tokens
    try:
        text = render(card, count)
        return text, count(text), False
    except CardTooLong:
        pass
    words = card.note.split()
    lo, hi = 0, len(words)
    while lo < hi:  # largest prefix that fits
        mid = (lo + hi + 1) // 2
        if count(_text(card, " ".join(words[:mid]) + " ...")) <= MAX_TOKENS:
            lo = mid
        else:
            hi = mid - 1
    text = _text(card, " ".join(words[:lo]) + " ...")
    n = count(text)
    if n > MAX_TOKENS:
        raise CardTooLong(n)
    return text, n, True
