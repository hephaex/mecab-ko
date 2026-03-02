# 기본 사용법 튜토리얼

이 튜토리얼에서는 MeCab-Ko의 기본적인 사용법을 단계별로 배웁니다.

## 목차

1. [환경 설정](#환경-설정)
2. [첫 번째 분석](#첫-번째-분석)
3. [토큰 정보 활용](#토큰-정보-활용)
4. [다양한 분석 메서드](#다양한-분석-메서드)
5. [사용자 사전 적용](#사용자-사전-적용)

## 환경 설정

### Rust 프로젝트 생성

```bash
# 새 프로젝트 생성
cargo new mecab-tutorial
cd mecab-tutorial

# Cargo.toml에 의존성 추가
cat >> Cargo.toml << 'EOF'
[dependencies]
mecab-ko = "0.2"
EOF
```

### Python 환경 (선택사항)

```bash
# pip으로 설치
pip install mecab-ko

# 또는 Poetry 사용
poetry add mecab-ko
```

## 첫 번째 분석

### Rust 예제

```rust
use mecab_ko::Tokenizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 토크나이저 생성
    let tokenizer = Tokenizer::new()?;

    // 텍스트 분석
    let text = "안녕하세요, 오늘 날씨가 좋네요!";
    let tokens = tokenizer.tokenize(text);

    // 결과 출력
    println!("입력: {}", text);
    println!("\n분석 결과:");
    println!("{:-<50}", "");

    for token in &tokens {
        println!("{:<10} {:>10}", token.surface, token.pos);
    }

    Ok(())
}
```

**출력:**
```
입력: 안녕하세요, 오늘 날씨가 좋네요!

분석 결과:
--------------------------------------------------
안녕             NNG
하               XSV
세요           EP+EF
,                 SC
오늘             NNG
날씨             NNG
가               JKS
좋               VA
네요           EF+EF
!                 SF
```

### Python 예제

```python
from mecab_ko import Mecab

# 분석기 생성
mecab = Mecab()

# 텍스트 분석
text = "안녕하세요, 오늘 날씨가 좋네요!"
result = mecab.pos(text)

print(f"입력: {text}")
print("\n분석 결과:")
for surface, pos in result:
    print(f"  {surface:<10} {pos:>10}")
```

## 토큰 정보 활용

### 토큰 구조체

각 토큰은 다음 정보를 포함합니다:

| 필드 | 타입 | 설명 |
|------|------|------|
| `surface` | String | 표면형 (실제 텍스트) |
| `pos` | String | 품사 태그 |
| `start` | usize | 시작 위치 (바이트) |
| `end` | usize | 끝 위치 (바이트) |
| `reading` | Option | 읽기 정보 |
| `lemma` | Option | 기본형 (원형) |

### 위치 정보 활용

```rust
use mecab_ko::Tokenizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = Tokenizer::new()?;
    let text = "한국어 형태소 분석";
    let tokens = tokenizer.tokenize(text);

    println!("원문: {}", text);
    println!("\n토큰별 위치 정보:");

    for token in &tokens {
        // 원문에서 해당 토큰 추출
        let extracted = &text[token.start..token.end];
        println!(
            "  '{}' @ [{}, {}) = '{}'",
            token.surface, token.start, token.end, extracted
        );
    }

    Ok(())
}
```

**출력:**
```
원문: 한국어 형태소 분석

토큰별 위치 정보:
  '한국어' @ [0, 9) = '한국어'
  '형태소' @ [10, 19) = '형태소'
  '분석' @ [20, 26) = '분석'
```

### 품사 필터링

```rust
use mecab_ko::Tokenizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = Tokenizer::new()?;
    let text = "오늘 날씨가 정말 좋습니다";
    let tokens = tokenizer.tokenize(text);

    // 명사만 추출 (NNG, NNP, NNB 등)
    let nouns: Vec<_> = tokens
        .iter()
        .filter(|t| t.pos.starts_with("NN"))
        .map(|t| &t.surface)
        .collect();

    println!("명사: {:?}", nouns);
    // 출력: 명사: ["오늘", "날씨"]

    // 동사/형용사 추출 (VV, VA)
    let verbs: Vec<_> = tokens
        .iter()
        .filter(|t| t.pos.starts_with("V"))
        .map(|t| &t.surface)
        .collect();

    println!("용언: {:?}", verbs);
    // 출력: 용언: ["좋"]

    Ok(())
}
```

## 다양한 분석 메서드

### morphs() - 형태소 추출

```rust
let morphs = tokenizer.morphs("오늘 날씨가 좋습니다");
println!("{:?}", morphs);
// ["오늘", "날씨", "가", "좋", "습니다"]
```

### nouns() - 명사 추출

```rust
let nouns = tokenizer.nouns("자연어 처리는 인공지능의 핵심 기술입니다");
println!("{:?}", nouns);
// ["자연어", "처리", "인공지능", "핵심", "기술"]
```

### pos() - 품사 태깅

```rust
let pos_tagged = tokenizer.pos("형태소 분석");
for (surface, pos) in pos_tagged {
    println!("{}: {}", surface, pos);
}
// 형태소: NNG
// 분석: NNG
```

### wakati() - 띄어쓰기 분리

```rust
let wakati = tokenizer.wakati("아버지가방에들어가신다");
println!("{}", wakati.join(" "));
// "아버지 가 방 에 들어가 시 ㄴ다"
```

## 사용자 사전 적용

### CSV 사전 생성

`user_dict.csv` 파일을 생성합니다:

```csv
# 신조어 사전
# 형식: 표면형,품사,비용,읽기

# IT 용어
딥러닝,NNG,-2000,딥러닝
머신러닝,NNG,-2000,머신러닝
챗GPT,NNP,-2000,챗지피티
LLM,NNG,-2000,엘엘엠

# 브랜드명
삼성전자,NNP,-2000,삼성전자
네이버,NNP,-2000,네이버

# 신조어
갓생,NNG,-1500,갓생
소확행,NNG,-1500,소확행
```

### 사용자 사전 로드

```rust
use mecab_ko::Tokenizer;
use mecab_ko_dict::UserDictionary;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 사용자 사전 로드
    let mut user_dict = UserDictionary::new();
    user_dict.load_from_csv("user_dict.csv")?;

    println!("사용자 사전 로드: {} 항목", user_dict.len());

    // 토크나이저 생성 (사용자 사전 적용)
    let tokenizer = Tokenizer::with_user_dict(user_dict)?;

    // 분석 테스트
    let text = "딥러닝과 머신러닝으로 챗GPT를 만들었습니다";
    let tokens = tokenizer.tokenize(text);

    for token in &tokens {
        println!("{:<12} {}", token.surface, token.pos);
    }

    Ok(())
}
```

### CLI에서 사용자 사전 사용

```bash
# 사용자 사전과 함께 분석
mecab-ko --user-dic user_dict.csv "딥러닝 기술이 발전하고 있습니다"

# JSON 출력과 함께
mecab-ko --user-dic user_dict.csv -O json "챗GPT가 인기입니다"
```

## 실습 예제

### 키워드 추출기

```rust
use mecab_ko::Tokenizer;
use std::collections::HashMap;

fn extract_keywords(text: &str, top_n: usize) -> Vec<(String, usize)> {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let tokens = tokenizer.tokenize(text);

    // 명사만 카운트
    let mut counter: HashMap<String, usize> = HashMap::new();
    for token in tokens {
        if token.pos.starts_with("NN") && token.surface.chars().count() > 1 {
            *counter.entry(token.surface).or_insert(0) += 1;
        }
    }

    // 빈도순 정렬
    let mut keywords: Vec<_> = counter.into_iter().collect();
    keywords.sort_by(|a, b| b.1.cmp(&a.1));
    keywords.truncate(top_n);

    keywords
}

fn main() {
    let article = r#"
        인공지능 기술이 빠르게 발전하고 있습니다.
        특히 자연어 처리 분야에서 대규모 언어 모델의 발전이 눈부십니다.
        인공지능은 의료, 금융, 교육 등 다양한 분야에서 활용되고 있으며,
        앞으로 더 많은 분야에서 인공지능 기술이 적용될 것으로 예상됩니다.
    "#;

    let keywords = extract_keywords(article, 5);

    println!("상위 키워드:");
    for (word, count) in keywords {
        println!("  {} ({}회)", word, count);
    }
}
```

**출력:**
```
상위 키워드:
  인공지능 (3회)
  기술 (2회)
  분야 (3회)
  발전 (2회)
  언어 (1회)
```

## 다음 단계

- [고급 기능](advanced-features.md): N-best 분석, 복합명사 분해 등
- [웹 서버 통합](web-integration.md): Actix-web, Axum과 통합
- [성능 튜닝](../advanced/performance-tuning.md): 대용량 처리 최적화

## 참고 자료

- [API 레퍼런스](../api-reference.md)
- [품사 태그](../reference/pos-tags.md)
- [사용자 사전 가이드](../user-dictionary.md)
