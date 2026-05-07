"""Shared pytest fixtures for mecab-ko Python E2E tests."""
import pytest


@pytest.fixture(scope="session")
def mecab():
    """Create a Mecab() instance; skip the test if no dictionary is available."""
    mecab_ko = pytest.importorskip("mecab_ko")
    try:
        instance = mecab_ko.Mecab()
    except Exception:
        pytest.skip("No MeCab dictionary available")
    return instance
