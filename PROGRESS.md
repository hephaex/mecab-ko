# PROGRESS — mecab-ko Sprint 144 (UD GSD CI 5번째 게이트)

> 마지막 업데이트: 2026-05-20

## Sprint 144 A — accuracy-gate CI에 UD GSD 추가 (4 → 5 gate)

| Task | 상태 | 비고 |
|------|------|------|
| S144-A1: UD GSD step 추가 | ✅ 완료 | Sprint 140 패턴 재사용 |
| S144-A2: PR comment + summary 갱신 | ✅ 완료 | 5번째 섹션 + allPassed 확장 |
| S144-A3: actionlint 검증 | ✅ 완료 | 기존 SC2129 패턴 동일 (style only) |

## 5-gate 시스템 완성

| Gate | Dataset | Sentences | Floor | 도메인 |
|------|---------|-----------|-------|--------|
| 1 | sample.tsv | 1,100 | Token 99.9%+ | curated quality |
| 2 | KLUE DP morph | 1,995 | morph 60%, eo 15% | 뉴스/리뷰 |
| 3 | KLUE DP surface-only | 1,995 | strict 50%, canon 80% | 검색/색인 use case |
| 4 | UD Korean-Kaist | 1,638 | morph strict 60% | 역사/철학/학술 |
| 5 | **UD Korean-GSD** | **971** | morph strict 60% | Google news/web |

## 신규 코드

### `.github/workflows/accuracy-gate.yml`
- `Run UD GSD silver gate` step (test_ud_gsd_dual_metric 실행 + floor check)
- Final gate summary 갱신 (5-gate 표시)
- PR comment script (gsd* variables, 5번째 섹션, `5 accuracy gates` 메시지)

## 효과

**도메인 회귀 감지 강화**: Sprint 138 NNG cost 조정이 sample.tsv 회귀를 일으켰음. 만약 한 silver에만 영향이면 다른 silver는 통과 → cross-domain 회귀 trade-off 분리.

## 측정값 (변경 없음 — CI infra만)

| 메트릭 | Sprint 143 | Sprint 144 |
|--------|-----------|-----------|
| 모든 평가 메트릭 | 동일 | 동일 |
| CI gate 수 | 4 | **5** |

## 핵심 학습 포인트

### 1. Sprint 140 패턴 재사용 효율

silver gate 표준 패턴 (extract → floor check → status output → PR comment section) 그대로 복제. 0.5 sprint 통합 완료.

### 2. 5-gate가 cost 조정 회귀의 안전망

다중 silver gate가 cross-domain 회귀 trade-off 분리.

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: all pass / 0 fail
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- actionlint: warning 5개 (모두 기존과 동일 SC2129 style 패턴)

## 변경 파일

- `.github/workflows/accuracy-gate.yml`: UD GSD step + PR comment + summary
- `docs/research/accuracy/2026-05-20_sprint144_ud_gsd_ci_gate.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 145 후보

- B [메인]: Full CRF Retrain (Track E)
- C: NIKL Modu 추가
- D: 다른 mecab 결합 토큰 패턴
- E: UD Korean-PUD 추가 (또 다른 silver)
