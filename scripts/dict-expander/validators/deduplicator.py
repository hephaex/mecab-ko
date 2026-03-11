"""Dictionary entry deduplication utilities.

This module provides deduplication logic for MeCab dictionary entries,
handling exact matches and near-duplicates with intelligent merging.
"""

from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass
from typing import Sequence

import sys
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from utils.mecab_format import MecabEntry


@dataclass
class DeduplicationStats:
    """Statistics from deduplication process.

    Attributes:
        total_entries: Total number of input entries.
        unique_entries: Number of unique entries after deduplication.
        duplicates_removed: Number of duplicate entries removed.
        merged_entries: Number of entries merged.
    """

    total_entries: int = 0
    unique_entries: int = 0
    duplicates_removed: int = 0
    merged_entries: int = 0

    def __str__(self) -> str:
        """Format statistics as human-readable string."""
        return (
            f"Deduplication Stats:\n"
            f"  Total entries: {self.total_entries:,}\n"
            f"  Unique entries: {self.unique_entries:,}\n"
            f"  Duplicates removed: {self.duplicates_removed:,}\n"
            f"  Merged entries: {self.merged_entries:,}\n"
            f"  Reduction rate: {self.reduction_rate:.1%}"
        )

    @property
    def reduction_rate(self) -> float:
        """Calculate reduction percentage."""
        if self.total_entries == 0:
            return 0.0
        return self.duplicates_removed / self.total_entries


class Deduplicator:
    """Deduplicates MeCab dictionary entries.

    Handles exact duplicates and merges entries with same surface form
    but different metadata intelligently.
    """

    def __init__(self, prefer_compound: bool = True):
        """Initialize deduplicator.

        Args:
            prefer_compound: Prefer compound entries when merging (default: True).
        """
        self.prefer_compound = prefer_compound
        self.stats = DeduplicationStats()

    def deduplicate(self, entries: Sequence[MecabEntry]) -> list[MecabEntry]:
        """Deduplicate entries with intelligent merging.

        Args:
            entries: List of entries to deduplicate.

        Returns:
            Deduplicated list of entries.
        """
        self.stats = DeduplicationStats(total_entries=len(entries))

        # Group by surface form
        surface_groups: dict[str, list[MecabEntry]] = defaultdict(list)
        for entry in entries:
            surface_groups[entry.surface].append(entry)

        # Process each group
        unique_entries: list[MecabEntry] = []
        for surface, group in surface_groups.items():
            if len(group) == 1:
                unique_entries.append(group[0])
            else:
                merged = self._merge_group(group)
                unique_entries.append(merged)
                self.stats.duplicates_removed += len(group) - 1
                if len(group) > 1:
                    self.stats.merged_entries += 1

        self.stats.unique_entries = len(unique_entries)
        return unique_entries

    def _merge_group(self, entries: list[MecabEntry]) -> MecabEntry:
        """Merge multiple entries with same surface form.

        Args:
            entries: Entries to merge.

        Returns:
            Single merged entry.
        """
        if len(entries) == 1:
            return entries[0]

        # Sort by priority
        sorted_entries = sorted(entries, key=self._entry_priority, reverse=True)
        return sorted_entries[0]

    def _entry_priority(self, entry: MecabEntry) -> tuple[int, int, int]:
        """Calculate priority score for entry selection.

        Args:
            entry: Entry to score.

        Returns:
            Tuple of (compound_score, semantic_score, expression_score).
        """
        # Prefer compound entries
        compound_score = 1 if self.prefer_compound and entry.entry_type == "Compound" else 0

        # Prefer entries with semantic info
        semantic_score = 1 if entry.semantic != "*" else 0

        # Prefer entries with expression
        expression_score = 1 if entry.expression != "*" else 0

        return (compound_score, semantic_score, expression_score)


def deduplicate_entries(
    entries: Sequence[MecabEntry],
    prefer_compound: bool = True,
) -> tuple[list[MecabEntry], DeduplicationStats]:
    """Deduplicate MeCab entries.

    Args:
        entries: Entries to deduplicate.
        prefer_compound: Prefer compound entries when merging.

    Returns:
        Tuple of (deduplicated entries, statistics).

    Examples:
        >>> entries = [
        ...     MecabEntry("서울", "NNP", "T", "서울"),
        ...     MecabEntry("서울", "NNP", "T", "서울"),
        ... ]
        >>> unique, stats = deduplicate_entries(entries)
        >>> len(unique)
        1
    """
    deduplicator = Deduplicator(prefer_compound=prefer_compound)
    unique = deduplicator.deduplicate(entries)
    return unique, deduplicator.stats


def deduplicate_by_key(
    entries: Sequence[MecabEntry],
    key_fn: callable = None,
) -> list[MecabEntry]:
    """Deduplicate entries using custom key function.

    Args:
        entries: Entries to deduplicate.
        key_fn: Function to extract deduplication key (default: surface form).

    Returns:
        Deduplicated entries.
    """
    if key_fn is None:
        key_fn = lambda e: e.surface

    seen_keys: set[str] = set()
    unique: list[MecabEntry] = []

    for entry in entries:
        key = key_fn(entry)
        if key not in seen_keys:
            seen_keys.add(key)
            unique.append(entry)

    return unique
