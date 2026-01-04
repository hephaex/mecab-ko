# MeCab-Ko Development Workflow

> **Project**: MeCab-Ko - Korean Morphological Analyzer in Rust  
> **Author**: hephaex (hephaex@gmail.com)  
> **Repository**: https://github.com/hephaex/mecab-ko

---

## 개발 프로세스 개요

MeCab-Ko 프로젝트는 **7단계 개발 프로세스**를 따릅니다. 각 단계는 명확한 입력, 산출물, 완료 기준을 가지며, 단계 간 전환은 명시적인 승인을 필요로 합니다.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        DEVELOPMENT WORKFLOW                             │
│                                                                         │
│   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐                │
│   │ 1.ISSUE │──▶│2.ANALYZE│──▶│3.DEVELOP│──▶│4.REVIEW │                │
│   └─────────┘   └─────────┘   └─────────┘   └─────────┘                │
│                                                   │                     │
│                                                   ▼                     │
│   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐                │
│   │ 7.CLOSE │◀──│ 6.DOCS  │◀──│5.VERIFY │◀──┘                          │
│   └─────────┘   └─────────┘   └─────────┘                              │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Issue Creation (이슈 생성)

### 목적
작업 범위를 명확히 정의하고 추적 가능한 작업 단위를 생성합니다.

### 입력
- 기능 요청 / 버그 리포트 / 기술 부채 식별
- 스프린트 백로그
- 외부 요구사항

### 프로세스

```yaml
Step 1.1: 이슈 분류
  - [ ] 이슈 유형 결정 (Feature / Bug / Task / Research)
  - [ ] 에픽 할당 (DIC / RST / BND / ELS / QA)
  - [ ] 우선순위 설정 (P0-P3)

Step 1.2: 이슈 작성
  - [ ] 제목 작성 (간결하고 명확하게)
  - [ ] 설명 작성 (배경, 목표, 범위)
  - [ ] 수용 기준 정의 (Acceptance Criteria)
  - [ ] 복잡도 추정 (S/M/L/XL)

Step 1.3: 메타데이터 설정
  - [ ] 라벨 추가
  - [ ] 마일스톤 할당
  - [ ] 담당자 지정 (선택)
  - [ ] 의존성 연결
```

### 이슈 템플릿

```markdown
## 📋 Summary
[이슈의 핵심을 1-2문장으로 요약]

## 🎯 Objective
[달성하고자 하는 목표]

## 📝 Description
[상세 설명, 배경, 컨텍스트]

## ✅ Acceptance Criteria
- [ ] 기준 1
- [ ] 기준 2
- [ ] 기준 3

## 🔗 Dependencies
- Blocked by: #XX
- Blocks: #YY

## 📊 Metadata
- **Epic**: [DIC|RST|BND|ELS|QA]
- **Priority**: [P0|P1|P2|P3]
- **Complexity**: [S|M|L|XL]
- **Sprint**: [Sprint N]
```

### 산출물
- GitHub Issue 생성 완료
- 라벨 및 메타데이터 설정 완료

### 완료 기준
- [ ] 수용 기준이 명확하게 정의됨
- [ ] 복잡도가 추정됨
- [ ] 의존성이 식별됨
- [ ] 스프린트에 할당됨

### 상태 전환
```
Status: Backlog → Ready for Analysis
Label: needs-analysis 추가
```

---

## Phase 2: Analysis (분석)

### 목적
기술적 접근 방식을 결정하고 구현 계획을 수립합니다.

### 담당
- **Primary**: Analyst Agent
- **Support**: Architect Agent (설계 관련)

### 입력
- 이슈 설명 및 수용 기준
- 관련 기술 문서
- 기존 코드베이스

### 프로세스

```yaml
Step 2.1: 요구사항 분석
  - [ ] 기능적 요구사항 추출
  - [ ] 비기능적 요구사항 식별
  - [ ] 엣지 케이스 파악

Step 2.2: 기술 조사
  - [ ] 기존 구현 분석
  - [ ] 대안 기술 조사
  - [ ] 참고 자료 수집

Step 2.3: 접근 방식 제안
  - [ ] 가능한 접근 방식 나열
  - [ ] 각 방식의 장단점 분석
  - [ ] 권장 방식 선택 및 근거

Step 2.4: 영향 분석
  - [ ] 영향받는 모듈 식별
  - [ ] 하위 호환성 검토
  - [ ] 성능 영향 예측
```

