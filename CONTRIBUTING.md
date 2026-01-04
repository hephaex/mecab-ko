# Contributing to MeCab-Ko

> **Project**: MeCab-Ko - Korean Morphological Analyzer  
> **Maintainer**: hephaex (hephaex@gmail.com)  
> **Repository**: https://github.com/hephaex/mecab-ko

---

MeCab-Ko 프로젝트에 기여해 주셔서 감사합니다! 이 문서는 프로젝트에 기여하는 방법을 안내합니다.

## 행동 강령

이 프로젝트는 [Contributor Covenant](https://www.contributor-covenant.org/) 행동 강령을 따릅니다. 참여함으로써 이 코드를 준수할 것에 동의하는 것입니다.

---

## 기여 방법

### 버그 리포트

버그를 발견했다면:

1. [기존 이슈](https://github.com/hephaex/mecab-ko/issues)에서 중복 여부 확인
2. 새 이슈 생성 시 다음 정보 포함:
   - MeCab-RS-KO 버전
   - Rust 버전 (`rustc --version`)
   - 운영체제
   - 재현 단계
   - 예상 동작 vs 실제 동작
   - 에러 메시지 (있는 경우)

### 기능 제안

새로운 기능을 제안하려면:

1. Discussion 또는 Issue에서 아이디어 공유
2. 유스케이스와 기대 효과 설명
3. 가능하다면 구현 방향 제안

### 코드 기여

#### 시작하기

```bash
# 1. 저장소 포크
# GitHub에서 Fork 버튼 클릭

# 2. 로컬 클론
git clone https://github.com/YOUR_USERNAME/mecab-ko.git
cd mecab-ko

# 3. Rust 개발 디렉토리로 이동
cd rust

# 4. 업스트림 설정
git remote add upstream https://github.com/hephaex/mecab-ko.git

# 4. 개발 환경 확인
rustc --version  # 1.75.0 이상 권장
cargo --version
```

#### 개발 워크플로우

```bash
# 1. 최신 main 동기화
git checkout main
git pull upstream main

# 2. 브랜치 생성
git checkout -b feature/RST-XXX-description

# 3. 개발 및 테스트
cargo build --all-features
cargo test
cargo clippy -- -D warnings
cargo fmt

# 4. 커밋
git add .
git commit -m "feat(module): description"

# 5. 푸시 및 PR 생성
git push origin feature/RST-XXX-description
```

---

## 코딩 표준

### Rust 스타일 가이드

```rust
// ✅ 좋은 예
/// 한글 음절을 자모로 분해합니다.
///
/// # Arguments
/// * `syllable` - 분해할 한글 음절
///
/// # Returns
/// 초성, 중성, 종성(옵션) 튜플
///
/// # Examples
/// ```
/// use mecab_rs_ko_hangul::decompose;
/// let (cho, jung, jong) = decompose('한').unwrap();
/// assert_eq!(cho, 'ㅎ');
/// ```
pub fn decompose(syllable: char) -> Option<(char, char, Option<char>)> {
    // 구현
}

// ❌ 피해야 할 예
pub fn d(c: char) -> Option<(char, char, Option<char>)> {
    // 문서 없음, 불명확한 이름
}
```

### 필수 검사

PR 제출 전 모든 검사 통과 필수:

```bash
# 빌드
cargo build --all-features

# 테스트 (문서 테스트 포함)
cargo test --all-features

# Clippy (경고 없이)
cargo clippy --all-features -- -D warnings

# 포맷팅
cargo fmt --all -- --check
```

### unsafe 코드

- `unsafe` 사용은 최소화
- 사용 시 반드시 `// SAFETY:` 주석으로 안전성 근거 설명
- 대안이 있다면 safe Rust 선호

```rust
// ✅ 필요한 경우
// SAFETY: ptr은 항상 유효한 메모리를 가리키며,
// length는 할당된 버퍼 크기를 초과하지 않음이 보장됨
unsafe {
    std::slice::from_raw_parts(ptr, length)
}
```

### 에러 처리

- 라이브러리 코드에서 `unwrap()`, `expect()` 금지
- `Result<T, E>` 또는 `Option<T>` 사용
- 커스텀 에러 타입은 `thiserror` 사용 권장

```rust
// ✅ 좋은 예
pub fn parse(input: &str) -> Result<Token, ParseError> {
    // ...
}

// ❌ 피해야 할 예
pub fn parse(input: &str) -> Token {
    input.parse().unwrap() // 패닉 가능!
}
```

---

## 커밋 컨벤션

[Conventional Commits](https://www.conventionalcommits.org/) 형식 사용:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### 타입

| Type | 설명 |
|------|------|
| `feat` | 새로운 기능 |
| `fix` | 버그 수정 |
| `docs` | 문서 변경 |
| `style` | 포맷팅 (코드 동작 변경 없음) |
| `refactor` | 리팩토링 |
| `perf` | 성능 개선 |
| `test` | 테스트 추가/수정 |
| `build` | 빌드 시스템 변경 |
| `ci` | CI 설정 변경 |
| `chore` | 기타 변경 |

### 스코프

- `hangul`: 한글 유틸리티
- `dict`: 사전 관련
- `core`: 핵심 알고리즘
- `cli`: CLI 도구
- `python`: Python 바인딩
- `wasm`: WASM 바인딩

### 예시

```
feat(hangul): add jamo decomposition function

Implement decompose() function that splits Korean syllables
into individual jamo components (choseong, jungseong, jongseong).

Closes #RST-008
```

---

## Pull Request 가이드

### PR 체크리스트

PR 제출 전 확인:

- [ ] 관련 이슈 번호 연결 (`Closes #XXX`)
- [ ] 모든 테스트 통과
- [ ] Clippy 경고 없음
- [ ] 포맷팅 적용됨
- [ ] 새 기능은 테스트 포함
- [ ] 공개 API는 문서화됨
- [ ] CHANGELOG 업데이트 (해당 시)
- [ ] Breaking change는 명시됨

### PR 템플릿

```markdown
## Summary
[변경 사항 요약]

## Related Issue
Closes #[이슈번호]

## Changes
- 변경 1
- 변경 2

## Testing
테스트 방법 설명

## Checklist
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Documentation updated
```

### 리뷰 프로세스

1. **자동 검사**: CI가 빌드, 테스트, 린트 실행
2. **코드 리뷰**: 메인테이너가 코드 품질 검토
3. **수정 요청**: 필요시 변경 요청
4. **승인 및 병합**: Squash and Merge로 병합

---

## 프로젝트 구조

```
mecab-ko/
├── crates/
│   ├── mecab-ko-core/      # 핵심 알고리즘
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── lattice.rs     # 래티스 구조
│   │   │   └── viterbi.rs     # Viterbi 알고리즘
│   │   └── Cargo.toml
│   │
│   ├── mecab-ko-dict/      # 사전 관리
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── trie.rs        # Double-Array Trie
│   │   │   └── connection.rs  # 연결 비용
│   │   └── Cargo.toml
│   │
│   ├── mecab-ko-hangul/    # 한글 유틸리티
│   │   ├── src/lib.rs
│   │   └── Cargo.toml
│   │
│   ├── mecab-ko-cli/       # CLI 도구
│   │   └── ...
│   │
│   └── mecab-ko-python/    # Python 바인딩
│       └── ...
│
├── docs/                       # 문서
│   ├── architecture/          # 아키텍처 문서
│   ├── analysis/              # 분석 문서
│   └── api/                   # API 문서
│
├── tests/                      # 통합 테스트
├── benches/                    # 벤치마크
├── examples/                   # 예제 코드
│
├── Cargo.toml                  # 워크스페이스 설정
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── LICENSE-MIT
└── LICENSE-APACHE
```

---

## 릴리스 프로세스

버전 관리는 [Semantic Versioning](https://semver.org/)을 따릅니다:

- **MAJOR**: 하위 호환되지 않는 API 변경
- **MINOR**: 하위 호환되는 기능 추가
- **PATCH**: 하위 호환되는 버그 수정

### Pre-release

- `0.x.y`: 초기 개발 단계
- `x.y.z-alpha.N`: 알파 릴리스
- `x.y.z-beta.N`: 베타 릴리스
- `x.y.z-rc.N`: 릴리스 후보

---

## 도움 받기

- **이슈**: https://github.com/hephaex/mecab-ko/issues
- **디스커션**: https://github.com/hephaex/mecab-ko/discussions
- **이메일**: hephaex@gmail.com

---

## 라이선스

이 프로젝트에 기여함으로써, 귀하의 기여가 프로젝트와 동일한 라이선스(MIT OR Apache-2.0)로 배포됨에 동의합니다.

---

*Last Updated: 2026-01-04*  
*Maintainer: hephaex (hephaex@gmail.com)*
