# 라이브러리 API

이 장에서는 MeCab-Ko 라이브러리의 주요 API를 설명합니다.

## 크레이트 구조

```
mecab-ko (통합 라이브러리)
├── mecab-ko-core     (핵심 분석 엔진)
├── mecab-ko-dict     (사전 관리)
└── mecab-ko-hangul   (한글 유틸리티)
```

대부분의 경우 `mecab-ko` 크레이트만 사용하면 됩니다. 개별 기능만 필요한 경우 해당 크레이트를 직접 의존할 수 있습니다.

## mecab-ko

### Tokenizer

형태소 분석의 메인 인터페이스입니다.

```rust
use mecab_ko::Tokenizer;
```

#### 생성

```rust
// Create with default dictionary
let tokenizer = Tokenizer::new()?;

// Create with custom dictionary path
let tokenizer = Tokenizer::with_dict("/path/to/dict")?;
```

#### 메서드

##### `tokenize(&self, text: &str) -> Vec<Token>`

텍스트를 형태소로 분석합니다.

```rust
let tokens = tokenizer.tokenize("안녕하세요");
for token in tokens {
    println!("{}: {} ({}-{})",
        token.surface, token.pos, token.start, token.end);
}
```

##### `morphs(&self, text: &str) -> Vec<String>`

형태소(표면형)만 추출합니다. `wakati`와 동일합니다.

```rust
let morphs = tokenizer.morphs("오늘 날씨가 좋습니다");
// ["오늘", "날씨", "가", "좋", "습니다"]
```

##### `wakati(&self, text: &str) -> Vec<String>`

형태소를 분리하여 표면형 목록으로 반환합니다.

```rust
let words = tokenizer.wakati("형태소 분석");
// ["형태소", "분석"]
```

##### `nouns(&self, text: &str) -> Vec<String>`

명사(NN으로 시작하는 품사)만 추출합니다.

```rust
let nouns = tokenizer.nouns("오늘 날씨가 좋습니다");
// ["오늘", "날씨"]
```

##### `pos(&self, text: &str) -> Vec<(String, String)>`

(표면형, 품사) 쌍의 목록을 반환합니다.

```rust
let pos_tagged = tokenizer.pos("안녕하세요");
// [("안녕", "NNG"), ("하", "XSV"), ("세요", "EP+EF")]
```

### Token

형태소 분석 결과를 나타내는 구조체입니다.

```rust
pub struct Token {
    pub surface: String,        // 표면형
    pub pos: String,            // 품사 태그
    pub start: usize,           // 시작 바이트 위치
    pub end: usize,             // 끝 바이트 위치
    pub reading: Option<String>, // 읽기
    pub lemma: Option<String>,  // 원형 (기본형)
}
```

예시:

```rust
let token = &tokens[0];
println!("Surface: {}", token.surface);
println!("POS: {}", token.pos);
println!("Position: {}-{}", token.start, token.end);
if let Some(reading) = &token.reading {
    println!("Reading: {}", reading);
}
```

## mecab-ko-hangul

한글 처리를 위한 유틸리티 함수들입니다.

```rust
use mecab_ko::hangul::*;
// or
use mecab_ko_hangul::*;
```

### 자모 분리/결합

#### `decompose(c: char) -> Option<(char, char, Option<char>)>`

한글 음절을 초성, 중성, 종성으로 분해합니다.

```rust
let result = decompose('한');
assert_eq!(result, Some(('ㅎ', 'ㅏ', Some('ㄴ'))));

let result = decompose('가');
assert_eq!(result, Some(('ㄱ', 'ㅏ', None)));

let result = decompose('a');
assert_eq!(result, None);  // Not a Hangul syllable
```

#### `compose(cho: char, jung: char, jong: Option<char>) -> Option<char>`

초성, 중성, 종성을 결합하여 한글 음절을 만듭니다.

```rust
let c = compose('ㅎ', 'ㅏ', Some('ㄴ'));
assert_eq!(c, Some('한'));

let c = compose('ㄱ', 'ㅏ', None);
assert_eq!(c, Some('가'));
```

#### `decompose_str(s: &str) -> String`

문자열의 모든 한글 음절을 자모로 분해합니다.