### 분석 문서 템플릿

```markdown
# Analysis Report: [Issue ID] - [Title]

## 1. 요구사항 요약
### 기능적 요구사항
- FR-1: ...
- FR-2: ...

### 비기능적 요구사항
- NFR-1: 성능 - ...
- NFR-2: 보안 - ...

## 2. 기술 조사
### 기존 구현 분석
[기존 코드/라이브러리 분석 결과]

### 참고 자료
- [링크1]: 설명
- [링크2]: 설명

## 3. 접근 방식 비교
| 방식 | 장점 | 단점 | 예상 공수 |
|------|------|------|-----------|
| A    |      |      |           |
| B    |      |      |           |

## 4. 권장 사항
### 선택된 방식: [A/B/C]
**근거**: ...

### 구현 계획
1. Step 1
2. Step 2
3. Step 3

## 5. 영향 분석
### 영향받는 모듈
- `crate_a`: 변경 필요
- `crate_b`: 인터페이스 변경

### 리스크
- Risk 1: [설명] → Mitigation: [대응책]
```

### 산출물
- 분석 문서 (docs/analysis/[issue-id].md)
- 이슈 코멘트 (분석 요약)
- 설계 결정 사항 (필요시 ADR)

### 완료 기준
- [ ] 분석 문서 작성 완료
- [ ] 접근 방식 결정됨
- [ ] 영향 분석 완료
- [ ] 아키텍트 검토 완료 (L/XL 이슈)

### 상태 전환
```
Status: Analysis → Ready for Development
Label: needs-analysis 제거, ready-for-dev 추가
Comment: "[Analysis Complete] 분석 문서: docs/analysis/xxx.md"
```

---

## Phase 3: Development (개발)

### 목적
분석 결과를 바탕으로 기능을 구현합니다.

### 담당
- **Primary**: Developer Agent
- **Tier**: 이슈 복잡도에 따라 동적 할당

### 입력
- 분석 문서
- 설계 결정 사항
- 코딩 표준

### 프로세스

```yaml
Step 3.1: 브랜치 생성
  - [ ] 브랜치명: feature/[issue-id]-[short-description]
  - [ ] 최신 main에서 분기

Step 3.2: 구현
  - [ ] 코드 작성
  - [ ] 인라인 문서화 (rustdoc)
  - [ ] 에러 처리

Step 3.3: 테스트 작성
  - [ ] 단위 테스트
  - [ ] 통합 테스트 (필요시)
  - [ ] 문서 테스트 (doctests)

Step 3.4: 로컬 검증
  - [ ] cargo build --all-features
  - [ ] cargo test
  - [ ] cargo clippy -- -D warnings
  - [ ] cargo fmt --check

Step 3.5: PR 생성
  - [ ] PR 템플릿 작성
  - [ ] 변경 사항 요약
  - [ ] 테스트 결과 첨부
```

### 브랜치 네이밍 컨벤션

```
feature/RST-004-double-array-trie    # 기능
bugfix/DIC-015-encoding-error        # 버그 수정
docs/RST-014-api-documentation       # 문서
refactor/RST-020-trie-optimization   # 리팩토링
```

### 커밋 메시지 컨벤션

```
<type>(<scope>): <subject>

<body>

<footer>

Types:
- feat: 새로운 기능
- fix: 버그 수정
- docs: 문서 변경
- style: 포맷팅, 세미콜론 등
- refactor: 리팩토링
- test: 테스트 추가/수정
- chore: 빌드, 설정 등

예시:
feat(hangul): add jamo decomposition utilities

Implement Unicode-compliant jamo decomposition and composition
functions for Korean syllable handling.

Closes #RST-008
```

### PR 템플릿

