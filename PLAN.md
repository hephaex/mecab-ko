# 현재 스프린트: Phase 17 - Sprint 28 (정확도 45% 목표)

## 목표
정확도 45% 달성, 저정확도 품사 집중 개선 (JKS, NNB, ETM)

## Sprint 28 작업 목록

### P0 (Critical)
- [ ] S28-01: 정확도 45% 달성
  - 현재: 40.0% (300문장, 세종 모드)
  - 목표: 45%+
  - 접근: JKS/NNB/ETM 개선, 공백 토큰 처리
- [ ] S28-02: npm mecab-ko-wasm v0.4.0 배포 (npm 토큰 필요)
  - npm 토큰 확보 및 publish 실행

### P1 (High)
- [ ] S28-03: JKS (주격조사) 정확도 개선
  - 현재: 23.1%
  - 목표: 40%+
  - 공백 연접 문제 해결 필요
- [ ] S28-04: NNB (의존명사) 정확도 개선
  - 현재: 21.7%
  - 목표: 35%+

### P2 (Medium)
- [ ] S28-05: ETM (관형형어미) 정확도 개선
  - 현재: 25.8%
  - 목표: 40%+
- [ ] S28-06: VX (보조용언) 정확도 추가 개선
  - 현재: 25.7%
  - 목표: 35%+

### P3 (Low)
- [ ] S28-07: PyPI 실제 배포 (토큰 확보 시)
- [ ] S28-08: 문서 사이트 v0.5.0 갱신

---

# 완료된 스프린트: Phase 16 - Sprint 27 (정확도 40% 달성) ✅

## 목표 (100% 달성)
정확도 40% 달성 ✅ (40.0%)

## Sprint 27 작업 목록 (완료)

### P0 (Critical) ✅
- [x] S27-01: 정확도 40% 달성 ✅
  - 시작: 35.2% → 최종: **40.0%** (+4.8%p)
  - EC 복원 로직, 복합명사 패턴, XSV 보정 수정
- [→] S27-02: npm mecab-ko-wasm v0.4.0 배포 → Sprint 28 이월

### P1 (High) ✅
- [x] S27-03: MAG (부사) 정확도 개선 ✅
  - 22.6% → 54.8% (+32.2%p)
- [x] S27-04: VX (보조용언) 정확도 개선 ✅
  - 20.0% → 25.7% (+5.7%p)

---

# 완료된 스프린트: Phase 15 - Sprint 26 (정확도 35% 달성) ✅

## 목표 (100% 달성 🎉)
정확도 35% 달성 ✅ (35.2%), 축약형 동사 처리 구현

## Sprint 26 작업 목록 (완료)

### P0 (Critical) ✅
- [x] S26-01: 정확도 35% 달성 ✅ 🎉
  - 시작: 30.4% → 최종: **35.2%** (+4.8%p)
  - 축약형 동사 분리 로직 추가 (try_split_contracted)
  - 사용자 사전 247개 패턴으로 확장
- [→] S26-02: npm mecab-ko-wasm v0.4.0 배포 → Sprint 27 이월

### P1 (High) ✅
- [x] S26-03: XSV (파생접미사) 정확도 개선 ✅
  - 10.9% → 17.2% (+6.3%p)
- [x] S26-04: VX (보조용언) 정확도 개선 ✅
  - 8.6% → 20.0% (+11.4%p)
- [x] S26-05: EP (선어말어미) 정확도 개선 ✅
  - 15.0% → 21.2% (+6.2%p)

---

# 완료된 스프린트: Phase 14 - Sprint 25 (정확도 30% & npm 배포) ✅

## 목표 (100% 달성 🎉)
정확도 30% 달성 ✅ (30.4%), npm WASM 준비 완료

## Sprint 25 작업 목록 (완료)

### P0 (Critical) ✅
- [x] S25-01: 정확도 30% 달성 ✅ 🎉
  - 시작: 23.8% → 최종: **30.4%** (+6.6%p)
  - SpacePenalty 버그 수정 (c0edcb2)
  - 사용자 사전 149개 패턴 추가 (538815b)
