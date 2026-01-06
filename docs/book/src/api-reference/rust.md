# Rust API

MeCab-Ko는 Rust로 작성되어 안전하고 효율적인 API를 제공합니다.

## 빠른 시작

```rust
use mecab_ko::{Tagger, TaggerConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 기본 설정으로 Tagger 생성
    let tagger = Tagger::new(TaggerConfig::default())?;

    // 문장 분석
    let result = tagger.parse("안녕하세요")?;

    for node in result.iter() {
        println!("{}\t{}", node.surface, node.feature);
    }

    Ok(())
}
```

## 주요 타입

### `Tagger`

형태소 분석의 핵심 구조체입니다.

```rust
pub struct Tagger {
    // 내부 필드는 비공개
}

impl Tagger {
    /// 새로운 Tagger를 생성합니다.
    pub fn new(config: TaggerConfig) -> Result<Self, Error>;

    /// 문자열을 분석합니다.
    pub fn parse(&self, text: &str) -> Result<ParseResult, Error>;

    /// 여러 문장을 한 번에 분석합니다.
    pub fn parse_batch(&self, texts: &[&str]) -> Result<Vec<ParseResult>, Error>;

    /// 사전을 동적으로 추가합니다.
    pub fn add_user_dict(&mut self, dict_path: &Path) -> Result<(), Error>;
}
```

### `TaggerConfig`

Tagger의 동작을 제어하는 설정 구조체입니다.

```rust
#[derive(Debug, Clone)]
pub struct TaggerConfig {
    /// 사전 디렉토리 경로
    pub dict_dir: Option<PathBuf>,

    /// 사용자 사전 경로
    pub user_dict: Option<PathBuf>,

    /// 출력 포맷
    pub output_format: OutputFormat,

    /// 띄어쓰기 패널티
    pub space_penalty: i32,

    /// 최대 그룹화 크기
    pub max_grouping_size: usize,

    /// 부분 처리 활성화
    pub partial: bool,

    /// 전부 출력 (N-best)
    pub all_morphs: bool,

    /// N-best 개수
    pub nbest: usize,

    /// theta (N-best용)
    pub theta: f32,
}

impl Default for TaggerConfig {
    fn default() -> Self {
        Self {
            dict_dir: None,
            user_dict: None,
            output_format: OutputFormat::default(),
            space_penalty: -1000,
            max_grouping_size: 24,
            partial: false,
            all_morphs: false,
            nbest: 1,
            theta: 0.75,
        }
    }
}
```

### `ParseResult`

분석 결과를 담는 구조체입니다.

```rust
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// 원본 텍스트
    pub text: String,

    /// 분석된 노드 리스트
    pub nodes: Vec<Node>,
}

impl ParseResult {
    /// 노드 반복자를 반환합니다.
    pub fn iter(&self) -> impl Iterator<Item = &Node>;

    /// 특정 포맷으로 문자열 출력
    pub fn format(&self, format: OutputFormat) -> String;

    /// JSON 직렬화
    pub fn to_json(&self) -> serde_json::Value;
}
```

### `Node`

분석된 형태소 노드입니다.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// 표면형 (실제 텍스트)
    pub surface: String,

    /// 품사 및 의미 정보
    pub feature: String,

    /// 시작 위치 (바이트 단위)
    pub start: usize,

    /// 길이 (바이트 단위)
    pub length: usize,

    /// 품사 ID
    pub pos_id: u16,

    /// 비용
    pub cost: i32,
}

impl Node {
    /// 품사 태그만 추출
    pub fn pos(&self) -> &str;

    /// 기본형 (원형) 추출
    pub fn base_form(&self) -> Option<&str>;

    /// 읽기 정보
    pub fn reading(&self) -> Option<&str>;
}
```

### `OutputFormat`

출력 포맷을 지정합니다.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// MeCab 기본 포맷
    MeCab,

    /// 공백으로 구분된 형태소만
    Wakati,

    /// JSON 포맷
    Json,

    /// CSV 포맷
    Csv,

    /// 품사 태그만
    PosOnly,

    /// 커스텀 포맷
    Custom(String),
}
```

## 사용 예제

### 기본 사용

```rust
use mecab_ko::{Tagger, TaggerConfig};

let tagger = Tagger::new(TaggerConfig::default())?;
let result = tagger.parse("형태소 분석을 시작합니다.")?;

for node in result.iter() {
    println!("표면형: {}", node.surface);
    println!("품사: {}", node.pos());
    if let Some(base) = node.base_form() {
        println!("기본형: {}", base);
    }
    println!("---");
}
```

