# Sprint 7: crates.io 발행 준비 완료

## 작업 정보
- **완료일**: 2026-03-01
- **스프린트**: Phase 4 - Sprint 7

## 작업 요약

Sprint 7에서는 crates.io 발행 준비와 바인딩 검증을 수행했습니다.

### 완료된 작업

| Task | 설명 | 상태 |
|------|------|------|
| S7-01 | Path dependency에 version 추가 | ✅ |
| S7-02 | cargo publish --dry-run 검증 | ✅ |
| S7-04 | Full-dict Memory KPI 측정 | ⚠️ 215MB |
| S7-05 | Python 바인딩 테스트 | ✅ |
| S7-06 | WASM 바인딩 테스트 | ⚠️ zstd 이슈 |
| S7-07 | Node.js 바인딩 테스트 | ✅ |
| S7-08 | Clippy 경고 해결 | ✅ 0 warnings |

## 커밋 내역

```
9530a77 fix(clippy): Resolve all workspace clippy warnings
7ef84e6 docs: Sprint 7 progress - binding tests and memory KPI
16aa61b chore(deps): Add version to path dependencies for crates.io publish
```

## 변경 파일

| 파일 | 유형 | 설명 |
|------|------|------|
| rust/crates/*/Cargo.toml | 수정 | path dep에 version 추가 |
| rust/crates/mecab-ko-core/src/*.rs | 수정 | clippy 경고 수정 |
| rust/crates/mecab-ko-*/src/lib.rs | 수정 | doc backticks 추가 |
| docs/research/benchmarks/sprint7-memory-kpi.md | 생성 | Memory KPI 리포트 |

## 주요 결정 및 발견

### 1. crates.io 발행 순서
```
1. mecab-ko-hangul (이미 존재)
2. mecab-ko-dict
3. mecab-ko-core
4. mecab-ko-dict-validator
5. mecab-ko-dict-builder (dict 0.1.1 필요)
6. mecab-ko (facade)
```

### 2. Memory KPI
- 측정값: **215 MB** (목표 150MB 초과)
- Cold start: **0.13s** (목표 달성)
- 최적화 필요: entries 지연 로딩, mmap 활용

### 3. WASM 빌드 이슈
- zstd-sys가 wasm32 타겟 미지원
- 해결 방안: zstd 제거 또는 pure-Rust 대안

### 4. rust-version 요구사항
- mecab-ko-node: napi-build가 Rust 1.77+ 문법 사용
- 의도적으로 rust-version = "1.77" 유지

## 테스트 결과
- 단위 테스트: 746개 통과
- 실패: 0개
- Clippy: 0 warnings
- cargo doc: 0 warnings

## Sprint 8 계획

### P0 (필수)
- [ ] Memory 최적화 (entries 지연 로딩)
- [ ] WASM zstd 이슈 해결

### P1 (중요)
- [ ] crates.io 정식 발행
- [ ] PyPI 배포 (maturin publish)

### P2 (권장)
- [ ] npm 배포 (Node.js 바인딩)
- [ ] 사전 현대화 착수

## 참고 자료
- docs/research/benchmarks/sprint7-memory-kpi.md
- docs/PROJECT_PLAN.md
