# 사용자 사전

사용자 사전은 기본 사전에 없는 단어를 추가하여 형태소 분석 품질을 향상시키는 기능입니다. 신조어, 전문 용어, 고유명사 등을 등록할 수 있습니다.

## 사전 파일 형식

### CSV 포맷

사용자 사전은 CSV(Comma-Separated Values) 형식을 사용합니다.

```csv
# 주석 라인 (# 으로 시작)
# 표면형,품사,비용,읽기
딥러닝,NNG,-1000,딥러닝
머신러닝,NNG,-1000,머신러닝
챗GPT,NNP,-1000,챗지피티
앤트로픽,NNP,-1000,앤트로픽
```

### 필드 설명

| 필드 | 필수 | 설명 | 예시 |
|------|------|------|------|
| 표면형 | O | 등록할 단어 | `딥러닝` |
| 품사 | O | 품사 태그 | `NNG`, `NNP`, `VV` |
| 비용 | X | 선택 우선도 (낮을수록 우선) | `-1000` |
| 읽기 | X | 발음 정보 | `딥러닝` |

### 비용(Cost) 이해하기

비용은 형태소 분석 시 해당 단어의 선택 우선도를 결정합니다:

- **음수 값**: 우선 선택 (추천: -1000 ~ -500)
- **0**: 기본 비용
- **양수 값**: 후순위 선택

```csv
# 높은 우선순위 (신조어, 전문 용어)
딥러닝,NNG,-1000,
GPT,NNP,-1000,

# 보통 우선순위
인공지능,NNG,-500,

# 낮은 우선순위 (거의 사용되지 않는 단어)
오래된단어,NNG,100,
```

## 품사 태그

사용자 사전에서 자주 사용하는 품사 태그:

| 태그 | 의미 | 예시 |
|------|------|------|
| NNG | 일반 명사 | 딥러닝, 인공지능 |
| NNP | 고유 명사 | 앤트로픽, 오픈AI |
| NNB | 의존 명사 | 것, 수, 등 |
| VV | 동사 | 분석하다, 처리하다 |
| VA | 형용사 | 빠르다, 좋다 |
| MAG | 일반 부사 | 매우, 아주 |
| SL | 외국어 | API, GPU |
| SH | 한자 | 韓國, 人工 |
| SN | 숫자 | 123, 456 |

전체 품사 태그 목록은 [품사 태그](pos-tags.md) 장을 참조하세요.

## CLI에서 사용

### 기본 사용

```bash
mecab-ko --user-dic user.csv "딥러닝 기술이 발전하고 있습니다"
```

### 조용히 실행

사전 로드 메시지 숨기기:

```bash
mecab-ko -q --user-dic user.csv "테스트"
```

### 여러 사전 파일

현재는 하나의 사용자 사전만 지정 가능합니다. 여러 사전을 사용하려면 파일을 합치세요:

```bash
cat dict1.csv dict2.csv > combined.csv
mecab-ko --user-dic combined.csv "테스트"
```

## 라이브러리에서 사용

### 기본 사용법

```rust
use mecab_ko_dict::UserDictionary;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dict = UserDictionary::new();

    // Add entries programmatically
    dict.add_entry("딥러닝", "NNG", Some(-1000), None);
    dict.add_entry("머신러닝", "NNG", Some(-1000), Some("머신러닝".to_string()));

    // Or load from file
    dict.load_from_csv("user.csv")?;

    println!("Loaded {} entries", dict.len());
    Ok(())
}
```

### 빌더 패턴

```rust
use mecab_ko_dict::UserDictionaryBuilder;

let dict = UserDictionaryBuilder::new()
    .default_cost(-500)              // Set default cost
    .add("딥러닝", "NNG")            // Uses default cost
    .add_with_cost("GPT", "NNP", -1000)
    .add_full("챗GPT", "NNP", -1000, Some("챗지피티"))
    .load_csv("extra.csv")?         // Load additional entries
    .build();
```

### 엔트리 조회

```rust
let entries = dict.lookup("딥러닝");
for entry in entries {
    println!("Surface: {}", entry.surface);
    println!("POS: {}", entry.pos);
    println!("Cost: {}", entry.cost);
    if let Some(reading) = &entry.reading {
        println!("Reading: {}", reading);
    }
}
```

### 문자열에서 로드

```rust
let csv_content = r#"
# IT 용어 사전
딥러닝,NNG,-1000,
머신러닝,NNG,-1000,
자연어처리,NNG,-1000,자연어처리
"#;

dict.load_from_str(csv_content)?;
```

