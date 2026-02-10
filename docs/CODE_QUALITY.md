# Code Quality Standards

> **Project**: MeCab-Ko - Korean Morphological Analyzer in Rust  
> **Author**: hephaex (hephaex@gmail.com)  
> **Repository**: https://github.com/hephaex/mecab-ko

---

## 개요

이 문서는 MeCab-Ko 프로젝트의 코드 품질 기준과 측정 방법을 정의합니다.

---

## 코드 품질 지표

### 1. 테스트 커버리지

| 영역 | 최소 기준 | 목표 |
|------|-----------|------|
| 전체 | 80% | 90% |
| 핵심 모듈 (core, dict) | 85% | 95% |
| 유틸리티 (hangul) | 90% | 95% |
| CLI | 70% | 80% |
| 바인딩 | 75% | 85% |

#### 측정 방법

```bash
# 전체 커버리지 측정
cargo llvm-cov --all-features

# HTML 리포트 생성
cargo llvm-cov --all-features --html
```

### 2. 정적 분석

#### Clippy 규칙

모든 기본 Clippy 린트 + 추가 린트:

```toml
# Cargo.toml
[lints.clippy]
# 필수 (에러로 처리)
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"

# 권장 (경고로 처리)
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }

# 프로젝트별 설정
allow_attributes_without_reason = "warn"
missing_docs_in_private_items = "warn"
```

#### 실행 방법

```bash
# 기본 Clippy
cargo clippy --all-targets --all-features -- -D warnings

# 모든 린트 포함
cargo clippy --all-targets --all-features -- -W clippy::pedantic -D warnings
```

### 3. 문서화 수준

| 영역 | 요구 사항 |
|------|-----------|
| 공개 API | 100% rustdoc 필수 |
| 공개 타입 | Examples 필수 |
| 내부 함수 | 복잡한 로직에 주석 |
| 모듈 | `//!` 모듈 문서 필수 |

#### 검증 방법

```bash
# 문서 빌드 및 경고 확인
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
```

### 4. 복잡도 지표

#### Cyclomatic Complexity

- 함수당 최대: 15
- 권장: 10 이하

#### 측정 도구

```bash
# rust-code-analysis 사용
cargo install rust-code-analysis-cli
rust-code-analysis-cli -m -p crates/
```

### 5. 의존성 품질

| 기준 | 요구 사항 |
|------|-----------|
| 직접 의존성 | 최소화 (필요한 것만) |
| 라이선스 | MIT, Apache-2.0, BSD만 |
| 보안 취약점 | 0개 |
| 유지보수 상태 | 최근 1년 내 업데이트 |

#### 검증 방법

```bash
# 보안 취약점 검사
cargo audit

# 라이선스 검사
cargo deny check licenses

# 의존성 트리 확인
cargo tree --depth 1
```

---

## 자동화된 품질 검사

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

echo "Running pre-commit checks..."

# 포맷팅
cargo fmt --all -- --check

# Clippy
cargo clippy --all-targets --all-features -- -D warnings

# 테스트
cargo test --all-features

echo "All checks passed!"
```

### CI 품질 게이트

```yaml
# 품질 게이트 기준
quality_gate:
  - name: "Test Coverage"
    threshold: 80%
    action: fail
  
  - name: "Clippy Warnings"
    threshold: 0
    action: fail
  
  - name: "Security Vulnerabilities"
    threshold: 0
    action: fail
  
  - name: "Documentation Coverage"
    threshold: 100%  # public items
    action: warn
```

---

## 코드 리뷰 체크리스트

### 기능성

- [ ] 요구사항을 충족하는가?
- [ ] 엣지 케이스를 처리하는가?
- [ ] 에러 처리가 적절한가?

### 안전성

- [ ] unsafe 코드가 최소화되어 있는가?
- [ ] unsafe 사용 시 SAFETY 주석이 있는가?
- [ ] 패닉 가능성이 제거되었는가?

### 성능

- [ ] 불필요한 할당이 없는가?
- [ ] 적절한 데이터 구조를 사용하는가?
- [ ] 시간/공간 복잡도가 적절한가?

### 가독성

- [ ] 변수/함수명이 명확한가?
- [ ] 복잡한 로직에 주석이 있는가?
- [ ] 함수가 단일 책임을 가지는가?

### 테스트

- [ ] 단위 테스트가 있는가?
- [ ] 엣지 케이스 테스트가 있는가?
- [ ] 테스트 커버리지가 충분한가?

### 문서화

- [ ] 공개 API에 rustdoc이 있는가?
- [ ] 예제 코드가 포함되어 있는가?
- [ ] CHANGELOG가 업데이트되었는가?

---

## 품질 개선 프로세스

### 기술 부채 관리

```markdown
# 기술 부채 기록 형식

## DEBT-XXX: [제목]

**위치**: `crate::module::function`
**심각도**: High/Medium/Low
**예상 공수**: X hours

**설명**:
현재 상태와 문제점

**제안된 해결책**:
개선 방향

**영향**:
- 성능: +/-X%
- 유지보수성: 개선/저하
```

### 정기 품질 리뷰

| 주기 | 활동 |
|------|------|
| 매주 | 의존성 업데이트 검토 |
| 격주 | 기술 부채 검토 |
| 월간 | 품질 지표 리뷰 |
| 분기 | 아키텍처 리뷰 |

---

## 품질 대시보드 지표

### 추적 지표

```yaml
code_quality_metrics:
  - name: test_coverage
    current: 82%
    target: 90%
    trend: improving

  - name: clippy_warnings
    current: 0
    target: 0
    trend: stable

  - name: doc_coverage
    current: 95%
    target: 100%
    trend: stable

  - name: security_vulnerabilities
    current: 0
    target: 0
    trend: stable

  - name: tech_debt_items
    current: 5
    target: 0
    trend: improving
```

---

## 참고 자료

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)

---

*Last Updated: 2026-01-04*  
*Maintainer: hephaex (hephaex@gmail.com)*
