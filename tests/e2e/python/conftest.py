"""Pytest configuration for MeCab-Ko Python E2E tests."""

import json
import os
from pathlib import Path
from typing import Any

import pytest


@pytest.fixture(scope="session")
def project_root() -> Path:
    """Get the project root directory."""
    return Path(__file__).parent.parent.parent.parent


@pytest.fixture(scope="session")
def fixtures_dir(project_root: Path) -> Path:
    """Get the fixtures directory."""
    return project_root / "tests" / "e2e" / "fixtures"


@pytest.fixture(scope="session")
def test_sentences(fixtures_dir: Path) -> dict[str, Any]:
    """Load test sentences from JSON fixture."""
    with open(fixtures_dir / "test_sentences.json", encoding="utf-8") as f:
        return json.load(f)


@pytest.fixture(scope="session")
def user_dict_path(fixtures_dir: Path) -> Path:
    """Get the user dictionary path."""
    return fixtures_dir / "user_dict.csv"


@pytest.fixture(scope="session")
def mecab_tagger():
    """Create a MeCab tagger instance.

    Note: This will be implemented once the Python binding is ready.
    For now, we'll use a mock or skip tests.
    """
    try:
        import mecab_ko
        return mecab_ko.Tagger()
    except ImportError:
        pytest.skip("mecab_ko Python module not installed")


@pytest.fixture
def temp_output_dir(tmp_path: Path) -> Path:
    """Create a temporary directory for test outputs."""
    output_dir = tmp_path / "output"
    output_dir.mkdir(exist_ok=True)
    return output_dir
