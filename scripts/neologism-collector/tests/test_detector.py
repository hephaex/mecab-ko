"""Tests for neologism detector."""

from __future__ import annotations

import pytest
from sqlalchemy.orm import Session

from src.config import Config
from src.detector import NeologismDetector
from src.models import CandidateWord, RawText


def test_detector_initialization(test_session: Session, test_config: Config) -> None:
    """Test detector initialization."""
    detector = NeologismDetector(test_session, test_config)
    assert detector.session is test_session
    assert detector.config is test_config


def test_extract_candidates(
    test_session: Session,
    test_config: Config,
    sample_text: str,
) -> None:
    """Test candidate extraction from text."""
    detector = NeologismDetector(test_session, test_config)

    candidates = detector.extract_candidates(sample_text, source="test")

    # Should extract some candidates
    assert len(candidates) > 0

    # Check candidate structure
    for surface, data in candidates.items():
        assert "frequency" in data
        assert "contexts" in data
        assert "sources" in data
        assert "features" in data


def test_is_valid_candidate(test_session: Session, test_config: Config) -> None:
    """Test candidate validation."""
    detector = NeologismDetector(test_session, test_config)

    # Valid candidates
    assert detector._is_valid_candidate("갓생") is True
    assert detector._is_valid_candidate("미라클모닝") is True

    # Invalid - too short
    assert detector._is_valid_candidate("ㄱ") is False

    # Invalid - only jamo
    assert detector._is_valid_candidate("ㅋㅋㅋ") is False

    # Invalid - only numbers
    assert detector._is_valid_candidate("123") is False


def test_has_jongseong(test_session: Session, test_config: Config) -> None:
    """Test jongseong detection."""
    detector = NeologismDetector(test_session, test_config)

    # Has jongseong
    assert detector._has_jongseong("한") is True
    assert detector._has_jongseong("밥") is True

    # No jongseong
    assert detector._has_jongseong("나") is False
    assert detector._has_jongseong("바") is False


def test_detect_from_raw_texts(
    test_session: Session,
    test_config: Config,
    sample_text: str,
) -> None:
    """Test detection from raw texts in database."""
    # Add raw text to database
    raw_text = RawText(
        source="test",
        content=sample_text,
        processed=False,
    )
    test_session.add(raw_text)
    test_session.commit()

    # Run detection
    detector = NeologismDetector(test_session, test_config)
    count = detector.detect_from_raw_texts(batch_size=10)

    assert count > 0

    # Check that raw text was marked as processed
    test_session.refresh(raw_text)
    assert raw_text.processed is True

    # Check that candidates were created
    candidates = test_session.query(CandidateWord).all()
    assert len(candidates) > 0


def test_get_statistics(test_session: Session, test_config: Config) -> None:
    """Test statistics generation."""
    detector = NeologismDetector(test_session, test_config)

    # Add some candidates
    candidate1 = CandidateWord(
        surface="갓생",
        frequency=10,
        length=2,
        is_neologism=True,
    )
    candidate2 = CandidateWord(
        surface="미라클모닝",
        frequency=5,
        length=5,
        is_neologism=None,
    )
    test_session.add_all([candidate1, candidate2])
    test_session.commit()

    stats = detector.get_statistics()

    assert stats["total_candidates"] == 2
    assert stats["neologisms"] == 1
    assert stats["unclassified"] == 1
