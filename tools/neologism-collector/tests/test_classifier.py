"""Tests for neologism classifier."""

from __future__ import annotations

import pytest
from sqlalchemy.orm import Session

from src.classifier import NeologismClassifier
from src.config import Config
from src.models import CandidateWord


def test_classifier_initialization(test_session: Session, test_config: Config) -> None:
    """Test classifier initialization."""
    classifier = NeologismClassifier(test_session, test_config)
    assert classifier.session is test_session
    assert classifier.config is test_config


def test_classify_candidate(test_session: Session, test_config: Config) -> None:
    """Test single candidate classification."""
    candidate = CandidateWord(
        surface="갓생",
        frequency=10,
        length=2,
        has_hangul=True,
        contexts=["갓생을 살다", "갓생 루틴"],
        sources=["news", "blog"],
    )

    classifier = NeologismClassifier(test_session, test_config)
    result = classifier.classify(candidate)

    assert "is_neologism" in result
    assert "is_typo" in result
    assert "confidence" in result
    assert "features" in result
    assert isinstance(result["is_neologism"], bool)
    assert isinstance(result["confidence"], float)
    assert 0.0 <= result["confidence"] <= 1.0


def test_extract_features(test_session: Session, test_config: Config) -> None:
    """Test feature extraction."""
    candidate = CandidateWord(
        surface="갓생",
        frequency=10,
        length=2,
        has_hangul=True,
        has_english=False,
        has_number=False,
        contexts=["context1", "context2", "context3"],
    )

    classifier = NeologismClassifier(test_session, test_config)
    features = classifier._extract_features(candidate)

    assert "length" in features
    assert "char_type_ratio" in features
    assert "frequency" in features
    assert "context_diversity" in features

    # All features should be normalized to 0-1
    for value in features.values():
        assert 0.0 <= value <= 1.0


def test_classify_candidates_batch(
    test_session: Session,
    test_config: Config,
) -> None:
    """Test batch classification."""
    # Add unclassified candidates
    candidates = [
        CandidateWord(surface="갓생", frequency=10, length=2, has_hangul=True),
        CandidateWord(surface="미라클모닝", frequency=5, length=5, has_hangul=True),
        CandidateWord(surface="ㅋㅋㅋ", frequency=2, length=3, has_hangul=True),
    ]
    test_session.add_all(candidates)
    test_session.commit()

    classifier = NeologismClassifier(test_session, test_config)
    count = classifier.classify_candidates(batch_size=10)

    assert count == 3

    # Check that all candidates were classified
    for candidate in candidates:
        test_session.refresh(candidate)
        assert candidate.is_neologism is not None
        assert candidate.confidence_score is not None
