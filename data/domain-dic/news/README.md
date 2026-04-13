# MeCab-Ko 도메인 사전: 뉴스 고유명사 (NNP)

> 한국어 뉴스에서 자동 수집·검증된 고유명사 사전

## 개요

| 항목 | 값 |
|------|-----|
| 품사 | NNP (고유명사) 전용 |
| 갱신 | 매주 토요일 자동 갱신 |
| 방식 | 전체 교체 (overwrite) |
| 수집 | 자동화된 뉴스 NNP 수집 파이프라인 |
| 검증 | LLM 기반 자동 검증 |
| 형식 | MeCab-Ko 12필드 CSV |
| 라이선스 | CC-BY-SA 4.0 |

## 파일

| 파일 | 설명 |
|------|------|
| `news-nnp.csv` | 검증된 뉴스 고유명사 |
| `news-nnp-stats.json` | 카테고리별 통계 |

## 수집 파이프라인

```
뉴스 크롤링
  → 형태소 분석 → NNP 추출 → 빈도 누적
  → LLM 자동 검증
    ├── valid: person/place/organization/brand/country/media
    └── invalid: 일반명사/성씨1글자/형용사/부사 등
  → 빈도·길이 필터링
  → 12필드 CSV 변환 → news-nnp.csv
```

## CSV 형식

```
# 표면형,좌ID,우ID,비용,품사,분류,종류,활용형,조합읽기,원형,읽기,타입
이재명,0,0,-9963,NNP,인명,*,*,이재명,이재명,이재명,*
삼성전자,0,0,-8293,NNP,단체명,*,*,삼성전자,삼성전자,삼성전자,*
```

### 필드 설명

| # | 필드 | 설명 | 기본값 |
|---|------|------|--------|
| 1 | 표면형 | 사전 등록 단어 | (수집값) |
| 2 | 좌문맥ID | 0=자동 | 0 |
| 3 | 우문맥ID | 0=자동 | 0 |
| 4 | 비용 | log 스케일 (아래 참조) | 계산값 |
| 5 | 품사 | 세종 태그 | NNP |
| 6 | 분류 | NNP 하위 분류 | (검증 결과) |
| 7-8 | 종류·활용형 | 미사용 | * |
| 9 | 조합읽기 | 원형과 동일 | (표면형) |
| 10 | 원형 | 표면형과 동일 | (표면형) |
| 11 | 읽기 | 한글 발음 | (표면형) |
| 12 | 타입 | 미사용 | * |

### 분류 매핑

| 검증 카테고리 | 분류 필드 |
|-------------|----------|
| person | 인명 |
| place | 지명 |
| organization | 단체명 |
| brand | 브랜드 |
| country | 국가명 |
| media | 매체명 |

### 비용(cost) 계산

빈도 기반 log 스케일 정규화:

```python
import math
cost = -min(10000, round(math.log(max(freq, 1) + 1) * 1200))
```

| 빈도 | cost | 비고 |
|------|------|------|
| 3 | -1663 | 최소 수집 기준 |
| 10 | -2879 | |
| 100 | -5537 | IT 용어(-5000)와 유사 스케일 |
| 1000 | -8293 | 고빈도 |
| 4000+ | -9963 | 최대 (cap: -10000) |

## 품질 기준

- LLM 검증 통과
- 출현 빈도 ≥ 3 (노이즈 필터)
- 표면형 길이 ≥ 2자
- 한국 뉴스 기사에서 실제 사용된 고유명사만

### 제외 대상

- 일반명사: 인공지능, 반도체, 솔루션
- 언론사 약칭: 파이낸셜, 투데이, 헤럴드 (전체 이름은 포함)
- 성씨 1글자: 김, 박, 이, 조
- 형용사/부사: 무능, 초연, 이대로
- 법률/제도명: 보호법, 특별법

## 갱신 프로세스

1. 자동 파이프라인이 뉴스 NNP를 수집·검증·변환
2. 매주 토요일 `dict/news-weekly-YYYYMMDD` 브랜치로 push
3. PR 자동 생성 → CI 검증 통과 → 수동 merge

### 전체 교체 방식

매주 `news-nnp.csv` 전체를 새 파일로 교체합니다.
- 빈도/cost가 매주 변동
- 검증 상태도 변경 가능
- CSV는 line-based이므로 git diff 가독성 양호

## 릴리즈

- **태그 형식**: `dict-news-vYYYY.MM.WNN` (예: `dict-news-v2026.04.W16`)
- **범위**: `news/` 단독 릴리즈 (IT 용어 사전과 독립)
- **주기**: 월간 (4~5주 누적분)

## CI 검증

PR 시 `.github/workflows/validate-domain-dict.yml`이 자동 실행:

- 12필드 포맷 일치
- 품사 태그 = NNP
- cost 범위: -10000 ~ 0
- 중복 표면형 없음
- UTF-8 인코딩

## 사용법

### CLI

```bash
mecab --user-dict data/domain-dic/news/news-nnp.csv "이재명 대통령이 삼성전자를 방문했다"
```

### Rust API

```rust
use mecab_ko::UserDictionary;

let mut dict = UserDictionary::new();
dict.load_csv("data/domain-dic/news/news-nnp.csv")?;

let tokenizer = Tokenizer::builder()
    .user_dictionary(dict)
    .build()?;
```

## 관련 사전

| 사전 | 경로 | 갱신 |
|------|------|------|
| IT 용어 | `data/domain-dic/it-terms/` | 수동 |
| 신조어 | `data/user-dict/neologisms.csv` | 자동 (OpenDict API) |
| 뉴스 NNP | `data/domain-dic/news/` | 자동 (주간) |

## 기여

이 사전은 자동 파이프라인으로 관리됩니다.
수동 수정은 다음 주간 갱신 시 덮어씌워집니다.
수동 항목을 추가하려면 `data/user-dict/neologisms.csv`를 사용하세요.

## 라이선스

이 사전은 MeCab-Ko 프로젝트의 일부이며, 프로젝트 라이선스를 따릅니다.
뉴스 고유명사는 공개 뉴스에서 추출한 사실 정보로, 별도 저작권이 없습니다.

---

**생성일**: 2026-04-13
**갱신 주기**: 주간 (토요일)
