# MeCab 바이너리 사전 포맷 상세 조사

**날짜**: 2026-02-23
**카테고리**: algorithms

## 요약 (3줄)
1. sys.dic = 72바이트 헤더 + DA Trie + Token 배열(16B/엔트리) + Feature 문자열 풀
2. 연접 비용 행렬: matrix.bin = u16 lsize + u16 rsize + i16[lsize*rsize], ~20MB
3. 미등록어: char.bin의 CharInfo(32비트 패킹) + unk.dic(sys.dic 동일 포맷)

## sys.dic 바이너리 레이아웃

### 헤더 (72 bytes)
| 오프셋 | 크기 | 필드 | 설명 |
|--------|------|------|------|
| 0 | 4 | magic | `file_size XOR 0xEF718F77` |
| 4 | 4 | version | 사전 버전 (0x66 = 102) |
| 8 | 4 | type | 0=SYS, 1=USR, 2=UNK |
| 12 | 4 | lexsize | 엔트리 수 |
| 16 | 4 | lsize | 좌문맥 ID 수 |
| 20 | 4 | rsize | 우문맥 ID 수 |
| 24 | 4 | dsize | DA Trie 데이터 크기 |
| 28 | 4 | tsize | Token 버퍼 크기 |
| 32 | 4 | fsize | Feature 버퍼 크기 |
| 36 | 4 | dummy | 패딩 |
| 40 | 32 | charset | 인코딩 문자열 ("UTF-8") |

### 데이터 섹션
```
[Header: 72B]
[Double-Array: dsize bytes]   ← Darts trie (base/check 배열)
[Token Buffer: tsize bytes]   ← Token 구조체 배열
[Feature Buffer: fsize bytes] ← 널 종료 자질 문자열
```

### Token 구조체 (16 bytes)
```rust
#[repr(C)]
struct Token {
    lc_attr: u16,    // 좌문맥 ID
    rc_attr: u16,    // 우문맥 ID
    posid: u16,      // 품사 ID
    wcost: i16,      // 단어 비용
    feature: u32,    // Feature 버퍼 오프셋
    compound: u32,   // 복합어 플래그
}
```

### 사전 검증
`(읽은_magic XOR 0xEF718F77) == 파일_크기`

## 연접 비용 행렬 (matrix.bin)

### 포맷
```
[u16: lsize] [u16: rsize]
[i16: cost_0_0] [i16: cost_0_1] ... [i16: cost_(lsize*rsize-1)]
```

### 인덱싱
```rust
fn cost(&self, right_id: u16, left_id: u16) -> i16 {
    self.matrix[right_id as usize * self.lsize as usize + left_id as usize]
}
```

### 크기 (mecab-ko-dic)
- lsize: ~2,690 / rsize: ~3,815
- 셀 수: ~10.26M
- 크기: ~20MB (비압축)

## 미등록어 처리

### char.def → char.bin
```
카테고리명  INVOKE(0|1)  GROUP(0|1)  LENGTH(n)
```

### CharInfo 비트 레이아웃 (32비트)
```
비트 31:     invoke (1비트)
비트 30:     group (1비트)
비트 26-29:  length (4비트)
비트 18-25:  default_type (8비트) - 기본 카테고리 인덱스
비트 0-17:   type (18비트) - 호환 카테고리 비트마스크
```

### 미등록어 생성 알고리즘
1. 현재 위치 문자의 CharInfo 조회
2. INVOKE=1 이거나 사전 매칭 없으면:
   - GROUP=1: 같은 카테고리 연속 문자를 하나의 노드로
   - LENGTH>0: 길이 1~N의 후보 노드 생성
3. unk.dic에서 해당 카테고리의 Token 데이터 조회

## DA Trie Common Prefix Search

### 핵심 알고리즘
```rust
fn common_prefix_search(&self, key: &[u8]) -> Vec<(u32, usize)> {
    let mut results = vec![];
    let mut node = 0; // root
    for (i, &c) in key.iter().enumerate() {
        let next = self.base[node] as usize + c as usize + 1;
        if self.check[next] != node { break; }
        node = next;
        // 터미널 체크
        let term = self.base[node] as usize + 1;
        if self.check[term] == node && self.base[term] < 0 {
            results.push(((-self.base[term] - 1) as u32, i + 1));
        }
    }
    results
}
```

## Rust 구현 매핑

| MeCab 컴포넌트 | mecab-ko 크레이트 | Rust 크레이트 |
|----------------|-------------------|---------------|
| sys.dic 로더 | mecab-ko-dict | memmap2 + zerocopy |
| DA Trie | mecab-ko-dict | yada |
| Token 파서 | mecab-ko-dict | zerocopy (#[repr(C)]) |
| matrix.bin | mecab-ko-dict | memmap2 + byteorder |
| char.bin | mecab-ko-dict | 커스텀 파서 |
| unk.dic | mecab-ko-dict | sys.dic 동일 로더 |
| Lattice | mecab-ko-core | 커스텀 구현 |
| Viterbi | mecab-ko-core | 커스텀 구현 |

## 학습 포인트
1. **magic 검증이 파일 무결성 체크 역할** - XOR 0xEF718F77로 크기와 매직 동시 검증
2. **Token 16바이트 고정 크기** - zerocopy로 제로카피 가능, 배열 인덱싱으로 O(1) 접근
3. **CharInfo 32비트 패킹**으로 65536 유니코드 코드포인트를 256KB에 저장

## 참고 자료
- [MeCab dictionary.cpp](https://github.com/taku910/mecab/blob/master/mecab/src/dictionary.cpp) - 바이너리 포맷 구현
- [MeCab connector.cpp](https://sources.debian.org/src/mecab/0.996-3.1/src/connector.cpp/) - 연접 비용 행렬
- [MeCab char_property.cpp](https://github.com/taku910/mecab/blob/master/mecab/src/char_property.cpp) - CharInfo 비트 레이아웃
