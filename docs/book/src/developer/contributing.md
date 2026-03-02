# 컨트리뷰션 가이드

MeCab-Ko 프로젝트에 기여하는 방법을 안내합니다.

## 시작하기

### 1. 저장소 포크

```bash
# GitHub에서 Fork 버튼 클릭 후
git clone https://github.com/YOUR_USERNAME/mecab-ko.git
cd mecab-ko
```

### 2. 개발 환경 설정

```bash
# Rust 설치 (1.75.0 이상)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 빌드
cd rust
cargo build

# 테스트
cargo test
```

### 3. 브랜치 생성

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

## 코딩 규칙

### Rust 스타일

```rust
// Good
pub fn parse_text(text: &str) -> Result<Vec<Token>, Error> {
    // 구현
}

// Bad
pub fn ParseText(Text: &str) -> Result<Vec<Token>, Error> {
    // 구현
}
```

### 네이밍 컨벤션

- **함수/메서드**: `snake_case`
- **타입/구조체**: `PascalCase`
- **상수**: `SCREAMING_SNAKE_CASE`
- **모듈**: `snake_case`

### 문서화

모든 public API에 rustdoc 주석 필수:

```rust
/// 텍스트를 형태소 분석합니다.
///
/// # Arguments
///
/// * `text` - 분석할 텍스트
///
/// # Returns
///
/// 분석된 토큰 리스트
///
/// # Errors
///
/// 사전을 찾을 수 없거나 파싱 오류 시 에러 반환
///
/// # Examples
///
/// ```
/// use mecab_ko::Tagger;
///
/// let tagger = Tagger::new(Default::default())?;
/// let result = tagger.parse("안녕하세요")?;
/// ```
pub fn parse(&self, text: &str) -> Result<Vec<Token>, Error> {
    // 구현
}
```

### 에러 처리

```rust
// Good - Result 반환
pub fn load_dictionary(path: &Path) -> Result<Dictionary, Error> {
    let file = File::open(path)?;
    // ...
}

// Bad - panic 사용
pub fn load_dictionary(path: &Path) -> Dictionary {
    let file = File::open(path).unwrap(); // 금지!
    // ...
}
```

라이브러리 코드에서 `unwrap()`, `expect()` 금지!

### Unsafe 최소화

```rust
// 꼭 필요한 경우만 사용
unsafe {
    // SAFETY: 이유 설명 필수
}
```

## 테스트

### 단위 테스트

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let tagger = Tagger::new(Default::default()).unwrap();
        let result = tagger.parse("테스트").unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_parse_empty() {
        let tagger = Tagger::new(Default::default()).unwrap();
        let result = tagger.parse("").unwrap();
        assert!(result.is_empty());
    }
}
```

### 통합 테스트

```rust
// tests/integration_test.rs
use mecab_ko::Tagger;

#[test]
fn test_full_workflow() {
    let tagger = Tagger::new(Default::default()).unwrap();
    let text = "형태소 분석을 테스트합니다.";
    let result = tagger.parse(text).unwrap();

    // 검증
    assert!(result.iter().any(|t| t.surface == "형태소"));
}
```

### 벤치마크

```rust
// benches/parse_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse(c: &mut Criterion) {
    let tagger = Tagger::new(Default::default()).unwrap();

    c.bench_function("parse Korean text", |b| {
        b.iter(|| {
            tagger.parse(black_box("형태소 분석 벤치마크"))
        })
    });
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
```

## 커밋 메시지

### 포맷

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type

- `feat`: 새로운 기능
- `fix`: 버그 수정
- `docs`: 문서 변경
- `style`: 코드 포맷팅
- `refactor`: 리팩토링
- `test`: 테스트 추가/수정
- `chore`: 빌드/설정 변경
- `perf`: 성능 개선

### 예시

