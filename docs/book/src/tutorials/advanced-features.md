# 고급 기능 튜토리얼

MeCab-Ko의 고급 기능을 활용하는 방법을 알아봅니다.

## 목차

1. [N-best 분석](#n-best-분석)
2. [복합명사 분해](#복합명사-분해)
3. [스트리밍 처리](#스트리밍-처리)
4. [정확도 평가](#정확도-평가)
5. [사전 품질 검증](#사전-품질-검증)

## N-best 분석

형태소 분석에서 최적 경로 외에 여러 후보를 얻을 수 있습니다.

### 기본 사용법

```rust
use mecab_ko::{Tokenizer, TokenizerConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = TokenizerConfig::default();
    config.nbest = 3;  // 상위 3개 결과

    let tokenizer = Tokenizer::with_config(config)?;
    let text = "아버지가방에들어가신다";

    let results = tokenizer.tokenize_nbest(text)?;

    for (i, tokens) in results.iter().enumerate() {
        println!("===== 후보 {} =====", i + 1);
        for token in tokens {
            println!("  {:<10} {}", token.surface, token.pos);
        }
        println!();
    }

    Ok(())
}
```

**출력:**
```
===== 후보 1 =====
  아버지       NNG
  가           JKS
  방           NNG
  에           JKB
  들어가       VV
  시           EP
  ㄴ다         EF

===== 후보 2 =====
  아버지       NNG
  가방         NNG
  에           JKB
  들어가       VV
  시           EP
  ㄴ다         EF

===== 후보 3 =====
  아버지       NNG
  가           JKS
  방           NNG
  에           JKB
  들어가시     VV+EP
  ㄴ다         EF
```

### 비용 기반 필터링

```rust
// theta 값으로 후보 범위 조정
// 낮은 theta = 더 다양한 후보
config.theta = 0.5;  // 기본값: 0.75

let tokenizer = Tokenizer::with_config(config)?;
```

## 복합명사 분해

### DecompoundMode 설정

```rust
use mecab_ko_core::DecompoundMode;

// None: 분해하지 않음
// Discard: 원본 제거, 분해 결과만
// Mixed: 원본 유지, 분해 결과도 포함

let mut config = TokenizerConfig::default();
config.decompound_mode = DecompoundMode::Mixed;
```

### 분해 예제

```rust
use mecab_ko::{Tokenizer, TokenizerConfig};
use mecab_ko_core::DecompoundMode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = "국립국어원에서 발표했습니다";

    // 분해 없음
    let config_none = TokenizerConfig {
        decompound_mode: DecompoundMode::None,
        ..Default::default()
    };
    let tokenizer_none = Tokenizer::with_config(config_none)?;
    println!("=== 분해 없음 ===");
    for t in tokenizer_none.tokenize(text) {
        println!("  {}", t.surface);
    }

    // Mixed 모드 (원본 + 분해)
    let config_mixed = TokenizerConfig {
        decompound_mode: DecompoundMode::Mixed,
        ..Default::default()
    };
    let tokenizer_mixed = Tokenizer::with_config(config_mixed)?;
    println!("\n=== Mixed 모드 ===");
    for t in tokenizer_mixed.tokenize(text) {
        println!("  {} (offset: {})", t.surface, t.start);
    }

    Ok(())
}
```

**출력:**
```
=== 분해 없음 ===
  국립국어원
  에서
  발표
  했
  습니다

=== Mixed 모드 ===
  국립국어원 (offset: 0)
  국립 (offset: 0)
  국어원 (offset: 6)
  에서 (offset: 15)
  발표 (offset: 21)
  했 (offset: 27)
  습니다 (offset: 30)
```

### 접두사/접미사 처리

v0.2.0에서 개선된 접두사/접미사 자동 감지:

```rust
// 접미사: 들, 님, 분, 꾼 등
// 접두사: 신, 구, 총, 부, 전, 후 등

let text = "선생님들께서 신기술을 소개합니다";
let tokens = tokenizer.tokenize(text);

for token in &tokens {
    if token.pos == "XSN" {
        println!("접미사 발견: {}", token.surface);
    }
    if token.pos == "XPN" {
        println!("접두사 발견: {}", token.surface);
    }
}
```

## 스트리밍 처리

대용량 텍스트를 메모리 효율적으로 처리합니다.

### StreamingTokenizer

```rust
use mecab_ko::StreamingTokenizer;
use std::io::{BufRead, BufReader};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = StreamingTokenizer::new()?;

    // 파일 스트리밍 처리
    let file = File::open("large_corpus.txt")?;
    let reader = BufReader::new(file);

    let mut total_tokens = 0;
    let mut total_nouns = 0;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let tokens = tokenizer.tokenize(&line);
        total_tokens += tokens.len();
        total_nouns += tokens.iter().filter(|t| t.pos.starts_with("NN")).count();
    }

    println!("총 토큰: {}", total_tokens);
    println!("총 명사: {}", total_nouns);

    Ok(())
}
```

### 배치 처리

```rust
use mecab_ko::Tokenizer;
use rayon::prelude::*;

fn process_batch(texts: &[String]) -> Vec<Vec<String>> {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    texts
        .par_iter()  // 병렬 처리
        .map(|text| {
            tokenizer
                .tokenize(text)
                .iter()
                .map(|t| t.surface.clone())
                .collect()
        })
        .collect()
}

fn main() {
    let texts = vec![
        "첫 번째 문장입니다.".to_string(),
        "두 번째 문장입니다.".to_string(),
        "세 번째 문장입니다.".to_string(),
    ];

    let results = process_batch(&texts);

    for (text, tokens) in texts.iter().zip(results.iter()) {
        println!("{} -> {:?}", text, tokens);
    }
}
```

## 정확도 평가

### 평가 데이터 형식 (TSV)

`eval_data.tsv`:
```
안녕하세요	안녕/NNG 하/XSV 세요/EP+EF
오늘 날씨가 좋습니다	오늘/NNG 날씨/NNG 가/JKS 좋/VA 습니다/EF
```

### CLI로 평가

```bash
# 정확도 측정
mecab-ko evaluate --data eval_data.tsv

# 상세 출력
mecab-ko evaluate --data eval_data.tsv --verbose

# 품사별 정확도
mecab-ko evaluate --data eval_data.tsv --pos-detail
```

### 프로그래밍 방식 평가

```rust
use mecab_ko_core::evaluate::{Evaluator, EvalResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let evaluator = Evaluator::new()?;

    let result = evaluator.evaluate_file("eval_data.tsv")?;

    println!("=== 평가 결과 ===");
    println!("토큰 정확도: {:.2}%", result.token_accuracy * 100.0);
    println!("문장 정확도: {:.2}%", result.sentence_accuracy * 100.0);
    println!("품사 정확도: {:.2}%", result.pos_accuracy * 100.0);
    println!();
    println!("Precision: {:.2}%", result.precision * 100.0);
    println!("Recall: {:.2}%", result.recall * 100.0);
    println!("F1 Score: {:.2}%", result.f1_score * 100.0);

    // 품사별 정확도
    println!("\n=== 품사별 정확도 ===");
    for (pos, accuracy) in &result.pos_stats {
        println!("  {}: {:.2}%", pos, accuracy * 100.0);
    }

    Ok(())
}
```

**출력:**
```
=== 평가 결과 ===
토큰 정확도: 96.45%
문장 정확도: 89.23%
품사 정확도: 94.87%

Precision: 95.12%
Recall: 94.56%
F1 Score: 94.84%

=== 품사별 정확도 ===
  NNG: 97.23%
  NNP: 92.45%
  VV: 95.67%
  VA: 96.12%
  ...
```

## 사전 품질 검증

v0.2.0에서 추가된 사전 분석 기능을 사용합니다.

### CLI로 검증

```bash
# 기본 검증
mecab-ko-dict-validator validate /path/to/dict

# 통계 분석 포함
mecab-ko-dict-validator validate /path/to/dict --analyze

# 자동 수정 제안
mecab-ko-dict-validator validate /path/to/dict --fix

# JSON 출력
mecab-ko-dict-validator validate /path/to/dict --output-format json
```

### 품사 분포 분석

```rust
use mecab_ko_dict_validator::{Analyzer, AnalysisReport};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let analyzer = Analyzer::new("/path/to/dict")?;
    let report = analyzer.analyze()?;

    println!("=== 품사 분포 ===");
    for stat in &report.pos_distribution.top_tags {
        println!(
            "  {}: {} ({:.1}%)",
            stat.tag,
            stat.count,
            stat.percentage
        );
    }

    println!("\n=== 비용 분포 ===");
    println!("  평균: {}", report.cost_distribution.mean);
    println!("  중앙값: {}", report.cost_distribution.median);
    println!("  표준편차: {}", report.cost_distribution.std_dev);

    // 이상치 탐지
    if !report.anomalies.is_empty() {
        println!("\n=== 이상치 ({}) ===", report.anomalies.len());
        for anomaly in report.anomalies.iter().take(5) {
            println!("  {} (비용: {})", anomaly.surface, anomaly.cost);
        }
    }

    Ok(())
}
```

### 일관성 검사

```rust
use mecab_ko_dict_validator::{Validator, ValidationResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let validator = Validator::new()?;
    let result = validator.validate("/path/to/dict")?;

    println!("=== 검증 결과 ===");
    println!("총 항목: {}", result.total_entries);
    println!("오류: {}", result.errors.len());
    println!("경고: {}", result.warnings.len());

    // 중복 검사
    if !result.duplicates.is_empty() {
        println!("\n중복 항목:");
        for dup in &result.duplicates {
            println!("  {} ({}회)", dup.surface, dup.count);
        }
    }

    // 품사 태그 검증
    for invalid in &result.invalid_pos_tags {
        println!("잘못된 품사: {} @ line {}", invalid.tag, invalid.line);
    }

    Ok(())
}
```

## Unknown 단어 패턴

v0.2.0에서 개선된 미등록어 처리:

```rust
use mecab_ko_core::unknown::{WordPattern, detect_pattern, estimate_pos};

fn main() {
    let words = vec![
        "ChatGPT",      // CamelCase -> NNP
        "iPhone15",     // NumberUnit
        "삼성Galaxy",   // HangulAlphaMix -> NNG
        "김철수",       // ProperNoun -> NNP
    ];

    for word in words {
        let pattern = detect_pattern(word);
        let pos = estimate_pos(&pattern);
        println!("{}: {:?} -> {}", word, pattern, pos);
    }
}
```

**출력:**
```
ChatGPT: CamelCase -> NNP
iPhone15: NumberUnit -> NNG
삼성Galaxy: HangulAlphaMix -> NNG
김철수: ProperNoun -> NNP
```

## 실습: 문서 분석 파이프라인

```rust
use mecab_ko::{Tokenizer, TokenizerConfig};
use mecab_ko_core::DecompoundMode;
use std::collections::HashMap;

struct DocumentAnalyzer {
    tokenizer: Tokenizer,
}

impl DocumentAnalyzer {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = TokenizerConfig {
            decompound_mode: DecompoundMode::Mixed,
            ..Default::default()
        };
        let tokenizer = Tokenizer::with_config(config)?;
        Ok(Self { tokenizer })
    }

    fn analyze(&self, text: &str) -> DocumentStats {
        let tokens = self.tokenizer.tokenize(text);

        let mut pos_count: HashMap<String, usize> = HashMap::new();
        let mut noun_freq: HashMap<String, usize> = HashMap::new();

        for token in &tokens {
            // 품사 카운트
            let pos_category = &token.pos[..token.pos.len().min(2)];
            *pos_count.entry(pos_category.to_string()).or_insert(0) += 1;

            // 명사 빈도
            if token.pos.starts_with("NN") && token.surface.chars().count() > 1 {
                *noun_freq.entry(token.surface.clone()).or_insert(0) += 1;
            }
        }

        DocumentStats {
            total_tokens: tokens.len(),
            pos_distribution: pos_count,
            top_nouns: Self::top_n(&noun_freq, 10),
        }
    }

    fn top_n(freq: &HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
        let mut items: Vec<_> = freq.iter().map(|(k, v)| (k.clone(), *v)).collect();
        items.sort_by(|a, b| b.1.cmp(&a.1));
        items.truncate(n);
        items
    }
}

struct DocumentStats {
    total_tokens: usize,
    pos_distribution: HashMap<String, usize>,
    top_nouns: Vec<(String, usize)>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let analyzer = DocumentAnalyzer::new()?;

    let document = r#"
        인공지능 기술의 발전으로 자연어 처리 분야가 급격히 성장하고 있습니다.
        특히 대규모 언어 모델은 다양한 자연어 처리 작업에서 뛰어난 성능을 보이고 있습니다.
        한국어 형태소 분석기는 이러한 자연어 처리 파이프라인의 핵심 구성 요소입니다.
    "#;

    let stats = analyzer.analyze(document);

    println!("=== 문서 통계 ===");
    println!("총 토큰: {}", stats.total_tokens);

    println!("\n품사 분포:");
    for (pos, count) in &stats.pos_distribution {
        println!("  {}: {}", pos, count);
    }

    println!("\n상위 명사:");
    for (noun, count) in &stats.top_nouns {
        println!("  {}: {}회", noun, count);
    }

    Ok(())
}
```

## 다음 단계

- [웹 서버 통합](web-integration.md): REST API 구축
- [성능 튜닝](../advanced/performance-tuning.md): 대용량 처리 최적화
- [커스텀 분석기](../advanced/custom-analyzer.md): 도메인 특화 분석기 구축
