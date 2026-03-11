"""Utility modules for dictionary expansion."""

from .mecab_format import MecabEntry, format_mecab_line, parse_mecab_line
from .korean_utils import (
    get_final_consonant,
    has_final_consonant,
    decompose_hangul,
    compose_hangul,
    is_hangul,
)

__all__ = [
    "MecabEntry",
    "format_mecab_line",
    "parse_mecab_line",
    "get_final_consonant",
    "has_final_consonant",
    "decompose_hangul",
    "compose_hangul",
    "is_hangul",
]
