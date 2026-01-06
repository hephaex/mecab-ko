# Publishing Documentation Index

crates.io 배포 관련 모든 문서와 스크립트의 인덱스입니다.

## 문서 구조

```
/home/mare/mecab-ko/rust/
├── PUBLISHING_INDEX.md           ← 이 파일 (문서 인덱스)
├── QUICK_PUBLISH.md              ← 빠른 참조 가이드
├── PUBLISHING_WORKFLOW.md        ← 배포 플로우 시각화
├── PUBLISHING_CHECKLIST.md       ← 실무 체크리스트
├── PUBLISHING.md                 ← 종합 배포 가이드
├── QA-005-SUMMARY.md             ← 작업 완료 요약
├── scripts/
│   ├── publish.sh                ← 자동 배포 스크립트
│   └── toggle-deps.sh            ← 의존성 전환 스크립트
└── crates/
    ├── mecab-ko-hangul/README.md ← 크레이트별 README
    ├── mecab-ko-dict/README.md
    ├── mecab-ko-core/README.md
    ├── mecab-ko-dict-builder/README.md
    ├── mecab-ko-cli/README.md
    └── mecab-ko/README.md
```

## 문서별 용도

### 1. QUICK_PUBLISH.md
**대상**: 빠르게 배포 명령을 찾는 개발자
**내용**:
- 한 줄 명령어
- 배포 순서
- 기본 체크리스트
- 주요 명령어만

**언제 읽나요?**
- 배포 명령을 빠르게 확인하고 싶을 때
- 자주 사용하는 명령어를 참조할 때

### 2. PUBLISHING_WORKFLOW.md
**대상**: 배포 프로세스를 이해하려는 개발자
**내용**:
- 의존성 그래프 시각화
- 배포 단계별 플로우 다이어그램
- 에러 처리 흐름
- 타임라인 예상
- 체크포인트

**언제 읽나요?**
- 처음 배포를 준비할 때
- 배포 프로세스 전체를 이해하고 싶을 때
- 각 단계의 순서와 의미를 파악하고 싶을 때

### 3. PUBLISHING_CHECKLIST.md
**대상**: 실제 배포를 수행하는 개발자
**내용**:
- 배포 전 필수 작업 목록
- Cargo.toml 수정 가이드 (각 크레이트별)
- Git 작업 절차
- 배포 후 확인 사항
- 문제 해결 팁

**언제 읽나요?**
- 실제 배포를 수행할 때
- 각 단계를 체크하면서 진행할 때
- 배포 후 확인 작업을 할 때

### 4. PUBLISHING.md
**대상**: 모든 개발자 (종합 레퍼런스)
**내용**:
- 배포 순서 상세 설명
- 버전 관리 전략
- 배포 전 체크리스트 (상세)
- 배포 명령어 모음
- 배포 후 작업
- 문제 해결 가이드
- CI/CD 자동화
- 참고 자료

**언제 읽나요?**
- 배포 관련 모든 정보를 찾고 싶을 때
- 특정 문제를 해결하고 싶을 때
- 배포 정책이나 전략을 이해하고 싶을 때

### 5. QA-005-SUMMARY.md
**대상**: 프로젝트 관리자, 리뷰어
**내용**:
- QA-005 이슈 작업 완료 보고서
- 완료된 작업 목록
- 크레이트별 배포 준비 상태
- 생성된 파일 목록
- 다음 단계

**언제 읽나요?**
- QA-005 작업 내용을 확인할 때
- 배포 준비 상태를 파악할 때
- 작업 완료 여부를 검토할 때

### 6. PUBLISHING_INDEX.md (이 파일)
**대상**: 모든 사용자
**내용**:
- 문서 구조
- 각 문서의 용도
- 시나리오별 가이드

**언제 읽나요?**
- 어떤 문서를 읽어야 할지 모를 때
- 문서 전체 구조를 파악하고 싶을 때

## 스크립트 사용법