- [x] S25-02: npm mecab-ko-wasm v0.4.0 준비 ✅
  - package.json 업데이트 완료
  - npm 토큰 대기 중 → Sprint 26 이월

### P1 (High) ✅
- [x] S25-03: ETM (관형형어미) 정확도 개선 ✅
  - 0% → 29.0% (+29.0%p)
- [x] S25-04: EC (연결어미) 정확도 개선 ✅
  - 14.5% → 22.2% (+7.7%p)

### P2 (Medium) ✅
- [x] S25-05: 사용자 사전으로 동사 활용형 보정 ✅
  - data/user-dict/verb-inflections.csv (149개)
- [x] S25-06: 신조어 자동 수집 파이프라인 ✅
  - .github/workflows/neologism-sync.yml

### P3 (Low)
- [→] S25-07: PyPI 실제 배포 → Sprint 26 이월
- [ ] S25-08: GitHub Discussions 설정 (미완료)

---

# 완료된 스프린트: Phase 13 - Sprint 24 (v0.4.0 릴리스 & 사전 품질 개선) ✅

## 목표 (87.5% 달성)
v0.4.0 crates.io 정식 릴리스 ✅, 사전 품질 근본 개선 시작 ✅

## Sprint 24 작업 목록 (완료)

### P0 (Critical) ✅
- [x] S24-01: v0.4.0 crates.io 정식 릴리스 ✅
  - 6개 크레이트 순서대로 배포 완료 🎉
- [x] S24-02: mecab-ko-dic 품질 분석 ✅

### P1 (High) ✅
- [x] S24-03: CI 자동 정확도 측정 ✅
- [x] S24-04: 평가 데이터셋 확장 ✅ (300문장)

### P2 (Medium) ⚠️
- [→] S24-05: mecab-ko-dic 비용값 튜닝 → Sprint 25 이월
- [x] S24-06: 문서 사이트 v0.4.0 업데이트 ✅

### P3 (Low) ✅
- [x] S24-07: PyPI 배포 준비 ✅
- [x] S24-08: 커뮤니티 기능 요청 검토 ✅

---

# 완료된 스프린트: Phase 12 - Sprint 23 (정확도 50% & crates.io 배포) ✅

## 목표 (87.5% 달성)
정확도 50% 달성 → **29.6% 달성** (사전 한계), crates.io 배포 준비 ✅

## Sprint 23 작업 목록 (완료)

### P0 (Critical) ✅
- [x] S23-01: crates.io 배포 준비 ✅
- [⚠️] S23-02: 정확도 45% 달성 → **29.6% 달성** (사전 품질 한계)

### P1 (High) ✅
- [x] S23-03: 어미 분리 로직 강화 ✅
- [x] S23-04: 고유명사 사전 확장 ✅ (~200개)

### P2 (Medium) ✅
- [x] S23-05: 성능 벤치마크 실행 ✅
- [x] S23-06: 테스트 커버리지 개선 ✅ (1060개)

### P3 (Low) ⚠️
- [→] S23-07: CI/CD 개선 → Sprint 24로 이월
- [x] S23-08: CHANGELOG 업데이트 ✅

---

# 완료된 스프린트: Phase 11 - Sprint 22 (v0.4.0 릴리스 & 정확도 30% 달성) ✅

## 목표 (완료)
v0.4.0 릴리스, 정확도 30% 달성 ✅ (16.8% → 35.4% 달성!)

## Sprint 22 작업 목록 (완료)

### P0 (Critical) ✅
- [x] S22-01: v0.4.0 버전 업그레이드
- [x] S22-02: 정확도 측정 및 분석

### P1 (High) ✅
- [x] S22-03: 조사 분리 로직 구현 (35.4% 달성!)
- [x] S22-04: mecab-ko-dic v3.0 Phase 2

### P2 (Medium) ✅
- [x] S22-05: 벤치마크 대시보드 개선
- [x] S22-06: CLI 개선 (--decomp, --sejong)

