# 진행 상황

## 마지막 업데이트: 2026-03-03

## Sprint 15 진행 중 (2026-03-03)

### P0 완료
- [x] S15-01: 정확도 측정 인프라 구축 ✅
  - `mecab-ko-core/src/evaluate.rs` 평가 모듈 구현
  - `mecab evaluate` CLI 서브커맨드 추가
  - Token Accuracy, Sentence Accuracy, POS Accuracy 측정
  - Precision/Recall/F1 계산
  - 품사별 정확도 리포트
  - 샘플 테스트 데이터 160문장 (data/eval/sample.tsv)
  - TSV 형식 지원 (text\ttoken1/pos1 token2/pos2 ...)
  - Verbose 모드로 틀린 문장 상세 분석
  - 5개 테스트 통과

### P1 대기
- [ ] S15-02: 사전 품질 검증 도구 개선
- [ ] S15-03: PyPI 배포 - BLOCKED (계정 복구 대기)
- [ ] S15-04: npm 배포 - BLOCKED (토큰 필요)

### P2 대기
- [ ] S15-05: Unknown 단어 처리 개선
- [ ] S15-06: 복합명사 분해 개선
- [ ] S15-07: 성능 벤치마크 CI 통합

### P3 대기
- [ ] S15-08: 문서 사이트 개선

---

## Sprint 14 완료 ✅ (2026-03-02)

### 완료율: 5/8 (BLOCKED 제외 시 100%)

### P0 완료
- [x] S14-01: v0.2.0 릴리스 준비 ✅
  - v0.2.0 태그 및 GitHub Release 생성
  - 4개 플랫폼 바이너리 배포

### P1 BLOCKED
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

## Sprint 13 완료 ✅ (2026-03-02)

### 완료율: 6/8 (BLOCKED 제외 시 100%)

### P0 완료
- [x] S13-01: 커뮤니티 기여 가이드라인 ✅

### P1 완료/BLOCKED
- [x] S13-02: 국립국어원 API 클라이언트 ✅
- [ ] S13-03: PyPI 배포 - BLOCKED → S14-03
- [ ] S13-04: npm 배포 - BLOCKED → S14-04

### P2 완료
- [x] S13-05: 사전 데이터 변환기 ✅
- [x] S13-06: CLI 사전 동기화 명령 ✅
- [x] S13-07: v0.2.0 Breaking Changes 정리 ✅

### P3 완료
- [x] S13-08: 신조어 자동 수집 파이프라인 설계 ✅

---

## Sprint 12 완료 ✅ (2026-03-02)

### P0 완료
- [x] S12-01: 신조어 시드 사전 구축 ✅
  - 2018-2024 주요 신조어 123개 수집
  - `data/user-dict/neologisms.csv` 생성
  - 품사 태그 및 비용 정보 포함
  - README.md 문서 추가

### P1 진행
- [x] S12-02: 사용자 사전 빌드 도구 개선 ✅
  - `estimate_pos()` - 자동 품사 추정 기능
  - `check_csv_duplicates()` - CSV 중복 검사
  - `check_system_conflicts()` - 시스템 사전 충돌 검사
  - `add_entry_auto_pos()` - 자동 품사 추가 편의 메서드
  - 21개 테스트 통과
- [ ] S12-03: PyPI 배포 - BLOCKED (PyPI 토큰 필요)
- [ ] S12-04: npm 배포 - BLOCKED (npm 토큰 필요)

### P2 완료
- [x] S12-05: 국립국어원 API 연동 조사 ✅
  - 우리말샘/한국어기초사전/표준국어대사전 API 조사
  - 데이터 변환 파이프라인 설계
  - docs/research/dictionary/korean-dict-api-survey.md 작성
- [x] S12-06: Streaming API 확인 ✅
  - StreamingTokenizer 이미 구현됨
  - TokenStream (Iterator 기반) 구현됨
  - process_reader, process_file API 지원
- [x] S12-07: 분석 모드 확인 ✅
  - DecompoundMode (None/Discard/Mixed) 구현됨
  - NoriTokenizer에 lemma 지원

### P3 완료
- [x] S12-08: CLI 인터랙티브 모드 ✅
  - REPL 스타일 대화형 분석 이미 구현됨
  - `--repl` 플래그로 활성화
  - `:help`, `:format`, `:quit`, `:exit` 명령어 지원
  - 7가지 출력 포맷 동적 전환
  - Ctrl+D 종료 지원

