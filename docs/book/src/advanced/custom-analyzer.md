# 커스텀 분석기

MeCab-Ko의 내부 동작을 이해하고 커스텀 분석기를 작성하는 방법을 소개합니다.

## 분석 파이프라인

MeCab-Ko의 형태소 분석은 다음 단계로 진행됩니다:

```
입력 텍스트
    ↓
문자 정규화 (Character Normalization)
    ↓
래티스 구축 (Lattice Construction)
    ↓
최적 경로 탐색 (Viterbi Algorithm)
    ↓
후처리 (Post-processing)
    ↓
출력
```

## 커스텀 Tokenizer

### 기본 구조

```rust
use mecab_ko_core::{Lattice, Tokenizer, Token};

pub struct CustomTokenizer {
    dict: Dictionary,
    config: Config,
}

impl Tokenizer for CustomTokenizer {
    fn tokenize(&self, text: &str) -> Result<Vec<Token>, Error> {
        // 1. 래티스 생성
        let mut lattice = Lattice::new(text);

        // 2. 사전 검색 및 노드 추가
        self.build_lattice(&mut lattice)?;

        // 3. 최적 경로 탐색
        let path = self.viterbi(&lattice)?;

        // 4. 토큰 변환
        Ok(self.path_to_tokens(path))
    }
}
```

### 래티스 빌더

```rust
impl CustomTokenizer {
    fn build_lattice(&self, lattice: &mut Lattice) -> Result<(), Error> {
        let text = lattice.text();
        let len = text.len();

        for pos in 0..len {
            // 사전에서 검색
            let entries = self.dict.lookup(&text[pos..])?;

            for entry in entries {
                // 노드 추가
                lattice.add_node(Node {
                    surface: entry.surface.clone(),
                    pos: entry.pos,
                    left_id: entry.left_id,
                    right_id: entry.right_id,
                    cost: entry.cost,
                    start: pos,
                    length: entry.surface.len(),
                });
            }

            // 미등록어 처리
            if entries.is_empty() {
                self.add_unknown_node(lattice, pos)?;
            }
        }

        Ok(())
    }
}
```

### Viterbi 알고리즘

```rust
impl CustomTokenizer {
    fn viterbi(&self, lattice: &Lattice) -> Result<Vec<usize>, Error> {
        let nodes = lattice.nodes();
        let mut best_cost = vec![i32::MAX; nodes.len()];
        let mut best_path = vec![None; nodes.len()];

        best_cost[0] = 0; // BOS

        for i in 1..nodes.len() {
            let node = &nodes[i];

            for j in 0..i {
                let prev = &nodes[j];

                if prev.end() != node.start {
                    continue;
                }

                // 연접 비용
                let conn_cost = self.dict.connection_cost(
                    prev.right_id,
                    node.left_id,
                );

                let total_cost = best_cost[j]
                    .saturating_add(conn_cost)
                    .saturating_add(node.cost);

                if total_cost < best_cost[i] {
                    best_cost[i] = total_cost;
                    best_path[i] = Some(j);
                }
            }
        }

        // 역추적
        self.backtrack(&best_path)
    }

    fn backtrack(&self, path: &[Option<usize>]) -> Result<Vec<usize>, Error> {
        let mut result = Vec::new();
        let mut current = path.len() - 1; // EOS

        while let Some(prev) = path[current] {
            result.push(prev);
            current = prev;
        }

        result.reverse();
        Ok(result)
    }
}
```

## 커스텀 필터

### 복합명사 분해기

```rust
pub struct CompoundNounDecomposer {
    min_length: usize,
}

impl CompoundNounDecomposer {
    pub fn decompose(&self, tokens: Vec<Token>) -> Vec<Token> {
        let mut result = Vec::new();

        for token in tokens {
            if token.pos == "NNG" && token.surface.chars().count() >= self.min_length {
                // 복합명사 분해
                result.extend(self.split_compound(&token));
            } else {
                result.push(token);
            }
        }

        result
    }

    fn split_compound(&self, token: &Token) -> Vec<Token> {
        // 사전 기반 분해 로직
        let mut parts = Vec::new();
        let surface = &token.surface;

        // 예: "형태소분석기" -> ["형태소", "분석기"]
        // 실제 구현은 사전 기반으로 수행

        parts
    }
}
```

### 품사 필터