### P3 (Low) ✅
- [x] S22-07: 문서 업데이트
- [x] S22-08: 커뮤니티 피드백 반영

---

# 완료된 스프린트: Phase 10 - Sprint 21 (정확도 향상 & v0.4.0 준비) ✅

## 목표 (완료)
어미 분리 로직 구현, mecab-ko-dic v3.0 Phase 1, 복합명사 분해 개선

## Sprint 21 작업 목록 (완료)

### P0 (Critical)
- [x] S21-01: 어미 분리 로직 구현 ✅
  - DecomposedMorpheme 구조체 추가
  - mecab-ko-dic 12번째 컬럼 활용 (분석결과)
  - 25개 테스트 통과
  - 커밋: 038f775
- [ ] S21-02: PyPI 배포 - BLOCKED (토큰 필요)

### P1 (High)
- [x] S21-03: mecab-ko-dic v3.0 Phase 1 ✅ (511개 신조어)
- [x] S21-04: 불규칙 활용 사전 구축 ✅ (7가지 패턴 문서화)

### P2 (Medium)
- [x] S21-05: 복합명사 분해 정확도 향상 ✅
  - COMPOUND_DICT 50+ 패턴
  - PREFIXES 23개, SUFFIXES 27개
- [x] S21-06: 성능 회귀 테스트 ✅

### P3 (Low)
- [x] S21-07: v0.4.0 CHANGELOG 준비 ✅
- [ ] S21-08: 커뮤니티 피드백 반영

---

# 완료된 스프린트: Phase 9 - Sprint 20 (정확도 개선 & 현대화) ✅

## 목표 (완료)
세종 코퍼스 호환 모드로 정확도 향상, mecab-ko-dic v3.0 계획, PyPI 배포

## Sprint 20 작업 목록

### P0 (Critical)
- [ ] S20-01: PyPI 배포 - BLOCKED (토큰 필요) → S21-02
- [x] S20-02: 세종 코퍼스 호환 모드 ✅
  - `sejong.rs` 모듈: 복합 태그 분리, 어미 분리 규칙
  - 16개 단위 테스트, format_sejong() 출력

### P1 (High)
- [x] S20-03: mecab-ko-dic v3.0 현대화 계획 ✅
  - docs/research/dictionary/mecab-ko-dic-v3.0-plan.md 작성
  - 목표: 816K → 1M+ 엔트리, Token Accuracy 50%+
  - 로드맵: Phase 1-4 (Sprint 20-26)
- [x] S20-04: 신조어 자동 수집 실행 ✅
  - OpenDict API 연동 수정 (User-Agent, 응답 파싱)
  - neologism-sync.yml 워크플로우 수정 및 동작 확인
  - 커밋: 6d038b2

### P2 (Medium)
- [x] S20-05: v0.3.1 릴리스 ✅
  - workspace 버전 0.3.1
  - CHANGELOG 업데이트
  - 235개 테스트 통과
- [x] S20-06: 정확도 개선 측정 ✅
  - CLI --sejong 옵션 추가
  - Token Accuracy: 15.2% → 16.8% (+1.6%p)
  - 복합 태그 분리만으로는 한계, 실제 어미 분리 로직 필요

### P3 (Low)
- [ ] S20-07: 커뮤니티 피드백 통합 - 스킵 (P3)
- [x] S20-08: 문서 사이트 업데이트 ✅
  - sejong 모듈 문서 추가
  - 정확도 개선 결과 반영

---

# 완료된 스프린트: Phase 8 - Sprint 17 (v0.3.0 릴리스 & API 개선) ✅

## 목표
v0.3.0 정식 릴리스, 스트리밍 API 개선, 메모리 최적화, PyPI 배포

## Sprint 17 작업 목록

### P0 (Critical)
- [x] S17-01: v0.3.0 정식 릴리스 ✅
  - GitHub Release v0.3.0 생성
  - 릴리스 노트 작성
  - crates.io 6개 크레이트 v0.3.0 발행 완료

