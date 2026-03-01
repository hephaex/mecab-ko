# Sprint 7 Memory KPI Report

## 측정일: 2026-03-01

## 환경
- OS: macOS Darwin 25.3.0
- Rust: stable
- Build: release
- Dictionary: mecab-ko-dic 2.1.1 (816,283 entries)

## 측정 결과

### Memory Usage

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Peak Memory Footprint | 211 MB | < 150 MB | ⚠️ OVER |
| Maximum RSS | 215 MB | < 150 MB | ⚠️ OVER |
| Cold Start Time | 0.13s | < 0.2s | ✅ PASS |

### Dictionary File Sizes

| File | Size | Description |
|------|------|-------------|
| sys.dic | 16 MB | Double-Array Trie |
| matrix.bin | 21 MB | Connection cost matrix |
| entries.bin | 56 MB | Entry data (surface, features) |
| unk.bin | 486 B | Unknown word rules |
| **Total** | **93 MB** | On-disk size |

### Memory Amplification

- On-disk: 93 MB
- In-memory: 215 MB
- Amplification factor: 2.3x

## 분석

### 메모리 구성 추정

1. **Trie 구조** (~40 MB)
   - yada Double-Array Trie는 메모리에서 확장됨
   - 인덱스 오버헤드 포함

2. **연접 비용 행렬** (~50 MB)
   - 2차원 배열로 확장
   - i16 값들의 연속 배열

3. **엔트리 데이터** (~100 MB)
   - String 할당 오버헤드
   - Vec 구조 오버헤드
   - 해시맵 인덱싱

4. **기타** (~25 MB)
   - Rust 런타임
   - 임시 버퍼
   - 스택

## 최적화 제안

### 단기 (Sprint 8)
1. **entries 지연 로딩**: 전체 entries를 미리 로드하지 않고 필요시 로드
2. **String interning**: 중복 문자열 공유
3. **memory-mapped I/O**: matrix를 mmap으로 접근

### 중기
1. **압축 행렬**: 희소 행렬 압축 (COO, CSR 형식)
2. **Zero-copy 역직렬화**: rkyv 활용 강화
3. **사전 분할**: 자주 사용되는 엔트리만 메모리 로드

### 장기
1. **사전 재설계**: 더 컴팩트한 바이너리 포맷
2. **온디맨드 로딩**: LRU 캐시 기반 엔트리 관리

## 비교 (참고)

| Analyzer | Memory | Notes |
|----------|--------|-------|
| MeCab-Ko (C++) | ~80 MB | mmap 기반 |
| Kiwi | ~150 MB | 자체 모델 |
| mecab-ko (Rust) | ~215 MB | 현재 구현 |

## 결론

현재 메모리 사용량(215MB)은 목표(150MB)를 43% 초과합니다.
주요 원인은 entries 데이터의 String 할당과 연접 비용 행렬의 메모리 확장입니다.

Sprint 8에서 entries 지연 로딩과 mmap 적용을 통해 목표 달성을 시도할 예정입니다.