### 사용자 사전 추가

```rust
use mecab_ko::{Tagger, TaggerConfig};
use std::path::Path;

let mut config = TaggerConfig::default();
config.user_dict = Some(Path::new("user.csv").to_path_buf());

let tagger = Tagger::new(config)?;
let result = tagger.parse("딥러닝과 머신러닝")?;
```

### N-best 분석

```rust
let mut config = TaggerConfig::default();
config.nbest = 3;
config.theta = 0.75;

let tagger = Tagger::new(config)?;
let results = tagger.parse_nbest("아버지가방에들어가신다")?;

for (i, result) in results.iter().enumerate() {
    println!("=== 후보 {} ===", i + 1);
    println!("{}", result.format(OutputFormat::MeCab));
}
```

### 병렬 처리

```rust
use rayon::prelude::*;

let tagger = Tagger::new(TaggerConfig::default())?;
let sentences = vec![
    "첫 번째 문장",
    "두 번째 문장",
    "세 번째 문장",
];

let results: Vec<_> = sentences
    .par_iter()
    .map(|&text| tagger.parse(text))
    .collect::<Result<_, _>>()?;
```

### 커스텀 출력 포맷

```rust
let mut config = TaggerConfig::default();
config.output_format = OutputFormat::Custom("%m\t%f[0]\t%f[6]\n".to_string());

let tagger = Tagger::new(config)?;
let result = tagger.parse("분석할 텍스트")?;
println!("{}", result.format(config.output_format));
```

### 스트림 처리

```rust
use std::io::{BufRead, BufReader};
use std::fs::File;

let tagger = Tagger::new(TaggerConfig::default())?;
let file = File::open("large_text.txt")?;
let reader = BufReader::new(file);

for line in reader.lines() {
    let line = line?;
    if !line.trim().is_empty() {
        let result = tagger.parse(&line)?;
        // 결과 처리
    }
}
```

## Error 처리

MeCab-Ko는 `Result` 타입을 사용하여 에러를 처리합니다.

```rust
use mecab_ko::{Error, ErrorKind};

match tagger.parse(text) {
    Ok(result) => {
        // 성공 처리
    }
    Err(e) => match e.kind() {
        ErrorKind::DictNotFound => {
            eprintln!("사전을 찾을 수 없습니다: {}", e);
        }
        ErrorKind::ParseError => {
            eprintln!("파싱 오류: {}", e);
        }
        ErrorKind::InvalidConfig => {
            eprintln!("잘못된 설정: {}", e);
        }
        _ => {
            eprintln!("기타 오류: {}", e);
        }
    }
}
```

## Feature Flags

Cargo.toml에서 다음 feature를 사용할 수 있습니다:

```toml
[dependencies]
mecab-ko = { version = "0.1", features = ["builder", "python", "wasm"] }
```

| Feature | 설명 |
|---------|------|
| `builder` | 사전 빌더 기능 포함 |
| `python` | Python 바인딩 포함 |
| `wasm` | WASM 지원 포함 |
| `rayon` | 병렬 처리 지원 |
| `serde` | JSON 직렬화 지원 |

## 성능 최적화

### Tagger 재사용

Tagger 생성은 비용이 큰 작업입니다. 가능하면 재사용하세요.

```rust
// Bad
for text in texts {
    let tagger = Tagger::new(config.clone())?; // 매번 생성
    let result = tagger.parse(text)?;
}

// Good
let tagger = Tagger::new(config)?; // 한 번만 생성
for text in texts {
    let result = tagger.parse(text)?;
}
```

### 배치 처리

여러 문장을 한 번에 처리하면 더 효율적입니다.

```rust
let results = tagger.parse_batch(&texts)?; // 배치 처리
```

### Arc 공유

멀티스레드 환경에서는 Arc로 공유하세요.

```rust
use std::sync::Arc;
use rayon::prelude::*;

let tagger = Arc::new(Tagger::new(config)?);

texts.par_iter()
    .map(|text| {
        let tagger = Arc::clone(&tagger);
        tagger.parse(text)
    })
    .collect::<Result<Vec<_>, _>>()?;
```

## 전체 API 문서

상세한 API 문서는 [rustdoc](../api/mecab_ko/index.html)을 참조하세요.

## 관련 문서

- [Python API](./python.md)
- [Node.js API](./nodejs.md)
- [WASM API](./wasm.md)
- [CLI 사용법](../cli-usage.md)