### P1 (High)
- [ ] S17-02: PyPI 배포 (S16-03 계속)
  - PyPI 토큰 설정 후 배포
  - mecab-ko-python v0.3.0
- [x] S17-03: 스트리밍 API 개선 ✅
  - `TokenStream` VecDeque 최적화 (O(1) dequeue)
  - `ProgressStreamingTokenizer` 진행률 콜백
  - `LargeFileProcessor` 대용량 파일 스트리밍
  - 스마트 문장 경계 청킹
- [x] S17-04: Migration Guide v0.2.0 → v0.3.0 ✅
  - Breaking changes 문서화
  - 코드 예제 업데이트
  - 업그레이드 가이드

### P2 (Medium)
- [x] S17-05: 메모리 최적화 2차 ✅
  - PosTagInterner: 품사 태그 String interning
  - FeatureCache: Feature 문자열 중복 제거
  - MemoryStats: 메모리 사용량 추적 인프라
  - Lattice.memory_usage() 메서드 추가
  - Tokenizer.memory_stats() 메서드 추가
- [x] S17-06: API 문서 개선 ✅
  - lib.rs 모듈 문서 대폭 개선 (기능 목록, 예제, 모듈 구조표)
  - tokenizer.rs 메서드 문서 보강 (wakati, morphs, pos, set_user_dict)
  - nbest.rs HTML 태그 경고 수정
  - memory.rs 문서 예제 수정
  - 52개 문서 테스트 통과
- [x] S17-07: 벤치마크 결과 정리 ✅
  - docs/BENCHMARK_DASHBOARD_v0.3.0.md 생성
  - v0.2.0 대비 3x+ 성능 개선 문서화
  - PERFORMANCE_BASELINES.md v0.3.0 기준선 업데이트

### P3 (Low)
- [x] S17-08: 테스트 커버리지 향상 ✅
  - edge_cases.rs: 47개 edge case 테스트 추가
  - integration_batch.rs: test_batch_chunked 수정
  - 총 54개 테스트 통과 (47 edge + 7 batch)

---

# 완료된 스프린트: Phase 7 - Sprint 16 (고급 토큰화 & N-best) ✅

## 목표 (완료)
N-best 경로 탐색 개선, 사용자 정의 분석 모드, 토큰화 성능 최적화

## Sprint 16 작업 목록

### P0 완료
- [x] S16-01: N-best 경로 탐색 개선 ✅

### P1 완료/BLOCKED
- [x] S16-02: 사용자 정의 분석 모드 ✅
- [ ] S16-03: PyPI 배포 - BLOCKED → S17-02
- [x] S16-04: npm 배포 ✅ (mecab-ko-wasm v0.3.0)

### P2 완료
- [x] S16-05: Lattice 시각화 도구 ✅
- [x] S16-06: 토큰화 캐싱 ✅
- [x] S16-07: 병렬 토큰화 ✅

### P3 완료
- [x] S16-08: v0.3.0 준비 ✅

---

# 완료된 스프린트: Phase 7 - Sprint 15 (사전 품질 & 정확도) ✅

## 목표 (완료)
정확도 측정 인프라 구축, 사전 품질 개선, Unknown 단어 처리 강화

## Sprint 15 작업 목록

### P0 완료
- [x] S15-01: 정확도 측정 인프라 구축 ✅

### P1 완료/BLOCKED
- [x] S15-02: 사전 품질 검증 도구 개선 ✅
- [ ] S15-03: PyPI 배포 - BLOCKED → S16-03
- [ ] S15-04: npm 배포 - BLOCKED → S16-04

### P2 완료
- [x] S15-05: Unknown 단어 처리 개선 ✅
- [x] S15-06: 복합명사 분해 개선 ✅
- [x] S15-07: 성능 벤치마크 CI 통합 ✅

### P3 완료
- [x] S15-08: 문서 사이트 개선 ✅

---

# 완료된 스프린트: Phase 6 - Sprint 14 (v0.2.0 릴리스 & 사전 자동화) ✅

