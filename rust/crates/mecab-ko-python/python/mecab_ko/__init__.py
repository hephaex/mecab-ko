"""
MeCab-Ko Python: Korean Morphological Analyzer

This package provides Python bindings for MeCab-Ko, a Korean morphological
analyzer implemented in Rust. The API is compatible with KoNLPy's Mecab interface.

Example:
    >>> from mecab_ko import Mecab
    >>> mecab = Mecab()
    >>> mecab.morphs("안녕하세요")
    ['안녕', '하', '세요']
    >>> mecab.nouns("아버지가방에들어가신다")
    ['아버지', '가방']
    >>> mecab.pos("나는 학생입니다")
    [('나', 'NP'), ('는', 'JX'), ('학생', 'NNG'), ('이', 'VCP'), ('ㅂ니다', 'EF')]
"""

from .mecab_ko import Mecab, __version__, __doc__

__all__ = ["Mecab", "__version__"]
