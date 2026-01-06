# 빠른 시작

이 장에서는 MeCab-Ko를 사용하여 한국어 형태소 분석을 시작하는 방법을 설명합니다.

## CLI 빠른 시작

### 기본 사용법

가장 간단한 형태소 분석:

```bash
mecab-ko "안녕하세요"
```

출력:

```
안녕    NNG
하      XSV
세요    EP+EF
EOS
```

### 파이프라인 사용

표준 입력을 통한 분석:

```bash
echo "오늘 날씨가 좋습니다" | mecab-ko
```

파일 분석:

```bash
cat input.txt | mecab-ko > output.txt
```

### 분리만 수행 (Wakati)

띄어쓰기로 분리된 형태소만 출력:

```bash
mecab-ko -O wakati "형태소 분석 테스트"
```

출력:

```
형태소 분석 테스트
```

### JSON 출력

프로그래밍 친화적인 JSON 포맷:

```bash
mecab-ko -O json "안녕"
```

출력:

```json
[
  {
    "surface": "안녕",
    "pos": "NNG",
    "start": 0,
    "end": 6
  }
]
```

## 라이브러리 빠른 시작

### 프로젝트 설정

새 Rust 프로젝트 생성:

```bash
cargo new my-nlp-project
cd my-nlp-project
```

`Cargo.toml`에 의존성 추가:

```toml
[dependencies]
mecab-ko = "0.1"
```

### 기본 형태소 분석

```rust
use mecab_ko::Tokenizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create tokenizer instance
    let tokenizer = Tokenizer::new()?;

    // Tokenize text
    let text = "아버지가방에들어가신다";
    let tokens = tokenizer.tokenize(text);

    // Print results
    for token in tokens {
        println!("{}\t{}", token.surface, token.pos);
    }

    Ok(())
}
```

### 다양한 분석 메서드

```rust
use mecab_ko::Tokenizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = Tokenizer::new()?;
    let text = "오늘 날씨가 좋습니다";

    // Get morphemes only (surface forms)
    let morphs = tokenizer.morphs(text);
    println!("Morphs: {:?}", morphs);
    // Output: ["오늘", "날씨", "가", "좋", "습니다"]

    // Get nouns only
    let nouns = tokenizer.nouns(text);
    println!("Nouns: {:?}", nouns);
    // Output: ["오늘", "날씨"]

    // Get POS-tagged pairs
    let pos = tokenizer.pos(text);
    println!("POS: {:?}", pos);
    // Output: [("오늘", "NNG"), ("날씨", "NNG"), ("가", "JKS"), ...]

    Ok(())
}
```

### 한글 유틸리티 사용

```rust
use mecab_ko::hangul::{decompose, compose, is_hangul, has_jongseong};

fn main() {
    // Decompose Hangul syllable to Jamo
    if let Some((cho, jung, jong)) = decompose('한') {
        println!("초성: {}, 중성: {}, 종성: {:?}", cho, jung, jong);
        // Output: 초성: ㅎ, 중성: ㅏ, 종성: Some('ㄴ')
    }

    // Compose Jamo to Hangul syllable
    if let Some(c) = compose('ㅎ', 'ㅏ', Some('ㄴ')) {
        println!("Composed: {}", c);  // Output: 한
    }

    // Check if character is Hangul
    println!("Is '가' Hangul? {}", is_hangul('가'));  // true
    println!("Is 'a' Hangul? {}", is_hangul('a'));    // false

    // Check for jongseong (final consonant)
    println!("Has jongseong '한': {:?}", has_jongseong('한'));  // Some(true)
    println!("Has jongseong '하': {:?}", has_jongseong('하'));  // Some(false)
}
```

### 문자열 자모 분리/결합

```rust
use mecab_ko::hangul::{decompose_str, compose_str};

fn main() {
    // Decompose entire string
    let decomposed = decompose_str("한글");
    println!("{}", decomposed);  // Output: ㅎㅏㄴㄱㅡㄹ

    // Compose Jamo string back
    let composed = compose_str("ㅎㅏㄴㄱㅡㄹ");
    println!("{}", composed);    // Output: 한글
}
```

## 사용자 사전 활용

### CSV 사전 생성

`user.csv` 파일:

```csv
# Custom dictionary for technical terms
딥러닝,NNG,-1000,딥러닝
머신러닝,NNG,-1000,머신러닝
자연어처리,NNG,-1000,자연어처리
챗GPT,NNP,-1000,챗지피티
```

### CLI에서 사용

```bash
mecab-ko --user-dic user.csv "딥러닝 기술이 발전하고 있습니다"
```

### 라이브러리에서 사용

```rust
use mecab_ko_dict::UserDictionary;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create user dictionary
    let mut user_dict = UserDictionary::new();

    // Add entries programmatically
    user_dict.add_entry("딥러닝", "NNG", Some(-1000), None);
    user_dict.add_entry("챗GPT", "NNP", Some(-1000), Some("챗지피티".to_string()));

    // Or load from CSV file
    user_dict.load_from_csv("user.csv")?;

    // Check loaded entries
    println!("Loaded {} entries", user_dict.len());

    Ok(())
}
```

## 출력 포맷 선택

MeCab-Ko는 다양한 출력 포맷을 지원합니다:

| 포맷 | 옵션 | 설명 |
|------|------|------|
| Default | `-O default` | MeCab 호환 포맷 |
| Wakati | `-O wakati` | 형태소만 공백 분리 |
| JSON | `-O json` | JSON 배열 |
| CSV | `-O csv` | CSV 포맷 |
| Simple | `-O simple` | 표면형/품사 쌍 |
| POS | `-O pos` | 표면형/품사 형식 |
| Dump | `-O dump` | 디버그용 상세 정보 |

예시:

```bash
# Simple format
mecab-ko -O simple "테스트"
# Output: 테스트/NNG

# CSV format
mecab-ko -O csv "테스트"
# Output:
# surface,pos,start,end,reading,lemma
# 테스트,NNG,0,9,,
```

## 다음 단계

- [CLI 사용법](cli-usage.md): 모든 CLI 옵션 상세 설명
- [API 레퍼런스](api-reference.md): 라이브러리 API 상세 문서
- [사용자 사전](user-dictionary.md): 사전 커스터마이징 가이드