```rust
pub struct PosFilter {
    allowed_pos: HashSet<String>,
}

impl PosFilter {
    pub fn filter(&self, tokens: Vec<Token>) -> Vec<Token> {
        tokens
            .into_iter()
            .filter(|token| self.allowed_pos.contains(&token.pos))
            .collect()
    }
}

// 사용
let filter = PosFilter {
    allowed_pos: ["NNG", "NNP", "VV", "VA"].iter().map(|s| s.to_string()).collect(),
};

let filtered = filter.filter(tokens);
```

### 불용어 필터

```rust
pub struct StopwordFilter {
    stopwords: HashSet<String>,
}

impl StopwordFilter {
    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let content = std::fs::read_to_string(path)?;
        let stopwords = content
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();

        Ok(Self { stopwords })
    }

    pub fn filter(&self, tokens: Vec<Token>) -> Vec<Token> {
        tokens
            .into_iter()
            .filter(|token| !self.stopwords.contains(&token.surface))
            .collect()
    }
}
```

## 분석기 조합

### 파이프라인 구성

```rust
pub struct AnalyzerPipeline {
    tokenizer: Box<dyn Tokenizer>,
    filters: Vec<Box<dyn TokenFilter>>,
}

impl AnalyzerPipeline {
    pub fn new(tokenizer: Box<dyn Tokenizer>) -> Self {
        Self {
            tokenizer,
            filters: Vec::new(),
        }
    }

    pub fn add_filter(mut self, filter: Box<dyn TokenFilter>) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn analyze(&self, text: &str) -> Result<Vec<Token>, Error> {
        let mut tokens = self.tokenizer.tokenize(text)?;

        for filter in &self.filters {
            tokens = filter.apply(tokens);
        }

        Ok(tokens)
    }
}

// 사용
let pipeline = AnalyzerPipeline::new(Box::new(MecabTokenizer::new()?))
    .add_filter(Box::new(PosFilter::new(vec!["NNG", "NNP"])))
    .add_filter(Box::new(StopwordFilter::from_file("stopwords.txt")?))
    .add_filter(Box::new(LowercaseFilter::new()));

let tokens = pipeline.analyze("분석할 텍스트")?;
```

## 커스텀 비용 함수

### 띄어쓰기 가중치

```rust
pub struct SpaceCostCalculator {
    base_penalty: i32,
    position_weight: f32,
}

impl SpaceCostCalculator {
    pub fn calculate(&self, node: &Node, context: &Context) -> i32 {
        let mut cost = node.cost;

        // 띄어쓰기 패널티
        if context.has_space_before(node) {
            cost += self.base_penalty;

            // 위치 기반 가중치
            let position_factor = context.position() as f32 / context.total_length() as f32;
            cost += (self.base_penalty as f32 * position_factor * self.position_weight) as i32;
        }

        cost
    }
}
```

### 도메인 특화 비용

```rust
pub struct DomainCostAdjuster {
    domain_terms: HashMap<String, i32>,
}

impl DomainCostAdjuster {
    pub fn adjust_cost(&self, node: &mut Node) {
        if let Some(&adjustment) = self.domain_terms.get(&node.surface) {
            node.cost += adjustment;
        }
    }
}

// 사용
let adjuster = DomainCostAdjuster {
    domain_terms: [
        ("딥러닝".to_string(), -2000),   // 선호
        ("머신러닝".to_string(), -2000),
        ("AI".to_string(), -2000),
    ].iter().cloned().collect(),
};
```

## N-best 분석기

### N-best 경로 탐색

```rust
pub struct NBestAnalyzer {
    tokenizer: MecabTokenizer,
    n: usize,
    theta: f32,
}

impl NBestAnalyzer {
    pub fn analyze(&self, text: &str) -> Result<Vec<Vec<Token>>, Error> {
        let lattice = self.tokenizer.build_lattice(text)?;
        let paths = self.find_nbest_paths(&lattice)?;

        Ok(paths
            .into_iter()
            .map(|path| self.path_to_tokens(path))
            .collect())
    }

    fn find_nbest_paths(&self, lattice: &Lattice) -> Result<Vec<Vec<usize>>, Error> {
        // A* 알고리즘 또는 Forward-DP + Backward-A* 사용
        let mut candidates = Vec::new();
        let best_cost = self.find_best_cost(lattice)?;

        // theta 범위 내의 경로만 수집
        self.collect_paths(lattice, best_cost, &mut candidates)?;

        // 상위 N개 선택
        candidates.sort_by_key(|path| path.cost);
        Ok(candidates.into_iter().take(self.n).map(|p| p.nodes).collect())
    }
}
```

