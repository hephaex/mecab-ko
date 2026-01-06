# 한국어 말뭉치 저작권 및 라이선스 가이드

이 문서는 한국어 말뭉치 사용 시 저작권 관련 주의사항을 설명합니다.

## 목차

1. [개요](#개요)
2. [주요 말뭉치 라이선스](#주요-말뭉치-라이선스)
3. [라이선스별 사용 가능 범위](#라이선스별-사용-가능-범위)
4. [사전 배포 시 준수사항](#사전-배포-시-준수사항)
5. [체크리스트](#체크리스트)

## 개요

한국어 말뭉치로 생성한 MeCab 사전은 **원본 말뭉치의 저작권 및 라이선스를 계승**합니다. 따라서 말뭉치 선택 시 사용 목적에 맞는 라이선스를 가진 말뭉치를 선택해야 합니다.

### 핵심 원칙

1. **파생 저작물**: 말뭉치에서 추출한 사전은 파생 저작물로 간주됩니다.
2. **라이선스 계승**: 원본 말뭉치의 라이선스 조건을 따라야 합니다.
3. **혼합 사전**: 여러 말뭉치를 혼합한 경우 가장 제한적인 라이선스를 적용합니다.
4. **출처 표시**: 대부분의 라이선스에서 출처 표시를 요구합니다.

## 주요 말뭉치 라이선스

### 1. 모두의 말뭉치 (Modu Corpus)

| 항목 | 내용 |
|------|------|
| **제공 기관** | 국립국어원 (National Institute of Korean Language) |
| **라이선스** | CC BY-SA 2.0 KR (Creative Commons Attribution-ShareAlike 2.0 Korea) |
| **웹사이트** | https://corpus.korean.go.kr |
| **규모** | 약 1억 어절 (지속 확장 중) |

#### 사용 조건

✅ **허용되는 사용**
- 상업적 사용 가능
- 2차 저작물 작성 가능
- 수정 및 변경 가능
- 배포 및 재배포 가능

⚠️ **준수 사항**
- **출처 표시 필수** (Attribution)
- **동일 조건 공유 필수** (Share-Alike)
  - 파생 저작물도 동일한 CC BY-SA 2.0 KR 라이선스로 배포해야 함
- 라이선스 변경 불가

#### 출처 표시 방법

**표준 형식:**
```
이 저작물은 국립국어원의 "모두의 말뭉치"를 활용하여 작성되었으며,
CC BY-SA 2.0 KR 라이선스를 따릅니다.

출처: 국립국어원 모두의 말뭉치 (https://corpus.korean.go.kr)
라이선스: CC BY-SA 2.0 KR (https://creativecommons.org/licenses/by-sa/2.0/kr/)
```

**영문 형식:**
```
This work is based on "Modu Corpus" by the National Institute of
Korean Language and is licensed under CC BY-SA 2.0 KR.

Source: Modu Corpus (https://corpus.korean.go.kr)
License: CC BY-SA 2.0 KR (https://creativecommons.org/licenses/by-sa/2.0/kr/)
```

#### 실무 가이드

**Q: 모두의 말뭉치로 만든 사전을 독점 소프트웨어에 포함할 수 있나요?**

A: 가능하지만, 사전 자체는 CC BY-SA 2.0 KR로 배포해야 합니다. 사전과 소프트웨어를 분리하여 배포하는 것을 권장합니다.

**Q: 모두의 말뭉치와 독점 말뭉치를 섞을 수 있나요?**

A: 가능하지만, 최종 사전은 CC BY-SA 2.0 KR 라이선스를 따라야 합니다 (Share-Alike 조건).

### 2. 세종 말뭉치 (Sejong Corpus)

| 항목 | 내용 |
|------|------|
| **제공 기관** | 국립국어원 |
| **라이선스** | 연구 목적 제한 라이선스 |
| **웹사이트** | https://ithub.korean.go.kr |
| **규모** | 약 1,000만 어절 |

#### 사용 조건

✅ **허용되는 사용**
- 학술 연구 목적 사용
- 교육 목적 사용
- 비영리 목적 사용

❌ **제한되는 사용**
- 상업적 사용 **사전 승인 필요**
- 재배포 **제한**
- 상업용 제품 포함 **승인 필요**

#### 상업적 사용 절차

1. **국립국어원 언어정보나눔터** 접속
2. **말뭉치 저작권 승인 신청서** 작성
3. 사용 목적 및 범위 명시
4. 승인 대기 (통상 2-4주)
5. 승인 후 조건에 따라 사용

⚠️ **주의**: 승인 없이 상업적으로 사용 시 저작권법 위반

### 3. 국립국어원 기타 말뭉치

#### 3.1. 한국어 학습자 말뭉치
- **라이선스**: CC BY-NC-SA 2.0 KR
- **상업적 사용**: 불가 (Non-Commercial)
- **용도**: 연구 및 교육 목적

#### 3.2. 일상 대화 말뭉치
- **라이선스**: CC BY-NC-SA 4.0
- **개인정보**: 익명화 처리됨
- **상업적 사용**: 불가

### 4. AI Hub 말뭉치

| 항목 | 내용 |
|------|------|
| **제공 기관** | 한국지능정보사회진흥원 (NIA) |
| **웹사이트** | https://aihub.or.kr |
| **라이선스** | 데이터셋별 상이 |

⚠️ **주의**: AI Hub의 각 데이터셋은 개별 라이선스를 가지므로 반드시 확인 필요

### 5. 기타 공개 말뭉치

#### 5.1. KLUE (Korean Language Understanding Evaluation)
- **라이선스**: CC BY-SA 4.0
- **용도**: 연구 및 상업적 사용 가능
- **출처**: https://klue-benchmark.com/

#### 5.2. KAIST 말뭉치
- **라이선스**: 개별 확인 필요
- **용도**: 주로 연구 목적

## 라이선스별 사용 가능 범위

### 비교표

| 라이선스 | 상업적 사용 | 수정 | 배포 | 출처 표시 | Share-Alike | 비고 |
|---------|------------|------|------|-----------|-------------|------|
| CC BY-SA 2.0 KR | ✅ | ✅ | ✅ | 필수 | 필수 | 모두의 말뭉치 |
| CC BY-NC-SA 2.0 KR | ❌ | ✅ | ✅ | 필수 | 필수 | 일부 교육용 |
| 세종 (연구 제한) | ⚠️ 승인 필요 | ✅ | ❌ | 필수 | - | 연구 우선 |
| CC BY 4.0 | ✅ | ✅ | ✅ | 필수 | - | 일부 공개 데이터 |
| CC0 (Public Domain) | ✅ | ✅ | ✅ | - | - | 드물음 |

### Creative Commons 라이선스 요소 설명

- **BY (Attribution)**: 출처 표시 필수
- **SA (Share-Alike)**: 동일 조건 공유 (파생물도 같은 라이선스)
- **NC (Non-Commercial)**: 비상업적 사용만 허용
- **ND (No-Derivatives)**: 변경 금지 (말뭉치에서는 드묾)

## 사전 배포 시 준수사항

### 1. LICENSES 파일 작성

사전 배포 시 반드시 포함해야 할 정보:

```markdown
# Dictionary Licenses

## Dictionary Information
- Name: MyKorean MeCab Dictionary
- Version: 1.0.0
- Release Date: 2026-01-05

## Source Corpora

### Modu Corpus (70% of entries)
- Provider: National Institute of Korean Language
- License: CC BY-SA 2.0 KR
- URL: https://corpus.korean.go.kr
- License URL: https://creativecommons.org/licenses/by-sa/2.0/kr/

### Custom Corpus (30% of entries)
- Provider: MyCompany Inc.
- License: Proprietary
- Usage: Internal use only

## Dictionary License

This dictionary is licensed under CC BY-SA 2.0 KR due to the
share-alike requirement of the Modu Corpus.

## Attribution

This work is based on "모두의 말뭉치" by the National Institute
of Korean Language, used under CC BY-SA 2.0 KR.

## Contact
- Email: contact@mycompany.com
- Issues: https://github.com/mycompany/mecab-dict/issues
```

### 2. README에 라이선스 정보 포함

```markdown
## License

This dictionary is licensed under CC BY-SA 2.0 KR.

### Attribution

Based on the following corpora:
- 모두의 말뭉치 (Modu Corpus) - CC BY-SA 2.0 KR

See the [LICENSES](LICENSES.md) file for full license information.
```

### 3. 소스 코드 헤더

각 CSV 파일 상단에 주석으로 포함:

```csv
# MeCab Korean Dictionary
# Based on 모두의 말뭉치 (Modu Corpus)
# License: CC BY-SA 2.0 KR
# Source: https://corpus.korean.go.kr
# Generated: 2026-01-05
#
surface,left_id,right_id,cost,pos,...
```

### 4. 메타데이터 파일

`metadata.json` 예시:

```json
{
  "name": "MyKorean MeCab Dictionary",
  "version": "1.0.0",
  "license": "CC BY-SA 2.0 KR",
  "sources": [
    {
      "name": "Modu Corpus",
      "provider": "National Institute of Korean Language",
      "license": "CC BY-SA 2.0 KR",
      "url": "https://corpus.korean.go.kr",
      "percentage": 70
    }
  ],
  "generated_at": "2026-01-05T00:00:00Z",
  "generator": "mecab-ko-corpus-tools v1.0"
}
```

## 체크리스트

### 사전 생성 전

- [ ] 사용할 말뭉치의 라이선스 확인
- [ ] 상업적 사용 계획이 있으면 라이선스 호환성 검토
- [ ] 필요시 사전 승인 신청 (세종 말뭉치 등)
- [ ] 혼합 사전의 경우 라이선스 충돌 여부 확인

### 사전 배포 전

- [ ] LICENSES 또는 LICENSE.md 파일 작성
- [ ] README에 라이선스 정보 포함
- [ ] 출처 표시 문구 작성
- [ ] 메타데이터 파일 생성 (선택사항)
- [ ] 각 소스 말뭉치의 비율 기록
- [ ] 라이선스 URL 링크 포함

### 상업적 배포 전

- [ ] 모든 말뭉치가 상업적 사용 가능한지 확인
- [ ] Share-Alike 조건 준수 방안 마련
- [ ] 법무팀 또는 변호사 검토 (권장)
- [ ] 필요시 보험 가입 검토

### 오픈소스 프로젝트

- [ ] GitHub/GitLab에 LICENSE 파일 포함
- [ ] 각 릴리스에 라이선스 정보 포함
- [ ] NOTICE 파일 작성 (Apache 스타일)
- [ ] CHANGELOG에 라이선스 변경사항 기록

## 위반 시 책임

### 저작권법 위반 시 처벌

- **민사상 책임**: 손해배상 청구 가능
- **형사상 책임**: 5년 이하 징역 또는 5천만원 이하 벌금
- **부정경쟁방지법**: 추가 처벌 가능

### 리스크 최소화 방안

1. **라이선스 준수**: 모든 라이선스 조건 철저히 준수
2. **문서화**: 사용한 말뭉치와 라이선스 명확히 기록
3. **분리 배포**: 가능하면 사전과 소프트웨어 분리
4. **전문가 상담**: 불확실한 경우 법률 전문가 상담
5. **보험**: 상업적 배포 시 배상책임보험 고려

## 추가 자료

### 관련 법률

- 저작권법 (Copyright Act)
- 부정경쟁방지 및 영업비밀보호에 관한 법률
- 개인정보 보호법 (말뭉치에 개인정보 포함 시)

### 유용한 링크

- [국립국어원 언어정보나눔터](https://ithub.korean.go.kr)
- [Creative Commons Korea](https://creativecommons.or.kr)
- [한국저작권위원회](https://www.copyright.or.kr)
- [AI Hub](https://aihub.or.kr)

### 문의처

- **국립국어원**: 02-2669-9775
- **한국저작권위원회**: 1800-5455
- **Creative Commons Korea**: info@cckorea.org

## 면책 조항

이 문서는 일반적인 가이드를 제공하며, 법률 자문을 대체하지 않습니다. 상업적 사용이나 복잡한 라이선스 문제의 경우 반드시 법률 전문가와 상담하시기 바랍니다.

---

**작성일**: 2026-01-05
**최종 수정**: 2026-01-05
**버전**: 1.0

**참고**: 이 문서는 2026년 1월 기준으로 작성되었으며, 라이선스 및 법률은 변경될 수 있습니다. 최신 정보는 각 기관의 공식 웹사이트를 참조하세요.