## 목표 (완료)
v0.2.0 정식 릴리스, 신조어 수집 워크플로우 검증, 사전 빌드 자동화 개선

## Sprint 14 작업 목록

### P0 완료
- [x] S14-01: v0.2.0 릴리스 준비 ✅

### P1 완료/BLOCKED
- [ ] S14-02: 신조어 수집 워크플로우 테스트 - 대기 (secret 필요)
- [ ] S14-03: PyPI 배포 - BLOCKED → S15-03
- [ ] S14-04: npm 배포 - BLOCKED → S15-04

### P2 완료
- [x] S14-05: 한국어기초사전 API 클라이언트 ✅
- [x] S14-06: CLI collect 서브커맨드 ✅
- [x] S14-07: 사전 빌드 자동화 ✅

### P3 완료
- [x] S14-08: 성능 벤치마크 문서화 ✅

---

# 완료된 스프린트: Phase 6 - Sprint 13 (커뮤니티 & API 통합) ✅

## 목표 (완료)
커뮤니티 기여 시스템 구축, 국립국어원 API 클라이언트 구현, v0.2.0 준비

## Sprint 13 작업 목록

### P0 (Critical)
- [x] S13-01: 커뮤니티 기여 가이드라인 ✅

### P1 (High)
- [x] S13-02: 국립국어원 API 클라이언트 ✅
- [ ] S13-03: PyPI 배포 - BLOCKED → S14-03
- [ ] S13-04: npm 배포 - BLOCKED → S14-04

### P2 (Medium)
- [x] S13-05: 사전 데이터 변환기 ✅
- [x] S13-06: CLI 사전 동기화 명령 ✅
- [x] S13-07: v0.2.0 Breaking Changes 정리 ✅

### P3 (Low)
- [x] S13-08: 신조어 자동 수집 파이프라인 설계 ✅

---

# 완료된 스프린트: Phase 5 - Sprint 12 (사전 현대화 & 기능 확장) ✅

## 목표 (완료)
신조어 사전 구축, 사용자 사전 도구 개선, API 확장

## Sprint 12 작업 목록

### P0 (Critical)
- [x] S12-01: 신조어 시드 사전 구축 ✅
  - 2018-2024 주요 신조어 123개 수집
  - `data/user-dict/neologisms.csv` 생성
  - 품사 태그 및 비용 정보 포함
  - README.md 문서 추가

### P1 (High)
- [x] S12-02: 사용자 사전 빌드 도구 개선 ✅
  - 신조어 자동 품사 추정 기능 (`estimate_pos()`)
  - CSV 중복 검사 기능 (`check_csv_duplicates()`)
  - 시스템 사전과 충돌 검사 (`check_system_conflicts()`)
- [ ] S12-03: PyPI 배포 - BLOCKED → S13-03
- [ ] S12-04: npm 배포 - BLOCKED → S13-04

### P2 (Medium)
- [x] S12-05: 국립국어원 API 연동 조사 ✅
- [x] S12-06: Streaming API 확인 ✅
- [x] S12-07: 분석 모드 확인 ✅

### P3 (Low)
- [x] S12-08: CLI 인터랙티브 모드 ✅

---

# 완료된 스프린트: Phase 5 - Sprint 11 (배포 & 생태계 확장) ✅

## 목표 (완료)
PyPI/npm 배포 준비, 문서 사이트 구축, 사전 현대화 조사

## Sprint 11 작업 목록

### P0 (Critical)
- [ ] S11-01: PyPI 배포 (mecab-ko-python) - BLOCKED (PyPI 토큰 필요) → S12-03
- [ ] S11-02: npm 배포 (mecab-ko-wasm) - BLOCKED (npm 토큰 필요) → S12-04

### P1 (High)
- [x] S11-03: GitHub Releases 자동화 ✅
- [x] S11-04: 성능 회귀 탐지 CI ✅

### P2 (Medium)
- [x] S11-05: 문서 사이트 구축 ✅
- [x] S11-06: mecab-ko-dic 최신화 조사 ✅
- [x] S11-07: Docker 이미지 배포 ✅

