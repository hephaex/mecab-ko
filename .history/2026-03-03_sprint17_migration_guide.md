# Sprint 17 - S17-04: Migration Guide v0.2.0 → v0.3.0 (2026-03-03)

## 세션 개요
v0.2.0에서 v0.3.0으로 업그레이드하기 위한 마이그레이션 가이드 작성

## 완료된 작업

### S17-04: Migration Guide v0.2.0 → v0.3.0 ✅

#### 문서 위치
`docs/MIGRATION_GUIDE.md` - 기존 v0.1.x → v0.2.0 가이드에 새 섹션 추가

#### Breaking Changes

1. **TokenStream 내부 버퍼 변경**
   - `Vec` → `VecDeque`로 변경
   - O(n) → O(1) dequeue 성능 개선
   - Public API는 동일 (내부 구현 변경)

2. **StreamingTokenizer 모듈 재구성**
   - 새로운 타입 export 추가:
     - `ProgressStreamingTokenizer`
     - `StreamingProgress`
     - `ProgressCallback`
     - `ChunkedTokenIterator`

#### New Features 문서화

1. **Improved N-best Path Search**
   ```rust
   use mecab_ko_core::ImprovedNbestSearcher;
   let searcher = ImprovedNbestSearcher::new(&lattice, k);
   let results = searcher.search();
   ```

2. **User-defined Analysis Modes**
   ```rust
   use mecab_ko_core::{AnalysisMode, PosFilter, AnalyzerConfig};
   let nouns = extract_nouns(&tokens);
   let config = AnalyzerConfig::new()
       .with_mode(AnalysisMode::Custom)
       .with_filter(PosFilter::include(&["NNG", "NNP", "VV"]));
   ```

3. **Lattice Visualization Tool**
   ```rust
   let dot = lattice_to_dot(&lattice);
   let html = lattice_to_html(&lattice);
   ```

4. **Tokenization Caching**
   ```rust
   let cache = TokenCache::with_config(config);
   let caching_tokenizer = CachingTokenizer::new(tokenizer, cache);
   ```

5. **Progress-aware Streaming**
   ```rust
   let stream = ProgressStreamingTokenizer::new(tokenizer)
       .with_progress_callback(|progress| { ... });
   ```

6. **Large File Processing**
   ```rust
   let processor = LargeFileProcessor::new()?
       .with_buffer_size(65536);
   let tokens = processor.process_file("large_corpus.txt")?;
   ```

7. **Smart Chunking**
   ```rust
   let chunks = BatchTokenizer::split_into_chunks_smart(text, 1000, &['.', '!', '?']);
   let overlapping = BatchTokenizer::split_with_overlap(text, 1000, 100);
   ```

8. **npm Package**
   ```javascript
   import { Tokenizer } from 'mecab-ko-wasm';
   const tokenizer = await Tokenizer.new();
   ```

#### 성능 개선 비교표

| Operation | v0.2.0 | v0.3.0 | Improvement |
|-----------|--------|--------|-------------|
| TokenStream dequeue | O(n) | O(1) | ~10x faster |
| Smart chunking | N/A | O(n) | Memory-efficient |
| Cache hit | N/A | O(1) | Instant for repeated |

#### 버전 호환성 매트릭스

| Component | v0.2.0 | v0.3.0 |
|-----------|--------|--------|
| Rust | 1.75+ | 1.75+ |
| Python | 3.8-3.13 | 3.8-3.13 |
| Node.js | 18, 20, 22 | 18, 20, 22 |
| npm | N/A | mecab-ko-wasm@0.3.0 |

#### 마이그레이션 체크리스트

- [ ] Update `Cargo.toml` dependencies to v0.3.0
- [ ] Review `NbestSearcher` usage → consider `ImprovedNbestSearcher`
- [ ] Update npm package: `npm update mecab-ko-wasm`
- [ ] Test tokenization with new features
- [ ] Consider adding caching for repeated text processing
- [ ] Update documentation for new analysis modes

## 변경된 파일
- `docs/MIGRATION_GUIDE.md` - v0.2.0 → v0.3.0 섹션 추가
- `PLAN.md` - S17-04 완료 표시
- `PROGRESS.md` - 진행 상황 업데이트

## 학습 포인트
1. 마이그레이션 가이드는 Breaking Changes를 명확히 문서화해야 함
2. 코드 예제는 실제 사용 가능한 형태로 제공
3. 버전 호환성 매트릭스로 플랫폼별 지원 상황 명시
4. 체크리스트로 업그레이드 단계 안내

## 다음 작업
- S17-02: PyPI 배포 (BLOCKED - 토큰 필요)
- S17-05: 메모리 최적화 2차
- S17-06: API 문서 개선
- S17-07: 벤치마크 결과 정리
- S17-08: 테스트 커버리지 향상