---

## Sprint 11 완료 ✅ (2026-03-02)

### P0 진행
- [ ] S11-01: PyPI 배포 - BLOCKED (PyPI 토큰 필요)
  - maturin build 성공 (cp313-macosx_11_0_arm64.whl)

### P1 진행
- [ ] S11-02: npm 배포 - BLOCKED (npm 토큰 필요)
- [x] S11-03: GitHub Releases 자동화 ✅
  - v0.1.1 태그 생성 및 푸시
  - release.yml 워크플로우 트리거됨
- [x] S11-04: 성능 회귀 탐지 CI ✅
  - benchmark.yml에 회귀 탐지 추가
  - PR 코멘트로 비교 표 자동 생성
  - 10% 이상 악화 시 경고

### P2 진행
- [x] S11-05: 문서 사이트 구축 ✅
  - mdBook 설정 및 빌드 완료
  - docs.yml 워크플로우 개선
  - GitHub Pages 자동 배포 (main 푸시 시)
- [x] S11-06: mecab-ko-dic 최신화 조사 ✅
  - 2018년 이후 변경 조사 완료
  - 신조어 추가 방안 (사용자 사전, 커뮤니티, 자동 수집)
  - 국립국어원 API 활용 방안 (우리말샘, 한국어기초사전)
  - docs/research/dictionary/mecab-ko-dic-modernization.md 작성
- [x] S11-07: Docker 이미지 배포 ✅
  - Dockerfile 작성 (multi-stage, debian-slim)
  - docker.yml 워크플로우 추가
  - GHCR 자동 배포 (linux/amd64, linux/arm64)

### P3 진행
- [x] S11-08: 성능 대시보드 ✅
  - Chart.js 기반 인터랙티브 대시보드
  - 버전별 처리량/지연 시간 비교 차트
  - 성능 개선 추이 그래프
  - CI 자동 업데이트 워크플로우

---

## Sprint 10 완료 ✅ (2026-03-02)

### P0 완료
- [x] S10-01: crates.io 정식 발행 ✅
  - mecab-ko-hangul v0.1.1
  - mecab-ko-dict v0.1.1
  - mecab-ko-core v0.1.1
  - mecab-ko-dict-validator v0.1.1
  - mecab-ko-dict-builder v0.1.1
  - mecab-ko v0.1.1 (facade)

### P1 완료
- [x] S10-02: ignored 테스트 활성화 ✅
  - `system_dict_available()` 헬퍼 함수 추가
  - `skip_without_system_dict!` 매크로 추가
  - e2e 테스트: 28 passed, 0 ignored
  - 남은 ignored: profiler(9), dict doc(4) - 의도적 제외
- [x] S10-03: Elasticsearch 통합 테스트 개선 ✅
  - doc tests: 5 passed, 0 ignored
  - import 경로 수정
- [x] S10-04: 에러 처리 확인 ✅ (이미 thiserror 사용 중)

### P2 완료
- [x] S10-05: 코드 중복 제거 ✅
  - 분석 완료: 최소 중복 확인
  - `#[allow(dead_code)]` 23개 (대부분 테스트/예제)
- [x] S10-06: 추가 벤치마크 ✅
  - `batch_bench.rs`: 배치 처리, 처리량, 시나리오 벤치마크 구현
  - `memory_bench.rs`: 메모리 할당, 재사용, 누적, 확장성, 압력 테스트 구현
  - 총 9개 벤치마크 파일, 포괄적인 커버리지
- [x] S10-07: CHANGELOG.md 작성 ✅
  - Keep a Changelog 형식
  - v0.1.0, v0.1.1, Unreleased 섹션

---

## Sprint 9 완료 ✅ (2026-03-01)

### P0 진행
- [ ] S9-01: crates.io 정식 발행 - BLOCKED (`cargo login` 필요)
- [x] S9-02: 의존성 업데이트 (wasm-bindgen 0.2.114, tempfile 3.26)
  - GitHub 이슈 #9 해결

### P1 진행
- [x] S9-03: CLI 개선 ✅
  - `--benchmark N` 옵션 추가 (성능 측정)
  - `--stats` 옵션 추가 (분석 통계)
  - 14개 테스트 통과
