"""Pytest configuration and fixtures."""

from __future__ import annotations

import tempfile
from pathlib import Path
from typing import Generator

import pytest
from sqlalchemy import create_engine
from sqlalchemy.orm import Session, sessionmaker

from src.config import Config
from src.models import Base


@pytest.fixture
def temp_db() -> Generator[str, None, None]:
    """Create a temporary database for testing.

    Yields:
        Path to temporary database file.
    """
    with tempfile.NamedTemporaryFile(suffix=".db", delete=False) as f:
        db_path = f.name

    db_url = f"sqlite:///{db_path}"
    yield db_url

    # Cleanup
    Path(db_path).unlink(missing_ok=True)


@pytest.fixture
def test_config(temp_db: str) -> Config:
    """Create test configuration.

    Args:
        temp_db: Temporary database URL.

    Returns:
        Test configuration object.
    """
    config = Config(
        database={"url": temp_db},
        crawler={
            "user_agent": "Test Crawler",
            "rate_limit": {"requests_per_second": 10, "burst": 5},
            "timeout": {"connect": 5, "read": 10},
            "retry": {"max_attempts": 1, "backoff_factor": 1},
            "respect_robots_txt": False,
        },
        detector={
            "min_frequency": 2,
            "min_length": 2,
            "max_length": 20,
            "allow_mixed_script": True,
            "allow_special_chars": False,
            "thresholds": {
                "min_occurrences": 3,
                "tfidf_threshold": 0.1,
                "min_context_diversity": 2,
            },
        },
        sources={},
    )
    return config


@pytest.fixture
def test_session(test_config: Config) -> Generator[Session, None, None]:
    """Create test database session.

    Args:
        test_config: Test configuration.

    Yields:
        SQLAlchemy session.
    """
    engine = create_engine(test_config.database.url)
    Base.metadata.create_all(engine)

    SessionLocal = sessionmaker(bind=engine)
    session = SessionLocal()

    yield session

    session.close()
    Base.metadata.drop_all(engine)


@pytest.fixture
def sample_text() -> str:
    """Sample Korean text for testing.

    Returns:
        Sample text string.
    """
    return """
    최근 갓생을 살고 있는 MZ세대가 늘고 있다.
    갓생이란 God과 인생을 합친 신조어로, 부지런하고 성실한 삶을 의미한다.
    많은 사람들이 미라클모닝과 함께 갓생을 실천하고 있다.
    특히 N잡러들 사이에서 갓생 살기가 유행이다.
    """


@pytest.fixture
def sample_candidates() -> list[dict[str, str]]:
    """Sample candidate neologisms for testing.

    Returns:
        List of candidate dictionaries.
    """
    return [
        {"surface": "갓생", "frequency": 10},
        {"surface": "미라클모닝", "frequency": 5},
        {"surface": "N잡러", "frequency": 3},
        {"surface": "MZ세대", "frequency": 15},
    ]
