# mecab-ko-dict-validator

MeCab-Ko 사전 검증 도구 - MeCab 한국어 사전 파일의 유효성을 검증하는 종합적인 도구입니다.

## 기능

- **CSV 포맷 검증**: 필드 개수, 빈 필드, 최대 길이 검사
- **품사 태그 검증**: 한국어 품사 태그 유효성 및 계층 구조 검증
- **비용 범위 검증**: 좌/우측 컨텍스트 ID 및 단어 비용 범위 검사
- **중복 엔트리 탐지**: 정확한 중복 및 의미적 중복 탐지
- **유니코드 정규화 검증**: NFC/NFD/NFKC/NFKD 정규화 검사
- **인코딩 검증**: UTF-8 유효성 및 BOM 검사
- **한글 조합 검증**: 한글 자모 조합 상태 검사
- **통계 정보**: 품사 분포, 비용 통계, 고유 표면형 개수 등
- **커스터마이징 가능**: TOML 설정 파일로 검증 규칙 조정
- **다양한 출력 포맷**: JSON, 텍스트 리포트

## 설치

```bash
cd rust
cargo build --release -p mecab-ko-dict-validator
```

빌드된 바이너리는 `target/release/dict-validate`에 생성됩니다.

## 사용법

### 기본 사용

```bash
# 단일 사전 파일 검증
dict-validate dictionary.csv

# 여러 파일 검증
dict-validate dict1.csv dict2.csv dict3.csv

# JSON 출력
dict-validate --format json dictionary.csv

# 파일로 저장
dict-validate --output report.txt dictionary.csv
```

### 고급 사용

```bash
# 커스텀 설정 파일 사용
dict-validate --config validation.toml dictionary.csv

# 경고를 에러로 처리 (엄격 모드)
dict-validate --strict dictionary.csv

# 진행률 표시
dict-validate --progress large-dictionary.csv

# 상세 로그 출력
dict-validate -vv dictionary.csv

# 조용한 모드 (에러만 표시)
dict-validate --quiet dictionary.csv
```

### 설정 파일 생성

```bash
# 기본 설정 파일 생성
dict-validate --generate-config validation.toml

# 생성된 파일을 편집하여 검증 규칙 커스터마이징
vim validation.toml
```

## 설정 파일 형식

```toml
[csv_rules]
expected_field_count = 13
allow_empty_fields = false
trim_fields = true
max_field_length = 0

[pos_rules]
validate_hierarchy = true
max_tag_depth = 4
tag_separator = "+"

[cost_rules]
warn_unusual_costs = true
unusual_high_cost = 8000
unusual_low_cost = -8000

[encoding_rules]
expected_encoding = "UTF-8"
validate_utf8 = true
detect_encoding_issues = true
allow_bom = false

[duplicate_rules]
detect_exact_duplicates = true
detect_semantic_duplicates = true
allow_cost_variants = true

[normalization_rules]
check_unicode_normalization = true
preferred_normalization = "Nfc"
check_width_consistency = true
check_hangul_composition = true
warn_on_whitespace = true
```

## 검증 항목

### 1. CSV 포맷 검증

MeCab 사전 표준 형식 준수 여부를 확인합니다:

```csv
표면형,좌문맥ID,우문맥ID,비용,품사,품사세분류1,품사세분류2,품사세분류3,읽기,타입,첫번째형태,두번째형태,원형
한글,1,2,100,NNG,*,F,한글,*,*,*,*
```

- 필드 개수 (기본: 13개)
- 빈 필드 검사
- 필드 길이 제한

### 2. 품사 태그 검증

한국어 품사 태그 유효성:

```
유효한 품사 태그 예시:
- NNG (일반 명사)
- VV (동사)
- JKS (주격 조사)
- NNG+JKS (복합 품사)
```

지원하는 품사 태그:
- 체언: NNG, NNP, NNB, NP, NR
- 용언: VV, VA, VX, VCP, VCN
- 수식언: MM, MAG, MAJ
- 관계언: JKS, JKC, JKG, JKO, JKB, JKV, JKQ, JX, JC
- 독립언: IC
- 어미: EP, EF, EC, ETN, ETM
- 접사: XPN, XSN, XSV, XSA
- 기타: XR, SF, SE, SSO, SSC, SC, SY, SL, SH, SN

### 3. 비용 검증

```
좌문맥 ID: 0 ~ 10000
우문맥 ID: 0 ~ 10000
단어 비용: -10000 ~ 10000

경고 임계값:
- 높은 비용: > 8000
- 낮은 비용: < -8000
```