### P3 (Low)
- [x] S11-08: 성능 대시보드 ✅

---

# 완료된 스프린트: Phase 4 - Sprint 10 (안정화 & 품질) ✅

## 목표 (완료)
테스트 커버리지 향상, ignored 테스트 활성화, 코드 품질 개선, crates.io 발행

## Sprint 10 작업 목록

### P0 (Critical)
- [x] S10-01: crates.io 정식 발행 ✅ (6개 크레이트 v0.1.1 발행 완료)

### P1 (High)
- [x] S10-02: ignored 테스트 활성화 ✅ (e2e: 28 passed, 0 ignored)
- [x] S10-03: Elasticsearch 통합 테스트 개선 ✅ (doc: 5 passed, 0 ignored)
- [x] S10-04: 에러 처리 개선 ✅ (이미 thiserror 사용 중)

### P2 (Medium)
- [x] S10-05: 코드 중복 제거 ✅ (분석 완료: 중복 최소화 확인)
- [x] S10-06: 추가 벤치마크 ✅ (배치/메모리 벤치마크 이미 구현됨)
- [x] S10-07: CHANGELOG.md 작성 ✅

---

# 완료된 스프린트 요약

| Sprint | Phase | 목표 | 상태 |
|--------|-------|------|------|
| 1-2 | 1 | 프로젝트 셋업 | ✅ |
| 3 | 1 | 코어 데이터 구조 | ✅ |
| 4 | 2 | 코어 엔진 + 바인딩 | ✅ |
| 5 | 3 | 안정화 | ✅ |
| 6 | 3 | 성능 최적화 | ✅ |
| 7 | 4 | crates.io 발행 준비 | ✅ |
| 8 | 4 | Memory 최적화 | ✅ |
| 9 | 4 | 사전 현대화 & 발행 | ✅ |
| 10 | 4 | 안정화 & 품질 | ✅ |
| 11 | 5 | 배포 & 생태계 확장 | ✅ |
| 12 | 5 | 사전 현대화 & 기능 확장 | ✅ |
| 13 | 6 | 커뮤니티 & API 통합 | ✅ |
| 14 | 6 | v0.2.0 릴리스 & 사전 자동화 | ✅ |
| 15 | 7 | 사전 품질 & 정확도 | ✅ |
| 16 | 7 | 고급 토큰화 & N-best | ✅ |
| 17 | 8 | v0.3.0 릴리스 & API 개선 | ✅ |

## 크레이트 발행 현황

| 크레이트 | 버전 | 플랫폼 | 상태 |
|---------|------|--------|------|
| mecab-ko-hangul | v0.3.0 | crates.io | ✅ |
| mecab-ko-dict | v0.3.0 | crates.io | ✅ |
| mecab-ko-core | v0.3.0 | crates.io | ✅ |
| mecab-ko-dict-validator | v0.3.0 | crates.io | ✅ |
| mecab-ko-dict-builder | v0.3.0 | crates.io | ✅ |
| mecab-ko | v0.3.0 | crates.io | ✅ |
| mecab-ko-python | - | PyPI | BLOCKED |
| mecab-ko-wasm | v0.3.0 | npm | ✅ |
| mecab-ko-cli | v0.1.1 | GitHub Releases | ✅ |
| mecab-ko (docker) | latest | GHCR | ✅ |

---

# 현재 스프린트: Phase 9 - Sprint 18 (정확도 개선 & 사전 현대화)

## 목표
정확도 측정 및 개선, mecab-ko-dic v3.0 준비, 커뮤니티 피드백 반영

## Sprint 18 작업 목록

### P0 (Critical)
- [ ] S18-01: 정확도 벤치마크 실행 - BLOCKED (전체 사전 필요)
  - 샘플 평가 데이터로 현재 정확도 측정
  - Token/Sentence/POS Accuracy 기준선 확립
  - 벤치마크 결과 문서화
  - **Note**: mini-dict로는 0.5% 정확도, 전체 mecab-ko-dic 필요