```markdown
## 📋 Summary
[변경 사항을 1-2문장으로 요약]

## 🔗 Related Issue
Closes #[issue-number]

## 🔄 Changes
- Change 1
- Change 2
- Change 3

## ✅ Checklist
- [ ] 코드가 Rust 안전성 가이드라인을 따름
- [ ] 단위 테스트 추가됨
- [ ] cargo clippy 통과
- [ ] cargo fmt 적용됨
- [ ] rustdoc 문서 작성됨

## 🧪 Test Results
```
cargo test 결과 붙여넣기
```

## 📸 Screenshots (if applicable)
```

### 산출물
- 기능 구현 코드
- 단위 테스트
- PR 생성

### 완료 기준
- [ ] 모든 테스트 통과
- [ ] clippy 경고 없음
- [ ] 포맷팅 적용됨
- [ ] PR 생성됨

### 상태 전환
```
Status: In Progress → Ready for Review
Label: ready-for-dev 제거, needs-review 추가
PR: Draft → Ready for Review
```

---

## Phase 4: Code Review (코드 리뷰)

### 목적
코드 품질, 보안, 성능을 검토하고 개선점을 제안합니다.

### 담당
- **Primary**: Reviewer Agent
- **Secondary**: Architect Agent (아키텍처 변경 시)

### 입력
- PR 코드
- 테스트 결과
- 분석 문서

### 프로세스

```yaml
Step 4.1: 자동 검사
  - [ ] CI 빌드 통과 확인
  - [ ] 테스트 커버리지 확인
  - [ ] 정적 분석 결과 확인

Step 4.2: 코드 품질 검토
  - [ ] 가독성
  - [ ] 유지보수성
  - [ ] 코딩 표준 준수
  - [ ] 에러 처리

Step 4.3: 안전성 검토
  - [ ] unsafe 코드 최소화
  - [ ] 메모리 안전성
  - [ ] 입력 검증
  - [ ] 보안 취약점

Step 4.4: 성능 검토
  - [ ] 알고리즘 복잡도
  - [ ] 메모리 사용
  - [ ] 불필요한 할당/복사

Step 4.5: 피드백 작성
  - [ ] 인라인 코멘트
  - [ ] 전체 리뷰 요약
  - [ ] 결정 (Approve / Request Changes)
```

### 리뷰 체크리스트

```markdown
## Code Review Checklist

### 🔒 Safety & Security
- [ ] No unnecessary `unsafe` blocks
- [ ] Input validation present
- [ ] No potential panics in public APIs
- [ ] Proper error handling (no unwrap() in library code)

### 📐 Code Quality
- [ ] Clear variable/function names
- [ ] Functions are focused (single responsibility)
- [ ] No code duplication
- [ ] Appropriate abstraction level

### 🧪 Testing
- [ ] Unit tests cover main functionality
- [ ] Edge cases tested
- [ ] Error cases tested
- [ ] Test coverage >= 80%

### 📚 Documentation
- [ ] Public APIs documented (rustdoc)
- [ ] Complex logic explained
- [ ] Examples provided
- [ ] CHANGELOG updated (if applicable)

### ⚡ Performance
- [ ] No obvious inefficiencies
- [ ] Appropriate data structures used
- [ ] No unnecessary allocations
```

### 리뷰 코멘트 가이드라인

```markdown
## 코멘트 유형

🔴 **[BLOCKING]**: 반드시 수정 필요
예: `[BLOCKING] This unwrap() can panic on invalid input`

🟡 **[SUGGESTION]**: 개선 권장
예: `[SUGGESTION] Consider using Vec::with_capacity() here`

🟢 **[NIT]**: 사소한 제안
예: `[NIT] Prefer snake_case for this variable`

💡 **[QUESTION]**: 이해를 위한 질문
예: `[QUESTION] Why is this clone necessary?`

👍 **[PRAISE]**: 좋은 코드에 대한 칭찬
예: `[PRAISE] Nice use of the builder pattern here!`
```

### 산출물
- 코드 리뷰 코멘트
- 리뷰 결정 (Approve / Request Changes)
- 보안 검토 노트 (필요시)

### 완료 기준
- [ ] 모든 BLOCKING 이슈 해결됨
- [ ] 리뷰어 승인 획득
- [ ] CI 모든 검사 통과

### 상태 전환

