"""MeCab-Ko Dictionary Expander.

A comprehensive toolkit for expanding MeCab Korean dictionary entries.

Modules:
    - expand_proper_nouns: Extract and generate proper noun entries
    - expand_compounds: Generate compound noun entries
    - expand_conjugations: Generate verb/adjective conjugations
    - expand_abbreviations: Generate abbreviation and acronym entries
    - data_sources: Data source fetchers (Wikipedia, public data)
    - validators: Quality validation and deduplication
    - utils: Utility functions for MeCab format and Korean text

Example:
    >>> from utils.mecab_format import MecabEntry
    >>> from utils.korean_utils import get_jongseong_marker
    >>> entry = MecabEntry(
    ...     surface="서울",
    ...     pos="NNP",
    ...     semantic="지명",
    ...     has_jongseong=get_jongseong_marker("서울"),
    ...     reading="서울"
    ... )
    >>> print(entry.to_csv_line())
    서울,0,0,0,NNP,지명,T,서울,*,*,*,*
"""

__version__ = "1.0.0"
__author__ = "MeCab-Ko Project"

# Make commonly used classes available at package level
from utils.mecab_format import MecabEntry, format_mecab_line, parse_mecab_line
from utils.korean_utils import (
    get_jongseong_marker,
    has_final_consonant,
    is_hangul,
)

__all__ = [
    "MecabEntry",
    "format_mecab_line",
    "parse_mecab_line",
    "get_jongseong_marker",
    "has_final_consonant",
    "is_hangul",
    "__version__",
]
