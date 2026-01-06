#!/usr/bin/env python3
"""
Example usage of mecab-ko Python bindings

This demonstrates the KoNLPy-compatible API for Korean morphological analysis.
"""

from mecab_ko import Mecab


def main():
    # Create tokenizer instance
    print("Creating Mecab tokenizer...")
    mecab = Mecab()
    print(f"Tokenizer: {mecab}")
    print()

    # Example 1: Extract morphemes
    text1 = "안녕하세요"
    print(f"Text: {text1}")
    print(f"morphs(): {mecab.morphs(text1)}")
    print()

    # Example 2: Extract nouns
    text2 = "아버지가방에들어가신다"
    print(f"Text: {text2}")
    print(f"nouns(): {mecab.nouns(text2)}")
    print()

    # Example 3: Part-of-speech tagging
    text3 = "나는 학생입니다"
    print(f"Text: {text3}")
    print(f"pos(): {mecab.pos(text3)}")
    print()

    # Example 4: MeCab format output
    text4 = "형태소 분석"
    print(f"Text: {text4}")
    print("parse():")
    print(mecab.parse(text4))

    # Example 5: Complex sentence
    text5 = "자연어 처리는 인공지능의 중요한 분야입니다"
    print(f"Text: {text5}")
    print(f"morphs(): {mecab.morphs(text5)}")
    print(f"nouns(): {mecab.nouns(text5)}")
    print(f"pos(): {mecab.pos(text5)}")
    print()


if __name__ == "__main__":
    main()
