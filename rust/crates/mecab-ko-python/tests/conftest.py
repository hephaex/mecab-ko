import pytest

try:
    from mecab_ko import Mecab
    _mecab = Mecab()
    _has_full_dict = len(_mecab.morphs("테스트")) > 0
except Exception:
    _has_full_dict = False

requires_dict = pytest.mark.skipif(
    not _has_full_dict,
    reason="requires system dictionary (sys.dic) for Korean analysis",
)