### 컨텍스트 ID 지정

고급 사용자를 위해 좌/우 문맥 ID를 직접 지정할 수 있습니다:

```rust
dict.add_entry_with_ids(
    "특수단어",
    "NNG",
    -1000,    // cost
    1234,     // left_id
    5678,     // right_id
    None,     // reading
);
```

### 사전 저장

```rust
dict.save_to_csv("output.csv")?;
```

## 도메인별 사전 예시

### IT/기술 용어

```csv
# IT 기술 용어 사전
딥러닝,NNG,-1000,딥러닝
머신러닝,NNG,-1000,머신러닝
인공지능,NNG,-1000,인공지능
자연어처리,NNG,-1000,자연어처리
GPT,NNP,-1000,지피티
API,SL,-1000,에이피아이
클라우드,NNG,-1000,클라우드
쿠버네티스,NNP,-1000,쿠버네티스
도커,NNP,-1000,도커
```

### 브랜드/회사명

```csv
# 회사 및 브랜드명
앤트로픽,NNP,-1000,앤트로픽
오픈AI,NNP,-1000,오픈에이아이
구글,NNP,-1000,구글
마이크로소프트,NNP,-1000,마이크로소프트
메타,NNP,-1000,메타
테슬라,NNP,-1000,테슬라
```

### 신조어/인터넷 용어

```csv
# 신조어 및 인터넷 용어
갓생,NNG,-1000,갓생
소확행,NNG,-1000,소확행
워라밸,NNG,-1000,워라밸
MZ세대,NNG,-1000,엠지세대
플렉스,NNG,-1000,플렉스
TMI,NNG,-1000,티엠아이
```

## 사전 작성 팁

### 1. 적절한 비용 설정

```csv
# 일반적인 경우: -1000
신조어,NNG,-1000,

# 자주 사용되는 경우: -1500
아주자주쓰는단어,NNG,-1500,

# 덜 중요한 경우: -500
덜중요한단어,NNG,-500,
```

### 2. 같은 표면형, 다른 품사

하나의 단어가 여러 품사로 사용될 수 있습니다:

```csv
# '분석'은 명사와 동사 어간 모두 가능
분석,NNG,-1000,
```

### 3. 복합어 등록

띄어쓰기 없는 복합어를 하나의 단어로 등록:

```csv
자연어처리,NNG,-1000,자연어처리
인공지능,NNG,-1000,인공지능
```

### 4. 외래어 표기 변형

다양한 표기를 모두 등록:

```csv
# 같은 단어의 다른 표기
쿠버네티스,NNP,-1000,
쿠버네테스,NNP,-1000,
kubernetes,SL,-1000,
```

### 5. 읽기 정보 활용

발음이 표기와 다른 경우 읽기 정보 제공:

```csv
GPT,NNP,-1000,지피티
API,SL,-1000,에이피아이
CEO,SL,-1000,씨이오
```

## 검증 및 테스트

### 사전 로드 테스트

```rust
use mecab_ko_dict::UserDictionary;

fn test_dictionary() -> Result<(), Box<dyn std::error::Error>> {
    let mut dict = UserDictionary::new();
    dict.load_from_csv("user.csv")?;

    // Verify entry count
    println!("Total entries: {}", dict.len());

    // Test specific lookup
    let entries = dict.lookup("딥러닝");
    assert!(!entries.is_empty(), "Entry 'Deep Learning' not found");

    Ok(())
}
```

### 분석 결과 비교

```bash
# Without user dictionary
echo "딥러닝 기술" | mecab-ko

# With user dictionary
echo "딥러닝 기술" | mecab-ko --user-dic user.csv
```

## 문제 해결

### 엔트리가 적용되지 않음

1. 비용이 너무 높은지 확인 (음수 값 사용)
2. 품사 태그가 올바른지 확인
3. CSV 형식이 정확한지 확인 (콤마 구분)

### 파싱 오류

```
Error: Invalid user dictionary format at line 5
```

해당 라인의 형식 확인:
- 최소 2개 필드 (표면형, 품사) 필요
- 빈 필드 허용 (예: `단어,NNG,,`)
- 콤마가 포함된 값은 큰따옴표로 감싸기

### 인코딩 문제

사전 파일은 **UTF-8** 인코딩을 사용해야 합니다:

```bash
# Check encoding
file user.csv

# Convert from EUC-KR to UTF-8
iconv -f EUC-KR -t UTF-8 user_euckr.csv > user.csv
```