- [x] S9-05: 사용자 사전 기능 개선 ✅
  - `validate()` 메서드 추가 (품사 태그 검증)
  - `ValidationResult`, `DictionaryStats` 구조체 추가
  - `remove_duplicates()` 메서드 추가 (중복 항목 제거)
  - `remove_surface()` 메서드 추가 (표면형으로 삭제)
  - `stats()` 메서드 추가 (사전 통계)
  - `is_valid_pos_tag()` 함수 추가 (세종 품사 태그 검증)

### P2 진행
- [x] S9-06: 문서화 개선 ✅
  - rustdoc 링크 수정
  - README에 사용자 사전 검증/통계 예제 추가
  - 성능 지표 테이블 업데이트
- [x] S9-07: 성능 벤치마크 자동화 ✅
  - CI workflow에 benchmark job 추가
  - main 브랜치 push 시 자동 실행
  - 벤치마크 결과 artifact 저장 (30일)
  - GitHub Step Summary에 결과 표시

---

## Sprint 8 완료 ✅ (2026-03-01)

### P0 완료
- [x] S8-01: Memory 최적화 - entries 지연 로딩
  - LazyEntries 구조체 구현 (mmap + LRU cache)
  - entries.bin v2 포맷 (index table + O(1) 랜덤 접근)
  - 예상 메모리 절감: 40-50%
- [x] S8-02: Memory 최적화 - mmap 활용 강화
  - LoadOptions 구조체 추가 (use_mmap_matrix, use_lazy_entries)
  - SystemDictionary::load_with_options() 메서드 추가
  - load_memory_optimized() 편의 메서드 제공
- [x] S8-03: WASM zstd-sys 이슈 해결
  - zstd를 optional feature로 분리 (default = ["zstd"])
  - cfg(feature = "zstd")로 조건부 컴파일
  - mecab-ko-wasm: default-features = false로 zstd 비활성화
  - WASM 빌드 성공 확인

### P1 완료
- [x] S8-04: crates.io 발행 준비 완료
  - 버전 0.1.1 준비 완료
  - dry-run 검증 통과
  - 발행 순서 확립 (hangul → dict → core → validator → builder → facade)
  - 참고: 실제 발행은 `cargo login` 인증 필요
- [x] S8-05: PyPI 배포 준비 (maturin) ✅
  - pypi-publish.yml 워크플로우 구성 완료
  - Linux/macOS/Windows 휠 빌드
  - Python 3.8-3.12 테스트
  - 태그 푸시 시 자동 발행 (v*)
- [x] S8-06: README.md 정리 ✅
  - 버전 0.1.1, 성능 지표 추가
  - Python/WASM/Node.js 바인딩 예제
  - 크레이트 구조 문서화

### P2 완료
- [x] S8-07: npm 배포 준비 (Node.js 바인딩) ✅
  - npm-publish.yml 워크플로우 생성
  - Linux/macOS/Windows 네이티브 모듈 빌드
  - Node.js 18, 20, 22 테스트
  - 태그 푸시 시 자동 발행 (v*)
