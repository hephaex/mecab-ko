# Sprint 144 A — accuracy-gate CI에 UD GSD 추가 (4 → 5 gate)

> **결과**: Sprint 140 패턴 재사용으로 5번째 silver gate 통합. PR comment + final summary 갱신. 도메인 회귀 감지 능력 강화.

---

## 1. 변경 사항

### 1.1 신규 CI step

`.github/workflows/accuracy-gate.yml`에 `Run UD GSD silver gate` step 추가:

```yaml
- name: Run UD GSD silver gate
  id: udgsd
  run: |
    cd rust
    OUTPUT=$(cargo test --release -p mecab-ko-core --test accuracy_eval \
      -- --ignored --nocapture --exact test_ud_gsd_dual_metric 2>&1) || true
    # ... STRICT_MORPH/PRACTICAL_MORPH/STRICT_EO/PRACTICAL_EO 추출
    # ... CI-level floor check (60% morph)
    # ... status output (passed/failed/skipped)
```

### 1.2 Final gate summary 확장

5-gate 모두 표시:

```
=================================
  ACCURACY GATE RESULT (5-gate)
=================================
Sample.tsv:    Token / Sentence
KLUE strict:   morph / eojeol
KLUE pract:    morph / eojeol
Surface-only:  strict / canonical / lenient
UD Kaist:      strict morph / practical / strict eo
UD GSD:        strict morph / practical / strict eo
Status: sample / klue / surface / udkaist / udgsd
```

### 1.3 PR comment 5번째 섹션

```markdown
### {gsdEmoji} UD Korean-GSD Silver (Sprint 143 C, 971 sentences)
| Mode | Morpheme | Eojeol |
|------|----------|--------|
| Strict | **${gsdStrictMorph}%** | **${gsdStrictEo}%** |
| Practical (POS lenient) | **${gsdPracMorph}%** | **${gsdPracEo}%** |

**Floors**: morph strict ≥ 60% (silver)
ℹ️ Silver dataset: GSD XPOS는 Sejong 직접 사용 (identity mapping, 98.2% 변환률)
도메인: Google news/web (KLUE에 가장 가까움)
```

---

## 2. 5-gate 시스템

| Gate | Dataset | Sentences | Floor | 도메인 |
|------|---------|-----------|-------|--------|
| 1 | sample.tsv | 1,100 | Token 99.9%+ | curated quality |
| 2 | KLUE DP morph | 1,995 | morph 60%, eo 15% | 뉴스/리뷰 |
| 3 | KLUE DP surface-only | 1,995 | strict 50%, canon 80% | 검색/색인 use case |
| 4 | **UD Korean-Kaist** | 1,638 | morph strict 60% | 역사/철학/학술 |
| 5 | **UD Korean-GSD** [신규] | 971 | morph strict 60% | Google news/web |

### 도메인 다양성 강화

기존 4-gate는 KLUE + UD Kaist로 뉴스+학술 cover. 신규 GSD는 Google news/web 도메인 — KLUE와 유사하나 다른 source.

**Sprint 138 같은 회귀 감지 강화**: 한 도메인에서만 회귀하는 경우 (예: 뉴스에는 영향 적고 학술에 큰 영향) 다중 silver로 격리 가능.

---

## 3. actionlint 검증

shellcheck warnings는 기존 step과 동일 패턴 (SC2129 style — `{ cmd1; cmd2; } >> file` 권장). 새 step만의 추가 에러 없음. style 일관성 유지.

```
SC2129:style:23:1 — Consider using { cmd1; cmd2; } >> file instead of individual redirects
```

기존 step들도 모두 같은 패턴 사용 → 변경 없이 유지.

---

## 4. 측정값 (변경 없음 — CI infra 확장만)

| 메트릭 | Sprint 143 | Sprint 144 |
|--------|-----------|-----------|
| 모든 평가 메트릭 | 동일 | 동일 |
| **CI gate 수** | 4 | **5** |

---

## 5. 핵심 학습 포인트

### 5.1 Sprint 140 패턴 재사용 효율

Sprint 140에서 정립한 silver gate 패턴 (extract → floor check → status output → PR comment section)을 그대로 복제. 0.5 sprint로 통합 완료. 표준화의 가치.

### 5.2 5-gate가 cost 조정 회귀의 안전망

Sprint 138 NNG cost 조정이 sample.tsv 회귀를 일으켰고, KLUE/UD 양쪽도 영향 받음. 만약 cost 조정이 한 도메인에만 영향이면 다른 silver는 통과 → 안전 격리. **다중 silver gate가 cross-domain 회귀 trade-off 분리**.

---

## 6. Sprint 145 후보

### 후보 B [메인]: Full CRF Retrain (Track E)
3-5 sprint. 학습 데이터 (KLUE train + UD Kaist train + UD GSD train) 풀.

### 후보 C: NIKL Modu 수동 다운로드
Academic license, 로컬 only.

### 후보 D: 다른 mecab 결합 토큰 패턴 (Sprint 141 연장)

### 후보 E: UD Korean-PUD 추가
또 다른 silver source (1,000 sentences).

---

## 7. 변경 파일

- `.github/workflows/accuracy-gate.yml`:
  - `Run UD GSD silver gate` step (5번째 gate)
  - `Final gate summary` 갱신 (5-gate 표시)
  - PR comment script 갱신 (gsd* variables, 5번째 섹션, `allPassed` 확장)
- `docs/research/accuracy/2026-05-20_sprint144_ud_gsd_ci_gate.md` (본 문서)
- `PLAN.md`, `PROGRESS.md` 갱신

---

*작성: 2026-05-20 (Sprint 144 A)*