```rust
let result = decompose_str("한글");
assert_eq!(result, "ㅎㅏㄴㄱㅡㄹ");

let result = decompose_str("Hello 한글");
assert_eq!(result, "Hello ㅎㅏㄴㄱㅡㄹ");
```

#### `compose_str(s: &str) -> String`

자모 문자열을 한글 음절로 결합합니다.

```rust
let result = compose_str("ㅎㅏㄴㄱㅡㄹ");
assert_eq!(result, "한글");
```

### 문자 판별

#### `is_hangul(c: char) -> bool`

한글(음절 또는 자모)인지 확인합니다.

```rust
assert!(is_hangul('가'));
assert!(is_hangul('ㄱ'));
assert!(is_hangul('ㅏ'));
assert!(!is_hangul('a'));
```

#### `is_hangul_syllable(c: char) -> bool`

완성형 한글 음절인지 확인합니다.

```rust
assert!(is_hangul_syllable('가'));
assert!(is_hangul_syllable('힣'));
assert!(!is_hangul_syllable('ㄱ'));  // Jamo, not syllable
```

#### `is_jamo(c: char) -> bool`

한글 자모인지 확인합니다.

```rust
assert!(is_jamo('ㄱ'));
assert!(is_jamo('ㅏ'));
assert!(!is_jamo('가'));  // Syllable, not jamo
```

#### `is_choseong(c: char) -> bool`

초성 자모인지 확인합니다.

```rust
assert!(is_choseong('ㄱ'));
assert!(is_choseong('ㅎ'));
assert!(!is_choseong('ㅏ'));  // Jungseong, not choseong
```

#### `is_jungseong(c: char) -> bool`

중성 자모인지 확인합니다.

```rust
assert!(is_jungseong('ㅏ'));
assert!(is_jungseong('ㅣ'));
assert!(!is_jungseong('ㄱ'));
```

#### `has_jongseong(c: char) -> Option<bool>`

한글 음절에 종성이 있는지 확인합니다.

```rust
assert_eq!(has_jongseong('한'), Some(true));
assert_eq!(has_jongseong('하'), Some(false));
assert_eq!(has_jongseong('a'), None);  // Not Hangul
```

### 문자 분류

#### `classify_char(c: char) -> CharType`

문자의 종류를 판별합니다.

```rust
use mecab_ko::CharType;

assert_eq!(classify_char('한'), CharType::HangulSyllable);
assert_eq!(classify_char('ㄱ'), CharType::HangulJamo);
assert_eq!(classify_char('韓'), CharType::Hanja);
assert_eq!(classify_char('ア'), CharType::Katakana);
assert_eq!(classify_char('あ'), CharType::Hiragana);
assert_eq!(classify_char('a'), CharType::Alphabet);
assert_eq!(classify_char('1'), CharType::Digit);
assert_eq!(classify_char(' '), CharType::Whitespace);
assert_eq!(classify_char('.'), CharType::Punctuation);
```

#### CharType 열거형

```rust
pub enum CharType {
    HangulSyllable,  // 한글 음절
    HangulJamo,      // 한글 자모
    Hanja,           // 한자
    Katakana,        // 가타카나
    Hiragana,        // 히라가나
    Alphabet,        // ASCII 알파벳
    Digit,           // 숫자
    Whitespace,      // 공백 문자
    Punctuation,     // 구두점
    Other,           // 기타
}
```

## mecab-ko-dict

사전 관리 기능을 제공합니다.

```rust
use mecab_ko::dict::*;
// or
use mecab_ko_dict::*;
```

### UserDictionary

사용자 정의 사전을 관리합니다.

#### 생성 및 엔트리 추가

```rust
use mecab_ko_dict::UserDictionary;

let mut dict = UserDictionary::new();

// Add entry with surface, POS, cost, and optional reading
dict.add_entry("딥러닝", "NNG", Some(-1000), None);
dict.add_entry("챗GPT", "NNP", Some(-1000), Some("챗지피티".to_string()));
```

#### CSV 파일 로드

```rust
dict.load_from_csv("user.csv")?;
```

CSV 포맷:

```csv
# Comment line (starts with #)
표면형,품사,비용,읽기
딥러닝,NNG,-1000,딥러닝
챗GPT,NNP,-1000,챗지피티
```

