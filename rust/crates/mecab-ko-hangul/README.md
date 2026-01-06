# mecab-ko-hangul

한글 처리 유틸리티 라이브러리 - 자모 분리/결합, 음절 처리, 정규화

## 특징

- **자모 분리/결합**: 한글 음절을 초성, 중성, 종성으로 분리하고 다시 결합
- **음절 처리**: 한글 음절 단위 처리 및 검증
- **정규화**: 한글 텍스트 정규화 (호환 자모 → 조합형)
- **Zero-cost abstractions**: 성능 오버헤드 없는 안전한 API
- **No dependencies**: 외부 의존성 없는 경량 라이브러리

## 사용 예제

```rust
use mecab_ko_hangul::{decompose, compose, is_hangul_syllable};

// 자모 분리
let (cho, jung, jong) = decompose('한').unwrap();
assert_eq!(cho, 'ㅎ');
assert_eq!(jung, 'ㅏ');
assert_eq!(jong, Some('ㄴ'));

// 자모 결합
let syllable = compose('ㅎ', 'ㅏ', Some('ㄴ')).unwrap();
assert_eq!(syllable, '한');

// 음절 검증
assert!(is_hangul_syllable('한'));
assert!(!is_hangul_syllable('a'));
```

## 라이선스

MIT 또는 Apache-2.0 중 선택
