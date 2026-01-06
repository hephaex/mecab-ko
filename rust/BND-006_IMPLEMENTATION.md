# BND-006: Kiwi 형태소 분석기 호환 레이어 구현

## 개요

MeCab-Ko와 Kiwi 형태소 분석기 간의 상호 운용성을 위한 품사 태그 매핑 및 변환 기능을 구현했습니다.

## 구현 내용

### 파일 위치
- `/home/mare/mecab-ko/rust/crates/mecab-ko-core/src/kiwi_compat.rs` (798 lines)

### 주요 구성 요소

#### 1. KiwiPosTag Enum
Kiwi 형태소 분석기의 모든 품사 태그를 정의:
- 체언 (NNG, NNP, NNB, NR, NP)
- 용언 (VV, VA, VX, VCP, VCN)
- 수식언 (MM, MAG, MAJ)
- 독립언 (IC)
- 조사 (JKS, JKC, JKG, JKO, JKB, JKV, JKQ, JX, JC)
- 어미 (EP, EF, EC, ETN, ETM)
- 접사 (XPN, XSN, XSV, XSA, XR)
- 기호 (SF, SP, SS, SE, SO, SW, SL, SH, SN)
- **웹 관련 (Kiwi 전용)**: W_URL, W_EMAIL, W_HASHTAG, W_MENTION, W_EMOJI, W_OTHER
- 특수 (Unknown)

#### 2. 변환 함수

##### `to_kiwi_tag(mecab_tag: PosTag) -> KiwiPosTag`
MeCab-Ko 품사 태그를 Kiwi 품사 태그로 변환:
- **1:1 매핑**: 대부분의 세종 태그 (NNG, VV, JKS 등)
- **통합 매핑**:
  - `NNBC` (단위 의존 명사) → `NNB` (의존 명사)
  - `SSO`/`SSC` (여는/닫는 괄호) → `SS` (괄호 통합)
  - `SC` (구분자) → `SP` (쉼표)
  - `SY` (기타 기호) → `SO` (그외 기호)

##### `from_kiwi_tag(kiwi_tag: KiwiPosTag) -> PosTag`
Kiwi 품사 태그를 MeCab-Ko 품사 태그로 역변환:
- **1:1 역매핑**: 공통 세종 태그
- **정보 손실 매핑**:
  - `NNB` → `NNB` (NNBC 정보 손실)
  - `SS` → `SSO` (기본값, SSC 정보 손실)
  - `SP` → `SP` (SC 정보 손실)
  - 웹 관련 태그 (W_*) → `SL` (외국어)

#### 3. KiwiToken 구조체
Kiwi 호환 토큰 표현:
```rust
pub struct KiwiToken {
    pub form: String,      // 형태소 표면형
    pub tag: KiwiPosTag,   // 품사 태그
    pub start: usize,      // 시작 위치 (바이트)
    pub length: usize,     // 길이 (바이트)
    pub score: f64,        // 분석 점수 (로그 확률)
}
```

메서드:
- `new()`: 토큰 생성
- `end()`: 끝 위치 계산
- `to_mecab_tag()`: MeCab 품사 태그로 변환
- `Display` trait 구현: "형태소/품사" 형식

## 테스트 커버리지

총 16개의 단위 테스트 구현 (모두 통과):

1. `test_kiwi_tag_from_str` - 문자열 파싱
2. `test_kiwi_tag_as_str` - 문자열 변환
3. `test_to_kiwi_tag_nominals` - 체언 변환
4. `test_to_kiwi_tag_predicates` - 용언 변환
5. `test_to_kiwi_tag_particles` - 조사 변환
6. `test_to_kiwi_tag_symbols` - 기호 통합 변환
7. `test_from_kiwi_tag_nominals` - 역변환 (체언)
8. `test_from_kiwi_tag_symbols` - 역변환 (기호)
9. `test_from_kiwi_tag_web` - 웹 태그 처리
10. `test_roundtrip_conversion` - 왕복 변환 검증
11. `test_lossy_conversion` - 정보 손실 검증
12. `test_kiwi_token_creation` - 토큰 생성
13. `test_kiwi_token_display` - Display trait
14. `test_kiwi_token_to_mecab` - 토큰 변환
15. `test_all_kiwi_tags_covered` - 모든 Kiwi 태그 변환 가능 검증
16. `test_all_mecab_tags_covered` - 모든 MeCab 태그 변환 가능 검증

