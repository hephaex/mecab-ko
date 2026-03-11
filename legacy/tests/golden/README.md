# MeCab-Ko Golden Test Set

## 개요

이 디렉토리는 MeCab-Ko 형태소 분석기의 정확성을 검증하기 위한 골든 테스트셋을 포함합니다.
골든 테스트는 예상되는 정확한 출력과 실제 출력을 비교하여 회귀 테스트를 수행합니다.

## 파일 구조

| 파일명 | 설명 | 테스트 케이스 수 |
|--------|------|-----------------|
| `basic.json` | 기본 문장 테스트 | 50개 |
| `nouns.json` | 명사 추출 테스트 | 30개 |
| `complex.json` | 복합 문장 테스트 | 20개 |

## JSON 형식

```json
{
  "input": "입력 문장",
  "expected_morphs": ["형태소1", "형태소2", ...],
  "expected_pos": [["형태소1", "품사1"], ["형태소2", "품사2"], ...]
}
```

## 필드 설명

- `input`: 분석할 입력 문장
- `expected_morphs`: 예상되는 형태소 리스트
- `expected_pos`: 예상되는 형태소-품사 쌍 리스트

## 품사 태그 (POS Tags)

MeCab-Ko는 세종 품사 태그셋을 기반으로 합니다.

### 주요 품사 태그

| 태그 | 설명 | 예시 |
|------|------|------|
| NNG | 일반 명사 | 사람, 컴퓨터 |
| NNP | 고유 명사 | 서울, 한국 |
| NNB | 의존 명사 | 것, 수, 바 |
| NP | 대명사 | 나, 너, 우리 |
| NR | 수사 | 하나, 둘 |
| VV | 동사 | 먹다, 가다 |
| VA | 형용사 | 예쁘다, 크다 |
| VX | 보조 용언 | 있다, 없다 |
| VCP | 긍정 지정사 | 이다 |
| VCN | 부정 지정사 | 아니다 |
| MM | 관형사 | 새, 헌 |
| MAG | 일반 부사 | 매우, 아주 |
| MAJ | 접속 부사 | 그리고, 하지만 |
| IC | 감탄사 | 아, 오 |
| JKS | 주격 조사 | 이/가 |
| JKC | 보격 조사 | 이/가 |
| JKG | 관형격 조사 | 의 |
| JKO | 목적격 조사 | 을/를 |
| JKB | 부사격 조사 | 에, 에서 |
| JKV | 호격 조사 | 아/야 |
| JKQ | 인용격 조사 | 라고, 고 |
| JX | 보조사 | 은/는, 도 |
| JC | 접속 조사 | 와/과 |
| EP | 선어말 어미 | 시, 었 |
| EF | 종결 어미 | 다, 요 |
| EC | 연결 어미 | 고, 아서 |
| ETN | 명사형 전성 어미 | 기, 음 |
| ETM | 관형형 전성 어미 | 은, 는, 을 |
| XPN | 체언 접두사 | 풋, 햇 |
| XSN | 명사 파생 접미사 | 님, 적 |
| XSV | 동사 파생 접미사 | 하, 되 |
| XSA | 형용사 파생 접미사 | 스럽, 롭 |
| XR | 어근 | 똑똑 |
| SF | 마침표, 물음표, 느낌표 | . ? ! |
| SP | 쉼표, 가운뎃점, 콜론, 빗금 | , / : |
| SS | 따옴표, 괄호표, 줄표 | " ' ( ) |
| SE | 줄임표 | ... |
| SO | 붙임표 | - ~ |
| SW | 기타 기호 | @ # $ |
| SL | 외국어 | A, B, C |
| SH | 한자 | |
| SN | 숫자 | 1, 2, 3 |

## 테스트 카테고리

### basic.json
- 기본 인사말
- 일상 대화
- 간단한 평서문
- 의문문, 명령문

### nouns.json
- 일반 명사
- 고유 명사
- 복합 명사
- 외래어 명사
- 신조어
- 기술 용어 (IT, AI)

### complex.json
- 뉴스 문장
- 긴 복합문
- 인용문
- 수식어가 많은 문장
- 전문 용어 포함 문장

## 사용 방법

### Rust 테스트에서 사용

```rust
use std::fs;
use serde_json::Value;

fn load_golden_tests(filename: &str) -> Vec<Value> {
    let content = fs::read_to_string(format!("tests/golden/{}", filename))
        .expect("Failed to read golden test file");
    serde_json::from_str(&content).expect("Failed to parse JSON")
}

#[test]
fn test_basic_sentences() {
    let tests = load_golden_tests("basic.json");
    for test in tests {
        let input = test["input"].as_str().unwrap();
        let expected = &test["expected_morphs"];
        // 형태소 분석 수행 및 비교
    }
}
```

## 테스트셋 업데이트 가이드

1. 새로운 테스트 케이스 추가 시:
   - 입력 문장이 명확하고 재현 가능해야 합니다
   - 예상 결과는 MeCab-Ko의 표준 출력과 일치해야 합니다
   - 카테고리에 맞는 파일에 추가합니다

2. 테스트 케이스 수정 시:
   - 변경 이유를 명확히 기록합니다
   - 관련 이슈 번호를 참조합니다

3. 새로운 카테고리 추가 시:
   - 별도의 JSON 파일로 생성합니다
   - 이 README에 카테고리 설명을 추가합니다

## 관련 이슈

- DIC-009: 형태소 분석 검증용 골든 테스트셋 구축

## 참고 자료

- [세종 품사 태그셋](https://ithub.korean.go.kr/user/guide/corpus/guide1.do)
- [MeCab 한국어 사전](https://bitbucket.org/eunjeon/mecab-ko-dic)
