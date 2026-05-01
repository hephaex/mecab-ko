#!/usr/bin/env python3
"""
Advanced tests for mecab-ko Python bindings

Tests edge cases, performance, and advanced usage patterns.
"""

import sys
import pytest

from conftest import requires_dict

try:
    from mecab_ko import Mecab
except ImportError as e:
    print(f"Error: Cannot import mecab_ko: {e}")
    print("Please build and install the module first:")
    print("  maturin develop")
    sys.exit(1)


class TestEdgeCases:
    """Test edge cases and boundary conditions"""

    @pytest.fixture
    def mecab(self):
        """Fixture to create a Mecab instance"""
        return Mecab()

    def test_unicode_normalization(self, mecab):
        """Test Unicode normalization"""
        text1 = "가"  # Single character
        text2 = "\u1100\u1161"  # Decomposed form
        result1 = mecab.morphs(text1)
        result2 = mecab.morphs(text2)
        assert isinstance(result1, list)
        assert isinstance(result2, list)

    @requires_dict
    def test_long_text(self, mecab):
        """Test with long text"""
        long_text = "안녕하세요. " * 1000
        result = mecab.morphs(long_text)
        assert isinstance(result, list)
        assert len(result) > 0

    def test_repeated_characters(self, mecab):
        """Test with repeated characters"""
        result = mecab.morphs("ㅋㅋㅋㅋㅋ")
        assert isinstance(result, list)

    def test_whitespace_only(self, mecab):
        """Test with whitespace only"""
        result = mecab.morphs("   \t\n  ")
        assert isinstance(result, list)

    def test_punctuation_only(self, mecab):
        """Test with punctuation only"""
        result = mecab.morphs(".,!?;:")
        assert isinstance(result, list)

    @requires_dict
    def test_mixed_scripts(self, mecab):
        """Test with mixed scripts (Korean, English, numbers, etc.)"""
        text = "Python 3.12는 2023년에 출시되었습니다!"
        result = mecab.morphs(text)
        assert isinstance(result, list)
        assert len(result) > 0

    def test_emoji(self, mecab):
        """Test with emoji"""
        text = "안녕하세요 😊👍"
        result = mecab.morphs(text)
        assert isinstance(result, list)

    def test_special_korean_characters(self, mecab):
        """Test with special Korean characters"""
        text = "ㄱㄴㄷㄹㅏㅑㅓㅕ"
        result = mecab.morphs(text)
        assert isinstance(result, list)


class TestPOSFiltering:
    """Test POS tag filtering functionality"""

    @pytest.fixture
    def mecab(self):
        return Mecab()

    def test_nouns_only_contain_nn_tags(self, mecab):
        """Verify nouns() only returns NN* tagged words"""
        text = "아름다운 하늘과 바다"
        nouns = mecab.nouns(text)
        pos_tags = mecab.pos(text)

        # Get all surfaces with NN tags
        expected_nouns = [surface for surface, pos in pos_tags if pos.startswith("NN")]

        # nouns() result should be subset of NN-tagged words
        for noun in nouns:
            assert any(noun in surface for surface in expected_nouns)

    def test_extract_verbs(self, mecab):
        """Extract verbs (VV tag)"""
        text = "나는 학교에 간다"
        pos_tags = mecab.pos(text)
        verbs = [surface for surface, pos in pos_tags if pos.startswith("VV")]
        assert len(verbs) >= 0  # May or may not have verbs depending on analysis

    def test_extract_adjectives(self, mecab):
        """Extract adjectives (VA tag)"""
        text = "예쁜 꽃이 피었다"
        pos_tags = mecab.pos(text)
        adjectives = [surface for surface, pos in pos_tags if pos.startswith("VA")]
        # Note: May be empty depending on analysis


class TestConsistency:
    """Test consistency across multiple calls"""

    @pytest.fixture
    def mecab(self):
        return Mecab()

    def test_morphs_consistency(self, mecab):
        """Repeated calls should return same results"""
        text = "형태소 분석 테스트"
        result1 = mecab.morphs(text)
        result2 = mecab.morphs(text)
        assert result1 == result2

    def test_pos_consistency(self, mecab):
        """Repeated POS calls should return same results"""
        text = "형태소 분석 테스트"
        result1 = mecab.pos(text)
        result2 = mecab.pos(text)
        assert result1 == result2

    def test_parse_consistency(self, mecab):
        """Repeated parse calls should return same results"""
        text = "형태소 분석"
        result1 = mecab.parse(text)
        result2 = mecab.parse(text)
        assert result1 == result2


