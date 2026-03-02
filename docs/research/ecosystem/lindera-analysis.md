# Lindera 소스 코드 심층 분석

**날짜**: 2026-02-23
**카테고리**: ecosystem

## 요약 (3줄)
1. Lindera는 3층 구조: 빌드타임(CSV→바이너리) + 로드타임(바이너리→메모리) + 런타임(Viterbi 래티스 검색)
2. PrefixDictionary의 DA Aho-Corasick이 핵심 - 전체 텍스트를 한 번에 스캔하여 모든 매칭 수집
3. rkyv 제로카피 + Data enum(Static/Vec/Mmap) + 전치 연접 행렬이 성능 핵심 기법

## 1. 핵심 사전 구조체

### Dictionary (최상위)
```rust
pub struct Dictionary {
    pub prefix_dictionary: PrefixDictionary,      // DA Aho-Corasick + 형태소 데이터
    pub connection_cost_matrix: ConnectionCostMatrix, // 연접 비용 행렬
    pub character_definition: CharacterDefinition,    // 문자 카테고리 (char.def)
    pub unknown_dictionary: UnknownDictionary,        // 미등록어 템플릿 (unk.def)
    pub metadata: Metadata,                           // 인코딩, 스키마, 압축 설정
}
```

### PrefixDictionary
```rust
pub struct PrefixDictionary {
    pub da: DoubleArrayAhoCorasick<u32>,  // daachorse 기반 DA Aho-Corasick
    pub vals_data: Data,       // WordEntry 직렬화 (10바이트/엔트리)
    pub words_idx_data: Data,  // 단어 데이터 인덱스 (u32 오프셋)
    pub words_data: Data,      // 단어 상세 문자열 (NUL 구분)
    pub is_system: bool,
}
```

**DA 값 인코딩**: `상위 27비트 = vals_data 오프셋`, `하위 5비트 = 엔트리 수`

### WordEntry (10바이트 고정)
```rust
pub struct WordEntry {
    pub word_id: WordId,    // 4바이트 (u32 id + LexType)
    pub word_cost: i16,     // 2바이트
    pub left_id: u16,       // 2바이트
    pub right_id: u16,      // 2바이트
}
```

### ConnectionCostMatrix
```rust
pub struct ConnectionCostMatrix {
    pub costs_data: Vec<i16>,   // 전치된 2D 행렬 (캐시 효율)
    pub backward_size: u32,
    pub forward_size: u32,
}
// 인덱싱: costs_data[forward_id + backward_id * forward_size]
```
**핵심**: 행렬이 **전치**되어 저장됨 → Viterbi 전방 패스에서 캐시 지역성 최적화

### CharacterDefinition
```rust
pub struct CharacterDefinition {
    pub category_definitions: Vec<CategoryData>,  // invoke/group/length
    pub category_names: Vec<String>,
    pub mapping: LookupTable<CategoryId>,         // 유니코드 → 카테고리 매핑
}

pub struct CategoryData {
    pub invoke: bool,   // 항상 미등록어 처리할지
    pub group: bool,    // 같은 카테고리 연속 문자 그룹핑
    pub length: u32,    // 최대 그룹핑 길이
}
```
`LookupTable`은 정렬된 경계값 배열 + 이진 탐색으로 O(log n) 매핑

### UnknownDictionary
```rust
pub struct UnknownDictionary {
    pub category_references: Vec<Vec<u32>>,  // category_id → [word_id, ...]
    pub costs: Vec<WordEntry>,               // word_id → WordEntry
    pub words_idx_data: Vec<u32>,            // word_id → words_data 오프셋
    pub words_data: Vec<u8>,                 // NUL 구분 상세 문자열
}
```

## 2. Viterbi / 래티스 구조

### Edge (래티스 노드)
```rust
pub struct Edge {
    pub edge_type: EdgeType,      // KNOWN, UNKNOWN, USER, INSERTED
    pub word_entry: WordEntry,
    pub path_cost: i32,           // 누적 Viterbi 비용
    pub left_index: u16,          // 최적 선행 에지 인덱스
    pub start_index: u32,         // 텍스트 시작 바이트 오프셋
    pub stop_index: u32,          // 텍스트 종료 바이트 오프셋
    pub kanji_only: bool,         // 검색 모드 페널티용
}
```

### Lattice
```rust
pub struct Lattice {
    capacity: usize,
    ends_at: Vec<Vec<Edge>>,              // ends_at[바이트_위치] = 여기서 끝나는 에지들
    char_info_buffer: Vec<CharData>,      // 사전 계산된 문자 정보
    categories_buffer: Vec<CategoryId>,   // 평탄화된 카테고리 버퍼
    // ... N-Best, 캐시 필드들
}
```
**핵심**: `ends_at`가 **바이트 위치**로 인덱싱 → UTF-8 문자 경계 계산 불필요