## API 사용 예제

```rust
use mecab_ko_core::{from_kiwi_tag, to_kiwi_tag, KiwiPosTag, KiwiToken, PosTag};

// MeCab -> Kiwi 변환
let kiwi_tag = to_kiwi_tag(PosTag::NNG);
assert_eq!(kiwi_tag, KiwiPosTag::NNG);

// Kiwi -> MeCab 변환
let mecab_tag = from_kiwi_tag(KiwiPosTag::NNG);
assert_eq!(mecab_tag, PosTag::NNG);

// 웹 태그 처리
let url_tag = from_kiwi_tag(KiwiPosTag::W_URL);
assert_eq!(url_tag, PosTag::SL); // 외국어로 매핑

// KiwiToken 사용
let token = KiwiToken::new("안녕", KiwiPosTag::NNG, 0, 6, -10.5);
println!("{}", token); // 출력: "안녕/NNG"
assert_eq!(token.to_mecab_tag(), PosTag::NNG);
```

## 품사 태그 매핑 상세

### 완전 호환 (1:1 매핑)
대부분의 세종 품사 태그는 완전히 호환됩니다:
- 체언: NNG, NNP, NNB, NR, NP
- 용언: VV, VA, VX, VCP, VCN
- 수식언: MM, MAG, MAJ
- 독립언: IC
- 조사: JKS, JKC, JKG, JKO, JKB, JKV, JKQ, JX, JC
- 어미: EP, EF, EC, ETN, ETM
- 접사: XPN, XSN, XSV, XSA, XR
- 기호: SF, SE, SL, SH, SN

### 통합 매핑 (정보 손실 가능)

| MeCab 태그 | Kiwi 태그 | 역변환 | 비고 |
|-----------|----------|--------|------|
| NNBC | NNB | NNB | 단위명사 정보 손실 |
| SSO | SS | SSO | 괄호 방향 정보 손실 |
| SSC | SS | SSO | 괄호 방향 정보 손실 |
| SC | SP | SP | 구분자/쉼표 구분 손실 |
| SY | SO | SY | 왕복 가능 |
| SP | SP | SP | 호환 |

### Kiwi 전용 태그 처리

Kiwi의 웹 관련 태그는 MeCab-Ko의 SL (외국어)로 매핑:
- W_URL → SL
- W_EMAIL → SL
- W_HASHTAG → SL
- W_MENTION → SL
- W_EMOJI → SL
- W_OTHER → SL

## 코드 품질

- ✅ 모든 테스트 통과 (71/71)
- ✅ `cargo clippy -- -D warnings` 통과
- ✅ `cargo fmt` 적용
- ✅ 모든 public API에 rustdoc 주석 포함
- ✅ `unsafe` 코드 없음
- ✅ `unwrap()`/`expect()` 사용 없음

## 문서화

- 모듈 레벨 rustdoc 주석
- 모든 public 함수/구조체에 예제 포함
- 품사 태그 매핑 상세 설명
- 정보 손실 케이스 명시

## 향후 개선 사항

1. **성능 최적화**: const 함수를 활용한 컴파일 타임 변환
2. **매핑 테이블**: 런타임 매핑 테이블 추가 고려
3. **통계 정보**: 태그 변환 통계 수집 기능
4. **사용자 정의 매핑**: 커스텀 매핑 규칙 지원

## 관련 이슈

- BND-006: Kiwi 형태소 분석기 호환 레이어

## 참고 자료

- [Kiwi GitHub](https://github.com/bab2min/Kiwi)
- 세종 품사 태그 체계
- `/home/mare/mecab-ko/rust/crates/mecab-ko-core/src/pos_tag.rs`
