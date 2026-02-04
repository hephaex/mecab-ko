#!/usr/bin/env python3
"""
Advanced usage examples for mecab-ko Python bindings

Demonstrates keyword extraction, sentence analysis, and filtering.
"""

from collections import Counter
from typing import List, Tuple, Dict, Any

try:
    from mecab_ko import Mecab
except ImportError:
    print("Please install mecab-ko first: maturin develop")
    raise


def extract_keywords(text: str, top_n: int = 5) -> List[Tuple[str, int]]:
    """
    Extract top N keywords (nouns) from text.

    Args:
        text: Input Korean text
        top_n: Number of top keywords to return

    Returns:
        List of (keyword, count) tuples
    """
    mecab = Mecab()
    nouns = mecab.nouns(text)
    counter = Counter(nouns)
    return counter.most_common(top_n)


def analyze_sentence_structure(text: str) -> Dict[str, Any]:
    """
    Analyze sentence structure by POS tags.

    Args:
        text: Input Korean text

    Returns:
        Dictionary with analysis results
    """
    mecab = Mecab()
    pos_tags = mecab.pos(text)

    # Count POS tag categories (first 2 chars)
    pos_counts = Counter(pos[1][:2] for pos in pos_tags)

    return {
        'total_tokens': len(pos_tags),
        'unique_tokens': len(set(t[0] for t in pos_tags)),
        'pos_distribution': dict(pos_counts),
        'tokens': pos_tags
    }


def extract_verb_phrases(text: str) -> List[str]:
    """
    Extract verbs from text.

    Args:
        text: Input Korean text

    Returns:
        List of verb surface forms
    """
    mecab = Mecab()
    pos_tags = mecab.pos(text)
    return [surface for surface, pos in pos_tags if pos.startswith('VV')]


def extract_by_pos(text: str, pos_filter: str) -> List[str]:
    """
    Extract tokens matching a POS tag prefix.

    Args:
        text: Input Korean text
        pos_filter: POS tag prefix to filter (e.g., 'NN', 'VV', 'VA')

    Returns:
        List of matching surface forms
    """
    mecab = Mecab()
    pos_tags = mecab.pos(text)
    return [surface for surface, pos in pos_tags if pos.startswith(pos_filter)]


def tokenize_with_pos(text: str) -> List[Dict[str, str]]:
    """
    Tokenize text and return detailed information.

    Args:
        text: Input Korean text

    Returns:
        List of token dictionaries
    """
    mecab = Mecab()
    pos_tags = mecab.pos(text)

    return [
        {'surface': surface, 'pos': pos}
        for surface, pos in pos_tags
    ]


def compare_texts(text1: str, text2: str) -> Dict[str, Any]:
    """
    Compare morphological structure of two texts.

    Args:
        text1: First Korean text
        text2: Second Korean text

    Returns:
        Comparison dictionary
    """
    mecab = Mecab()

    morphs1 = set(mecab.morphs(text1))
    morphs2 = set(mecab.morphs(text2))

    nouns1 = set(mecab.nouns(text1))
    nouns2 = set(mecab.nouns(text2))

    return {
        'common_morphs': morphs1 & morphs2,
        'unique_to_text1': morphs1 - morphs2,
        'unique_to_text2': morphs2 - morphs1,
        'common_nouns': nouns1 & nouns2,
        'similarity': len(morphs1 & morphs2) / len(morphs1 | morphs2) if morphs1 | morphs2 else 0
    }


def batch_process(texts: List[str]) -> List[Dict[str, Any]]:
    """
    Process multiple texts efficiently.

    Args:
        texts: List of Korean texts

    Returns:
        List of analysis results
    """
    mecab = Mecab()  # Reuse single instance

    results = []
    for text in texts:
        results.append({
            'text': text,
            'morphs': mecab.morphs(text),
            'nouns': mecab.nouns(text),
            'pos': mecab.pos(text)
        })

    return results


def main():
    print("=" * 60)
    print("MeCab-Ko Python Advanced Usage Examples")
    print("=" * 60)

    # Example 1: Keyword extraction
    print("\n[1] Keyword Extraction")
    print("-" * 40)
    text1 = """
    인공지능은 현대 기술의 핵심입니다.
    머신러닝과 딥러닝을 통해 인공지능은 빠르게 발전하고 있습니다.
    자연어 처리는 인공지능의 중요한 응용 분야입니다.
    """
    keywords = extract_keywords(text1)
    print("Top keywords:")
    for keyword, count in keywords:
        print(f"  {keyword}: {count}")

    # Example 2: Sentence structure analysis
    print("\n[2] Sentence Structure Analysis")
    print("-" * 40)
    text2 = "나는 학교에 가서 친구를 만났다"
    analysis = analyze_sentence_structure(text2)
    print(f"Input: {text2}")
    print(f"Total tokens: {analysis['total_tokens']}")
    print(f"Unique tokens: {analysis['unique_tokens']}")
    print(f"POS distribution: {analysis['pos_distribution']}")

    # Example 3: Verb extraction
    print("\n[3] Verb Extraction")
    print("-" * 40)
    text3 = "나는 밥을 먹고 학교에 갔다"
    verbs = extract_verb_phrases(text3)
    print(f"Input: {text3}")
    print(f"Verbs: {verbs}")

    # Example 4: Filter by POS
    print("\n[4] Filter by POS Tag")
    print("-" * 40)
    text4 = "아름다운 서울의 야경이 멋지다"
    nouns = extract_by_pos(text4, 'NN')
    adjectives = extract_by_pos(text4, 'VA')
    print(f"Input: {text4}")
    print(f"Nouns (NN*): {nouns}")
    print(f"Adjectives (VA*): {adjectives}")

    # Example 5: Text comparison
    print("\n[5] Text Comparison")
    print("-" * 40)
    text5a = "오늘 날씨가 좋습니다"
    text5b = "내일 날씨가 나쁩니다"
    comparison = compare_texts(text5a, text5b)
    print(f"Text 1: {text5a}")
    print(f"Text 2: {text5b}")
    print(f"Common morphs: {comparison['common_morphs']}")
    print(f"Similarity: {comparison['similarity']:.2%}")

    # Example 6: Batch processing
    print("\n[6] Batch Processing")
    print("-" * 40)
    texts = [
        "첫 번째 문장입니다",
        "두 번째 문장입니다",
        "세 번째 문장입니다"
    ]
    results = batch_process(texts)
    for i, result in enumerate(results, 1):
        print(f"  [{i}] {result['text']}")
        print(f"      Morphs: {len(result['morphs'])}, Nouns: {len(result['nouns'])}")

    print("\n" + "=" * 60)
    print("Examples completed!")


if __name__ == "__main__":
    main()