### P1 (High)
- [ ] S18-02: PyPI 배포 (계속)
  - PyPI 토큰 설정 후 배포
  - mecab-ko-python v0.3.0
- [x] S18-03: 사전 엔트리 품질 개선 ✅
  - Unknown 단어 비용 조정 최적화:
    - HangulAlphaMix: +200 패널티 → -100 보너스 (신조어 패턴)
    - ProperNoun/CamelCase: 더 강한 선호 (-600/-400)
    - Plain 길이 패널티: 5→6자, 100→80 per char
  - 커밋: 9e8942b

### P2 (Medium)
- [ ] S18-04: 복합명사 분해 정확도 향상 - BLOCKED (전체 사전 필요)
  - 잘못 분해되는 패턴 수집
  - 분해 규칙 조정
  - **Note**: 전체 mecab-ko-dic으로 테스트 필요
- [x] S18-05: 사용자 사전 자동 검증 ✅
  - CI에서 사용자 사전 검증 자동화 (validate-user-dict job)
  - CSV 포맷 검증 (최소 5개 필드)
  - POS 태그 검증 (한국어 품사 태그 목록)
  - 중복 표면형 검출
  - GitHub Step Summary 품질 리포트
- [x] S18-06: Elasticsearch 플러그인 테스트 ✅
  - 63개 테스트 통과 (28 unit + 30 integration + 5 doc)
  - Nori 호환성: NoriAnalyzer, NoriTokenizer, DecompoundMode
  - 검색 시나리오: 캐싱, 배치 분석, 필터 체인, 직렬화
  - POS 필터, Reading Form 필터, Length 필터 검증

### P3 (Low)
- [x] S18-07: 문서 사이트 SEO 개선 ✅
  - theme/head.hbs 추가 (OG, Twitter Card, JSON-LD)
  - sitemap.xml, robots.txt 자동 생성
  - introduction.md v0.3.0 업데이트
  - main index.html SEO 메타태그 추가
- [x] S18-08: 커뮤니티 이슈 대응 ✅
  - GitHub 이슈 #6: 이미 답변 완료 (answered 라벨)
  - 추가 대응 필요 없음

## Sprint 18 완료 요약

**완료**: 5/8 (62.5%)
- S18-03: 사전 품질 개선 ✅
- S18-05: 사용자 사전 검증 CI ✅
- S18-06: Elasticsearch 테스트 ✅
- S18-07: SEO 개선 ✅
- S18-08: 커뮤니티 이슈 ✅

**BLOCKED**: 3/8 (37.5%)
- S18-01: 정확도 벤치마크 (전체 사전 필요)
- S18-02: PyPI 배포 (토큰 필요)
- S18-04: 복합명사 분해 (전체 사전 필요)

---

# 다음 스프린트: Phase 9 - Sprint 19 (전체 사전 & Python 배포)

## 목표
mecab-ko-dic 전체 사전 통합, PyPI 배포, 정확도 기준선 확립

## Sprint 19 작업 목록

### P0 (Critical)
- [x] S19-01: mecab-ko-dic 전체 사전 빌드 ✅
  - mecab-ko-dic-2.1.1-20180720 다운로드 (47.4MB)
  - dict-builder로 바이너리 사전 생성 (37.5초)
  - 816,283개 엔트리, 768,190개 고유 표면형
  - data/dict-output/에 저장
  - 토큰화 테스트 통과

### P1 (High)
- [ ] S19-02: PyPI 배포 (토큰 설정 후)
  - PyPI API 토큰 설정
  - mecab-ko-python v0.3.0 발행
  - pip install mecab-ko 테스트
- [x] S19-03: 정확도 벤치마크 (S18-01 계속) ✅
  - 전체 사전으로 Token/Sentence/POS Accuracy 측정
  - **기준선 확립**: Token 15.2%, Sentence 8.1%, F1 0.165
  - 명사 48%, 동사 1.5% → 형태소 경계 불일치 분석 필요
