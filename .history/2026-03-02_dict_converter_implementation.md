# 세션 로그: 사전 데이터 변환기 구현

**날짜**: 2026-03-02
**작업**: Sprint 13 - S13-05 (사전 데이터 변환기)
**담당**: PM Agent

---

## 목표

국립국어원 API에서 가져온 사전 데이터를 MeCab-Ko 형식으로 변환하는 컨버터 모듈 구현.

---

## 구현 내용

### 1. 크레이트 생성

**파일**: `rust/crates/mecab-ko-dict-sync/`

새로운 크레이트를 워크스페이스에 추가:
- `Cargo.toml`: 의존성 설정 (thiserror, csv, serde)
- `src/lib.rs`: 크레이트 진입점 및 에러 타입 정의
- `src/converter.rs`: 핵심 변환 로직
- `README.md`: 사용법 문서

### 2. 데이터 구조

#### `ConverterEntry` (입력)
```rust
pub struct ConverterEntry {
    pub surface: String,        // 표면형
    pub pos: String,            // 국립국어원 품사 (예: "명사", "동사")
    pub reading: Option<String>, // 발음/읽기
    pub frequency: Option<u32>,  // 빈도 (비용 계산용)
}
```

#### `UserEntry` (출력)
```rust
pub struct UserEntry {
    pub surface: String,    // 표면형
    pub left_id: i16,       // 좌문맥 ID (0=자동)
    pub right_id: i16,      // 우문맥 ID (0=자동)
    pub cost: i16,          // 비용 (낮을수록 우선순위 높음)
    pub pos: String,        // MeCab-Ko 품사 태그
    pub reading: Option<String>,
}
```

### 3. POS 태그 매핑

국립국어원 → MeCab-Ko 변환 테이블 구현:

| 국립국어원 | MeCab-Ko | 설명 |
|-----------|----------|------|
| 명사 | NNG | 일반명사 |
| 고유명사 | NNP | 고유명사 |
| 동사 | VV | 동사 |
| 형용사 | VA | 형용사 |
| 부사 | MAG | 일반부사 |
| 감탄사 | IC | 감탄사 |
| 관형사 | MM | 관형사 |
| 대명사 | NP | 대명사 |

총 **30개 이상의 매핑** 지원 (조사, 어미, 접사 등 포함).

### 4. 비용 계산 알고리즘

```rust
pub fn calculate_cost(&self, entry: &ConverterEntry) -> i16 {
    // 빈도 기반 기본 비용
    let base_cost = match entry.frequency {
        Some(freq) if freq >= 1000 => 0,     // 고빈도
        Some(freq) if freq >= 100 => 500,    // 중빈도
        Some(_) => 1000,                     // 저빈도
        None => 500,                         // 기본값
    };

    // 단어 길이 조정 (긴 단어 우대)
    let length_adjustment = if entry.surface.chars().count() > 5 {
        -100
    } else {
        0
    };

    (base_cost + length_adjustment).max(0)
}
```

### 5. CSV 출력 형식

MeCab-Ko 사용자 사전 호환:
```
표면형,좌ID,우ID,비용,품사,*,*,*,읽기,원형,읽기,*
```

예시:
```
챗GPT,0,0,0,NNP,*,*,*,챗지피티,챗GPT,챗지피티,*
메타버스,0,0,500,NNG,*,*,*,메타버스,메타버스,메타버스,*
```

### 6. API 설계

#### 주요 메서드

```rust
impl DictConverter {
    // 새 컨버터 생성 (기본 POS 매핑 포함)
    pub fn new() -> Self;

    // POS 태그 매핑
    pub fn map_pos(&self, nikl_pos: &str) -> Result<&str>;

    // 비용 계산
    pub fn calculate_cost(&self, entry: &ConverterEntry) -> i16;

    // 단일 항목 변환
    pub fn convert_entry(&self, entry: &ConverterEntry) -> Result<UserEntry>;

    // 배치 변환 → CSV 라인
    pub fn convert_to_csv(&self, entries: &[ConverterEntry]) -> Result<Vec<String>>;

    // 커스텀 매핑 추가
    pub fn add_pos_mapping(&mut self, nikl_pos: String, mecab_pos: String);

    // 모든 매핑 조회
    pub fn pos_mappings(&self) -> impl Iterator<Item = (&str, &str)>;
}

impl UserEntry {
    // CSV 라인 생성
    #[must_use]
    pub fn to_csv_line(&self) -> String;
}
```

---

## 테스트 커버리지

### 단위 테스트 (22개)

1. POS 매핑 테스트
   - `test_pos_mapping_nouns`: 명사류 매핑
   - `test_pos_mapping_verbs`: 동사/형용사 매핑
   - `test_pos_mapping_adverbs`: 부사류 매핑
   - `test_pos_mapping_interjections`: 감탄사 매핑
   - `test_pos_mapping_unknown`: 미지원 태그 에러 처리
   - `test_comprehensive_pos_categories`: 전체 카테고리 검증