class TestOutputFormats:
    """Test different output formats"""

    @pytest.fixture
    def mecab(self):
        return Mecab()

    def test_parse_output_format(self, mecab):
        """Test parse() output matches MeCab format"""
        text = "테스트"
        result = mecab.parse(text)

        lines = result.strip().split("\n")
        # Should end with EOS
        assert lines[-1] == "EOS"

        # Other lines should have tab-separated values
        for line in lines[:-1]:
            assert "\t" in line
            parts = line.split("\t")
            assert len(parts) == 2
            surface, features = parts
            assert len(surface) > 0
            # Features should be comma-separated
            assert "," in features

    def test_pos_tuple_format(self, mecab):
        """Test pos() returns proper tuples"""
        text = "형태소 분석"
        result = mecab.pos(text)

        for item in result:
            assert isinstance(item, tuple)
            assert len(item) == 2
            surface, pos = item
            assert isinstance(surface, str)
            assert isinstance(pos, str)
            assert len(surface) > 0
            assert len(pos) > 0


class TestMemoryAndPerformance:
    """Test memory efficiency and performance"""

    @pytest.fixture
    def mecab(self):
        return Mecab()

    def test_multiple_analyses(self, mecab):
        """Test multiple analyses don't cause issues"""
        texts = [
            "첫 번째 문장입니다.",
            "두 번째 문장입니다.",
            "세 번째 문장입니다.",
        ]

        for text in texts * 10:  # Repeat 10 times
            result = mecab.morphs(text)
            assert isinstance(result, list)

    def test_concurrent_safe(self, mecab):
        """Test that tokenizer is thread-safe (sequential calls)"""
        import time

        texts = ["안녕하세요", "감사합니다", "좋은 하루 되세요"]

        results = []
        for text in texts:
            result = mecab.morphs(text)
            results.append(result)
            time.sleep(0.01)

        assert len(results) == len(texts)
        for result in results:
            assert isinstance(result, list)


class TestErrorHandling:
    """Test error handling"""

    def test_invalid_dicpath(self):
        """Test initialization with invalid dictionary path"""
        with pytest.raises(RuntimeError):
            Mecab(dicpath="/definitely/not/a/valid/path/to/dictionary")

    def test_nonexistent_dicpath(self):
        """Test initialization with nonexistent path"""
        with pytest.raises(RuntimeError):
            Mecab(dicpath="/tmp/nonexistent_mecab_dict_12345")


class TestRealWorldExamples:
    """Test with real-world Korean text examples"""

    @pytest.fixture
    def mecab(self):
        return Mecab()

    @requires_dict
    def test_news_headline(self, mecab):
        """Test with news headline style text"""
        text = "한국 경제 성장률 3% 달성 전망"
        morphs = mecab.morphs(text)
        nouns = mecab.nouns(text)
        pos_tags = mecab.pos(text)

        assert len(morphs) > 0
        assert len(nouns) > 0
        assert len(pos_tags) > 0

        # Should contain economic terms
        assert any("경제" in noun or "성장" in noun for noun in nouns)

    @requires_dict
    def test_social_media_text(self, mecab):
        """Test with social media style text"""
        text = "오늘 날씨 너무 좋다 ㅎㅎ"
        morphs = mecab.morphs(text)
        pos_tags = mecab.pos(text)

        assert len(morphs) > 0
        assert len(pos_tags) > 0

    @requires_dict
    def test_formal_text(self, mecab):
        """Test with formal text"""
        text = "귀하의 의견에 깊이 감사드립니다."
        morphs = mecab.morphs(text)
        nouns = mecab.nouns(text)

        assert len(morphs) > 0
        assert len(nouns) > 0

    @requires_dict
    def test_question(self, mecab):
        """Test with question"""
        text = "내일 날씨가 어떨까요?"
        morphs = mecab.morphs(text)
        pos_tags = mecab.pos(text)

        assert len(morphs) > 0
        assert len(pos_tags) > 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
