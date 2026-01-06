# 한국어 품사 태그 통합 매핑 가이드

> **문서 버전**: 1.0
> **작성일**: 2026-01-04
> **이슈**: DIC-002 세종 품사 태그 체계 검토 및 확장 설계

이 문서는 세종 품사 태그 체계를 기준으로 주요 한국어 형태소 분석기들의 품사 태그를 비교하고 통합 매핑 테이블을 제공합니다.

---

## 목차

1. [세종 품사 태그 체계](#1-세종-품사-태그-체계)
2. [분석기별 태그 비교](#2-분석기별-태그-비교)
3. [통합 매핑 테이블](#3-통합-매핑-테이블)
4. [태그 확장 권장사항](#4-태그-확장-권장사항)
5. [Rust 구현 가이드](#5-rust-구현-가이드)

---

## 1. 세종 품사 태그 체계

### 1.1 개요

**21세기 세종계획** (1998-2007)에서 정의된 품사 태그 체계로, 한국어 형태소 분석의 사실상 표준입니다.

- **총 태그 수**: 43-45개
- **기반**: 학교문법 5언 체계 + 기호
- **적용**: mecab-ko-dic, Kiwi, KoNLPy, khaiii 등

### 1.2 5언 분류 체계

```
세종 품사 태그 (5언 + 기타)
├── 체언 (Nominals) ─────────── 명사, 대명사, 수사
├── 용언 (Predicates) ────────── 동사, 형용사, 보조용언, 지정사
├── 수식언 (Modifiers) ───────── 관형사, 부사
├── 독립언 (Interjection) ────── 감탄사
├── 관계언 (Particles) ───────── 조사
└── 기타
    ├── 어미 (Endings)
    ├── 접사 (Affixes)
    ├── 어근 (Root)
    └── 기호 (Symbols)
```

### 1.3 세종 품사 태그 전체 목록 (43개)

#### 체언 (5개)
| 태그 | 명칭 | 영문 | 예시 |
|------|------|------|------|
| **NNG** | 일반 명사 | General Noun | 사과, 컴퓨터, 사랑 |
| **NNP** | 고유 명사 | Proper Noun | 서울, 세종대왕, 삼성 |
| **NNB** | 의존 명사 | Dependent Noun | 것, 수, 줄, 뿐 |
| **NP** | 대명사 | Pronoun | 나, 너, 우리, 이것 |
| **NR** | 수사 | Numeral | 하나, 둘, 첫째 |

#### 용언 (5개)
| 태그 | 명칭 | 영문 | 예시 |
|------|------|------|------|
| **VV** | 동사 | Verb | 먹다, 가다, 하다 |
| **VA** | 형용사 | Adjective | 예쁘다, 크다, 좋다 |
| **VX** | 보조 용언 | Auxiliary Predicate | -아/어 있다, -고 싶다 |
| **VCP** | 긍정 지정사 | Positive Copula | 이다 |
| **VCN** | 부정 지정사 | Negative Copula | 아니다 |

#### 수식언 (3개)
| 태그 | 명칭 | 영문 | 예시 |
|------|------|------|------|
| **MM** | 관형사 | Determiner | 새, 헌, 이, 그, 저 |
| **MAG** | 일반 부사 | General Adverb | 매우, 아주, 빨리 |
| **MAJ** | 접속 부사 | Conjunctive Adverb | 그러나, 그리고 |

#### 독립언 (1개)
| 태그 | 명칭 | 영문 | 예시 |
|------|------|------|------|
| **IC** | 감탄사 | Interjection | 아, 와, 어머나 |

#### 관계언 - 조사 (9개)
| 태그 | 명칭 | 영문 | 예시 |
|------|------|------|------|
| **JKS** | 주격 조사 | Subjective Case | 이/가 |
| **JKC** | 보격 조사 | Complement Case | 이/가 |
| **JKG** | 관형격 조사 | Genitive Case | 의 |
| **JKO** | 목적격 조사 | Objective Case | 을/를 |
| **JKB** | 부사격 조사 | Adverbial Case | 에, 에서, 로 |
| **JKV** | 호격 조사 | Vocative Case | 아, 야 |
| **JKQ** | 인용격 조사 | Quotative Case | 라고, 고 |
| **JX** | 보조사 | Auxiliary Particle | 은/는, 도, 만 |
| **JC** | 접속 조사 | Conjunctive Particle | 와/과, 하고 |

#### 어미 (5개)
| 태그 | 명칭 | 영문 | 예시 |
|------|------|------|------|
| **EP** | 선어말 어미 | Pre-final Ending | -시-, -었-, -겠- |
| **EF** | 종결 어미 | Final Ending | -다, -니, -라 |
| **EC** | 연결 어미 | Connective Ending | -고, -면, -어서 |
| **ETN** | 명사형 전성 어미 | Nominal Trans. Ending | -ㅁ, -기 |
| **ETM** | 관형형 전성 어미 | Adnominal Trans. Ending | -ㄴ, -는, -ㄹ |

#### 접사 및 어근 (5개)
| 태그 | 명칭 | 영문 | 예시 |
|------|------|------|------|
| **XPN** | 체언 접두사 | Noun Prefix | 풋-, 맨-, 헛- |
| **XSN** | 명사파생 접미사 | Noun Suffix | -님, -적, -화 |
| **XSV** | 동사파생 접미사 | Verb Suffix | -하다, -되다 |
| **XSA** | 형용사파생 접미사 | Adj. Suffix | -스럽다, -롭다 |
| **XR** | 어근 | Root | 꼼꼼, 달콤 |

#### 기호 (9개)
| 태그 | 명칭 | 영문 | 예시 |
|------|------|------|------|
| **SF** | 마침 부호 | Terminal Punctuation | . ? ! |
| **SP** | 쉼표/콜론 | Comma/Colon | , ; : |
| **SS** | 따옴표/괄호 | Quote/Bracket | " ' ( ) |
| **SE** | 줄임표 | Ellipsis | ... |
| **SO** | 붙임표 | Hyphen | - ~ |
| **SW** | 기타 기호 | Other Symbol | @ # $ |
| **SL** | 외국어 | Foreign | hello, ABC |
| **SH** | 한자 | Hanja | 韓國, 大學 |
| **SN** | 숫자 | Number | 123, 45.6 |

#### 분석 불능 (3개)
| 태그 | 명칭 | 영문 |
|------|------|------|
| **NF** | 명사 추정 | Noun Estimation |
| **NV** | 용언 추정 | Predicate Estimation |
| **NA** | 분석 불능 | Unknown |

---

## 2. 분석기별 태그 비교

### 2.1 비교 대상

| 분석기 | 기반 | 특징 |
|--------|------|------|
| **mecab-ko-dic** | MeCab + 세종 | 사실상 표준, Viterbi 알고리즘 |
| **Kiwi** | 세종 | 현대적, 웹 텍스트 특화 |
| **Nori** | mecab-ko-dic | Elasticsearch 공식 플러그인 |

### 2.2 주요 차이점 요약

| 기능 | 세종 | mecab-ko-dic | Kiwi | Nori |
|------|------|--------------|------|------|
| 기본 태그 | 43개 | 43개+ | 43개+ | 36개 |
| 단위 명사 분리 | NNB | **NNBC** | NNB | **NNBC** |
| 조사 세분화 | 9개 | 9개 | 9개 | **J (통합)** |
| 어미 세분화 | 5개 | 5개 | 5개 | **E (통합)** |
| 괄호 세분화 | SS | **SSO/SSC** | **SSO/SSC** | **SSO/SSC** |
| 불규칙 활용 | - | - | **-R/-I** | - |
| 웹 패턴 | - | - | **W_*** | - |
| 사용자 정의 | - | - | **USER0-4** | - |

### 2.3 mecab-ko-dic 확장

```
세종 기본 태그 + mecab-ko-dic 확장
├── NNBC     : 단위 의존 명사 (세종 NNB에서 분리)
├── SSO      : 여는 괄호 (세종 SS에서 분리)
├── SSC      : 닫는 괄호 (세종 SS에서 분리)
├── SC       : 구분자 (쉼표 등)
├── SY       : 기타 기호
├── UNKNOWN  : 미등록어
├── Compound : 복합어 타입
├── Inflect  : 활용형 타입
└── Preanalysis : 기분석 타입
```

### 2.4 Kiwi 확장

```
세종 기본 태그 + Kiwi 확장
├── 불규칙 활용
│   ├── VV-R, VA-R, VX-R   : 규칙 활용
│   └── VV-I, VA-I, VX-I   : 불규칙 활용
├── 웹 패턴
│   ├── W_URL      : URL
│   ├── W_EMAIL    : 이메일
│   ├── W_HASHTAG  : 해시태그
│   ├── W_MENTION  : 멘션
│   ├── W_SERIAL   : 일련번호
│   └── W_EMOJI    : 이모지
├── 음운 현상
│   ├── Z_CODA     : 덧붙는 받침
│   └── Z_SIOT     : 사이시옷
├── 기타
│   ├── XSM        : 부사 파생 접미사
│   ├── SB         : 순서 글머리
│   └── USER0-4    : 사용자 정의
└── UN             : 분석 불능
```

### 2.5 Nori 간소화

```
Nori 태그 간소화
├── J   : 모든 조사 통합 (JKS, JKO, JX 등 → J)
├── E   : 모든 어미 통합 (EP, EF, EC 등 → E)
└── 기본 태그는 mecab-ko-dic과 동일
```

---

## 3. 통합 매핑 테이블

### 3.1 체언 매핑

| 세종 | mecab-ko-dic | Kiwi | Nori | 설명 |
|------|--------------|------|------|------|
| NNG | NNG | NNG | NNG | 일반 명사 |
| NNP | NNP | NNP | NNP | 고유 명사 |
| NNB | NNB | NNB | NNB | 의존 명사 |
| NNB* | **NNBC** | NNB | **NNBC** | 단위 의존 명사 |
| NP | NP | NP | NP | 대명사 |
| NR | NR | NR | NR | 수사 |

### 3.2 용언 매핑

| 세종 | mecab-ko-dic | Kiwi | Nori | 설명 |
|------|--------------|------|------|------|
| VV | VV | VV / VV-R / VV-I | VV | 동사 |
| VA | VA | VA / VA-R / VA-I | VA | 형용사 |
| VX | VX | VX / VX-R / VX-I | VX | 보조 용언 |
| VCP | VCP | VCP | VCP | 긍정 지정사 |
| VCN | VCN | VCN | VCN | 부정 지정사 |

### 3.3 조사 매핑

| 세종 | mecab-ko-dic | Kiwi | Nori | 설명 |
|------|--------------|------|------|------|
| JKS | JKS | JKS | **J** | 주격 조사 |
| JKC | JKC | JKC | **J** | 보격 조사 |
| JKG | JKG | JKG | **J** | 관형격 조사 |
| JKO | JKO | JKO | **J** | 목적격 조사 |
| JKB | JKB | JKB | **J** | 부사격 조사 |
| JKV | JKV | JKV | **J** | 호격 조사 |
| JKQ | JKQ | JKQ | **J** | 인용격 조사 |
| JX | JX | JX | **J** | 보조사 |
| JC | JC | JC | **J** | 접속 조사 |

### 3.4 어미 매핑

| 세종 | mecab-ko-dic | Kiwi | Nori | 설명 |
|------|--------------|------|------|------|
| EP | EP | EP | **E** | 선어말 어미 |
| EF | EF | EF | **E** | 종결 어미 |
| EC | EC | EC | **E** | 연결 어미 |
| ETN | ETN | ETN | **E** | 명사형 전성 어미 |
| ETM | ETM | ETM | **E** | 관형형 전성 어미 |

### 3.5 접사 매핑

| 세종 | mecab-ko-dic | Kiwi | Nori | 설명 |
|------|--------------|------|------|------|
| XPN | XPN | XPN | XPN | 체언 접두사 |
| XSN | XSN | XSN | XSN | 명사파생 접미사 |
| XSV | XSV | XSV | XSV | 동사파생 접미사 |
| XSA | XSA | XSA / XSA-R / XSA-I | XSA | 형용사파생 접미사 |
| - | - | **XSM** | - | 부사파생 접미사 (Kiwi 전용) |
| XR | XR | XR | XR | 어근 |

### 3.6 기호 매핑

| 세종 | mecab-ko-dic | Kiwi | Nori | 설명 |
|------|--------------|------|------|------|
| SF | SF | SF | SF | 마침 부호 |
| SP | SP | SP | SP | 공백 |
| SS | SSO, SSC | SSO, SSC | SSO, SSC | 괄호 |
| SE | SE | SE | SE | 줄임표 |
| SO | SY | SO | SY | 붙임표/기타 기호 |
| SW | SY | SW | SY | 기타 기호 |
| SL | SL | SL | SL | 외국어 |
| SH | SH | SH | SH | 한자 |
| SN | SN | SN | SN | 숫자 |
| - | SC | SC | SC | 구분자 |
| - | - | **SB** | - | 순서 글머리 (Kiwi 전용) |

### 3.7 특수/확장 태그 매핑

| 용도 | mecab-ko-dic | Kiwi | Nori | 설명 |
|------|--------------|------|------|------|
| 미등록어 | UNKNOWN | UN | UNKNOWN | 사전에 없는 단어 |
| URL | - | W_URL | - | 웹 주소 |
| 이메일 | - | W_EMAIL | - | 이메일 주소 |
| 해시태그 | - | W_HASHTAG | - | #태그 |
| 멘션 | - | W_MENTION | - | @멘션 |
| 이모지 | - | W_EMOJI | - | 이모지 문자 |
| 사용자 정의 | - | USER0-4 | - | 커스텀 태그 |

---

## 4. 태그 확장 권장사항

### 4.1 mecab-ko Rust 구현 권장 태그

기존 mecab-ko-dic 호환성을 유지하면서 다음 확장을 권장합니다:

#### 필수 유지 (세종 + mecab-ko-dic)
```rust
// 기본 43개 세종 태그 + mecab-ko-dic 확장
pub enum PosTag {
    // 체언
    NNG, NNP, NNB, NNBC, NP, NR,
    // 용언
    VV, VA, VX, VCP, VCN,
    // 수식언
    MM, MAG, MAJ,
    // 독립언
    IC,
    // 조사
    JKS, JKC, JKG, JKO, JKB, JKV, JKQ, JX, JC,
    // 어미
    EP, EF, EC, ETN, ETM,
    // 접사
    XPN, XSN, XSV, XSA, XR,
    // 기호
    SF, SP, SSO, SSC, SC, SE, SY, SL, SH, SN,
    // 특수
    Unknown,
}
```

#### 선택적 확장 (Kiwi 참고)
```rust
// 웹 텍스트 지원 (선택적)
pub enum WebPattern {
    Url,
    Email,
    Hashtag,
    Mention,
    Emoji,
}

// 불규칙 활용 표시 (선택적)
pub enum Regularity {
    Regular,    // 규칙 활용
    Irregular,  // 불규칙 활용
}
```

### 4.2 태그 변환 함수

분석기 간 호환을 위한 변환 함수:

```rust
impl PosTag {
    /// Nori 호환 태그로 변환
    pub fn to_nori(&self) -> NoriTag {
        match self {
            // 조사 통합
            Self::JKS | Self::JKC | Self::JKG | Self::JKO |
            Self::JKB | Self::JKV | Self::JKQ | Self::JX | Self::JC => NoriTag::J,
            // 어미 통합
            Self::EP | Self::EF | Self::EC | Self::ETN | Self::ETM => NoriTag::E,
            // 나머지는 동일
            _ => self.clone().into(),
        }
    }

    /// 세종 기본 태그로 정규화
    pub fn to_sejong(&self) -> SejongTag {
        match self {
            Self::NNBC => SejongTag::NNB,  // 단위명사 → 의존명사
            Self::SSO | Self::SSC => SejongTag::SS,  // 괄호 통합
            Self::SC | Self::SY => SejongTag::SO,  // 기호 통합
            _ => self.clone().into(),
        }
    }
}
```

### 4.3 신조어 카테고리 확장

현대 언어 현상을 반영한 확장 제안:

| 태그 | 설명 | 예시 |
|------|------|------|
| `NNG+coined` | 신조어 명사 | 갓생, 소확행 |
| `NNG+abbrev` | 약어 | ㅋㅋ, ㅎㅎ |
| `SL+loanword` | 외래어 표기 | 컴퓨터, 인터넷 |
| `IC+onomatopoeia` | 의성어 | 멍멍, 야옹 |
| `IC+mimetic` | 의태어 | 반짝반짝, 살금살금 |

---

## 5. Rust 구현 가이드

### 5.1 PosTag Enum 정의

```rust
// rust/mecab-ko-core/src/pos_tag.rs

/// 품사 태그 (세종 품사 태그 체계 기반)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PosTag {
    // === 체언 (Nominals) ===
    /// 일반 명사 (General Noun)
    NNG = 0,
    /// 고유 명사 (Proper Noun)
    NNP = 1,
    /// 의존 명사 (Dependent Noun)
    NNB = 2,
    /// 단위 의존 명사 (Counter/Unit Noun) - mecab-ko-dic 확장
    NNBC = 3,
    /// 대명사 (Pronoun)
    NP = 4,
    /// 수사 (Numeral)
    NR = 5,

    // === 용언 (Predicates) ===
    /// 동사 (Verb)
    VV = 10,
    /// 형용사 (Adjective)
    VA = 11,
    /// 보조 용언 (Auxiliary Predicate)
    VX = 12,
    /// 긍정 지정사 '이다' (Positive Copula)
    VCP = 13,
    /// 부정 지정사 '아니다' (Negative Copula)
    VCN = 14,

    // === 수식언 (Modifiers) ===
    /// 관형사 (Determiner)
    MM = 20,
    /// 일반 부사 (General Adverb)
    MAG = 21,
    /// 접속 부사 (Conjunctive Adverb)
    MAJ = 22,

    // === 독립언 (Interjection) ===
    /// 감탄사 (Interjection)
    IC = 30,

    // === 관계언 - 조사 (Particles) ===
    /// 주격 조사 (Subjective Case)
    JKS = 40,
    /// 보격 조사 (Complement Case)
    JKC = 41,
    /// 관형격 조사 (Genitive Case)
    JKG = 42,
    /// 목적격 조사 (Objective Case)
    JKO = 43,
    /// 부사격 조사 (Adverbial Case)
    JKB = 44,
    /// 호격 조사 (Vocative Case)
    JKV = 45,
    /// 인용격 조사 (Quotative Case)
    JKQ = 46,
    /// 보조사 (Auxiliary Particle)
    JX = 47,
    /// 접속 조사 (Conjunctive Particle)
    JC = 48,

    // === 어미 (Endings) ===
    /// 선어말 어미 (Pre-final Ending)
    EP = 50,
    /// 종결 어미 (Final Ending)
    EF = 51,
    /// 연결 어미 (Connective Ending)
    EC = 52,
    /// 명사형 전성 어미 (Nominal Transformative)
    ETN = 53,
    /// 관형형 전성 어미 (Adnominal Transformative)
    ETM = 54,

    // === 접사 (Affixes) ===
    /// 체언 접두사 (Noun Prefix)
    XPN = 60,
    /// 명사파생 접미사 (Noun Derivational Suffix)
    XSN = 61,
    /// 동사파생 접미사 (Verb Derivational Suffix)
    XSV = 62,
    /// 형용사파생 접미사 (Adjective Derivational Suffix)
    XSA = 63,
    /// 어근 (Root)
    XR = 64,

    // === 기호 (Symbols) ===
    /// 마침 부호 (. ? !)
    SF = 70,
    /// 공백 (Space)
    SP = 71,
    /// 여는 괄호 (Opening Bracket)
    SSO = 72,
    /// 닫는 괄호 (Closing Bracket)
    SSC = 73,
    /// 구분자 (Comma, Colon)
    SC = 74,
    /// 줄임표 (Ellipsis)
    SE = 75,
    /// 기타 기호 (Other Symbol)
    SY = 76,
    /// 외국어 (Foreign)
    SL = 77,
    /// 한자 (Hanja)
    SH = 78,
    /// 숫자 (Number)
    SN = 79,

    // === 특수 (Special) ===
    /// 미등록어 (Unknown)
    Unknown = 99,
}
```

### 5.2 태그 파싱 및 변환

```rust
impl PosTag {
    /// 문자열에서 품사 태그 파싱
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "NNG" => Some(Self::NNG),
            "NNP" => Some(Self::NNP),
            "NNB" => Some(Self::NNB),
            "NNBC" => Some(Self::NNBC),
            "NP" => Some(Self::NP),
            "NR" => Some(Self::NR),
            "VV" => Some(Self::VV),
            "VA" => Some(Self::VA),
            "VX" => Some(Self::VX),
            "VCP" => Some(Self::VCP),
            "VCN" => Some(Self::VCN),
            "MM" => Some(Self::MM),
            "MAG" => Some(Self::MAG),
            "MAJ" => Some(Self::MAJ),
            "IC" => Some(Self::IC),
            "JKS" => Some(Self::JKS),
            "JKC" => Some(Self::JKC),
            "JKG" => Some(Self::JKG),
            "JKO" => Some(Self::JKO),
            "JKB" => Some(Self::JKB),
            "JKV" => Some(Self::JKV),
            "JKQ" => Some(Self::JKQ),
            "JX" => Some(Self::JX),
            "JC" => Some(Self::JC),
            "EP" => Some(Self::EP),
            "EF" => Some(Self::EF),
            "EC" => Some(Self::EC),
            "ETN" => Some(Self::ETN),
            "ETM" => Some(Self::ETM),
            "XPN" => Some(Self::XPN),
            "XSN" => Some(Self::XSN),
            "XSV" => Some(Self::XSV),
            "XSA" => Some(Self::XSA),
            "XR" => Some(Self::XR),
            "SF" => Some(Self::SF),
            "SP" => Some(Self::SP),
            "SSO" => Some(Self::SSO),
            "SSC" => Some(Self::SSC),
            "SC" => Some(Self::SC),
            "SE" => Some(Self::SE),
            "SY" => Some(Self::SY),
            "SL" => Some(Self::SL),
            "SH" => Some(Self::SH),
            "SN" => Some(Self::SN),
            "UNKNOWN" => Some(Self::Unknown),
            _ => None,
        }
    }

    /// 품사 태그 문자열 반환
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NNG => "NNG",
            Self::NNP => "NNP",
            Self::NNB => "NNB",
            Self::NNBC => "NNBC",
            Self::NP => "NP",
            Self::NR => "NR",
            Self::VV => "VV",
            Self::VA => "VA",
            Self::VX => "VX",
            Self::VCP => "VCP",
            Self::VCN => "VCN",
            Self::MM => "MM",
            Self::MAG => "MAG",
            Self::MAJ => "MAJ",
            Self::IC => "IC",
            Self::JKS => "JKS",
            Self::JKC => "JKC",
            Self::JKG => "JKG",
            Self::JKO => "JKO",
            Self::JKB => "JKB",
            Self::JKV => "JKV",
            Self::JKQ => "JKQ",
            Self::JX => "JX",
            Self::JC => "JC",
            Self::EP => "EP",
            Self::EF => "EF",
            Self::EC => "EC",
            Self::ETN => "ETN",
            Self::ETM => "ETM",
            Self::XPN => "XPN",
            Self::XSN => "XSN",
            Self::XSV => "XSV",
            Self::XSA => "XSA",
            Self::XR => "XR",
            Self::SF => "SF",
            Self::SP => "SP",
            Self::SSO => "SSO",
            Self::SSC => "SSC",
            Self::SC => "SC",
            Self::SE => "SE",
            Self::SY => "SY",
            Self::SL => "SL",
            Self::SH => "SH",
            Self::SN => "SN",
            Self::Unknown => "UNKNOWN",
        }
    }

    /// 품사 대분류 반환
    pub fn category(&self) -> PosCategory {
        match self {
            Self::NNG | Self::NNP | Self::NNB | Self::NNBC | Self::NP | Self::NR => {
                PosCategory::Nominal
            }
            Self::VV | Self::VA | Self::VX | Self::VCP | Self::VCN => PosCategory::Predicate,
            Self::MM | Self::MAG | Self::MAJ => PosCategory::Modifier,
            Self::IC => PosCategory::Interjection,
            Self::JKS | Self::JKC | Self::JKG | Self::JKO | Self::JKB | Self::JKV | Self::JKQ
            | Self::JX | Self::JC => PosCategory::Particle,
            Self::EP | Self::EF | Self::EC | Self::ETN | Self::ETM => PosCategory::Ending,
            Self::XPN | Self::XSN | Self::XSV | Self::XSA | Self::XR => PosCategory::Affix,
            Self::SF | Self::SP | Self::SSO | Self::SSC | Self::SC | Self::SE | Self::SY
            | Self::SL | Self::SH | Self::SN => PosCategory::Symbol,
            Self::Unknown => PosCategory::Unknown,
        }
    }

    /// 내용어 여부 (검색 인덱싱용)
    pub fn is_content_word(&self) -> bool {
        matches!(
            self,
            Self::NNG | Self::NNP | Self::NNB | Self::NNBC |
            Self::VV | Self::VA | Self::XR
        )
    }
}

/// 품사 대분류
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosCategory {
    Nominal,      // 체언
    Predicate,    // 용언
    Modifier,     // 수식언
    Interjection, // 독립언
    Particle,     // 관계언 (조사)
    Ending,       // 어미
    Affix,        // 접사
    Symbol,       // 기호
    Unknown,      // 미상
}
```

---

## 참고 자료

### 세종 품사 태그
- [21세기 세종계획](https://www.korean.go.kr/niklintro2/20years05_04_01.jsp)
- [꼬꼬마 품사 태그표](http://kkma.snu.ac.kr/documents/?doc=postag)
- [KOMORAN 품사표](https://docs.komoran.kr/firststep/postypes.html)

### 형태소 분석기
- [mecab-ko-dic](https://bitbucket.org/eunjeon/mecab-ko-dic)
- [Kiwi GitHub](https://github.com/bab2min/Kiwi)
- [Elasticsearch Nori](https://www.elastic.co/guide/en/elasticsearch/plugins/current/analysis-nori.html)

### 태그 스프레드시트
- [mecab-ko-dic 품사 태그 스프레드시트](https://docs.google.com/spreadsheets/d/1-9blXKjtjeKZqsf4NzHeYJCrr49-nXeRF6D80udfcwY)
