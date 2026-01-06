"""Part-of-speech tag inference for Korean words.

This module provides intelligent POS tag inference based on:
- Word patterns and morphology
- Semantic clues
- Frequency analysis
- Korean linguistic rules
"""

from __future__ import annotations

import re
from typing import ClassVar

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from utils.korean_utils import get_final_consonant, is_hangul


class POSInferencer:
    """Infers POS tags for Korean words based on patterns and rules."""

    # Common name patterns
    FAMILY_NAMES: ClassVar[set[str]] = {
        '김', '이', '박', '최', '정', '강', '조', '윤', '장', '임',
        '한', '오', '서', '신', '권', '황', '안', '송', '류', '홍'
    }

    # Place name suffixes
    PLACE_SUFFIXES: ClassVar[set[str]] = {
        '시', '도', '군', '구', '읍', '면', '동', '리', '가',
        '로', '길', '역', '항', '산', '강', '천', '호', '섬'
    }

    # Organization suffixes
    ORG_SUFFIXES: ClassVar[set[str]] = {
        '회사', '기업', '그룹', '은행', '병원', '학교', '대학', '연구소',
        '센터', '재단', '협회', '조합', '위원회', '부', '청', '원'
    }

    # Verb/adjective endings
    VERB_ENDINGS: ClassVar[set[str]] = {
        '하다', '되다', '이다', '아니다', '스럽다', '롭다'
    }

    # Common noun suffixes
    NOUN_SUFFIXES: ClassVar[set[str]] = {
        '성', '적', '화', '력', '도', '율', '치', '량', '권'
    }

    def __init__(self):
        """Initialize POS inferencer."""
        self.pattern_cache: dict[str, str] = {}

    def infer_pos(self, word: str, context: str | None = None) -> str:
        """Infer POS tag for word.

        Args:
            word: Word to analyze.
            context: Optional context (semantic category, source, etc.).

        Returns:
            Inferred POS tag (NNG, NNP, VV, VA, etc.).

        Examples:
            >>> inferencer = POSInferencer()
            >>> inferencer.infer_pos("서울")
            'NNP'
            >>> inferencer.infer_pos("사랑하다")
            'VV'
        """
        if word in self.pattern_cache:
            return self.pattern_cache[word]

        # Check context hints
        if context:
            context_lower = context.lower()
            if any(hint in context_lower for hint in ['인명', 'person', 'name']):
                return 'NNP'
            if any(hint in context_lower for hint in ['지명', 'place', 'location']):
                return 'NNP'
            if any(hint in context_lower for hint in ['기관', 'organization', 'company']):
                return 'NNP'

        # Apply inference rules
        pos = self._infer_by_pattern(word)
        self.pattern_cache[word] = pos
        return pos

    def _infer_by_pattern(self, word: str) -> str:
        """Infer POS based on word patterns.

        Args:
            word: Word to analyze.

        Returns:
            Inferred POS tag.
        """
        if not word:
            return 'NNG'

        # Check for person names (family name + given name pattern)
        if len(word) >= 2 and word[0] in self.FAMILY_NAMES:
            if len(word) <= 4:  # Most Korean names are 2-4 syllables
                return 'NNP'

        # Check for place names
        if any(word.endswith(suffix) for suffix in self.PLACE_SUFFIXES):
            return 'NNP'

        # Check for organization names
        if any(word.endswith(suffix) for suffix in self.ORG_SUFFIXES):
            return 'NNP'

        # Check for verbs/adjectives
        if any(word.endswith(ending) for ending in self.VERB_ENDINGS):
            if '하다' in word:
                return 'VV'  # 동사
            elif any(word.endswith(e) for e in ['스럽다', '롭다']):
                return 'VA'  # 형용사
            else:
                return 'VV'

        # Check for English letters (foreign words)
        if any(c.isalpha() and not is_hangul(c) for c in word):
            return 'NNG'

        # Check for numbers
        if any(c.isdigit() for c in word):
            return 'NR'  # 수사

        # Check for common noun patterns
        if any(word.endswith(suffix) for suffix in self.NOUN_SUFFIXES):
            return 'NNG'

        # Default to common noun
        return 'NNG'

    def infer_semantic(self, word: str, pos: str) -> str:
        """Infer semantic category for proper nouns.

        Args:
            word: Word to analyze.
            pos: POS tag.

        Returns:
            Semantic category (인명, 지명, 기관, or *).
        """
        if pos != 'NNP':
            return '*'

        # Person names
        if len(word) >= 2 and word[0] in self.FAMILY_NAMES:
            if len(word) <= 4:
                return '인명'

        # Place names
        if any(word.endswith(suffix) for suffix in self.PLACE_SUFFIXES):
            return '지명'

        # Organizations
        if any(word.endswith(suffix) for suffix in self.ORG_SUFFIXES):
            return '기관'

        return '*'


def infer_pos_tag(word: str, context: str | None = None) -> str:
    """Infer POS tag for word (convenience function).

    Args:
        word: Word to analyze.
        context: Optional context hint.

    Returns:
        Inferred POS tag.

    Examples:
        >>> infer_pos_tag("서울")
        'NNP'
        >>> infer_pos_tag("컴퓨터")
        'NNG'
    """
    inferencer = POSInferencer()
    return inferencer.infer_pos(word, context)