## 실전 예제

### 의료 도메인 분석기

```rust
pub struct MedicalAnalyzer {
    base_tokenizer: MecabTokenizer,
    medical_dict: HashMap<String, MedicalTerm>,
}

impl MedicalAnalyzer {
    pub fn analyze(&self, text: &str) -> Result<Vec<MedicalToken>, Error> {
        // 1. 기본 형태소 분석
        let tokens = self.base_tokenizer.tokenize(text)?;

        // 2. 의료 용어 인식
        let medical_tokens = self.recognize_medical_terms(&tokens)?;

        // 3. 의료 용어 정규화
        let normalized = self.normalize_medical_terms(medical_tokens)?;

        Ok(normalized)
    }

    fn recognize_medical_terms(&self, tokens: &[Token]) -> Result<Vec<MedicalToken>, Error> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            // 연속된 토큰을 결합하여 의료 용어 검색
            let mut matched = false;

            for len in (1..=5).rev() {
                if i + len > tokens.len() {
                    continue;
                }

                let phrase = tokens[i..i+len]
                    .iter()
                    .map(|t| t.surface.as_str())
                    .collect::<Vec<_>>()
                    .join("");

                if let Some(term) = self.medical_dict.get(&phrase) {
                    result.push(MedicalToken {
                        surface: phrase,
                        term_type: term.term_type.clone(),
                        standard_code: term.code.clone(),
                    });
                    i += len;
                    matched = true;
                    break;
                }
            }

            if !matched {
                result.push(MedicalToken::from_token(&tokens[i]));
                i += 1;
            }
        }

        Ok(result)
    }
}
```

### 법률 문서 분석기

```rust
pub struct LegalAnalyzer {
    tokenizer: MecabTokenizer,
    legal_patterns: Vec<Regex>,
}

impl LegalAnalyzer {
    pub fn analyze(&self, text: &str) -> Result<LegalDocument, Error> {
        let tokens = self.tokenizer.tokenize(text)?;

        Ok(LegalDocument {
            tokens,
            articles: self.extract_articles(text)?,
            clauses: self.extract_clauses(text)?,
            references: self.extract_references(text)?,
        })
    }

    fn extract_articles(&self, text: &str) -> Result<Vec<Article>, Error> {
        // "제1조", "제2조" 등 추출
        let pattern = Regex::new(r"제(\d+)조")?;
        let articles = pattern
            .captures_iter(text)
            .map(|cap| Article {
                number: cap[1].parse().unwrap(),
                content: self.extract_article_content(&cap),
            })
            .collect();

        Ok(articles)
    }
}
```

## 성능 최적화

### 캐싱

```rust
use lru::LruCache;

pub struct CachedAnalyzer {
    analyzer: MecabTokenizer,
    cache: Arc<Mutex<LruCache<String, Vec<Token>>>>,
}

impl CachedAnalyzer {
    pub fn analyze(&self, text: &str) -> Result<Vec<Token>, Error> {
        // 캐시 확인
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(text) {
                return Ok(cached.clone());
            }
        }

        // 분석
        let tokens = self.analyzer.tokenize(text)?;

        // 캐시 저장
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(text.to_string(), tokens.clone());
        }

        Ok(tokens)
    }
}
```

## 테스트

### 단위 테스트

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_tokenizer() {
        let tokenizer = CustomTokenizer::new().unwrap();
        let tokens = tokenizer.tokenize("테스트 문장").unwrap();

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].surface, "테스트");
        assert_eq!(tokens[1].surface, "문장");
    }

    #[test]
    fn test_pos_filter() {
        let filter = PosFilter::new(vec!["NNG"]);
        let tokens = vec![
            Token::new("테스트", "NNG"),
            Token::new("하", "VV"),
            Token::new("다", "EF"),
        ];

        let filtered = filter.filter(tokens);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].surface, "테스트");
    }
}
```

## 참고 자료

- [MeCab 논문](http://taku910.github.io/mecab/)
- [Viterbi 알고리즘](https://en.wikipedia.org/wiki/Viterbi_algorithm)
- [Rust API](../api-reference/rust.md)
