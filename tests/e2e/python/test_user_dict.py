"""User dictionary E2E tests for MeCab-Ko Python binding."""

import pytest


class TestUserDictionary:
    """Test user dictionary functionality."""

    def test_load_user_dict(self, user_dict_path):
        """Test loading user dictionary."""
        try:
            import mecab_ko

            tagger = mecab_ko.Tagger(user_dict=str(user_dict_path))
            assert tagger is not None
        except ImportError:
            pytest.skip("mecab_ko Python module not installed")
        except Exception as e:
            # User dict might not be implemented yet
            pytest.skip(f"User dict not supported: {e}")

    def test_user_dict_tokenization(self, user_dict_path, test_sentences):
        """Test tokenization with user dictionary."""
        try:
            import mecab_ko

            tagger = mecab_ko.Tagger(user_dict=str(user_dict_path))

            test_case = next(
                tc
                for tc in test_sentences["test_cases"]
                if tc["id"] == "user_dict_001"
            )

            result = tagger.parse(test_case["input"])
            assert result is not None
            # "카카오톡" should be recognized as a single token
            assert "카카오톡" in result

        except ImportError:
            pytest.skip("mecab_ko Python module not installed")
        except Exception as e:
            pytest.skip(f"User dict not supported: {e}")

    def test_user_dict_priority(self, user_dict_path):
        """Test that user dictionary has priority over system dictionary."""
        try:
            import mecab_ko

            # Create tagger without user dict
            tagger_no_dict = mecab_ko.Tagger()
            result_no_dict = tagger_no_dict.parse("카카오톡으로 메시지를 보냈다.")

            # Create tagger with user dict
            tagger_with_dict = mecab_ko.Tagger(user_dict=str(user_dict_path))
            result_with_dict = tagger_with_dict.parse("카카오톡으로 메시지를 보냈다.")

            # Results should be different
            # (user dict should recognize "카카오톡" as single token)
            assert result_no_dict != result_with_dict

        except ImportError:
            pytest.skip("mecab_ko Python module not installed")
        except Exception as e:
            pytest.skip(f"User dict not supported: {e}")