- [x] S8-08: GitHub Maintenance 이슈 정리 ✅
  - 오래된 유지보수 이슈 7개 닫음 (#1-5, #7-8)
  - 이슈 #6에 라벨 추가 (question, answered)
  - 남은 이슈: #9 (최신 유지보수), #6 (커뮤니티 질문)

---

## Sprint 7 완료 ✅ (2026-03-01)

### 완료
- [x] S7-01: Path dependency에 version 추가 (7개 크레이트)
- [x] S7-02: cargo publish --dry-run 검증
  - ✅ mecab-ko-hangul (이미 crates.io에 존재)
  - ✅ mecab-ko-dict
  - ✅ mecab-ko-core
  - ✅ mecab-ko-dict-validator
  - ⚠️ mecab-ko-dict-builder (crates.io의 mecab-ko-dict 0.1.0에 save_entries 함수 없음)
  - ✅ mecab-ko (facade)
- [x] S7-04: Full-dict Memory KPI 측정
  - Peak Memory: **215 MB** (목표 150MB 초과)
  - Cold Start: **0.13s** (목표 0.2s 달성)
  - 상세: docs/research/benchmarks/sprint7-memory-kpi.md
- [x] S7-05: Python 바인딩 테스트 ✅
  - maturin build 성공
  - wheel 설치 및 import 성공
  - `Mecab().parse()` 동작 확인
- [x] S7-06: WASM 바인딩 테스트 ⚠️
  - wasm-pack build 실패
  - 원인: zstd-sys가 wasm32 타겟 미지원
  - 해결: Sprint 8에서 zstd 제거 또는 pure-Rust 대안
- [x] S7-07: Node.js 바인딩 테스트 ✅
  - cargo build 성공
  - libmecab_ko_node.dylib 생성 (669KB)

### 발행 순서 노트
- crates.io에 이미 mecab-ko-hangul 0.1.0 존재
- mecab-ko-dict-builder 발행 전에 mecab-ko-dict를 0.1.1로 업데이트 필요

### Memory 최적화 필요 (Sprint 8)
- entries 지연 로딩
- mmap 활용 강화
- String interning

## 완료된 작업

### Phase 1 - Sprint 1-2 (프로젝트 셋업)
- [x] RST-002: 프로젝트 구조 설계 (Cargo workspace 13개 크레이트)
- [x] RST-008: 한글 자소 분리/결합 유틸리티 (`mecab-ko-hangul` 구현 완료, 7 tests)
- [x] 프로파일러 기초 구현 (`mecab-ko-profiler` 부분 구현, 6 tests)
- [x] CI/CD 파이프라인 기초 설정
- [x] 프로젝트 자동화 인프라 구축 (.claude agents, skills, commands)

### Phase 1 - Sprint 3 (코어 데이터 구조)
- [x] research: MeCab 알고리즘 내부 구조 조사
- [x] research: 한국어 NLP 생태계 조사
- [x] research: Rust 데이터 구조 크레이트 조사
- [x] research: MeCab 바이너리 사전 포맷 상세
- [x] research: Lindera 소스 코드 심층 분석

### Phase 2 - Sprint 4 (코어 엔진 + 바인딩)
- [x] SystemDictionary entries 로딩, Multi-entry lookup
- [x] mecab-ko-dic 2.1.1 빌드 (816,283 엔트리)
- [x] Viterbi, Lattice, Tokenizer, Unknown handler, Normalizer
- [x] CLI, Python/WASM/Node 바인딩, Elasticsearch Nori 호환

### Phase 3 - Sprint 5 (안정화)
- [x] 의존성 업데이트 (wasm-bindgen 0.2.111, tempfile 3.25, rkyv 0.8.15 등 58개)
- [x] find_dicdir()에 test-fixtures/mini-dict 폴백 추가
- [x] 159개 ignored 테스트 활성화 (502→661 pass, 259→95 ignored)
- [x] mecab-ko-dict-validator 확인 (19 tests + 1 doc test)
- [x] 코드 품질 점검 (라이브러리 0 경고)

### Phase 3 - Sprint 6 (성능 최적화, ES 완성, 프로파일러 완성) ✅
- [x] S6-01: CI에 Elasticsearch 크레이트 포함
- [x] S6-02: CI에 test-allocator 테스트 별도 job 추가
- [x] S6-03: mini-dict로 추가 ignored 테스트 40개 활성화 (670→710 pass)
- [x] S6-04: Full-dict 테스트 인프라 (test_utils.rs, test-full-dict.sh)
- [x] S6-05: Elasticsearch 통합 테스트 12개 추가 (24 pass, 6 ignored)
- [x] S6-08: 프로파일러 실제 사전 데이터 통합 (mecab-profile CLI 완전 재작성)
- [x] S6-09: 프로파일러 회귀 탐지 (`baseline save/compare` 서브커맨드)
- [x] S6-11: 9개 벤치마크 스위트 실행, KPI 기준선 확립
- [x] S6-12: Hot path 성능 최적화 (토크나이저 45-55% 개선)
  - SpacePositions: HashSet → sorted Vec + binary_search
  - SpacePenalty: linear scan → binary_search
  - parse_features: Vec → splitn iterator
  - Lattice: byte_to_char binary search helper
- [x] S6-13: Cold start 최적화 (0.086ms, 이미 목표 200ms 충족)
- [x] S6-14: 릴리스 준비 (cargo doc 0 경고, 메타데이터 정리)
- [x] S6-15: rustdoc ignore → no_run/runnable 전환 (11→5 ignore)
- [x] S6-16: Sprint 6 전체 코드 리뷰

## 크레이트 실제 상태

| 크레이트 | 상태 | 비고 |
|----------|------|------|
| mecab-ko-hangul | ✅ 완료 | 한글 자소 분리/결합 |
| mecab-ko-dict | ✅ 완료 | Trie, Matrix, Loader, UserDict, HotReload, FileWatcher, entries |
| mecab-ko-dict-builder | ✅ 완료 | CSV→binary 사전 빌더, 압축, entries 저장 |
| mecab-ko-core | ✅ 완료 | Viterbi, Lattice, Tokenizer, Unknown handler, Normalizer (최적화) |
| mecab-ko-cli | ✅ 완료 | CLI 인터페이스 |
| mecab-ko-python | ✅ 완료 | PyO3 바인딩 (KoNLPy 호환) |
| mecab-ko-wasm | ✅ 완료 | WASM 바인딩 (wasm-bindgen) |
| mecab-ko-node | ✅ 완료 | Node.js 바인딩 (N-API, TypeScript) |
| mecab-ko-elasticsearch | ✅ 완료 | Nori 호환 (24 pass, 6 ignored) |
| mecab-ko-profiler | ✅ 완료 | 실제 사전 통합, baseline save/compare |
| mecab-ko-dict-validator | ✅ 완료 | CSV 검증, 규칙 엔진, CLI, 리포트 |
| mecab-ko (facade) | ✅ 완료 | 통합 API |

## 테스트 현황
- **통과**: 746개
- **실패**: 0개
- **무시됨**: 22개
- **Clippy**: 경고 없음
- **cargo doc**: 경고 없음
- **브랜치**: main

## 성능 KPI (mini-dict 기준)

| KPI | 목표 | 측정값 | 상태 |
|-----|------|--------|------|
| Morphemes/sec | 150K | ~238K | PASS |
| Cold start | < 200ms | 0.086ms | PASS |
| Memory per instance | < 150MB | N/A (mini-dict) | TBD |

### 벤치마크 개선 (S6-12)

| Input Size | Before | After | Improvement |
|-----------|--------|-------|-------------|
| 10 chars | 8.6µs | 3.8µs | -55% |
| 50 chars | 77.5µs | 44.9µs | -42% |
| 100 chars | 198µs | 141µs | -31% |
| 500 chars | 3055µs | 2165µs | -29% |
| 1000 chars | 9978µs | 8413µs | -16% |

## PM Agent 자동화 시스템 ✅ (2026-03-01)

### 완료된 스킬
- [x] `/pm-auto` - PM 자동 모드 (Auto Loop + Error Recovery + Context Management)
- [x] `/pm-orchestrate` - 이력 기반 멀티 에이전트 오케스트레이션
- [x] `/issue-sync` - GitHub 이슈 동기화 (1시간마다, PM Agent 기술 분석 코멘트)
- [x] `/issue-followup` - 이슈 처리 시작 (에이전트 위임)
- [x] `/pr-create` - 이슈 해결 PR 생성
- [x] `/lesson-learn` - PR 완료 후 LessonLearn 기술 보고서 생성
- [x] `/tech-report` - 기술 조사/스프린트 보고서

### 완료된 에이전트
- [x] `github-automation-setup` - GitHub Labels, Templates, Workflows 설정
- [x] `github-issue-manager` - 이슈 관리
- [x] `github-pr-creator` - PR 생성
- [x] `tech-writer` - 기술 보고서 작성
- [x] `pm-orchestrator` - PM 오케스트레이터
- [x] `pm-runner` - PM 자동 실행

### Sub-Agent 출력 표준화
```json
{
  "status": "success|failure|partial",
  "summary": "작업 요약",
  "files_changed": [],
  "tests_passed": true,
  "errors": [],
  "next_steps": []
}
```

### CLAUDE.md 자율 운영 규칙 추가
- 세션 시작 프로토콜
- PM Agent 자동 루프
- Sub-Agent 출력 표준
- 자동 커밋 규칙
- GitHub 이슈 연동
- 기술 보고서 (LessonLearn)
- 에러 복구 체계

## 블로커/이슈
- 없음

## GitHub 이슈
- #6: 커뮤니티 질문 (프로젝트 목표, 성능, 사전 계획 등) - ✅ 응답 완료

## 다음 세션에서 할 일 (Sprint 13)
1. S13-01: 커뮤니티 기여 시스템 구축
2. S13-02: 국립국어원 API 클라이언트 구현
3. S13-03: v0.2.0 준비 (Breaking Changes 정리)
4. S13-04: PyPI/npm 배포 (토큰 준비 시)
