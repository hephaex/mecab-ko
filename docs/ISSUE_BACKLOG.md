# 📋 이슈 리스트 (Issue Backlog)

> **Project**: MeCab-Ko - Korean Morphological Analyzer in Rust  
> **Author**: hephaex (hephaex@gmail.com)  
> **Repository**: https://github.com/hephaex/mecab-ko

이 문서는 프로젝트의 모든 이슈를 GitHub Issues 형식으로 정리한 것입니다.

---

## 🏷️ 라벨 정의

| 라벨 | 설명 |
|-----|------|
| `epic:dictionary` | 사전 관련 Epic |
| `epic:rust-core` | Rust 코어 Epic |
| `epic:bindings` | 바인딩/통합 Epic |
| `epic:elasticsearch` | ES 플러그인 Epic |
| `epic:qa` | 품질 관리 Epic |
| `priority:P0` | 필수 (Must Have) |
| `priority:P1` | 중요 (Should Have) |
| `priority:P2` | 보통 (Could Have) |
| `priority:P3` | 낮음 (Won't Have This Time) |
| `size:S` | 1-2일 |
| `size:M` | 3-5일 |
| `size:L` | 1-2주 |
| `size:XL` | 2주+ |
| `type:feature` | 기능 |
| `type:research` | 조사/분석 |
| `type:docs` | 문서 |
| `type:test` | 테스트 |

---

## Epic 1: 사전 현대화 (Dictionary Modernization)

### DIC-001: mecab-ko-dic 소스 분석 및 포맷 문서화

```yaml
title: "[Research] mecab-ko-dic 소스 분석 및 포맷 문서화"
labels: [epic:dictionary, priority:P0, size:M, type:research]
assignees: []
```

**설명**
mecab-ko-dic의 구조와 데이터 포맷을 완전히 이해하고 문서화합니다.

**작업 내용**
- [ ] CSV 파일별 구조 분석 (NNG, VV, etc.)
- [ ] 연접 비용 행렬 (matrix.def) 분석
- [ ] 문자 정의 (char.def) 분석
- [ ] 미등록어 정의 (unk.def) 분석
- [ ] 사전 빌드 프로세스 분석
- [ ] 분석 결과 문서화

**산출물**
- `docs/dictionary-format-v2.md`
- `docs/build-process.md`

**참고 자료**
- https://bitbucket.org/eunjeon/mecab-ko-dic
- https://docs.google.com/spreadsheets/d/1-9blXKjtjeKZqsf4NzHeYJCrr49-nXeRF6D80udfcwY

---

### DIC-002: 세종 말뭉치 v2.0 품사 태그 체계 검토

```yaml
title: "[Research] 세종 품사 태그 체계 검토 및 확장 설계"
labels: [epic:dictionary, priority:P0, size:S, type:research]
assignees: []
```

**설명**
세종 품사 태그와 mecab-ko-dic 태그 간의 매핑을 검토하고, 확장 가능성을 분석합니다.

**작업 내용**
- [ ] 세종 품사 태그 체계 정리 (45개 태그)
- [ ] mecab-ko-dic 태그와의 매핑 검증
- [ ] Kiwi, Nori와의 태그 호환성 분석
- [ ] 태그 확장 필요성 검토 (신조어 카테고리 등)
- [ ] 통합 매핑 테이블 작성

**산출물**
- `docs/pos-tag-mapping.md`
- `src/tag_mapping.rs` (코드)

**의존성**
- DIC-001

---

### DIC-003: 모두의 말뭉치 데이터셋 수집 및 라이센스 검토

```yaml
title: "[Data] 모두의 말뭉치 데이터셋 수집 및 라이센스 검토"
labels: [epic:dictionary, priority:P0, size:M, type:research]
assignees: []
```

**설명**
국립국어원의 '모두의 말뭉치'에서 활용 가능한 데이터를 수집하고 라이센스를 검토합니다.

**작업 내용**
- [ ] corpus.korean.go.kr 가입 및 데이터 신청
- [ ] 형태 분석 말뭉치 다운로드
- [ ] 일상 대화 말뭉치 다운로드
- [ ] 라이센스 조건 검토 및 문서화
- [ ] 데이터 전처리 스크립트 작성
- [ ] 데이터 품질 검증

**산출물**
- `data/modu-corpus/` (원시 데이터)
- `scripts/preprocess_modu.py`
- `docs/data-license.md`

---

### DIC-004: AI Hub 말뭉치 활용 가능성 조사

```yaml
title: "[Research] AI Hub 말뭉치 활용 가능성 조사"
labels: [epic:dictionary, priority:P1, size:S, type:research]
assignees: []
```

**설명**
AI Hub에서 제공하는 한국어 데이터셋의 형태소 분석 학습 활용 가능성을 조사합니다.

**작업 내용**
- [ ] AI Hub 한국어 데이터셋 목록 조사
- [ ] 형태소 분석 관련 데이터셋 식별
- [ ] 라이센스 및 이용 조건 확인
- [ ] 활용 가능성 평가 리포트 작성

**산출물**
- `docs/aihub-evaluation.md`

---

### DIC-005: 신조어 수집 파이프라인 구축

```yaml
title: "[Feature] 신조어 수집 파이프라인 구축"
labels: [epic:dictionary, priority:P1, size:L, type:feature]
assignees: []
```

**설명**
나무위키, 위키피디아 등에서 신조어를 자동으로 수집하는 파이프라인을 구축합니다.

**작업 내용**
- [ ] 나무위키 크롤러 구현 (robots.txt 준수)
- [ ] 위키피디아 한국어 덤프 처리
- [ ] 신조어 후보 추출 알고리즘
- [ ] 품사 태깅 자동화 (기존 분석기 활용)
- [ ] 수동 검증 워크플로우 설계
- [ ] 주기적 업데이트 스케줄링

**산출물**
- `tools/neologism-collector/`
- `data/neologisms/`
- 수집 자동화 스크립트

**의존성**
- DIC-001

---

### DIC-006: IT/기술 용어 도메인 사전 구축

```yaml
title: "[Data] IT/기술 용어 도메인 사전 구축"
labels: [epic:dictionary, priority:P1, size:M, type:feature]
assignees: []
```

**설명**
IT, 프로그래밍, 기술 분야의 전문 용어 사전을 구축합니다.

**작업 내용**
- [ ] 기술 블로그/문서에서 용어 추출
- [ ] 프로그래밍 언어 관련 용어 정리
- [ ] 클라우드/인프라 용어 정리
- [ ] AI/ML 관련 용어 정리
- [ ] 외래어 한글 표기 정규화
- [ ] 품사 및 의미 태깅

**산출물**
- `data/domain-dic/it-tech.csv` (10,000+ 엔트리 목표)

**의존성**
- DIC-001

---

### DIC-007: 외래어 표기 정규화 규칙 정의

```yaml
title: "[Feature] 외래어 표기 정규화 규칙 정의"
labels: [epic:dictionary, priority:P1, size:M, type:feature]
assignees: []
```

**설명**
외래어의 다양한 한글 표기를 정규화하는 규칙을 정의합니다.

**작업 내용**
- [ ] 국립국어원 외래어 표기법 정리
- [ ] 일반적 변이형 패턴 분석 (예: 커피/코피)
- [ ] IT 용어 특수 표기 정리 (예: 쿠버네티스/쿠베르네테스)
- [ ] 정규화 규칙 구현
- [ ] 예외 처리 로직

**산출물**
- `src/normalizer/foreign_word.rs`
- `data/foreign-word-variants.csv`

**의존성**
- DIC-002

---

### DIC-008: 연접 비용 행렬 재학습 (CRF 기반)

```yaml
title: "[ML] 연접 비용 행렬 재학습 (CRF 기반)"
labels: [epic:dictionary, priority:P0, size:XL, type:feature]
assignees: []
```

**설명**
최신 말뭉치를 기반으로 CRF 모델을 학습하여 연접 비용 행렬을 재생성합니다.

**작업 내용**
- [ ] 학습 말뭉치 준비 (모두의 말뭉치 + 세종)
- [ ] CRF 학습 환경 구축 (Lindera trainer 또는 자체)
- [ ] 피처 엔지니어링
- [ ] 모델 학습
- [ ] 비용 행렬 생성
- [ ] 품질 검증

**산출물**
- `models/cost-matrix-v3/`
- `tools/crf-trainer/`
- 학습 파이프라인

**의존성**
- DIC-003

---

### DIC-009: 사전 검증 테스트셋 구축

```yaml
title: "[Test] 사전 검증 테스트셋 구축"
labels: [epic:dictionary, priority:P1, size:M, type:test]
assignees: []
```

**설명**
형태소 분석 정확도를 측정하기 위한 골든 테스트셋을 구축합니다.

**작업 내용**
- [ ] 세종 말뭉치에서 테스트셋 추출
- [ ] 장르별 균형 잡힌 샘플링 (뉴스, 문학, 대화 등)
- [ ] 최신 텍스트 추가 (2020-2024)
- [ ] 수동 검증 및 교정
- [ ] 테스트 자동화 스크립트

**산출물**
- `tests/golden/` (1,000+ 문장)
- `scripts/evaluate_accuracy.py`

**의존성**
- DIC-003

---

### DIC-010: 바이너리 사전 포맷 v3.0 설계

```yaml
title: "[Design] 바이너리 사전 포맷 v3.0 설계"
labels: [epic:dictionary, priority:P0, size:L, type:feature]
assignees: []
```

**설명**
압축 효율과 로딩 속도를 최적화한 새로운 바이너리 사전 포맷을 설계합니다.

**작업 내용**
- [ ] 기존 Lindera 포맷 분석
- [ ] 압축 알고리즘 비교 (LZMA, Zstd, Brotli)
- [ ] 메모리 매핑 지원 설계
- [ ] 버전 관리 메커니즘
- [ ] 포맷 명세서 작성
- [ ] 프로토타입 구현

**산출물**
- `docs/binary-format-v3.md`
- `src/dictionary/format.rs`

**의존성**
- DIC-001

---

## Epic 2: Rust 코어 구현

### RST-001: Lindera 코드베이스 분석 및 전략 수립

```yaml
title: "[Research] Lindera 코드베이스 분석 및 fork 전략 수립"
labels: [epic:rust-core, priority:P0, size:M, type:research]
assignees: []
```

**설명**
Lindera 프로젝트를 분석하고, fork vs 신규 개발 전략을 결정합니다.

**작업 내용**
- [ ] Lindera 아키텍처 분석
- [ ] 한국어 지원 현황 파악 (lindera-ko-dic)
- [ ] 코드 품질 및 유지보수성 평가
- [ ] 라이센스 호환성 검토
- [ ] Fork vs 신규 개발 의사결정
- [ ] 개발 전략 문서화

**산출물**
- `docs/lindera-analysis.md`
- `docs/development-strategy.md`

---

### RST-002: 프로젝트 구조 설계

```yaml
title: "[Setup] 프로젝트 구조 및 Cargo workspace 설계"
labels: [epic:rust-core, priority:P0, size:S, type:feature]
assignees: []
```

**설명**
모듈화된 Cargo workspace 구조를 설계하고 초기화합니다.

**작업 내용**
- [ ] Workspace 구조 설계
- [ ] Crate 분리 전략 결정
- [ ] 공통 의존성 관리 방안
- [ ] CI/CD 기초 설정
- [ ] 코딩 컨벤션 정의

**제안 구조**
```
mecab-ko/
├── Cargo.toml (workspace)
├── crates/
│   ├── mecab-ko-core/      # 핵심 알고리즘
│   ├── mecab-ko-dict/      # 사전 관리
│   ├── mecab-ko-hangul/    # 한글 유틸리티
│   ├── mecab-ko-cli/       # CLI 도구
│   └── mecab-ko-python/    # Python 바인딩
├── data/
├── docs/
└── tests/
```

**산출물**
- 초기화된 저장소
- `CONTRIBUTING.md`

**의존성**
- RST-001

---

### RST-003: 바이너리 사전 로더 구현

```yaml
title: "[Feature] 바이너리 사전 로더 구현"
labels: [epic:rust-core, priority:P0, size:L, type:feature]
assignees: []
```

**설명**
v3.0 바이너리 포맷의 사전을 효율적으로 로드하는 모듈을 구현합니다.

**작업 내용**
- [ ] 메모리 매핑 기반 로더
- [ ] 압축 해제 처리
- [ ] 버전 검증
- [ ] 에러 처리
- [ ] 벤치마크

**API 설계**
```rust
pub struct Dictionary {
    pub tokens: TokenDict,
    pub matrix: CostMatrix,
    pub unknown: UnknownDict,
}

impl Dictionary {
    pub fn load(path: &Path) -> Result<Self, DictError>;
    pub fn load_embedded() -> Result<Self, DictError>;
}
```

**산출물**
- `crates/mecab-ko-dict/src/loader.rs`

**의존성**
- DIC-010

---

### RST-004: Double-Array Trie 구현

```yaml
title: "[Feature] Double-Array Trie 구현"
labels: [epic:rust-core, priority:P0, size:XL, type:feature]
assignees: []
```

**설명**
사전 검색을 위한 Double-Array Trie 자료구조를 구현합니다.

**작업 내용**
- [ ] Double-Array Trie 알고리즘 구현
- [ ] 공통 접두사 검색
- [ ] 압축 지원
- [ ] 직렬화/역직렬화
- [ ] 성능 최적화
- [ ] 단위 테스트

**산출물**
- `crates/mecab-ko-core/src/trie.rs`

**의존성**
- RST-002

---

### RST-005: Viterbi 알고리즘 구현

```yaml
title: "[Feature] Viterbi 알고리즘 구현"
labels: [epic:rust-core, priority:P0, size:XL, type:feature]
assignees: []
```

**설명**
최적 형태소 분석 경로를 찾는 Viterbi 알고리즘을 구현합니다.

**작업 내용**
- [ ] Lattice 구조 정의
- [ ] 노드/엣지 표현
- [ ] Forward 패스 구현
- [ ] Backward 패스 구현
- [ ] 최적 경로 추출
- [ ] N-best 지원 준비
- [ ] 단위 테스트

**API 설계**
```rust
pub struct Tokenizer {
    dictionary: Dictionary,
}

impl Tokenizer {
    pub fn tokenize(&self, text: &str) -> Vec<Token>;
    pub fn tokenize_to_lattice(&self, text: &str) -> Lattice;
}
```

**산출물**
- `crates/mecab-ko-core/src/viterbi.rs`
- `crates/mecab-ko-core/src/lattice.rs`

**의존성**
- RST-003, RST-004

---

### RST-006: 연접 비용 행렬 로더 구현

```yaml
title: "[Feature] 연접 비용 행렬 로더 구현"
labels: [epic:rust-core, priority:P0, size:M, type:feature]
assignees: []
```

**설명**
품사 간 연접 비용을 저장한 행렬을 효율적으로 로드합니다.

**작업 내용**
- [ ] 희소 행렬 표현
- [ ] 압축 포맷 지원
- [ ] 빠른 조회 인터페이스
- [ ] 메모리 최적화

**산출물**
- `crates/mecab-ko-dict/src/matrix.rs`

**의존성**
- RST-003

---

### RST-007: 미등록어 처리 모듈 구현

```yaml
title: "[Feature] 미등록어(Unknown Word) 처리 모듈 구현"
labels: [epic:rust-core, priority:P1, size:L, type:feature]
assignees: []
```

**설명**
사전에 없는 단어를 처리하는 모듈을 구현합니다.

**작업 내용**
- [ ] 문자 유형 분류 (한글, 영문, 숫자 등)
- [ ] 문자 유형별 기본 비용 설정
- [ ] 미등록어 후보 생성
- [ ] 그룹핑 전략 (최소/최대 단위)

**산출물**
- `crates/mecab-ko-core/src/unknown.rs`

**의존성**
- RST-005

---

### RST-008: 한글 자소 분리/결합 유틸리티

```yaml
title: "[Feature] 한글 자소(Jamo) 분리/결합 유틸리티"
labels: [epic:rust-core, priority:P0, size:M, type:feature]
assignees: []
```

**설명**
한글 처리를 위한 기본 유틸리티 함수들을 구현합니다.

**작업 내용**
- [ ] 자모 분리 (가 → ㄱ+ㅏ)
- [ ] 자모 결합 (ㄱ+ㅏ → 가)
- [ ] 초/중/종성 분리
- [ ] 한글 여부 판별
- [ ] 종성 유무 판별
- [ ] 유니코드 정규화

**API 설계**
```rust
pub mod hangul {
    pub fn decompose(c: char) -> Option<(char, char, Option<char>)>;
    pub fn compose(cho: char, jung: char, jong: Option<char>) -> Option<char>;
    pub fn is_hangul(c: char) -> bool;
    pub fn has_jongseong(c: char) -> bool;
}
```

**산출물**
- `crates/mecab-ko-hangul/`

---

### RST-009: 띄어쓰기 특화 비용 조정

```yaml
title: "[Feature] 띄어쓰기 특화 비용 조정 (left-space-penalty)"
labels: [epic:rust-core, priority:P1, size:M, type:feature]
assignees: []
```

**설명**
한국어 띄어쓰기 특성을 반영한 비용 조정 기능을 구현합니다.

**작업 내용**
- [ ] mecab-ko의 left-space-penalty 로직 분석
- [ ] 띄어쓰기 컨텍스트 추적
- [ ] 품사별 페널티 적용
- [ ] 설정 가능한 인터페이스

**산출물**
- `crates/mecab-ko-core/src/space_penalty.rs`

**의존성**
- RST-006

---

### RST-010: N-best 결과 출력 기능

```yaml
title: "[Feature] N-best 결과 출력 기능"
labels: [epic:rust-core, priority:P2, size:M, type:feature]
assignees: []
```

**설명**
상위 N개의 분석 결과를 출력하는 기능을 구현합니다.

**작업 내용**
- [ ] N-best Viterbi 알고리즘
- [ ] 확률/비용 기반 정렬
- [ ] Iterator 인터페이스

**산출물**
- `crates/mecab-ko-core/src/nbest.rs`

**의존성**
- RST-005

---

### RST-011: 사용자 정의 사전 지원

```yaml
title: "[Feature] 사용자 정의 사전 지원"
labels: [epic:rust-core, priority:P1, size:M, type:feature]
assignees: []
```

**설명**
사용자가 추가 단어를 등록할 수 있는 기능을 구현합니다.

**작업 내용**
- [ ] 사용자 사전 포맷 정의
- [ ] 런타임 사전 추가
- [ ] 우선순위 처리
- [ ] CSV 임포트

**산출물**
- `crates/mecab-ko-dict/src/user_dict.rs`

**의존성**
- RST-003

---

### RST-012: CLI 인터페이스 구현

```yaml
title: "[Feature] CLI 인터페이스 구현"
labels: [epic:rust-core, priority:P1, size:S, type:feature]
assignees: []
```

**설명**
명령줄에서 사용할 수 있는 인터페이스를 구현합니다.

**작업 내용**
- [ ] clap 기반 인자 파싱
- [ ] 표준 입출력 처리
- [ ] 출력 포맷 옵션 (기본, wakati, JSON)
- [ ] 사전 경로 옵션
- [ ] 인터랙티브 모드

**사용 예시**
```bash
$ mecab-ko "안녕하세요"
안녕    NNG,인사,T,안녕,*,*,*,*
하      XSV,*,F,하,*,*,*,*
세요    EP+EF,*,F,세요,Inflect,EP,EF,시/EP/*+어요/EF/*
EOS

$ echo "형태소 분석" | mecab-ko -O wakati
형태소 분석
```

**산출물**
- `crates/mecab-ko-cli/`

**의존성**
- RST-005

---

### RST-013: 단위 테스트 및 벤치마크

```yaml
title: "[Test] 단위 테스트 및 벤치마크"
labels: [epic:rust-core, priority:P1, size:M, type:test]
assignees: []
```

**설명**
전체 코드에 대한 단위 테스트와 성능 벤치마크를 작성합니다.

**작업 내용**
- [ ] 각 모듈 단위 테스트
- [ ] 통합 테스트
- [ ] criterion 기반 벤치마크
- [ ] 코드 커버리지 설정
- [ ] 성능 회귀 테스트

**산출물**
- `tests/`
- `benches/`

**의존성**
- RST-005

---

### RST-014: 문서화 (rustdoc + mdbook)

```yaml
title: "[Docs] API 문서화 및 사용자 가이드"
labels: [epic:rust-core, priority:P2, size:M, type:docs]
assignees: []
```

**설명**
API 문서와 사용자 가이드를 작성합니다.

**작업 내용**
- [ ] rustdoc 주석 작성
- [ ] mdbook 기반 가이드북
- [ ] 예제 코드
- [ ] FAQ

**산출물**
- `docs/book/`
- docs.rs 페이지

**의존성**
- RST-012

---

## Epic 3: 바인딩 및 통합

### BND-001: Python 바인딩 설계

```yaml
title: "[Design] Python 바인딩 API 설계 (konlpy 호환)"
labels: [epic:bindings, priority:P1, size:M, type:feature]
assignees: []
```

**설명**
konlpy 및 기존 mecab-ko Python 바인딩과 호환되는 API를 설계합니다.

**작업 내용**
- [ ] 기존 python-mecab-ko API 분석
- [ ] konlpy.tag 인터페이스 분석
- [ ] 호환 API 설계
- [ ] 비호환 확장 API 설계

**제안 API**
```python
from mecab_rs_ko import Mecab

mecab = Mecab()
mecab.morphs("안녕하세요")  # ['안녕', '하', '세요']
mecab.nouns("안녕하세요")   # []
mecab.pos("안녕하세요")     # [('안녕', 'NNG'), ('하', 'XSV'), ('세요', 'EP+EF')]
```

**산출물**
- `docs/python-api.md`

**의존성**
- RST-005

---

### BND-002 ~ BND-007: (바인딩 관련 이슈들)

*(상세 내용은 BND-001과 유사한 형식으로 작성)*

---

## Epic 4: Elasticsearch 플러그인

### ELS-001 ~ ELS-004: (ES 관련 이슈들)

*(상세 내용은 위 형식과 유사하게 작성)*

---

## Epic 5: 품질 및 배포

### QA-001 ~ QA-006: (QA 관련 이슈들)

*(상세 내용은 위 형식과 유사하게 작성)*

---

## 📝 이슈 생성 가이드

### GitHub CLI로 이슈 일괄 생성

```bash
#!/bin/bash
# create-issues.sh

# DIC-001 생성 예시
gh issue create \
  --title "[Research] mecab-ko-dic 소스 분석 및 포맷 문서화" \
  --label "epic:dictionary,priority:P0,size:M,type:research" \
  --body-file docs/issues/DIC-001.md

# ... 이하 동일 패턴
```

### 이슈 템플릿

`.github/ISSUE_TEMPLATE/feature.yml` 활용

---

## 🔄 스프린트 진행 워크플로우

```
┌─────────────────────────────────────────────────────────────────┐
│  1. 스프린트 계획                                               │
│     - 백로그에서 이슈 선택                                       │
│     - 스프린트 마일스톤에 할당                                   │
│     - 담당자 지정                                               │
├─────────────────────────────────────────────────────────────────┤
│  2. 개발                                                        │
│     - 이슈별 feature branch 생성                                │
│     - 코드 리뷰 (PR)                                            │
│     - main 머지                                                 │
├─────────────────────────────────────────────────────────────────┤
│  3. 스프린트 리뷰                                               │
│     - 완료된 기능 데모                                          │
│     - 회고                                                      │
│     - 다음 스프린트 계획 조정                                    │
└─────────────────────────────────────────────────────────────────┘
```

---

**문서 버전**: 0.1.0  
**최종 수정**: 2025-01-04