### scripts/publish.sh
**용도**: 크레이트 자동 배포

**주요 기능**:
- 의존성 순서대로 자동 배포
- Dry-run 모드 지원
- 버전 검증
- 테스트, Clippy, 문서 자동 확인
- 에러 시 즉시 중단

**옵션**:
```bash
--dry-run           # 실제 배포 없이 테스트
--version VER       # 배포 버전 지정
--skip-tests        # 테스트 건너뛰기 (비권장)
--help              # 도움말
```

**사용 예제**:
```bash
# 도움말 확인
./scripts/publish.sh --help

# Dry-run 테스트
./scripts/publish.sh --dry-run --version 0.1.0

# 실제 배포
./scripts/publish.sh --version 0.1.0
```

### scripts/toggle-deps.sh
**용도**: path ↔ version 의존성 전환

**모드**:
- `path`: version → path (개발용)
- `version`: path → version (배포용)

**사용 예제**:
```bash
# 배포 준비 (path → version)
./scripts/toggle-deps.sh version 0.1.0

# 개발 모드로 복귀 (version → path)
./scripts/toggle-deps.sh path
```

## 시나리오별 가이드

### 처음 배포하는 경우

1. **PUBLISHING_WORKFLOW.md** 읽기
   - 전체 프로세스 이해
   - 각 단계의 의미 파악

2. **PUBLISHING_CHECKLIST.md** 참조하며 작업
   - 체크리스트 따라가기
   - 각 단계 확인

3. **스크립트 사용**
   ```bash
   ./scripts/toggle-deps.sh version 0.1.0
   ./scripts/publish.sh --dry-run --version 0.1.0
   ./scripts/publish.sh --version 0.1.0
   ```

### 급하게 배포 명령만 필요한 경우

1. **QUICK_PUBLISH.md** 열기
2. 명령어 복사 & 실행

### 배포 중 문제가 발생한 경우

1. **PUBLISHING.md** → 문제 해결 섹션
2. 에러 메시지 확인
3. 해당 섹션의 해결 방법 따라하기

### 배포 정책을 이해하고 싶은 경우

1. **PUBLISHING.md** → 버전 관리 전략 섹션
2. SemVer, 의존성 관리 정책 확인

### 작업 완료 상태를 확인하는 경우

1. **QA-005-SUMMARY.md** 읽기
2. 완료된 작업 및 배포 준비 상태 확인

## 크레이트별 README

각 크레이트의 README는 다음 정보를 포함합니다:

- **mecab-ko-hangul**: 한글 처리 유틸리티, 자모 분리/결합
- **mecab-ko-dict**: 사전 관리, FST 검색, 연접 비용
- **mecab-ko-core**: 형태소 분석 엔진, Viterbi 알고리즘
- **mecab-ko-dict-builder**: CSV → 바이너리 사전 변환
- **mecab-ko-cli**: 명령줄 형태소 분석 도구
- **mecab-ko**: 통합 라이브러리 (facade)

## 추가 리소스

### 외부 문서
- [The Cargo Book - Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [crates.io Policies](https://crates.io/policies)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Semantic Versioning](https://semver.org/)

### 프로젝트 문서
- `docs/PROJECT_PLAN.md` - 24주 로드맵
- `docs/ISSUE_BACKLOG.md` - 이슈 백로그
- `docs/AGENTS.md` - 멀티 에이전트 시스템

## 배포 체크리스트 요약

배포 전:
- [ ] 구현 완료
- [ ] 테스트 통과
- [ ] 문서 작성
- [ ] 의존성 전환

배포:
- [ ] Dry-run
- [ ] 실제 배포
- [ ] 확인

배포 후:
- [ ] crates.io 확인
- [ ] docs.rs 확인
- [ ] 개발 모드 복귀

## 문의

배포 관련 문제나 질문이 있으면:
- GitHub Issues: https://github.com/hephaex/mecab-ko/issues
- Repository: https://github.com/hephaex/mecab-ko
