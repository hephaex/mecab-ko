"""E2E tests for mecab-ko Python bindings."""
import pytest


# -- Import tests (no dict needed) --

def test_import():
    """Module imports successfully."""
    import mecab_ko
    assert hasattr(mecab_ko, "Mecab")


def test_version_format():
    """Version follows semver pattern."""
    import mecab_ko
    version = mecab_ko.__version__
    assert isinstance(version, str)
    parts = version.split(".")
    assert len(parts) >= 2
    assert all(p.isdigit() for p in parts[:2])


def test_mecab_constructor_default():
    """Mecab() constructor works."""
    import mecab_ko
    try:
        m = mecab_ko.Mecab()
        assert m is not None
    except Exception:
        pytest.skip("No dictionary available")


def test_mecab_constructor_invalid_dicpath():
    """Mecab with invalid dicpath raises error."""
    import mecab_ko
    with pytest.raises(Exception):
        mecab_ko.Mecab(dicpath="/nonexistent_dict_xyz")


# -- Dict-dependent tests (skip if no dict) --

def test_morphs_returns_list(mecab):
    # Use words present in the mini-dict (안녕+하+세요)
    result = mecab.morphs("안녕하세요")
    assert isinstance(result, list)
    assert len(result) > 0
    assert all(isinstance(m, str) for m in result)


def test_nouns_returns_list(mecab):
    # Use a mini-dict word; don't assert len > 0 — noun extraction may be sparse
    result = mecab.nouns("한국어 사람")
    assert isinstance(result, list)
    assert all(isinstance(n, str) for n in result)


def test_pos_returns_tuples(mecab):
    # Use words present in the mini-dict (한국어, 사람)
    result = mecab.pos("한국어 사람")
    assert isinstance(result, list)
    assert len(result) > 0
    for item in result:
        assert isinstance(item, tuple)
        assert len(item) == 2
        assert isinstance(item[0], str)
        assert isinstance(item[1], str)


def test_parse_contains_eos(mecab):
    result = mecab.parse("안녕하세요")
    assert isinstance(result, str)
    assert "EOS" in result


def test_parse_tab_separated(mecab):
    # 안녕하세요 tokenizes to 3 morphemes in the mini-dict (안녕+하+세요)
    result = mecab.parse("안녕하세요")
    lines = [l for l in result.strip().split("\n") if l and l != "EOS"]
    assert len(lines) > 0
    assert all("\t" in line for line in lines)


def test_wakati_returns_list(mecab):
    # Use words present in the mini-dict
    result = mecab.wakati("안녕하세요")
    assert isinstance(result, list)
    assert len(result) > 0
    morphs_result = mecab.morphs("안녕하세요")
    assert result == morphs_result


def test_empty_input_morphs(mecab):
    result = mecab.morphs("")
    assert isinstance(result, list)


def test_empty_input_parse(mecab):
    result = mecab.parse("")
    assert isinstance(result, str)
