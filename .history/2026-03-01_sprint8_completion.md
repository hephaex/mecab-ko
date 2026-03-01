# Sprint 8: Memory 최적화 & 발행 준비

## 작업 정보
- **완료일**: 2026-03-01
- **스프린트**: Phase 4 - Sprint 8

## 작업 요약

Sprint 8에서는 메모리 최적화, WASM 호환성, 배포 인프라 구축을 완료했습니다.

### 주요 성과

1. **Memory 최적화** (S8-01, S8-02)
   - LazyEntries 구조체: mmap + LRU 캐시 기반 지연 로딩
   - LoadOptions: 사전 로딩 전략 선택 가능 (메모리 vs 속도)
   - 예상 메모리 절감: 40-50%

2. **WASM 호환성** (S8-03)
   - zstd를 optional feature로 분리
   - WASM 빌드 성공 (zstd-sys 의존성 제거)

3. **배포 인프라** (S8-05, S8-07)
   - PyPI 배포 워크플로우 (maturin, 3 platforms, Python 3.8-3.12)
   - npm 배포 워크플로우 (napi-rs, 5 targets, Node 18/20/22)

4. **문서화** (S8-06)
   - README.md 전면 개선
   - 성능 지표, 사용 예제, 크레이트 구조 문서화

5. **GitHub 정리** (S8-08)
   - 오래된 유지보수 이슈 7개 닫음

## 변경 파일

| 파일 | 유형 | 설명 |
|------|------|------|
| rust/crates/mecab-ko-dict/src/dictionary.rs | Modified | LoadOptions, load_with_options 추가 |
| rust/crates/mecab-ko-dict/src/entries.rs | Modified | LazyEntries 구현 |
| rust/crates/mecab-ko-dict/Cargo.toml | Modified | zstd optional feature |
| rust/crates/mecab-ko-dict/src/trie.rs | Modified | cfg(feature = "zstd") 조건부 컴파일 |
| rust/crates/mecab-ko-dict/src/matrix/mod.rs | Modified | cfg(feature = "zstd") 조건부 컴파일 |
| rust/crates/mecab-ko-dict/src/loader.rs | Modified | zstd 조건부 함수 |
| rust/crates/mecab-ko-core/Cargo.toml | Modified | zstd feature 전파 |
| rust/crates/mecab-ko-wasm/Cargo.toml | Modified | default-features = false |
| rust/Cargo.toml | Modified | 버전 0.1.1 |
| rust/README.md | Modified | 전면 개선 |
| .github/workflows/npm-publish.yml | Created | npm 배포 워크플로우 |
| .github/workflows/pypi-publish.yml | Created | PyPI 배포 워크플로우 |

## 주요 결정

1. **zstd를 optional feature로**: WASM 타겟에서 C 의존성 문제 해결
2. **LoadOptions 패턴**: 사용자가 메모리/속도 트레이드오프 선택 가능
3. **배포 전략**: crates.io (Rust), PyPI (Python), npm (Node.js) 분리

## 테스트 결과

- 단위 테스트: 191개 통과
- 무시됨: 19개 (full-dict 필요)
- Clippy: 경고 없음 (라이브러리)

## 커밋 내역

```
82fc906 docs: Complete Sprint 8 P1/P2 tasks
d36be77 feat(ci): Add npm publish workflow for Node.js bindings
ce1cb89 docs: Update README with current features and usage
0be64fb chore: Bump version to 0.1.1 for crates.io release
4c88f65 docs: Update Sprint 8 progress - P0 tasks complete
525b42e feat(dict): Make zstd compression optional for WASM support
187395e feat(dict): Add LoadOptions and mmap-enabled dictionary loading
f87d5a8 feat(dict): Add lazy loading entries for memory optimization
```

## 다음 단계

1. **S9-01**: crates.io 정식 발행 (cargo login 필요)
2. **S9-02**: mecab-ko-dic 최신 버전 지원
3. **S9-03**: CLI 개선

## 참고 자료

- [NAPI-RS documentation](https://napi.rs/)
- [Maturin Python wheels](https://www.maturin.rs/)
- [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html)
