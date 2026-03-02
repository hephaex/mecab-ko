# Rust 데이터 구조 및 알고리즘 조사 보고서

**날짜**: 2026-02-23
**카테고리**: rust-crates

## 요약 (3줄)
1. `yada`(DA Trie) + `fst`(FST) 이중 구조가 최적 - yada는 MeCab 호환, fst는 퍼지검색/압축
2. `vibrato`와 `mecrab`이 Rust MeCab 재구현의 핵심 레퍼런스 - Viterbi, 래티스, SIMD 최적화 참고
3. `memmap2` + `zerocopy`(고정크기) + `rkyv`(가변크기) 조합으로 제로카피 사전 로딩 구현

## 1. Double-Array Trie

### yada (워크스페이스에 포함: `yada = "0.5"`)
- **URL**: https://crates.io/crates/yada | https://github.com/takuyaa/yada
- Darts-clone 방식의 컴팩트 DA Trie
- `exact_match_search`, `common_prefix_search` 지원
- 시스템 사전 용으로 최적

### daachorse
- **URL**: https://github.com/daac-tools/daachorse
- Aho-Corasick + Double-Array. aho-corasick 크레이트 대비 3-5x 빠름
- 멀티패턴 매칭에 적합 (사용자 사전 오버레이)
- `CharwiseDoubleArrayAhoCorasick`로 한글 멀티바이트 처리

### cedarwood
- **URL**: https://github.com/MnO2/cedarwood
- **동적 삽입/삭제 가능** - 사용자 사전 핫리로딩에 적합
- 베타 품질, 정적 사전에는 yada가 나음

### 권장: yada (시스템 사전) + cedarwood (사용자 사전 옵션)

## 2. FST (Finite State Transducer)

### fst (워크스페이스에 포함: `fst = "0.4"`)
- **URL**: https://github.com/BurntSushi/fst | https://burntsushi.net/transducers/
- 접두사+접미사 모두 압축 (Trie는 접두사만)
- Levenshtein 퍼지 검색 내장
- 메모리맵 1급 지원
- 9.8M 용어 → 69MB FST, 8초 빌드

### FST vs DA Trie 비교

| 특성 | FST (fst) | DA Trie (yada) |
|------|-----------|----------------|
| 압축률 | 더 좋음 (접두사+접미사) | 보통 (접두사만) |
| 조회 속도 | 빠름 (오토마타) | 매우 빠름 (배열 인덱싱) |
| 퍼지 검색 | 내장 | 미지원 |
| MeCab 호환 | Lindera 방식 | 원본 MeCab 방식 |

### 권장: 둘 다 유지 - yada(주 조회), fst(보조/퍼지)

## 3. Viterbi 구현 레퍼런스

### vibrato (핵심 레퍼런스)
- **URL**: https://github.com/daac-tools/vibrato
- MeCab 토크나이제이션 완전 재구현
- 캐시 효율적 ID 매핑으로 대형 행렬(459MB) 처리
- 사전 학습(비용 추정) 기능 포함

### vibrato-rkyv
- **URL**: https://github.com/stellanomia/vibrato-rkyv
- rkyv 제로카피로 사전 즉시 로딩
- `Dictionary::from_zstd()` 패턴

### mecrab (SIMD 최적화 레퍼런스)
- **URL**: https://github.com/cool-japan/mecrab
- 순수 Rust, AVX2 SIMD 비용 계산
- memmap2 + DA Trie + Viterbi
- A* 알고리즘 N-best 검색

### MeCab Viterbi 알고리즘 흐름
1. **래티스 구축**: 입력 각 위치에서 DA Trie common-prefix 검색 → 노드 생성
2. **전방 패스**: `cost = min(prev.cost + connection_cost[prev.right_id][cur.left_id] + cur.word_cost)`
3. **역추적**: EOS → BOS 방향으로 prev 포인터 따라 최적 경로 추출

## 4. 바이너리 사전 포맷

### memmap2 (워크스페이스: `memmap2 = "0.9"`)
- **URL**: https://crates.io/crates/memmap2
- OS 페이지 캐시 활용, RAM 초과 사전도 가능
- `MmapOptions::new().populate()` 로 prefaulting

### zerocopy (추가 권장)
- **URL**: https://docs.rs/zerocopy
- `#[repr(C)]` 구조체를 바이트 슬라이스에서 직접 캐스팅
- 고정 크기 구조체(DictEntry, 비용 행렬)에 최적

### rkyv (워크스페이스: `rkyv = "0.8.13"`)
- **URL**: https://rkyv.org/
- 가변 길이 데이터(String, Vec) 제로카피
- rust_serialization_benchmark에서 최고 성능

### 권장 사전 레이아웃
```
system.dic:
+------------------+
| Header (fixed)   |  ← zerocopy
| DA Trie bytes    |  ← yada raw bytes
| Entry table      |  ← zerocopy DictEntry[]
| Connection matrix|  ← zerocopy i16[][]
| String pool      |  ← offset-indexed
| Footer/checksum  |
+------------------+
전체 파일: memmap2로 메모리 매핑
```

## 5. CRF (비용 학습)

### crfsuite-rs (향후 추가)
- **URL**: https://github.com/messense/crfsuite-rs
- C crfsuite 바인딩, 학습+추론 모두 지원
- `mecab-ko-dict-builder`에 통합 예정

### 초기 전략: 기존 mecab-ko-dic의 사전 학습된 비용 값 그대로 사용

## 6. 압축

### zstd (워크스페이스: `zstd = "0.13"`) - **1순위**
- 해제 속도: LZMA 대비 5-10x 빠름
- vibrato-rkyv이 이 방식 사용
- 딕셔너리 모드로 소규모 항목 압축률 향상

### lzma-rs (워크스페이스: `lzma-rs = "0.3"`) - 호환용
- 기존 MeCab-Ko 배포 포맷 호환

### 권장: zstd(배포) + memmap2(런타임), lzma-rs(호환)

## 학습 포인트
1. **제로카피 계층화**: 고정크기(zerocopy) + 가변크기(rkyv) + 파일매핑(memmap2) 3단 구성
2. **DA Trie의 common-prefix-search**가 래티스 구축의 핵심 - 한 위치에서 가능한 모든 형태소를 한번에 탐색
3. **vibrato의 캐시 효율적 ID 매핑**이 대형 연접 비용 행렬 성능의 핵심

## 참고 자료
- [Index 1.6B Keys with Automata and Rust](https://burntsushi.net/transducers/) - FST 심층 해설
- [How Japanese Tokenizers Work](https://medium.com/data-science/how-japanese-tokenizers-work-87ab6b256984) - Viterbi 래티스 시각화
- [vibrato GitHub](https://github.com/daac-tools/vibrato) - Rust MeCab 재구현 코드

## 프로젝트 적용 방안
- `mecab-ko-dict`: yada + fst + memmap2 + zerocopy 조합으로 사전 로더 구현
- `mecab-ko-core`: vibrato 아키텍처 참고하여 Lattice + Viterbi 구현
- `mecab-ko-dict-builder`: zstd 압축 + rkyv 직렬화로 사전 빌드 파이프라인
