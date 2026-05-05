"""Basic E2E test for mecab-ko Python bindings."""
import pytest


def test_import():
    """Verify mecab_ko can be imported."""
    import mecab_ko
    assert hasattr(mecab_ko, "Mecab")


def test_version():
    """Verify version is accessible."""
    import mecab_ko
    assert hasattr(mecab_ko, "__version__")
