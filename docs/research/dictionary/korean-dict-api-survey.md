# 국립국어원 사전 API 조사 보고서

## 조사일: 2026-03-02
## 작성: PM Agent (Sprint 12 - S12-05)

---

## 1. 우리말샘 (Open Dictionary)

### 1.1 개요
- **URL**: https://opendict.korean.go.kr/
- **운영**: 국립국어원
- **시작**: 2016년 10월
- **특징**: 개방형 사전 (사용자 참여)
- **라이선스**: CC-BY-SA 2.0 KR

### 1.2 API 정보
- **포털**: https://www.data.go.kr/data/15019347/openapi.do
- **엔드포인트**: `https://opendict.korean.go.kr/api/search`
- **인증**: API 키 (공공데이터포털에서 발급)
- **응답형식**: XML, JSON

### 1.3 주요 API

#### 검색 API
```
GET /api/search
Parameters:
  - key: API 인증키
  - q: 검색어
  - part: 어휘, 뜻풀이, 용례
  - start: 시작위치
  - num: 결과수
```

#### 상세정보 API
```
GET /api/view
Parameters:
  - key: API 인증키
  - target_code: 어휘 코드
```

### 1.4 수록 범위
- 표준국어대사전 기본 어휘
- 옛말, 방언, 북한어
- 신조어, 전문용어
- 다문화 어휘
- **예상 규모**: 100만+ 어휘

---

## 2. 한국어기초사전

### 2.1 개요
- **URL**: https://krdict.korean.go.kr/
- **운영**: 국립국어원
- **특징**: 한국어 학습자용 사전
- **라이선스**: CC-BY-SA 2.0 KR

### 2.2 API 정보
- **포털**: https://www.data.go.kr/data/15105059/openapi.do
- **엔드포인트**: `https://krdict.korean.go.kr/api/search`
- **인증**: API 키
- **응답형식**: XML

### 2.3 주요 특징
- 5만 기초 어휘
- 다국어 번역 (13개 언어)
- 발음 정보
- 예문

---

## 3. 표준국어대사전

### 3.1 개요
- **URL**: https://stdict.korean.go.kr/
- **운영**: 국립국어원
- **특징**: 규범 사전 (표준어)
- **라이선스**: CC-BY-SA 2.0 KR

### 3.2 API 정보
- **포털**: https://stdict.korean.go.kr/openapi/openApiInfo.do
- **엔드포인트**: `https://stdict.korean.go.kr/api/search.do`
- **인증**: API 키
- **응답형식**: XML

---

## 4. MeCab-Ko 통합 방안

### 4.1 데이터 변환 파이프라인

```
┌──────────────────────────────────────────────────┐
│  국립국어원 API 연동 파이프라인                    │
├──────────────────────────────────────────────────┤
│                                                  │
│  [API 호출]     [파싱]        [변환]     [저장]   │
│  ┌────────┐   ┌────────┐   ┌────────┐  ┌─────┐  │
│  │우리말샘 │──▶│XML/JSON│──▶│MeCab   │──▶│CSV  │  │
│  │krdict  │──▶│Parser  │──▶│Format  │──▶│File │  │
│  └────────┘   └────────┘   └────────┘  └─────┘  │
│                    │                             │
│                    ▼                             │
│              ┌──────────┐                        │
│              │품사 매핑  │                        │
│              │비용 계산  │                        │
│              │중복 제거  │                        │
│              └──────────┘                        │
│                                                  │
└──────────────────────────────────────────────────┘
```

### 4.2 품사 태그 매핑

| 국립국어원 | MeCab-Ko | 설명 |
|-----------|----------|------|
| 명사 | NNG/NNP | 일반명사/고유명사 |
| 동사 | VV | 동사 |
| 형용사 | VA | 형용사 |
| 부사 | MAG | 일반부사 |
| 감탄사 | IC | 감탄사 |

### 4.3 구현 계획

#### Phase 1: API 클라이언트
```rust
// 예상 인터페이스
pub struct OpenDictClient {
    api_key: String,
    base_url: String,
}

impl OpenDictClient {
    pub async fn search(&self, query: &str) -> Result<Vec<DictEntry>>;
    pub async fn get_detail(&self, code: &str) -> Result<DictDetail>;
}
```

#### Phase 2: 변환기
```rust
pub fn convert_to_mecab_entry(dict_entry: &DictEntry) -> UserEntry {
    UserEntry::new(
        dict_entry.word,
        map_pos(&dict_entry.pos),
        calculate_cost(&dict_entry),
        Some(dict_entry.pronunciation),
    )
}
```

#### Phase 3: 동기화 도구
```bash
# CLI 예상
mecab-ko-sync --source opendict --api-key KEY --output neologisms.csv
```

---

## 5. 참고 자료

### 5.1 웹 리소스
- [우리말샘](https://opendict.korean.go.kr/)
- [한국어기초사전](https://krdict.korean.go.kr/)
- [표준국어대사전](https://stdict.korean.go.kr/)
- [공공데이터포털](https://www.data.go.kr/)

### 5.2 GitHub 프로젝트
- [OpenDictAPI](https://github.com/joyhong85/OpenDictAPI): 우리말샘 API 활용 예제
- [korean-dict-nikl](https://github.com/spellcheck-ko/korean-dict-nikl): FOSS 버전

---

## 6. 결론

### 6.1 권장 우선순위
1. **우리말샘**: 신조어 포함, 가장 포괄적
2. **한국어기초사전**: 기본 어휘, 발음 정보
3. **표준국어대사전**: 규범 어휘

### 6.2 다음 단계
1. 공공데이터포털에서 API 키 발급
2. API 클라이언트 구현 (`mecab-ko-dict-sync` 크레이트)
3. 정기 동기화 CI/CD 파이프라인 구축

### 6.3 예상 일정
- API 클라이언트: Sprint 13
- 동기화 도구: Sprint 14
- CI/CD 통합: Sprint 15
