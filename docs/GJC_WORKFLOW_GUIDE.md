# GJC 워크플로우 가이드 — mecab-ko 프로젝트
> 작성: 2026-06-25 | /skill:review-project 리뷰 결과물

---

## 개요

이 가이드는 mecab-ko 프로젝트에서 GJC 에이전트를 **효과적으로** 사용하는 방법을 설명합니다.
핵심 원칙: **메모리 먼저 → Opus 리뷰 → Sonnet 실행** 

---

## 1. 프로젝트 리뷰 커맨드

### 사용법

```bash
# 세션 시작 시 프로젝트 전체 리뷰
gjc
> /skill:review-project

# 특정 서브시스템만 리뷰
gjc
> /skill:review-project rust/crates/mecab-ko-core/src/viterbi
```

### 스킬 위치

```
~/.gjc/agent/skills/review-project/SKILL.md
```

### 리뷰 흐름

```
메모리 게이트 → 프로젝트 조사 → Opus 아키텍처 리뷰 → 메모리 저장 → 하이라이트 추출
```

---

## 2. 메모리 시스템

### 구조

```
~/.gjc/memory/
├── MEMORY.md                          # 인덱스 (모든 항목의 한 줄 요약)
├── project_mecab_ko.md                # mecab-ko 전체 프로젝트 메모리
├── project_mecab_ko_dict_publish.md   # NNP 사전 자동 발행 파이프라인
├── feedback_*.md                      # 행동 규칙 피드백
└── reference_*.md                     # 인프라/기술 참조
```

### 프로젝트 메모리 (`project_mecab_ko.md`) 포함 항목

- 아키텍처 요약 (크레이트 맵, 파이프라인)
- 성능/정확도 지표
- 위험 영역 (Danger Zones)
- 인프라 정보 (CT118, deploy key, cron)
- Opus 판정 (`verdict: WATCH`)
- Sprint 재개 트리거

### 메모리 조회 방법

```bash
# 프로젝트 메모리 확인
cat ~/.gjc/memory/project_mecab_ko.md

# 인덱스에서 빠른 확인
grep "mecab-ko" ~/.gjc/memory/MEMORY.md

# 인프라 관련 확인
cat ~/.gjc/memory/project_mecab_ko_dict_publish.md
```

---

## 3. 인프라 작업 전 필수 절차

**CT118, deploy key, cron, PAT 관련 작업 시 반드시 먼저 메모리 확인:**

```bash
# 1. 프로젝트 메모리 확인
cat ~/.gjc/memory/project_mecab_ko.md

# 2. 인프라 상세 확인
cat ~/.gjc/memory/project_mecab_ko_dict_publish.md

# 3. 하이라이트 확인
cat docs/REVIEW_HIGHLIGHTS.md
```

**이유**: CT118 서버 IP/포트, deploy key 위치, PAT 경로, cron 스케줄이 메모리에 있음. 묻지 말고 읽을 것.

---

## 4. 모델 사용 정책

`~/.gjc/memory/feedback_model_usage_policy.md` 기반:

| 작업 유형 | 모델 | GJC 커맨드 |
|----------|------|-----------|
| 코드 리뷰, 아키텍처 분석 | **Opus** | `gjc --model opus` 또는 `task(agent: "architect")` |
| 코드 작성, 버그 수정 | **Sonnet** | 기본값 |
| 메모리/파일 갱신 | **Sonnet** | 기본값 |
| 단순 변경 | **Haiku** | `gjc --model haiku` |

**Phase 게이트 핸드오프 패턴 (대규모 작업)**:
```
Opus(설계·리뷰) ──PLAN.md계약──▶ Sonnet(구현 자율 실행) ──▶ Opus(Phase 경계 리뷰)
```

---

## 5. 자주 쓰는 GJC 커맨드

### 프로젝트 컨텍스트 진입

