"""Korean language utilities for Hangul processing.

This module provides utilities for Korean text processing including:
- Hangul syllable decomposition and composition
- Final consonant (종성) detection
- Jamo manipulation
"""

from __future__ import annotations


# Hangul Unicode ranges
HANGUL_START = 0xAC00  # '가'
HANGUL_END = 0xD7A3    # '힣'

# Jamo counts
CHOSUNG_COUNT = 19   # 초성 (initial consonants)
JUNGSUNG_COUNT = 21  # 중성 (medial vowels)
JONGSUNG_COUNT = 28  # 종성 (final consonants, including none)

# Chosung (초성) - Initial consonants
CHOSUNG = [
    'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ',
    'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'
]

# Jungsung (중성) - Medial vowels
JUNGSUNG = [
    'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ',
    'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ'
]

# Jongsung (종성) - Final consonants
JONGSUNG = [
    '', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ',
    'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ', 'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ',
    'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'
]


def is_hangul(char: str) -> bool:
    """Check if character is Hangul syllable.

    Args:
        char: Single character to check.

    Returns:
        True if character is Hangul syllable.

    Examples:
        >>> is_hangul('가')
        True
        >>> is_hangul('A')
        False
    """
    if len(char) != 1:
        return False
    code = ord(char)
    return HANGUL_START <= code <= HANGUL_END


def decompose_hangul(syllable: str) -> tuple[str, str, str]:
    """Decompose Hangul syllable into jamo components.

    Args:
        syllable: Single Hangul syllable.

    Returns:
        Tuple of (chosung, jungsung, jongsung).

    Raises:
        ValueError: If input is not a Hangul syllable.

    Examples:
        >>> decompose_hangul('한')
        ('ㅎ', 'ㅏ', 'ㄴ')
        >>> decompose_hangul('가')
        ('ㄱ', 'ㅏ', '')
    """
    if not is_hangul(syllable):
        raise ValueError(f"Not a Hangul syllable: {syllable}")

    code = ord(syllable) - HANGUL_START
    jongsung_idx = code % JONGSUNG_COUNT
    jungsung_idx = (code // JONGSUNG_COUNT) % JUNGSUNG_COUNT
    chosung_idx = code // (JUNGSUNG_COUNT * JONGSUNG_COUNT)

    return (
        CHOSUNG[chosung_idx],
        JUNGSUNG[jungsung_idx],
        JONGSUNG[jongsung_idx],
    )


def compose_hangul(chosung: str, jungsung: str, jongsung: str = "") -> str:
    """Compose Hangul syllable from jamo components.

    Args:
        chosung: Initial consonant.
        jungsung: Medial vowel.
        jongsung: Final consonant (optional).

    Returns:
        Composed Hangul syllable.

    Raises:
        ValueError: If jamo components are invalid.

    Examples:
        >>> compose_hangul('ㅎ', 'ㅏ', 'ㄴ')
        '한'
        >>> compose_hangul('ㄱ', 'ㅏ')
        '가'
    """
    try:
        chosung_idx = CHOSUNG.index(chosung)
        jungsung_idx = JUNGSUNG.index(jungsung)
        jongsung_idx = JONGSUNG.index(jongsung)
    except ValueError as e:
        raise ValueError(f"Invalid jamo: {chosung}, {jungsung}, {jongsung}") from e

    code = (
        HANGUL_START
        + chosung_idx * JUNGSUNG_COUNT * JONGSUNG_COUNT
        + jungsung_idx * JONGSUNG_COUNT
        + jongsung_idx
    )

    return chr(code)


def get_final_consonant(text: str) -> str:
    """Get final consonant (종성) of last Hangul syllable.

    Args:
        text: Text to extract final consonant from.

    Returns:
        Final consonant jamo, or empty string if none.

    Examples:
        >>> get_final_consonant('한글')
        'ㄹ'
        >>> get_final_consonant('나무')
        ''
    """
    if not text:
        return ""

    # Find last Hangul character
    for char in reversed(text):
        if is_hangul(char):
            _, _, jongsung = decompose_hangul(char)
            return jongsung

    return ""


def has_final_consonant(text: str) -> bool:
    """Check if text ends with final consonant (받침).

    Args:
        text: Text to check.

    Returns:
        True if text ends with final consonant.

    Examples:
        >>> has_final_consonant('한글')
        True
        >>> has_final_consonant('나무')
        False
    """
    return bool(get_final_consonant(text))


def get_jongseong_marker(text: str) -> str:
    """Get MeCab jongseong marker ('T' or 'F').

    Args:
        text: Text to check.

    Returns:
        'T' if has final consonant, 'F' otherwise.

    Examples:
        >>> get_jongseong_marker('서울')
        'T'
        >>> get_jongseong_marker('부산')
        'F'
    """
    return "T" if has_final_consonant(text) else "F"


def normalize_hangul(text: str) -> str:
    """Normalize Hangul text by removing duplicates and invalid characters.

    Args:
        text: Text to normalize.

    Returns:
        Normalized text containing only valid Hangul and common characters.
    """
    # Remove leading/trailing whitespace
    text = text.strip()

    # Keep Hangul, numbers, basic punctuation, and spaces
    normalized = "".join(
        char for char in text
        if is_hangul(char) or char.isdigit() or char in " -·."
    )

    return normalized


def syllable_count(text: str) -> int:
    """Count number of Hangul syllables in text.

    Args:
        text: Text to count syllables.

    Returns:
        Number of Hangul syllables.

    Examples:
        >>> syllable_count('안녕하세요')
        5
        >>> syllable_count('Hello 세계')
        2
    """
    return sum(1 for char in text if is_hangul(char))