### 4. 중복 검증

- **정확한 중복**: 모든 필드가 동일한 엔트리
- **의미적 중복**: 표면형과 품사가 동일 (비용 차이 허용 가능)

### 5. 정규화 검증

- **유니코드 정규화**: NFC 형식 권장
- **한글 조합**: 완성형 한글 사용 권장 (자모 분리 경고)
- **공백 검사**: 표면형 내 공백 경고

### 6. 인코딩 검증

- UTF-8 인코딩 확인
- 잘못된 바이트 시퀀스 탐지
- BOM (Byte Order Mark) 검사

## 출력 예시

### 텍스트 포맷

```
═══════════════════════════════════════════════════════════
  MeCab-Ko Dictionary Validation Report
═══════════════════════════════════════════════════════════

File: dictionary.csv
Timestamp: 2026-01-06T12:34:56+00:00

Summary:
───────────────────────────────────────────────────────────
  Total entries:   1000
  Valid entries:   995
  Errors:          5
  Warnings:        12

Status: FAILED

Statistics:
───────────────────────────────────────────────────────────
  Cost statistics:
    Average: 523.45
    Min: -2000
    Max: 8500
  Unique surface forms: 950
  Duplicate entries: 50

  Top POS tags:
    NNG: 450
    VV: 200
    JKS: 150

Errors:
───────────────────────────────────────────────────────────
1. [Error] PosTag: Invalid POS tag: 'XXX' at line 42
    Suggestion: Check against valid Korean POS tags
2. [Error] Duplicate: Exact duplicate of line 15 at line 89
    Context: Surface: '테스트'

Warnings:
───────────────────────────────────────────────────────────
1. [Warning] Cost: Word cost 8500 is unusually high (threshold: 8000) at line 123
2. [Warning] Normalization: Surface form contains whitespace at line 234

═══════════════════════════════════════════════════════════
```

### JSON 포맷

```json
{
  "file_path": "dictionary.csv",
  "total_entries": 1000,
  "valid_entries": 995,
  "error_entries": 5,
  "warning_entries": 12,
  "issues": [
    {
      "severity": "Error",
      "category": "PosTag",
      "message": "Invalid POS tag: 'XXX'",
      "location": {
        "line": 42,
        "column": null
      },
      "suggestion": "Check against valid Korean POS tags",
      "context": null
    }
  ],
  "statistics": {
    "pos_tag_counts": {
      "NNG": 450,
      "VV": 200
    },
    "average_cost": 523.45,
    "min_cost": -2000,
    "max_cost": 8500,
    "unique_surface_forms": 950,
    "duplicate_count": 50
  },
  "timestamp": "2026-01-06T12:34:56+00:00"
}
```

## 라이브러리 사용

```rust
use mecab_ko_dict_validator::{DictValidator, ValidationConfig};

// 기본 설정으로 검증
let validator = DictValidator::with_defaults();
let report = validator.validate_file("dictionary.csv")?;

if report.is_valid() {
    println!("사전이 유효합니다!");
} else {
    println!("검증 실패: {} 개의 에러", report.error_entries);
    println!("{}", report.to_text());
}

// 커스텀 설정 사용
let mut config = ValidationConfig::default();
config.csv_rules.expected_field_count = 10;
config.cost_rules.word_cost_range = -5000..=5000;

let validator = DictValidator::new(config);
let report = validator.validate_file("custom_dict.csv")?;
```

## CI/CD 통합

### GitHub Actions

```yaml
- name: Validate Dictionary
  run: |
    cargo run --bin dict-validate -- \
      --format json \
      --output validation-report.json \
      --strict \
      dictionary.csv
```

### 종료 코드

- `0`: 검증 성공
- `1`: 검증 실패 (에러 발견 또는 `--strict` 모드에서 경고 발견)

## 성능

- **병렬 처리**: Rayon을 사용한 멀티스레드 검증
- **대용량 파일**: 수백만 엔트리 처리 가능
- **메모리 효율**: 스트리밍 방식 CSV 파싱

## 개발

### 테스트 실행

```bash
cargo test -p mecab-ko-dict-validator
```

### 린트 및 포맷팅

```bash
cargo clippy -p mecab-ko-dict-validator -- -D warnings
cargo fmt -p mecab-ko-dict-validator
```

## 라이선스

MIT OR Apache-2.0

## 기여

버그 리포트 및 기능 제안은 GitHub Issues를 이용해주세요.
