# S14-05: 한국어기초사전 API 클라이언트 추가

**날짜**: 2026-03-02
**작업 시간**: 약 1시간
**담당**: Claude Sonnet 4.5
**상태**: ✅ 완료

## 작업 개요

mecab-ko-dict-sync 크레이트에 한국어기초사전/표준국어대사전 API 클라이언트를 추가하여 다중 사전 소스 지원을 구현했습니다.

## 구현 내용

### 1. KrDictClient 구현 (`krdict_client.rs`)

**API 클라이언트 구조**:
```rust
pub struct KrDictClient {
    config: KrDictConfig,
    client: reqwest::Client,
}
```

**주요 메서드**:
- `new(config: KrDictConfig) -> Result<Self>` - 클라이언트 생성
- `search(query: &str) -> Result<Vec<DictEntry>>` - 검색
- `get_detail(target_code: &str) -> Result<DictDetail>` - 상세 정보 조회
- `search_paginated(query, start, num) -> Result<Vec<DictEntry>>` - 페이지네이션 검색

**XML 응답 파싱**:
- `KrDictSearchResponse` - 검색 응답 구조
- `KrDictChannel` - 결과 채널
- `KrDictItem` - 개별 항목
- `KrDictDetailItem` - 상세 항목

### 2. KrDictConfig 추가 (`config.rs`)

**Builder Pattern 구현**:
```rust
pub struct KrDictConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout_secs: u64,
    pub max_results: u32,
}

impl KrDictConfig {
    pub fn new(api_key: impl Into<String>) -> Self
    pub fn with_base_url(self, url: impl Into<String>) -> Self
    pub fn with_timeout_secs(self, secs: u64) -> Self
    pub fn with_max_results(self, max: u32) -> Self
    pub fn validate(&self) -> Result<()>
}
```

**기본값**:
- `DEFAULT_BASE_URL`: `https://krdict.korean.go.kr/api`
- `DEFAULT_TIMEOUT_SECS`: 30
- `DEFAULT_MAX_RESULTS`: 100

### 3. DictSource Enum 확장 (`main.rs`)

```rust
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum DictSource {
    /// 국립국어원 우리말샘 (`OpenDict` API)
    #[default]
    Opendict,
    /// 국립국어원 한국어기초사전/표준국어대사전 (`KrDict` API)
    Krdict,
}
```

### 4. CLI 통합

**Sync 커맨드 업데이트**:
```bash
# OpenDict 사용 (기본)
mecab sync -q "신조어" --api-key YOUR_KEY

# KrDict 사용
mecab sync -q "컴퓨터" --source krdict --api-key YOUR_KEY

# 환경변수 사용
export KRDICT_API_KEY=your_key
mecab sync -q "메타버스" --source krdict
```

**환경변수 지원**:
- `OPENDICT_API_KEY` - OpenDict API 키
- `KRDICT_API_KEY` - KrDict API 키

### 5. run_krdict_sync 함수 추가

OpenDictSync와 동일한 구조로 구현:
1. API 클라이언트 생성
2. 비동기 검색 실행 (tokio runtime)
3. 품사 태그 변환 (DictConverter)
4. CSV 출력 (파일 또는 stdout)

## 테스트 결과

### Unit Tests
```
running 47 tests
test config::tests::test_krdict_new_config ... ok
test config::tests::test_krdict_builder_pattern ... ok
test config::tests::test_krdict_validate_empty_api_key ... ok
test config::tests::test_krdict_validate_invalid_url ... ok
test config::tests::test_krdict_validate_zero_timeout ... ok
test config::tests::test_krdict_validate_zero_max_results ... ok
test config::tests::test_krdict_validate_success ... ok
test krdict_client::tests::test_new_client_valid_config ... ok
test krdict_client::tests::test_new_client_invalid_config ... ok
test krdict_client::tests::test_search_invalid_api_key ... ok
test krdict_client::tests::test_krdict_item_conversion ... ok
... (47 passed)
```

### Doc Tests
```
running 30 tests
test crates/mecab-ko-dict-sync/src/krdict_client.rs - krdict_client (line 21) - compile ... ok
test crates/mecab-ko-dict-sync/src/krdict_client.rs - krdict_client::KrDictClient (line 49) - compile ... ok
test crates/mecab-ko-dict-sync/src/krdict_client.rs - krdict_client::KrDictClient::new (line 84) ... ok
test crates/mecab-ko-dict-sync/src/krdict_client.rs - krdict_client::KrDictClient::search (line 118) - compile ... ok
test crates/mecab-ko-dict-sync/src/krdict_client.rs - krdict_client::KrDictClient::get_detail (line 190) - compile ... ok
test crates/mecab-ko-dict-sync/src/krdict_client.rs - krdict_client::KrDictClient::search_paginated (line 255) - compile ... ok
... (30 passed)
```