**승인 시:**
```
Status: Review → Ready for Verification
Label: needs-review 제거, needs-verification 추가
Comment: "[Review Approved] ✅"
```

**변경 요청 시:**
```
Status: Review → Changes Requested
Label: changes-requested 추가
Comment: "[Changes Requested] 다음 항목 수정 필요: ..."
```

---

## Phase 5: Verification (검증)

### 목적
기능이 요구사항을 충족하고 기존 기능에 영향을 주지 않음을 검증합니다.

### 담당
- **Primary**: QA Agent

### 입력
- 승인된 PR
- 수용 기준
- 기존 테스트 스위트

### 프로세스

```yaml
Step 5.1: 기능 검증
  - [ ] 수용 기준 충족 확인
  - [ ] 엣지 케이스 테스트
  - [ ] 예상 동작 확인

Step 5.2: 회귀 테스트
  - [ ] 전체 테스트 스위트 실행
  - [ ] 관련 모듈 테스트 집중 실행
  - [ ] 기존 기능 정상 동작 확인

Step 5.3: 성능 검증
  - [ ] 벤치마크 실행 (해당 시)
  - [ ] 성능 저하 없음 확인
  - [ ] 메모리 사용량 확인

Step 5.4: 통합 검증
  - [ ] 다른 모듈과의 통합 테스트
  - [ ] 바인딩 동작 확인 (해당 시)
  - [ ] 예제 코드 실행
```

### 검증 보고서 템플릿

```markdown
# Verification Report: PR #[number]

## Test Results
| Test Suite | Passed | Failed | Skipped |
|------------|--------|--------|---------|
| Unit Tests |   152  |   0    |    2    |
| Doc Tests  |    45  |   0    |    0    |
| Integration|    23  |   0    |    0    |

## Coverage
- Overall: 82.5%
- Changed files: 91.2%

## Benchmark Results (if applicable)
| Benchmark | Before | After | Change |
|-----------|--------|-------|--------|
| tokenize  | 120μs  | 118μs | -1.7%  |
| load_dict | 450ms  | 445ms | -1.1%  |

## Acceptance Criteria
- [x] AC-1: [설명]
- [x] AC-2: [설명]
- [x] AC-3: [설명]

## Verification Status: ✅ PASSED
```

### 산출물
- 검증 보고서
- 테스트 결과 로그
- 벤치마크 결과 (해당 시)

### 완료 기준
- [ ] 모든 테스트 통과
- [ ] 성능 회귀 없음
- [ ] 수용 기준 모두 충족
- [ ] 검증 보고서 작성

### 상태 전환
```
Status: Verification → Ready for Documentation
Label: needs-verification 제거, needs-docs 추가
Comment: "[Verification Passed] 검증 보고서: [링크]"
```

---

## Phase 6: Documentation (문서화)

### 목적
변경 사항을 문서화하고 사용자/개발자 가이드를 업데이트합니다.

### 담당
- **Primary**: Documentation Agent

### 입력
- 구현된 기능
- API 변경 사항
- 수용 기준

### 프로세스

```yaml
Step 6.1: API 문서 검토
  - [ ] rustdoc 완성도 확인
  - [ ] 예제 코드 검증
  - [ ] 링크 동작 확인

Step 6.2: 사용자 문서 업데이트
  - [ ] README 업데이트 (필요시)
  - [ ] 튜토리얼 업데이트 (필요시)
  - [ ] 마이그레이션 가이드 (Breaking changes)

Step 6.3: CHANGELOG 업데이트
  - [ ] 변경 유형 분류
  - [ ] 변경 내용 기술
  - [ ] 이슈/PR 링크

Step 6.4: 내부 문서 정리
  - [ ] ADR 업데이트 (필요시)
  - [ ] 아키텍처 문서 업데이트 (필요시)
```

### CHANGELOG 형식

```markdown
## [Unreleased]

### Added
- 한글 자모 분리/조합 유틸리티 추가 (#RST-008)
  - `decompose()`: 음절을 초/중/종성으로 분리
  - `compose()`: 자모를 음절로 조합
  - `has_jongseong()`: 받침 유무 확인

### Changed
- [변경 사항 설명] (#이슈번호)

### Fixed
- [버그 수정 설명] (#이슈번호)

### Deprecated
- [폐기 예정 기능] (#이슈번호)

### Removed
- [제거된 기능] (#이슈번호)

### Security
- [보안 관련 변경] (#이슈번호)
```