#### CSV 문자열 로드

```rust
let csv_content = r#"
딥러닝,NNG,-1000,
머신러닝,NNG,-1000,
"#;
dict.load_from_str(csv_content)?;
```

#### 엔트리 조회

```rust
let entries = dict.lookup("딥러닝");
for entry in entries {
    println!("{}: {} (cost: {})", entry.surface, entry.pos, entry.cost);
}
```

#### 기타 메서드

```rust
// Number of entries
let count = dict.len();

// Check if empty
let is_empty = dict.is_empty();

// Clear all entries
dict.clear();

// Save to CSV file
dict.save_to_csv("output.csv")?;

// Get all entries
let all_entries = dict.entries();
```

### UserDictionaryBuilder

빌더 패턴으로 사용자 사전을 생성합니다.

```rust
use mecab_ko_dict::UserDictionaryBuilder;

let dict = UserDictionaryBuilder::new()
    .default_cost(-500)
    .add("딥러닝", "NNG")
    .add_with_cost("머신러닝", "NNG", -300)
    .add_full("자연어처리", "NNG", -400, Some("자연어처리"))
    .load_csv("extra.csv")?
    .build();
```

### Entry

사전 엔트리를 나타내는 구조체입니다.

```rust
pub struct Entry {
    pub surface: String,   // 표면형
    pub left_id: u16,      // 좌문맥 ID
    pub right_id: u16,     // 우문맥 ID
    pub cost: i16,         // 비용 (낮을수록 우선)
    pub feature: String,   // 품사 정보
}
```

### UserEntry

사용자 사전 엔트리입니다.

```rust
pub struct UserEntry {
    pub surface: String,           // 표면형
    pub left_id: u16,              // 좌문맥 ID
    pub right_id: u16,             // 우문맥 ID
    pub cost: i16,                 // 비용
    pub pos: String,               // 품사 태그
    pub reading: Option<String>,   // 읽기
    pub lemma: Option<String>,     // 원형
}
```

## 에러 처리

### mecab-ko-core 에러

```rust
use mecab_ko::Error;

match tokenizer.new() {
    Ok(t) => { /* use tokenizer */ }
    Err(Error::Dict(e)) => eprintln!("Dictionary error: {}", e),
    Err(Error::Init(msg)) => eprintln!("Init error: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

### mecab-ko-dict 에러

```rust
use mecab_ko_dict::error::DictError;

match dict.load_from_csv("user.csv") {
    Ok(_) => println!("Loaded successfully"),
    Err(DictError::Io(e)) => eprintln!("IO error: {}", e),
    Err(DictError::Format(msg)) => eprintln!("Format error: {}", msg),
    Err(DictError::Version { expected, found }) => {
        eprintln!("Version mismatch: expected {}, found {}", expected, found);
    }
}
```

## 전체 예시

```rust
use mecab_ko::{Tokenizer, hangul};
use mecab_ko_dict::UserDictionary;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create tokenizer
    let tokenizer = Tokenizer::new()?;

    // Basic tokenization
    let text = "오늘 날씨가 좋습니다";
    let tokens = tokenizer.tokenize(text);

    println!("=== Tokens ===");
    for token in &tokens {
        println!("{}\t{}\t[{},{})",
            token.surface, token.pos, token.start, token.end);
    }

    // Extract nouns
    println!("\n=== Nouns ===");
    let nouns = tokenizer.nouns(text);
    println!("{:?}", nouns);

    // Hangul utilities
    println!("\n=== Hangul ===");
    let (cho, jung, jong) = hangul::decompose('한').unwrap();
    println!("'한' = {} + {} + {:?}", cho, jung, jong);

    println!("Decomposed '한글': {}", hangul::decompose_str("한글"));

    // User dictionary
    println!("\n=== User Dictionary ===");
    let mut dict = UserDictionary::new();
    dict.add_entry("딥러닝", "NNG", Some(-1000), None);

    let entries = dict.lookup("딥러닝");
    for entry in entries {
        println!("{}: {} (cost: {})", entry.surface, entry.pos, entry.cost);
    }

    Ok(())
}
```

## 추가 문서

자세한 API 문서는 [docs.rs](https://docs.rs/mecab-ko)에서 확인할 수 있습니다.