### CLI Tests
```
running 3 tests
test tests::test_sync_command_basic ... ok
test tests::test_sync_command_with_options ... ok
test tests::test_sync_command_krdict_source ... ok
```

### Clippy
```
✅ 0 warnings
```

## 파일 변경 사항

**신규 파일**:
- `rust/crates/mecab-ko-dict-sync/src/krdict_client.rs` (520 lines)

**수정 파일**:
- `rust/crates/mecab-ko-dict-sync/src/config.rs` (+169 lines)
- `rust/crates/mecab-ko-dict-sync/src/lib.rs` (+2 lines)
- `rust/crates/mecab-ko-cli/src/main.rs` (+126 lines)

**총 변경**:
- 4 files changed, 817 insertions(+), 5 deletions(-)

## API 응답 예시

### 검색 응답 (XML)
```xml
<channel>
  <total>100</total>
  <num>10</num>
  <item>
    <target_code>12345</target_code>
    <word>컴퓨터</word>
    <pos>명사</pos>
    <sense>
      <definition>전자 계산기</definition>
    </sense>
    <pronunciation>컴퓨터</pronunciation>
  </item>
</channel>
```

### 상세 응답 (XML)
```xml
<item>
  <target_code>12345</target_code>
  <word>컴퓨터</word>
  <pos>명사</pos>
  <sense>
    <definition>전자 계산기</definition>
    <example>컴퓨터로 작업한다</example>
    <related>전산기</related>
  </sense>
  <pronunciation>컴퓨터</pronunciation>
  <origin>영어 computer</origin>
</item>
```

## 코드 품질

### Error Handling
- 모든 HTTP 에러 처리 (401, 403, 429, 404 등)
- Custom error types (`SyncError`)
- Context 제공 (`anyhow::Context`)

### Documentation
- 모든 public API에 rustdoc 주석
- 사용 예제 포함
- 한국어/영어 혼용 문서화

### Testing
- Unit tests: 클라이언트 생성, 설정 검증
- Doc tests: API 사용 예제
- CLI tests: 커맨드 파싱

## 학습 포인트

### 1. XML 역직렬화 (quick-xml)
`quick_xml::de::from_str()` 사용하여 XML → Rust 구조체 변환:
```rust
let response: KrDictSearchResponse =
    quick_xml::de::from_str(&text).map_err(SyncError::from)?;
```

### 2. 다중 소스 패턴
Enum + match를 통한 확장 가능한 API 소스 관리:
```rust
match source {
    DictSource::Opendict => run_opendict_sync(...),
    DictSource::Krdict => run_krdict_sync(...),
}
```

### 3. Builder Pattern 일관성
OpenDictConfig와 동일한 패턴으로 KrDictConfig 구현하여 API 일관성 유지

## 다음 단계

### S14-06: CLI collect 서브커맨드
- 배치 수집 기능 구현
- 키워드 목록 파일 입력
- 진행률 표시

### S14-07: 사전 빌드 자동화 개선
- CI 워크플로우 추가
- 자동 리빌드 트리거

## 참고 자료

- **한국어기초사전 API**: https://krdict.korean.go.kr/openApi/openApiInfo
- **quick-xml 문서**: https://docs.rs/quick-xml/latest/quick_xml/
- **reqwest 문서**: https://docs.rs/reqwest/latest/reqwest/

## Commit

```
feat(dict-sync): Add Korean Dictionary API client (KrDict)

- Implement KrDictClient for 한국어기초사전/표준국어대사전 API
- Add KrDictConfig configuration with builder pattern
- Extend DictSource enum to support both OpenDict and KrDict
- Update CLI sync command to support --source krdict option
- Add KRDICT_API_KEY environment variable support
- Implement search(), get_detail(), search_paginated() methods
- Add comprehensive tests (47 unit tests + 30 doc tests)
- All tests passing with zero clippy warnings

Resolves S14-05: 한국어기초사전 API 클라이언트 추가
```

**Commit Hash**: 50972554
