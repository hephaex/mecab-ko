"""Tests for MeCab exporter."""

from __future__ import annotations

from pathlib import Path

import pytest
from sqlalchemy.orm import Session

from src.config import Config
from src.exporter import MeCabExporter
from src.models import Neologism


def test_exporter_initialization(test_session: Session, test_config: Config) -> None:
    """Test exporter initialization."""
    exporter = MeCabExporter(test_session, test_config)
    assert exporter.session is test_session
    assert exporter.config is test_config


def test_has_jongseong(test_session: Session, test_config: Config) -> None:
    """Test jongseong detection."""
    exporter = MeCabExporter(test_session, test_config)

    # Has jongseong
    assert exporter._has_jongseong("한") is True
    assert exporter._has_jongseong("밥") is True
    assert exporter._has_jongseong("김") is True

    # No jongseong
    assert exporter._has_jongseong("나") is False
    assert exporter._has_jongseong("가") is False
    assert exporter._has_jongseong("너") is False

    # Edge cases
    assert exporter._has_jongseong("") is False
    assert exporter._has_jongseong("abc") is False


def test_format_mecab_row(test_session: Session, test_config: Config) -> None:
    """Test MeCab row formatting."""
    neologism = Neologism(
        surface="갓생",
        total_score=0.85,
        frequency=10,
        pos="NNG",
        cost=4800,
        confidence_level="high",
    )

    exporter = MeCabExporter(test_session, test_config)
    row = exporter._format_mecab_row(neologism)

    # Check row structure
    assert len(row) == 14
    assert row[0] == "갓생"  # surface
    assert row[4] == "NNG"  # POS
    assert row[7] == "F"  # no jongseong
    assert row[8] == "갓생"  # reading


def test_export_neologisms(
    test_session: Session,
    test_config: Config,
    tmp_path: Path,
) -> None:
    """Test neologism export."""
    # Update config with temp output directory
    test_config.exporter.output_dir = str(tmp_path)

    # Add neologisms
    neologisms = [
        Neologism(
            surface="갓생",
            total_score=0.85,
            frequency=10,
            confidence_level="high",
        ),
        Neologism(
            surface="미라클모닝",
            total_score=0.65,
            frequency=5,
            confidence_level="medium",
        ),
    ]
    test_session.add_all(neologisms)
    test_session.commit()

    exporter = MeCabExporter(test_session, test_config)
    output_path = exporter.export_neologisms(
        min_confidence="medium",
        output_file="test_export.csv",
    )

    # Check that file was created
    assert output_path.exists()
    assert output_path.name == "test_export.csv"

    # Check that neologisms were marked as exported
    for neo in neologisms:
        test_session.refresh(neo)
        assert neo.exported is True


def test_get_export_summary(test_session: Session, test_config: Config) -> None:
    """Test export summary."""
    # Add neologisms with different confidence levels
    neologisms = [
        Neologism(
            surface="neo1",
            total_score=0.9,
            confidence_level="high",
            exported=True,
        ),
        Neologism(
            surface="neo2",
            total_score=0.7,
            confidence_level="medium",
            exported=False,
        ),
        Neologism(
            surface="neo3",
            total_score=0.4,
            confidence_level="low",
            exported=False,
        ),
    ]
    test_session.add_all(neologisms)
    test_session.commit()

    exporter = MeCabExporter(test_session, test_config)
    summary = exporter.get_export_summary()

    assert summary["total_neologisms"] == 3
    assert summary["total_exported"] == 1
    assert summary["pending_export"] == 2
    assert summary["by_confidence"]["high"]["total"] == 1
    assert summary["by_confidence"]["medium"]["total"] == 1
    assert summary["by_confidence"]["low"]["total"] == 1