### 산출물
- 업데이트된 API 문서
- CHANGELOG 항목
- 사용자 가이드 업데이트 (필요시)

### 완료 기준
- [ ] rustdoc 빌드 성공
- [ ] CHANGELOG 업데이트됨
- [ ] 예제 코드 동작 확인
- [ ] 문서 리뷰 완료

### 상태 전환
```
Status: Documentation → Ready for Merge
Label: needs-docs 제거, ready-to-merge 추가
Comment: "[Docs Complete] 문서화 완료"
```

---

## Phase 7: PR Close (PR 종료)

### 목적
모든 검토가 완료된 PR을 병합하고 이슈를 종료합니다.

### 담당
- **Maintainer**: hephaex

### 입력
- 승인된 PR
- 검증 보고서
- 완성된 문서

### 프로세스

```yaml
Step 7.1: 최종 확인
  - [ ] 모든 CI 검사 통과
  - [ ] 리뷰어 승인 확인
  - [ ] 검증 완료 확인
  - [ ] 문서화 완료 확인

Step 7.2: 병합
  - [ ] Squash and Merge 선택
  - [ ] 커밋 메시지 정리
  - [ ] 병합 실행

Step 7.3: 후처리
  - [ ] 브랜치 삭제
  - [ ] 이슈 자동 종료 확인
  - [ ] 마일스톤 진행률 업데이트
```

### Squash Merge 메시지 템플릿

```
<type>(<scope>): <summary> (#PR번호)

<상세 설명>

Closes #이슈번호

Co-authored-by: ...
```

### 산출물
- 병합된 코드
- 종료된 이슈
- 삭제된 브랜치

### 완료 기준
- [ ] PR 병합됨
- [ ] 이슈 종료됨
- [ ] 브랜치 정리됨
- [ ] 마일스톤 업데이트됨

### 상태 전환
```
Issue Status: → Closed
PR Status: → Merged
Branch: → Deleted
```

---

## 긴급 핫픽스 프로세스

프로덕션 이슈의 경우 단축된 프로세스를 따릅니다:

```
┌──────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐
│  Issue   │──▶│ Develop │──▶│ Review  │──▶│  Merge  │
│ (P0 Bug) │   │ (Fast)  │   │ (Quick) │   │ (Direct)│
└──────────┘   └─────────┘   └─────────┘   └─────────┘
                                                │
                    ┌─────────┐   ┌─────────┐   │
                    │  Docs   │◀──│ Verify  │◀──┘
                    │(후순위) │   │(사후)   │
                    └─────────┘   └─────────┘
```

### 핫픽스 규칙

1. **브랜치**: `hotfix/critical-issue-description`
2. **리뷰**: 최소 1명의 빠른 리뷰
3. **테스트**: 관련 테스트만 실행
4. **문서**: 병합 후 업데이트 가능
5. **배포**: 즉시 패치 릴리스

---

## 자동화 및 도구

### GitHub Actions Workflows

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: cargo build --all-features
      - name: Test
        run: cargo test --all-features
      - name: Clippy
        run: cargo clippy -- -D warnings
      - name: Format
        run: cargo fmt --check
```

### 라벨 자동화

```yaml
# .github/labeler.yml
needs-analysis:
  - any: ['**/*']
    
needs-review:
  - any: ['**/*.rs']
    
documentation:
  - any: ['**/*.md', 'docs/**/*']
```

### 이슈 자동 종료

PR 본문에 다음 키워드 사용:
- `Closes #123`
- `Fixes #123`
- `Resolves #123`

---

## 참고 자료

- [AGENTS.md](./AGENTS.md) - 에이전트 역할 정의
- [CONTRIBUTING.md](./CONTRIBUTING.md) - 기여 가이드라인
- [PROJECT_PLAN.md](./PROJECT_PLAN.md) - 전체 프로젝트 계획

---

*Last Updated: 2026-01-04*  
*Maintainer: hephaex (hephaex@gmail.com)*
