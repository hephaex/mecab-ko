#!/usr/bin/env python3
"""
Unit tests for mecab-ko Python bindings

These tests verify the KoNLPy-compatible API.
"""

import sys
import pytest


# Try to import the module
try:
    from mecab_ko import Mecab
except ImportError as e:
    print(f"Error: Cannot import mecab_ko: {e}")
    print("Please build and install the module first:")
    print("  maturin develop")
    sys.exit(1)


class TestMecab:
    """Test cases for Mecab class"""

    @pytest.fixture
    def mecab(self):
        """Fixture to create a Mecab instance"""
        return Mecab()

    def test_mecab_creation(self):
        """Test Mecab instance creation"""
        mecab = Mecab()
        assert mecab is not None
        assert str(mecab) == "MeCab-Ko tokenizer"

    def test_morphs(self, mecab):
        """Test morphs() method"""
        result = mecab.morphs("안녕하세요")
        assert isinstance(result, list)
        assert len(result) > 0
        assert all(isinstance(m, str) for m in result)

    def test_nouns(self, mecab):
        """Test nouns() method"""
        result = mecab.nouns("아버지가방에들어가신다")
        assert isinstance(result, list)
        # Should contain nouns like '아버지', '가방'
        assert all(isinstance(n, str) for n in result)

    def test_pos(self, mecab):
        """Test pos() method"""
        result = mecab.pos("나는 학생입니다")
        assert isinstance(result, list)
        assert len(result) > 0
        # Each element should be a tuple of (surface, pos_tag)
        assert all(isinstance(item, tuple) and len(item) == 2 for item in result)
        assert all(isinstance(item[0], str) and isinstance(item[1], str) for item in result)

    def test_parse(self, mecab):
        """Test parse() method"""
        result = mecab.parse("형태소 분석")
        assert isinstance(result, str)
        assert "EOS" in result  # Should end with EOS marker
        assert "\t" in result   # Should contain tab-separated values

    def test_wakati(self, mecab):
        """Test wakati() method (alias for morphs)"""
        text = "자연어 처리"
        morphs_result = mecab.morphs(text)
        wakati_result = mecab.wakati(text)
        assert morphs_result == wakati_result

    def test_empty_string(self, mecab):
        """Test with empty string"""
        result = mecab.morphs("")
        assert isinstance(result, list)

    def test_english_text(self, mecab):
        """Test with English text"""
        result = mecab.morphs("Hello World")
        assert isinstance(result, list)

    def test_mixed_text(self, mecab):
        """Test with mixed Korean and English text"""
        result = mecab.morphs("Hello 안녕하세요 World")
        assert isinstance(result, list)
        assert len(result) > 0

    def test_special_characters(self, mecab):
        """Test with special characters"""
        result = mecab.morphs("!@#$%^&*()")
        assert isinstance(result, list)

    def test_numbers(self, mecab):
        """Test with numbers"""
        result = mecab.morphs("12345")
        assert isinstance(result, list)


class TestMecabWithCustomDict:
    """Test cases for Mecab with custom dictionary"""

    def test_custom_dict_path(self):
        """Test Mecab creation with custom dictionary path"""
        # This should fail gracefully if dictionary doesn't exist
        with pytest.raises(RuntimeError):
            Mecab(dicpath="/nonexistent/path")


class TestModuleMetadata:
    """Test module-level metadata"""

    def test_module_version(self):
        """Test module version is accessible"""
        import mecab_ko
        assert hasattr(mecab_ko, "__version__")
        assert isinstance(mecab_ko.__version__, str)

    def test_module_doc(self):
        """Test module documentation is accessible"""
        import mecab_ko
        assert hasattr(mecab_ko, "__doc__")
        assert isinstance(mecab_ko.__doc__, str)


def test_konlpy_compatibility():
    """Test KoNLPy API compatibility"""
    mecab = Mecab()

    # These methods should exist and work like KoNLPy's Mecab
    text = "자연어 처리는 재미있다"

    # morphs
    morphs = mecab.morphs(text)
    assert isinstance(morphs, list)

    # nouns
    nouns = mecab.nouns(text)
    assert isinstance(nouns, list)

    # pos
    pos_tags = mecab.pos(text)
    assert isinstance(pos_tags, list)
    assert all(isinstance(item, tuple) for item in pos_tags)

    # parse
    parsed = mecab.parse(text)
    assert isinstance(parsed, str)


if __name__ == "__main__":
    # Run tests with pytest
    pytest.main([__file__, "-v"])