```
feat(tagger): N-best 분석 기능 추가

N-best 경로 탐색 알고리즘을 구현하여 여러 후보 결과를 반환할 수
있도록 개선했습니다.

- NBestAnalyzer 구조체 추가
- Viterbi 알고리즘 N-best 버전 구현
- 테스트 케이스 추가

Closes #123
```

## Pull Request

### 체크리스트

PR 생성 전 확인:

- [ ] 코드가 빌드됨 (`cargo build`)
- [ ] 모든 테스트 통과 (`cargo test`)
- [ ] Clippy 경고 없음 (`cargo clippy`)
- [ ] 포맷팅 적용 (`cargo fmt`)
- [ ] 문서 업데이트
- [ ] CHANGELOG.md 업데이트
- [ ] 테스트 추가/수정

### PR 템플릿

```markdown
## 변경 사항

<!-- 무엇을 변경했는지 설명 -->

## 동기

<!-- 왜 이 변경이 필요한지 설명 -->

## 테스트

<!-- 어떻게 테스트했는지 설명 -->

## 스크린샷 (해당되는 경우)

## 체크리스트

- [ ] 코드 빌드 확인
- [ ] 테스트 통과 확인
- [ ] 문서 업데이트
- [ ] CHANGELOG.md 업데이트
```

## CI/CD

### GitHub Actions

PR 생성 시 자동 실행:

- Rust 빌드 (stable, beta, nightly)
- 테스트 실행
- Clippy 린트
- 코드 포맷 확인
- 문서 빌드

### 로컬 확인

```bash
# 전체 CI 체크
./scripts/ci-check.sh
```

## 이슈 보고

### 버그 리포트

```markdown
## 버그 설명

<!-- 버그에 대한 명확한 설명 -->

## 재현 방법

1. ...
2. ...
3. ...

## 예상 동작

<!-- 어떻게 동작해야 하는지 -->

## 실제 동작

<!-- 실제로 어떻게 동작하는지 -->

## 환경

- OS: [e.g., Ubuntu 22.04]
- Rust 버전: [e.g., 1.75.0]
- MeCab-Ko 버전: [e.g., 0.1.0]

## 추가 정보

<!-- 스크린샷, 로그 등 -->
```

### 기능 제안

```markdown
## 기능 설명

<!-- 제안하는 기능에 대한 명확한 설명 -->

## 동기

<!-- 왜 이 기능이 필요한지 -->

## 사용 예시

```rust
// 코드 예시
```

## 대안

<!-- 고려한 다른 방법들 -->
```

## 릴리스 프로세스

### 버전 관리

Semantic Versioning 사용:

- MAJOR: 호환되지 않는 API 변경
- MINOR: 하위 호환되는 기능 추가
- PATCH: 하위 호환되는 버그 수정

### 릴리스 체크리스트

1. [ ] CHANGELOG.md 업데이트
2. [ ] Cargo.toml 버전 업데이트
3. [ ] 문서 버전 업데이트
4. [ ] 태그 생성 (`git tag v0.1.0`)
5. [ ] GitHub Release 생성
6. [ ] crates.io 배포 (`cargo publish`)

## 코드 리뷰

### 리뷰어 가이드

- 코드 스타일 확인
- 테스트 커버리지 확인
- 성능 영향 평가
- 문서화 확인
- 에러 처리 확인

### 리뷰이 가이드

- 피드백에 열린 자세
- 변경 이유 명확히 설명
- 요청된 수정사항 반영
- 토론이 필요한 경우 이슈 생성

## 라이선스

기여한 코드는 프로젝트의 Apache 2.0 또는 MIT 라이선스를 따릅니다.

## 행동 강령

- 존중하는 태도
- 건설적인 피드백
- 포용적인 언어 사용
- 협력적인 자세

## 연락처

- GitHub Issues: 버그 리포트, 기능 제안
- GitHub Discussions: 일반적인 질문, 토론
- Email: hephaex@gmail.com

## 참고 자료

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [The Rust Book](https://doc.rust-lang.org/book/)