2. 비용 계산 테스트
   - `test_calculate_cost_high_frequency`: 고빈도 (≥1000) → cost 0
   - `test_calculate_cost_medium_frequency`: 중빈도 (100-999) → cost 500
   - `test_calculate_cost_low_frequency`: 저빈도 (<100) → cost 1000
   - `test_calculate_cost_no_frequency`: 빈도 없음 → cost 500
   - `test_calculate_cost_long_word`: 긴 단어 (>5자) → -100 조정
   - `test_calculate_cost_long_word_boundary`: 경계 케이스 (=5자)

3. 변환 테스트
   - `test_convert_entry_basic`: 기본 변환
   - `test_convert_entry_without_reading`: 읽기 없는 경우
   - `test_convert_entry_unknown_pos`: 미지원 POS 에러
   - `test_convert_to_csv`: 배치 변환

4. CSV 출력 테스트
   - `test_user_entry_to_csv_line`: 표준 형식
   - `test_user_entry_to_csv_line_no_reading`: 읽기 없는 경우

5. 기타
   - `test_add_pos_mapping`: 커스텀 매핑
   - `test_pos_mappings_iter`: 매핑 순회
   - `test_dict_entry_equality`: 항목 동등성
   - `test_user_entry_equality`: 사용자 항목 동등성

### Doc 테스트 (8개)

모든 public API에 실행 가능한 doc 예제 포함:
- `DictConverter::new`
- `DictConverter::map_pos`
- `DictConverter::calculate_cost`
- `DictConverter::convert_entry`
- `DictConverter::convert_to_csv`
- `DictConverter::add_pos_mapping`
- `UserEntry::to_csv_line`
- 크레이트 레벨 예제

**결과**: 30/30 테스트 통과 ✅

---

## 예제 프로그램

**파일**: `examples/convert_neologisms.rs`

신조어 변환 데모:
- 9개 샘플 항목 (IT 용어, 사회/문화 용어, SNS 용어)
- POS 태그 분포 통계
- 우선순위별 분류 (고/중/저)

**실행**:
```bash
cargo run --example convert_neologisms -p mecab-ko-dict-sync
```

---

## 파일 변경 사항

### 생성된 파일
- `rust/crates/mecab-ko-dict-sync/Cargo.toml`
- `rust/crates/mecab-ko-dict-sync/README.md`
- `rust/crates/mecab-ko-dict-sync/src/lib.rs`
- `rust/crates/mecab-ko-dict-sync/src/converter.rs`
- `rust/crates/mecab-ko-dict-sync/examples/convert_neologisms.rs`

### 수정된 파일
- `PROGRESS.md`: S13-05 완료 표시

---

## 코드 품질

### Clippy
```bash
cargo clippy -p mecab-ko-dict-sync -- -D warnings
```
**결과**: 0 경고 ✅

### 테스트
```bash
cargo test -p mecab-ko-dict-sync
```
**결과**: 30/30 통과 ✅

### 빌드
```bash
cargo build -p mecab-ko-dict-sync
```
**결과**: 성공 ✅

---

## 기술적 세부사항

### 에러 처리

`Error` enum with `PartialEq + Eq` for testability:
```rust
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    UnknownPosTag(String),
    InvalidEntry(String),
    Csv(String),
    Io(String),
}
```

### 성능 고려사항

- `HashMap` 기반 POS 매핑 (O(1) 조회)
- `clone()` 최소화 (`&str` 반환)
- 비용 계산 시 정수 연산만 사용
- 배치 변환: iterator chain으로 메모리 효율적 처리

### 확장성

- `add_pos_mapping()`: 커스텀 매핑 추가 가능
- `pos_mappings()`: 모든 매핑 노출 (검사/디버깅용)
- `Default` trait 구현

---

## 다음 단계

### S13-06: CLI 사전 동기화 명령

`mecab-ko-cli`에 서브커맨드 추가 예정:
```bash
mecab-ko sync --source opendict --api-key KEY --output neologisms.csv
```

이 컨버터가 핵심 변환 로직을 담당.

---

## 참고 자료

- 국립국어원 API 조사: `docs/research/dictionary/korean-dict-api-survey.md`
- 사용자 사전 형식: `data/user-dict/README.md`
- 기존 신조어: `data/user-dict/neologisms.csv`

---

## 학습 포인트

1. **POS 태그 표준화**: 서로 다른 한국어 NLP 시스템 간 품사 매핑의 중요성
2. **비용 함수 설계**: 빈도 + 길이 기반 우선순위 계산으로 분석 품질 향상
3. **타입 안전성**: `ConverterEntry` vs `UserEntry` 분리로 변환 단계 명확화

---

**상태**: ✅ 완료
**커밋**: 대기 중
