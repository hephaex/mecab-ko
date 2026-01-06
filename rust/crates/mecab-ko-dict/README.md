# mecab-ko-dict

한국어 형태소 분석을 위한 사전 관리 라이브러리

## 주요 기능

- **SystemDictionary**: Trie + Matrix + Features를 통합한 시스템 사전
- **UserDictionary**: 사용자 정의 단어 추가 지원
- **Double-Array Trie**: 빠른 형태소 검색
- **연접 비용 행렬**: 형태소 간 연결 비용 계산
- **메모리 맵 지원**: 효율적인 메모리 사용

## 설치

```toml
[dependencies]
mecab-ko-dict = "0.1.0"
```

## 사용 예제

### 시스템 사전 로드

```rust
use mecab_ko_dict::{DictionaryLoader, Dictionary};

// 기본 경로에서 로드
let dict = DictionaryLoader::load_default()?;

// 특정 경로에서 로드
let dict = DictionaryLoader::load_system("/usr/local/lib/mecab/dic/mecab-ko-dic")?;

// 형태소 검색
let entries = dict.lookup("안녕");
for entry in entries {
    println!("{}: {}", entry.surface, entry.feature);
}

// 연접 비용 조회
let cost = dict.get_connection_cost(left_id, right_id);
```

### 사용자 사전

```rust
use mecab_ko_dict::UserDictionaryBuilder;

// 빌더 패턴으로 생성
let user_dict = UserDictionaryBuilder::new()
    .default_cost(-1000)
    .add("딥러닝", "NNG")
    .add_with_cost("머신러닝", "NNG", -800)
    .add_full("챗GPT", "NNP", -1000, Some("챗지피티"))
    .build_with_trie()?;

// 검색
let entries = user_dict.lookup("딥러닝");
```

### CSV 파일에서 로드

```rust
use mecab_ko_dict::UserDictionaryBuilder;

let user_dict = UserDictionaryBuilder::new()
    .load_csv("user_dict.csv")?
    .build_with_trie()?;
```

CSV 포맷:
```csv
# 사용자 사전
표면형,품사,비용,읽기
딥러닝,NNG,-1000,딥러닝
챗GPT,NNP,-1000,챗지피티
```

### 시스템 사전 + 사용자 사전

```rust
use mecab_ko_dict::{DictionaryLoader, UserDictionaryBuilder};

// 시스템 사전 로드
let mut system_dict = DictionaryLoader::load_default()?;

// 사용자 사전 추가
let user_dict = UserDictionaryBuilder::new()
    .add("딥러닝", "NNG")
    .build();

system_dict.set_user_dictionary(user_dict);

// 통합 검색
let entries = system_dict.lookup_combined("딥러닝");
```

## 사전 구조

### 파일 구조

```
mecab-ko-dic/
├── sys.dic          # Trie 데이터
├── matrix.bin       # 연접 비용 행렬 (바이너리)
└── matrix.def       # 연접 비용 행렬 (텍스트)
```

### 환경 변수

- `MECAB_DICDIR`: 사전 디렉토리 경로

기본 경로:
- `/usr/local/lib/mecab/dic/mecab-ko-dic`
- `/usr/lib/mecab/dic/mecab-ko-dic`
- `/opt/mecab/dic/mecab-ko-dic`
- `./dic/mecab-ko-dic`

## API 문서

### Dictionary Trait

```rust
pub trait Dictionary {
    fn lookup(&self, surface: &str) -> Vec<Entry>;
    fn get_connection_cost(&self, left_id: u16, right_id: u16) -> i16;
}
```

### SystemDictionary

```rust
pub struct SystemDictionary { /* ... */ }

impl SystemDictionary {
    pub fn load_default() -> Result<Self>;
    pub fn load<P: AsRef<Path>>(dicdir: P) -> Result<Self>;
    pub fn with_user_dictionary(self, user_dict: UserDictionary) -> Self;
    pub fn lookup_combined(&self, surface: &str) -> Vec<Entry>;
    pub fn common_prefix_search(&self, text: &str) -> Vec<(&DictEntry, usize)>;
}
```

### UserDictionary

```rust
pub struct UserDictionary { /* ... */ }

impl UserDictionary {
    pub fn new() -> Self;
    pub fn add_entry(&mut self, surface: &str, pos: &str, cost: Option<i16>, reading: Option<String>) -> &mut Self;
    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<&mut Self>;
    pub fn lookup(&self, surface: &str) -> Vec<&UserEntry>;
    pub fn build_trie(&mut self) -> Result<&[u8]>;
}
```

## 성능

- **Trie 검색**: O(m) (m = 검색 키 길이)
- **연접 비용 조회**: O(1)
- **메모리 사용**: 사전 크기에 비례 (mmap 사용 시 효율적)

## 테스트

```bash
# 단위 테스트
cargo test -p mecab-ko-dict

# 통합 테스트
cargo test -p mecab-ko-dict --test integration_test

# 예제 실행
cargo run -p mecab-ko-dict --example dictionary_usage
```

## 라이선스

MIT OR Apache-2.0

## 기여

이슈와 풀 리퀘스트는 언제나 환영합니다!