```bash
cd ~/Simon/mecab-ko
gjc --continue          # 이전 세션 이어서
gjc -c "viterbi 리뷰"  # 특정 주제로 이어서
```

### 리뷰 스킬

```bash
# 전체 프로젝트 리뷰 (Opus 아키텍처 분석 포함)
gjc
> /skill:review-project

# Rust 심화 가이드 (소유권, async, 에러 처리)
gjc
> /skill:rust
```

### 계획/분석 워크플로우

```bash
# 복잡한 기능 계획 (Planner → Architect → Critic 합의)
gjc
> /skill:ralplan

# 요구사항 불분명할 때 (Socratic 인터뷰)
gjc
> /skill:deep-interview
```

---

## 6. 위험 작업 전 체크리스트

### Viterbi/Matrix 변경 전

```bash
# 1. 현재 상태 확인
cargo test --workspace --exclude mecab-ko-ffi --lib

# 2. Silver 3개 baseline 측정
cargo bench --manifest-path rust/Cargo.toml -- tokenizer_baseline

# 3. 변경 후 즉시 재측정
# 5% 이상 회귀 → 즉시 rollback
```

### LazyEntries V3 관련 작업 전

```bash
# dictionary.rs:308-316 silent fallback 여부 확인
# tracing::warn 로그 확인
RUST_LOG=warn cargo run -- tokenize "테스트"
```

### N-best API 사용 전 (현재 알려진 버그)

```
주의: nbest.rs:360 에 clamp_oob_cost 누락
→ OOV 포함 문장에서 nbest[0] ≠ 1-best 불일치 가능
→ 수정 전까지 OOV 다량 포함 텍스트에서 N-best 결과 신뢰 주의
```

---

## 7. CI/CD 5-gate 통과 기준

```bash
# 전체 CI 로컬 시뮬레이션
cargo test --workspace --exclude mecab-ko-ffi --lib  # accuracy-gate
cargo clippy --workspace -- -D warnings               # code-quality
cargo audit                                            # security
cargo bench --quick                                    # benchmark (빠른 확인)
```

**PRO TIP**: `cargo test` 실패 시 PR 올리지 말 것 — 5-gate 중 1개라도 실패하면 CI 블록됨.

---

## 8. 메모리 저장 커맨드 (수동)

리뷰 스킬 없이 수동으로 메모리 업데이트:

```bash
# 프로젝트 메모리 편집
gjc
> 메모리에 저장해줘: [내용]

# 또는 직접 편집
vi ~/.gjc/memory/project_mecab_ko.md

# MEMORY.md 인덱스도 동기화
vi ~/.gjc/memory/MEMORY.md
```

---

## 9. 다음 리뷰 권장 시점

| 이벤트 | 리뷰 범위 |
|--------|----------|
| NIKL Modu 코퍼스 입수 | 정확도 sprint 계획 + viterbi 위험 재평가 |
| Sejong 코퍼스 Track B 재시도 | CRF retrain 위험 영역 집중 리뷰 |
| v1.0 릴리즈 전 | facade wildcard re-export + API 동결 |
| PAT 갱신 (2026-09-07) | CT118 인프라 상태 점검 |
| 새 바인딩 추가 (Java/Ruby 등) | FFI safety 패턴 리뷰 |

---

## 10. 참고 파일

```
docs/REVIEW_HIGHLIGHTS.md          # 이번 리뷰 핵심 요약 (인간 독자용)
~/.gjc/memory/project_mecab_ko.md  # 전체 프로젝트 메모리 (에이전트용)
~/.gjc/memory/MEMORY.md            # 전체 메모리 인덱스
~/.gjc/agent/skills/review-project/SKILL.md  # 리뷰 스킬 정의
docs/PERFORMANCE_BASELINES.md      # 성능 baseline 상세
docs/AGENTS.md                     # 에이전트 역할 정의
PLAN.md                            # Sprint 현황
PROGRESS.md                        # Sprint 이력
```
