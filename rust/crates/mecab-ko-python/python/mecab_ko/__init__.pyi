"""
Type stubs for mecab_ko package.

This file provides type hints for better IDE support and type checking.
"""

from typing import List, Optional, Tuple

__version__: str
__doc__: str

class Mecab:
    """
    MeCab-Ko tokenizer for Korean morphological analysis.

    This class provides a KoNLPy-compatible interface for Korean morphological analysis.
    """

    def __init__(self, dicpath: Optional[str] = None) -> None:
        """
        Create a new Mecab instance.

        Args:
            dicpath: Optional path to dictionary directory.

        Raises:
            RuntimeError: If the tokenizer fails to initialize.
        """
        ...

    def morphs(self, text: str) -> List[str]:
        """
        Extract morphemes from text.

        Args:
            text: Input text to analyze.

        Returns:
            List of morphemes (surface forms).

        Example:
            >>> mecab = Mecab()
            >>> mecab.morphs("안녕하세요")
            ['안녕', '하', '세요']
        """
        ...

    def nouns(self, text: str) -> List[str]:
        """
        Extract nouns from text.

        Args:
            text: Input text to analyze.

        Returns:
            List of nouns.

        Example:
            >>> mecab = Mecab()
            >>> mecab.nouns("아버지가방에들어가신다")
            ['아버지', '가방']
        """
        ...

    def pos(self, text: str) -> List[Tuple[str, str]]:
        """
        Perform part-of-speech tagging.

        Args:
            text: Input text to analyze.

        Returns:
            List of tuples (surface, pos_tag).

        Example:
            >>> mecab = Mecab()
            >>> mecab.pos("나는 학생입니다")
            [('나', 'NP'), ('는', 'JX'), ('학생', 'NNG'), ('이', 'VCP'), ('ㅂ니다', 'EF')]
        """
        ...

    def parse(self, text: str) -> str:
        """
        Parse text and return MeCab format output.

        Args:
            text: Input text to analyze.

        Returns:
            MeCab format string with tab-separated values.

        Example:
            >>> mecab = Mecab()
            >>> result = mecab.parse("안녕하세요")
            >>> print(result)
            안녕	NNG,*,*,안녕,*,*,*,*
            하	XSV,*,*,하,*,*,*,*
            세요	EF,*,*,세요,*,*,*,*
            EOS
        """
        ...

    def wakati(self, text: str) -> List[str]:
        """
        Alias for morphs() - extract morphemes.

        This method is provided for compatibility with some interfaces.

        Args:
            text: Input text to analyze.

        Returns:
            List of morphemes (surface forms).
        """
        ...

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

__all__ = ["Mecab", "__version__"]
