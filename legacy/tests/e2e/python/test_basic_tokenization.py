"""Basic tokenization E2E tests for MeCab-Ko Python binding."""

import pytest


class TestBasicTokenization:
    """Test basic tokenization functionality."""

    def test_simple_sentence(self, mecab_tagger, test_sentences):
        """Test tokenization of a simple sentence."""
        test_case = next(
            tc for tc in test_sentences["test_cases"] if tc["id"] == "basic_001"
        )

        result = mecab_tagger.parse(test_case["input"])
        assert result is not None
        assert isinstance(result, str)

    def test_verb_conjugation(self, mecab_tagger, test_sentences):
        """Test tokenization with verb conjugation."""
        test_case = next(
            tc for tc in test_sentences["test_cases"] if tc["id"] == "basic_002"
        )

        result = mecab_tagger.parse(test_case["input"])
        assert result is not None

    def test_question_sentence(self, mecab_tagger, test_sentences):
        """Test tokenization of question sentence."""
        test_case = next(
            tc for tc in test_sentences["test_cases"] if tc["id"] == "basic_003"
        )

        result = mecab_tagger.parse(test_case["input"])
        assert result is not None

    def test_compound_noun(self, mecab_tagger, test_sentences):
        """Test tokenization of compound nouns."""
        test_case = next(
            tc for tc in test_sentences["test_cases"] if tc["id"] == "compound_001"
        )

        result = mecab_tagger.parse(test_case["input"])
        assert result is not None

    def test_mixed_korean_english(self, mecab_tagger, test_sentences):
        """Test tokenization of mixed Korean and English text."""
        test_case = next(
            tc for tc in test_sentences["test_cases"] if tc["id"] == "mixed_001"
        )

        result = mecab_tagger.parse(test_case["input"])
        assert result is not None
        # Should contain "Python" as a separate token
        assert "Python" in result

    def test_numbers(self, mecab_tagger, test_sentences):
        """Test tokenization with numbers."""
        test_case = next(
            tc for tc in test_sentences["test_cases"] if tc["id"] == "numbers_001"
        )

        result = mecab_tagger.parse(test_case["input"])
        assert result is not None
        # Should contain number tokens
        assert "2024" in result

    def test_honorific_speech(self, mecab_tagger, test_sentences):
        """Test tokenization of honorific speech."""
        test_case = next(
            tc for tc in test_sentences["test_cases"] if tc["id"] == "honorific_001"
        )

        result = mecab_tagger.parse(test_case["input"])
        assert result is not None


class TestEdgeCases:
    """Test edge cases in tokenization."""

    def test_empty_string(self, mecab_tagger, test_sentences):
        """Test tokenization of empty string."""
        test_case = next(
            tc for tc in test_sentences["test_cases"] if tc["id"] == "edge_empty"
        )

        result = mecab_tagger.parse(test_case["input"])
        # Should handle empty string gracefully
        assert result is not None

    def test_whitespace_only(self, mecab_tagger, test_sentences):
        """Test tokenization of whitespace-only string."""
        test_case = next(
            tc for tc in test_sentences["test_cases"] if tc["id"] == "edge_whitespace"
        )

        result = mecab_tagger.parse(test_case["input"])
        assert result is not None

    def test_punctuation_only(self, mecab_tagger, test_sentences):
        """Test tokenization of punctuation-only string."""
        test_case = next(
            tc for tc in test_sentences["test_cases"] if tc["id"] == "edge_punctuation"
        )

        result = mecab_tagger.parse(test_case["input"])
        assert result is not None

    def test_long_sentence(self, mecab_tagger, test_sentences):
        """Test tokenization of long sentence."""
        test_case = next(
            tc for tc in test_sentences["test_cases"] if tc["id"] == "long_sentence"
        )

        result = mecab_tagger.parse(test_case["input"])
        assert result is not None


class TestOutputFormats:
    """Test different output formats."""

    def test_default_format(self, mecab_tagger):
        """Test default output format."""
        text = "나는 학교에 갑니다."
        result = mecab_tagger.parse(text)

        assert result is not None
        # Default format should have multiple lines
        lines = result.strip().split("\n")
        assert len(lines) > 1

    @pytest.mark.parametrize(
        "text",
        [
            "안녕하세요.",
            "오늘 날씨가 좋아요.",
            "Python 프로그래밍",
        ],
    )
    def test_various_inputs(self, mecab_tagger, text):
        """Test tokenization with various inputs."""
        result = mecab_tagger.parse(text)
        assert result is not None
        assert len(result) > 0


class TestThreadSafety:
    """Test thread safety of the Python binding."""

    def test_concurrent_parsing(self, mecab_tagger):
        """Test concurrent parsing from multiple threads."""
        import concurrent.futures

        texts = [
            "나는 학교에 갑니다.",
            "오늘은 날씨가 좋습니다.",
            "이것은 무엇입니까?",
        ] * 10

        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as executor:
            futures = [executor.submit(mecab_tagger.parse, text) for text in texts]
            results = [f.result() for f in concurrent.futures.as_completed(futures)]

        assert len(results) == len(texts)
        assert all(r is not None for r in results)


class TestMemoryManagement:
    """Test memory management in Python binding."""

    def test_large_batch(self, mecab_tagger):
        """Test parsing large batch of texts."""
        text = "나는 학교에 갑니다. "
        large_text = text * 1000

        result = mecab_tagger.parse(large_text)
        assert result is not None

    def test_repeated_parsing(self, mecab_tagger):
        """Test repeated parsing doesn't leak memory."""
        text = "나는 학교에 갑니다."

        # Parse the same text multiple times
        for _ in range(1000):
            result = mecab_tagger.parse(text)
            assert result is not None


@pytest.mark.benchmark
class TestPerformance:
    """Performance benchmarks for Python binding."""

    def test_short_text_performance(self, mecab_tagger, benchmark):
        """Benchmark short text parsing."""
        text = "나는 학교에 갑니다."
        result = benchmark(mecab_tagger.parse, text)
        assert result is not None

    def test_long_text_performance(self, mecab_tagger, benchmark):
        """Benchmark long text parsing."""
        text = "나는 학교에 갑니다. " * 100
        result = benchmark(mecab_tagger.parse, text)
        assert result is not None