### 전방 Viterbi (융합 구현)
래티스 구축과 비용 계산을 **단일 전방 패스**로 융합:
1. **Aho-Corasick 프리스캔**: `da.find_overlapping_iter(text)` 한 번 실행 → 모든 매칭을 시작 위치별 연결 리스트로 정리
2. **문자 전처리**: 전체 텍스트의 카테고리/한자 연속 길이를 미리 계산
3. **전방 패스**: 각 문자 위치에서 사전 매칭 + 미등록어 처리 → 최적 선행자 선택
4. **비용 계산**: `path_cost = best_left.path_cost + connection_cost(left.right_id, cur.left_id) + word_cost`
5. **역추적**: EOS → BOS 방향으로 `left_index` 포인터 추적

### 검색 모드 (Decompose)
```rust
pub enum Mode {
    Normal,
    Decompose(Penalty),
}
pub struct Penalty {
    pub kanji_penalty_length_threshold: usize,   // 기본 2
    pub kanji_penalty_length_penalty: i32,        // 기본 3000
    pub other_penalty_length_threshold: usize,    // 기본 7
    pub other_penalty_length_penalty: i32,        // 기본 1700
}
```

## 3. 빌드 파이프라인 (CSV → 바이너리)

### 빌드 순서
```
DictionaryBuilder::build_dictionary()
  1. build_metadata()              → metadata.json
  2. build_character_definition()  → char_def.bin (rkyv)
  3. build_unknown_dictionary()    → unk.bin (rkyv)
  4. build_prefix_dictionary()     → dict.da, dict.vals, dict.wordsidx, dict.words
  5. build_connection_cost_matrix()→ matrix.mtx
```

### PrefixDictionary 빌드 과정
1. 모든 `*.csv` 파일 읽기 → 표면형 기준 정렬
2. `BTreeMap<String, Vec<WordEntry>>` 구성
3. DA 값 인코딩: `(offset << 5) | count`
4. `DoubleArrayAhoCorasickBuilder::build_with_values(keyset)` 호출
5. 5개 파일 출력: dict.da, dict.vals, dict.wordsidx, dict.words

### 한국어 사전 스키마 (12필드)
```json
{
  "dictionary_schema": {
    "fields": [
      "surface", "left_context_id", "right_context_id", "cost",
      "part_of_speech_tag", "meaning", "presence_absence", "reading",
      "type", "first_part_of_speech", "last_part_of_speech", "expression"
    ]
  }
}
```

## 4. 사전 로딩 흐름

### Data 추상화
```rust
pub enum Data {
    Static(&'static [u8]),  // include_bytes! (컴파일타임 임베딩)
    Vec(Vec<u8>),           // 파일시스템 로딩
    Map(Arc<Mmap>),         // 메모리 매핑
}
```

### 직렬화 전략
| 컴포넌트 | 직렬화 방식 |
|----------|------------|
| CharacterDefinition | rkyv (제로카피) |
| UnknownDictionary | rkyv (제로카피) |
| ConnectionCostMatrix | 커스텀 바이너리 (i16 배열) |
| PrefixDictionary DA | daachorse 자체 serialize/deserialize |
| PrefixDictionary 데이터 | Raw 바이트 배열 |
| UserDictionary | rkyv 전체 직렬화 |
| Metadata | JSON (serde_json) |

### 핵심 의존 크레이트
| 크레이트 | 역할 |
|---------|------|
| daachorse | DA Aho-Corasick 오토마타 |
| rkyv | 제로카피 역직렬화 |
| byteorder | 리틀엔디안 바이너리 읽기/쓰기 |
| csv | MeCab CSV 파싱 |
| encoding_rs | 문자 인코딩 (EUC-KR 등) |
| flate2 | Deflate 압축 |
| memmap2 | 메모리 매핑 I/O |

## 5. mecab-ko 프로젝트 적용 방안

### mecab-ko-dict 적용
1. **DA Trie → daachorse 고려**: 현재 yada 사용 중이지만, Lindera처럼 daachorse로 전환하면 한 번의 전체 스캔으로 모든 매칭 수집 가능 (성능 대폭 향상)
2. **WordEntry 10바이트 구조**: 현재 Entry struct를 분리하여 컴팩트 WordEntry + 별도 상세 저장소
3. **연접 행렬 전치**: `forward_id + backward_id * forward_size` 인덱싱으로 캐시 최적화
4. **Data enum**: Static/Vec/Mmap 3가지 모드를 하나의 타입으로 추상화

### mecab-ko-core 적용
1. **융합 Viterbi**: 래티스 구축과 비용 계산을 단일 전방 패스로 통합
2. **바이트 인덱싱**: `ends_at[바이트_위치]`로 UTF-8 경계 계산 회피
3. **Aho-Corasick 프리스캔**: 전체 텍스트 한 번 스캔으로 모든 매칭 수집
4. **검색 모드**: Penalty 구조체로 장단어 분해 제어

### mecab-ko-dict-builder 적용
1. **5파일 출력 포맷**: dict.da, dict.vals, dict.wordsidx, dict.words, matrix.mtx
2. **DA 값 인코딩**: `(offset << 5) | count` 방식
3. **rkyv 직렬화**: CharacterDefinition, UnknownDictionary용

## 참고 자료
- [Lindera GitHub](https://github.com/lindera/lindera) - 소스 코드
- [daachorse](https://github.com/daac-tools/daachorse) - DA Aho-Corasick
- [rkyv](https://rkyv.org/) - 제로카피 직렬화