- [~] S19-04: 정확도 개선 분석 (S18-04 확장)
  - **원인 분석 완료**:
    - 토큰화 기준 차이: 세종 코퍼스 vs mecab-ko-dic
    - 어미 분리: 정답 "갔/VV 다/EF" vs MeCab "갔다/VV+ETM"
    - 품사 태그 체계 차이: JKO/JKS vs ETN/EF
  - **개선 방안**:
    1. 평가 데이터를 mecab-ko-dic 형식으로 변환
    2. 후처리로 세종 코퍼스 형식 출력 지원
  - **Note**: v0.3.2에서 세종 코퍼스 호환 모드 구현 예정

### P2 (Medium)
- [x] S19-05: v0.3.1 릴리스 준비 ✅
  - CHANGELOG v0.2.0, v0.3.0 섹션 추가 완료
  - Unreleased에 Sprint 19 변경사항 추가
- [x] S19-06: 성능 회귀 테스트 ✅
  - 전체 사전 기준 벤치마크 완료
  - **결과**: 3.55µs (11자), 22.13µs (67자)
  - 처리 속도: 3.0-3.7M chars/sec
  - 회귀 없음 확인

### P3 (Low)
- [x] S19-07: API 문서 개선 (rustdoc) ✅
  - facade 크레이트에 v0.3.0 기능 re-export
  - 고급 기능 문서 추가 (N-best, 분석 모드, 캐싱, 배치)
  - doc tests 6개 통과
- [x] S19-08: 커뮤니티 피드백 수집 ✅
  - Issue #6에 Sprint 19 진행 상황 업데이트
  - 정확도 기준선, 성능, npm/crates.io 배포 현황 공유

## Sprint 19 완료 요약

**완료율**: 7/8 (87.5%), BLOCKED 제외 100%

| 작업 | 상태 |
|------|------|
| S19-01: 전체 사전 빌드 | ✅ 816K 엔트리 |
| S19-02: PyPI 배포 | BLOCKED (토큰 필요) |
| S19-03: 정확도 벤치마크 | ✅ Token 15.2% |
| S19-04: 정확도 분석 | ✅ 토큰화 표준 차이 확인 |
| S19-05: CHANGELOG | ✅ v0.2.0/v0.3.0 |
| S19-06: 성능 회귀 테스트 | ✅ 3.55µs, 회귀 없음 |
| S19-07: API 문서 | ✅ facade re-export |
| S19-08: 커뮤니티 | ✅ Issue #6 업데이트 |

---

# 다음 스프린트: Phase 10 - Sprint 20 (정확도 향상 & 사전 현대화)

## 목표
세종 코퍼스 호환 모드, mecab-ko-dic v3.0 계획, 신조어 수집 실행

## Sprint 20 작업 목록

### P0 (Critical)
- [ ] S20-01: PyPI 배포 (S19-02 계속)
  - PyPI API 토큰 설정
  - mecab-ko-python v0.3.0 발행

### P1 (High)
- [ ] S20-02: 세종 코퍼스 호환 모드 설계
  - 어미 분리 후처리 구현
  - 품사 태그 변환 테이블
  - `--sejong` 출력 옵션
- [ ] S20-03: mecab-ko-dic v3.0 계획
  - 2018년 이후 누락 단어 분석
  - 신조어 추가 방안
  - 빌드 파이프라인 검토

### P2 (Medium)
- [ ] S20-04: 신조어 자동 수집 실행
  - neologism-sync.yml 워크플로우 테스트
  - 국립국어원 API 실제 호출
- [ ] S20-05: v0.3.1 릴리스
  - CHANGELOG Unreleased → v0.3.1
  - crates.io 발행
  - GitHub Release 생성

### P3 (Low)
- [ ] S20-06: 정확도 향상 측정
  - 세종 코퍼스 호환 모드로 재측정
  - 품사별 개선 분석
- [ ] S20-07: 커뮤니티 피드백 반영
  - Issue/Discussion 모니터링
  - 사용자 요청 사항 수집
- [ ] S20-08: 문서 사이트 업데이트
  - v0.3.0 기능 튜토리얼
  - 정확도 가이드
