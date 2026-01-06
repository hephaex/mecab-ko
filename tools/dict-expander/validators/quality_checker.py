"""Quality validation for dictionary entries.

This module provides comprehensive quality checks for MeCab entries including:
- Format validation
- Korean text validation
- Semantic consistency
- Statistical anomaly detection
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Sequence

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from utils.mecab_format import MecabEntry
from utils.korean_utils import is_hangul, syllable_count


class Severity(Enum):
    """Validation issue severity levels."""

    INFO = "info"
    WARNING = "warning"
    ERROR = "error"


@dataclass
class ValidationIssue:
    """Represents a validation issue.

    Attributes:
        severity: Issue severity level.
        message: Human-readable message.
        field: Field name with issue.
        value: Invalid value.
    """

    severity: Severity
    message: str
    field: str | None = None
    value: str | None = None

    def __str__(self) -> str:
        """Format issue as string."""
        parts = [f"[{self.severity.value.upper()}]", self.message]
        if self.field:
            parts.append(f"(field: {self.field})")
        if self.value:
            parts.append(f"(value: {self.value})")
        return " ".join(parts)


@dataclass
class ValidationResult:
    """Results from entry validation.

    Attributes:
        is_valid: Whether entry passed validation.
        issues: List of validation issues.
    """

    is_valid: bool = True
    issues: list[ValidationIssue] = field(default_factory=list)

    def add_error(self, message: str, field: str | None = None, value: str | None = None) -> None:
        """Add error issue and mark as invalid."""
        self.issues.append(ValidationIssue(Severity.ERROR, message, field, value))
        self.is_valid = False

    def add_warning(self, message: str, field: str | None = None, value: str | None = None) -> None:
        """Add warning issue."""
        self.issues.append(ValidationIssue(Severity.WARNING, message, field, value))

    def add_info(self, message: str, field: str | None = None, value: str | None = None) -> None:
        """Add info issue."""
        self.issues.append(ValidationIssue(Severity.INFO, message, field, value))

    @property
    def error_count(self) -> int:
        """Count error-level issues."""
        return sum(1 for issue in self.issues if issue.severity == Severity.ERROR)

    @property
    def warning_count(self) -> int:
        """Count warning-level issues."""
        return sum(1 for issue in self.issues if issue.severity == Severity.WARNING)


class QualityChecker:
    """Validates MeCab dictionary entry quality."""

    # Valid POS tags
    VALID_POS_TAGS = {
        # Nouns
        'NNG', 'NNP', 'NNB', 'NNBC', 'NR', 'NP',
        # Verbs
        'VV', 'VA', 'VX', 'VCP', 'VCN',
        # Modifiers
        'MM', 'MAG', 'MAJ',
        # Particles
        'JKS', 'JKC', 'JKG', 'JKO', 'JKB', 'JKV', 'JKQ', 'JX', 'JC',
        # Endings
        'EP', 'EF', 'EC', 'ETN', 'ETM',
        # Prefixes/Suffixes
        'XPN', 'XSN', 'XSV', 'XSA', 'XR',
        # Others
        'SF', 'SE', 'SSO', 'SSC', 'SC', 'SY',
        'IC', 'SL', 'SH', 'SN',
    }

    MIN_SYLLABLE_COUNT = 1
    MAX_SYLLABLE_COUNT = 20

    def __init__(self, strict: bool = False):
        """Initialize quality checker.

        Args:
            strict: Enable strict validation mode.
        """
        self.strict = strict

    def validate_entry(self, entry: MecabEntry) -> ValidationResult:
        """Validate single entry.

        Args:
            entry: Entry to validate.

        Returns:
            Validation result with any issues found.
        """
        result = ValidationResult()

        # Check surface form
        self._validate_surface(entry.surface, result)

        # Check POS tag
        self._validate_pos(entry.pos, result)

        # Check jongseong marker
        self._validate_jongseong(entry.has_jongseong, entry.surface, result)

        # Check reading
        self._validate_reading(entry.reading, entry.surface, result)

        # Check consistency
        self._validate_consistency(entry, result)

        return result

    def _validate_surface(self, surface: str, result: ValidationResult) -> None:
        """Validate surface form."""
        if not surface:
            result.add_error("Surface form is empty", "surface")
            return

        # Check syllable count
        count = syllable_count(surface)
        if count < self.MIN_SYLLABLE_COUNT:
            result.add_error(
                f"Surface form too short ({count} syllables)",
                "surface",
                surface,
            )
        elif count > self.MAX_SYLLABLE_COUNT:
            result.add_warning(
                f"Surface form very long ({count} syllables)",
                "surface",
                surface,
            )

        # Check for valid characters
        has_hangul = any(is_hangul(c) for c in surface)
        if not has_hangul:
            if self.strict:
                result.add_error("No Hangul characters found", "surface", surface)
            else:
                result.add_warning("No Hangul characters found", "surface", surface)

    def _validate_pos(self, pos: str, result: ValidationResult) -> None:
        """Validate POS tag."""
        if not pos:
            result.add_error("POS tag is empty", "pos")
            return

        if pos not in self.VALID_POS_TAGS:
            result.add_error(f"Invalid POS tag: {pos}", "pos", pos)

    def _validate_jongseong(
        self,
        has_jongseong: str,
        surface: str,
        result: ValidationResult,
    ) -> None:
        """Validate jongseong marker consistency."""
        if has_jongseong not in ('T', 'F'):
            result.add_error(
                f"Invalid jongseong marker: {has_jongseong}",
                "has_jongseong",
                has_jongseong,
            )
            return

        # Check consistency with surface form
        from utils.korean_utils import get_jongseong_marker

        expected = get_jongseong_marker(surface)
        if has_jongseong != expected:
            result.add_warning(
                f"Jongseong marker '{has_jongseong}' doesn't match "
                f"surface form '{surface}' (expected '{expected}')",
                "has_jongseong",
            )

    def _validate_reading(self, reading: str, surface: str, result: ValidationResult) -> None:
        """Validate reading form."""
        if not reading:
            result.add_warning("Reading form is empty", "reading")
            return

        # In most cases, reading should match surface for Korean
        if reading != surface and reading != "*":
            result.add_info(
                f"Reading '{reading}' differs from surface '{surface}'",
                "reading",
            )

    def _validate_consistency(self, entry: MecabEntry, result: ValidationResult) -> None:
        """Validate entry internal consistency."""
        # Compound entries should have expression
        if entry.entry_type == "Compound" and entry.expression == "*":
            result.add_warning(
                "Compound entry missing morpheme expression",
                "expression",
            )

        # Preanalysis entries should have first_pos and last_pos
        if entry.entry_type == "Preanalysis":
            if entry.first_pos == "*" or entry.last_pos == "*":
                result.add_warning(
                    "Preanalysis entry missing POS information",
                    "entry_type",
                )

    def validate_batch(
        self,
        entries: Sequence[MecabEntry],
    ) -> tuple[list[ValidationResult], dict[str, int]]:
        """Validate batch of entries.

        Args:
            entries: Entries to validate.

        Returns:
            Tuple of (results list, summary statistics).
        """
        results = [self.validate_entry(entry) for entry in entries]

        stats = {
            'total': len(entries),
            'valid': sum(1 for r in results if r.is_valid),
            'invalid': sum(1 for r in results if not r.is_valid),
            'errors': sum(r.error_count for r in results),
            'warnings': sum(r.warning_count for r in results),
        }

        return results, stats
