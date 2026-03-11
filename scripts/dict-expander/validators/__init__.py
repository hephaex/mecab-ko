"""Validation modules for dictionary quality assurance."""

from .deduplicator import Deduplicator, deduplicate_entries
from .pos_inference import POSInferencer, infer_pos_tag
from .quality_checker import QualityChecker, ValidationResult

__all__ = [
    "Deduplicator",
    "deduplicate_entries",
    "POSInferencer",
    "infer_pos_tag",
    "QualityChecker",
    "ValidationResult",
]
