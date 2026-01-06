"""MeCab CSV format handling utilities.

This module provides utilities for parsing and formatting MeCab dictionary entries.

MeCab CSV format:
    surface,left_id,right_id,cost,pos,semantic,has_jongseong,reading,type,first_pos,last_pos,expression

Example:
    서울,0,0,0,NNP,지명,T,서울,*,*,*,*
    가곡선,0,0,0,NNP,*,T,가곡선,Compound,*,*,가곡/NNG/*+선/NNG/*
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import ClassVar


@dataclass
class MecabEntry:
    """Represents a single MeCab dictionary entry.

    Attributes:
        surface: Surface form of the word (표층형).
        left_id: Left context ID (default: 0).
        right_id: Right context ID (default: 0).
        cost: Connection cost (default: 0).
        pos: Part-of-speech tag (품사).
        semantic: Semantic category (의미 분류).
        has_jongseong: Final consonant marker ('T' or 'F').
        reading: Reading form (읽기).
        entry_type: Entry type (Compound, Preanalysis, Inflect, etc.).
        first_pos: First POS for compound words.
        last_pos: Last POS for compound words.
        expression: Morpheme expression for compound words.
    """

    surface: str
    pos: str
    has_jongseong: str
    reading: str
    left_id: int = 0
    right_id: int = 0
    cost: int = 0
    semantic: str = "*"
    entry_type: str = "*"
    first_pos: str = "*"
    last_pos: str = "*"
    expression: str = "*"

    # POS tag categories
    NOUN_TAGS: ClassVar[set[str]] = {
        "NNG",  # 일반 명사
        "NNP",  # 고유 명사
        "NNB",  # 의존 명사
        "NNBC", # 단위성 의존 명사
        "NR",   # 수사
        "NP",   # 대명사
    }

    VERB_TAGS: ClassVar[set[str]] = {
        "VV",   # 동사
        "VA",   # 형용사
        "VX",   # 보조 용언
        "VCP",  # 긍정 지정사
        "VCN",  # 부정 지정사
    }

    SEMANTIC_CATEGORIES: ClassVar[set[str]] = {
        "인명",   # Person name
        "지명",   # Place name
        "기관",   # Organization
        "고유명", # Proper noun
        "*",     # Unspecified
    }

    def __post_init__(self) -> None:
        """Validate entry after initialization."""
        if not self.surface:
            raise ValueError("Surface form cannot be empty")
        if not self.pos:
            raise ValueError("POS tag cannot be empty")
        if self.has_jongseong not in ("T", "F"):
            raise ValueError(f"has_jongseong must be 'T' or 'F', got: {self.has_jongseong}")

    def to_csv_line(self) -> str:
        """Convert entry to MeCab CSV format line.

        Returns:
            CSV formatted string.
        """
        return format_mecab_line(
            surface=self.surface,
            left_id=self.left_id,
            right_id=self.right_id,
            cost=self.cost,
            pos=self.pos,
            semantic=self.semantic,
            has_jongseong=self.has_jongseong,
            reading=self.reading,
            entry_type=self.entry_type,
            first_pos=self.first_pos,
            last_pos=self.last_pos,
            expression=self.expression,
        )

    @classmethod
    def from_csv_line(cls, line: str) -> MecabEntry:
        """Parse MeCab CSV line into entry.

        Args:
            line: CSV formatted line.

        Returns:
            Parsed MecabEntry.

        Raises:
            ValueError: If line format is invalid.
        """
        return parse_mecab_line(line)


def format_mecab_line(
    surface: str,
    left_id: int,
    right_id: int,
    cost: int,
    pos: str,
    semantic: str,
    has_jongseong: str,
    reading: str,
    entry_type: str = "*",
    first_pos: str = "*",
    last_pos: str = "*",
    expression: str = "*",
) -> str:
    """Format MeCab dictionary entry as CSV line.

    Args:
        surface: Surface form.
        left_id: Left context ID.
        right_id: Right context ID.
        cost: Connection cost.
        pos: Part-of-speech tag.
        semantic: Semantic category.
        has_jongseong: Final consonant ('T' or 'F').
        reading: Reading form.
        entry_type: Entry type (default: '*').
        first_pos: First POS (default: '*').
        last_pos: Last POS (default: '*').
        expression: Morpheme expression (default: '*').

    Returns:
        CSV formatted line.
    """
    fields = [
        surface,
        str(left_id),
        str(right_id),
        str(cost),
        pos,
        semantic,
        has_jongseong,
        reading,
        entry_type,
        first_pos,
        last_pos,
        expression,
    ]
    return ",".join(fields)


def parse_mecab_line(line: str) -> MecabEntry:
    """Parse MeCab CSV line into structured entry.

    Args:
        line: CSV line to parse.

    Returns:
        Parsed MecabEntry.

    Raises:
        ValueError: If line has incorrect number of fields.
    """
    line = line.strip()
    if not line or line.startswith("#"):
        raise ValueError("Empty or comment line")

    parts = line.split(",")
    if len(parts) != 12:
        raise ValueError(f"Expected 12 fields, got {len(parts)}: {line}")

    return MecabEntry(
        surface=parts[0],
        left_id=int(parts[1]),
        right_id=int(parts[2]),
        cost=int(parts[3]),
        pos=parts[4],
        semantic=parts[5],
        has_jongseong=parts[6],
        reading=parts[7],
        entry_type=parts[8],
        first_pos=parts[9],
        last_pos=parts[10],
        expression=parts[11],
    )
