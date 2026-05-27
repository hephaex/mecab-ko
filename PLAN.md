# PLAN — mecab-ko Sprint 169+ (정확도 lift cycle 종료 후)

> 마지막 업데이트: 2026-05-27

## 완료: Sprint 168 — Track B 공식 종료

- 사용자 confirm: Option D 채택
- Track B (Full CRF Retrain) 종료
- 정확도 lift sprint cycle 마무리

## 정확도 Sprint Cycle 종합 (Sprint 122 → 167)

### 최종 누적 지표

| Metric | Baseline (S122) | 현재 (S167) | Δ |
|--------|----------------|------------|---|
| sample.tsv | 100%/99.9% | 100%/99.9% | — (보존) |
| **KLUE morph practical** | ~65.8% | **72.1%** | **+6.3pp** |
| KLUE eojeol practical | ~5000 | 5327 | +327 |
| **KLUE surface canonical_lenient** | ~89% | **95.6%** | **+6pp** |
| UD Kaist morph practical | — | 68.6% | (new) |
| UD GSD morph practical | — | 71.8% | (new) |
| accuracy_eval.rs 줄 수 | 4963 | 2406 | -51% |

### 검증된 안전/위험 영역

| 영역 | 결과 |
|------|------|
| ✅ Surface normalization | 누적 +6pp |
| ✅ PRACTICAL 동치 | 누적 +6.3pp |
| ✅ Silver dataset | 5-gate CI 완성 |
| ❌ Splitter rule | 영역 소진 |
| ❌ Dict cost 확장 | 회귀 |
| ❌ CRF matrix 수동 조정 | 회귀 |
| ❌ CRF Full Retrain | Track B 종료 |

## Sprint 169+ 옵션 (사용자 결정 필요)

### 시나리오 A: NIKL Modu 다운로드 완료 후 측정

사용자가 https://kli.korean.go.kr 학술 등록 + 다운로드 완료 시:
```bash
./tools/nikl_modu_setup.sh ~/Korpora/NIKL_MP/NXMP*.json
```
→ 자동 측정 + 분석. 새 도메인 (구어/SNS/문어) 정확도 확인.

### 시나리오 B: 정확도 외 영역 전환

다음 중 사용자 우선순위 결정 필요:

#### B-1: 언어 바인딩 강화
- Python (mecab-ko-python) 통합 보강
- WASM (mecab-ko-wasm) 사용성
- Node (mecab-ko-node) 안정성

#### B-2: 성능 최적화
- 프로파일링 (mecab-ko-profiler 활용)
- 핫스팟 식별 + 최적화
- 측정 + 보고 sprint

#### B-3: 사용자 기능 추가
- CLI 추가 옵션
- API 개선
- 신규 기능 (사용자 피드백 기반)

#### B-4: 유지보수 모드
- 정확도 sprint 종료
- 버그 픽스, 의존성 업데이트만
- 다음 메이저 작업 대기

### 시나리오 C: Sejong 코퍼스 입수 시 Track B 재시도

- 국립국어원 또는 KAIST 학술 입수
- Sprint 164~167 파이프라인 즉시 재활용
- 잠재 lift +1~5pp (정확한 학습 데이터로)

## 결정 프로세스

규칙 5 (자동 트랙 선택)이 정확도 영역에 적용. 정확도 외 영역은 사용자 우선순위 결정 필요.

## 검증 기준 (모든 시나리오 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과**
- sample.tsv baseline 100%/99.9% **회귀 금지**
