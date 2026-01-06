# MeCab-Ko 말뭉치 처리 스크립트 - 파일 인덱스

## 📁 파일 구조

```
/home/mare/mecab-ko/scripts/
├── README.md                    # 메인 문서
├── QUICKSTART.md               # 5분 빠른 시작 가이드
├── CORPUS_LICENSES.md          # 저작권 및 라이선스 상세 가이드
├── INDEX.md                    # 이 파일
│
├── corpus_to_dict.py           # 말뭉치 → 사전 변환
├── extract_neologisms.py       # 신조어 추출
├── merge_dictionaries.py       # 사전 병합 및 분석
│
├── example_workflow.sh         # 전체 워크플로우 예제
├── test_scripts.py            # 테스트 스위트
└── requirements.txt            # Python 의존성 (선택사항)
```

## 📚 문서

### [README.md](README.md)
**주요 문서 - 여기서 시작하세요!**

- 프로젝트 개요
- 스크립트 상세 설명
- 사용법 및 예제
- 지원 형식
- 문제 해결

**읽어야 할 사람:**
- 모든 사용자 (필수)
- 처음 사용하는 개발자
- 고급 사용 사례가 필요한 사용자

### [QUICKSTART.md](QUICKSTART.md)
**5분 빠른 시작**

- 최소한의 설정으로 빠르게 시작
- 샘플 데이터 생성 및 테스트
- 일반적인 시나리오 예제
- 유용한 명령어 모음

**읽어야 할 사람:**
- 빠르게 시작하고 싶은 사용자
- 개념 증명이 필요한 사용자
- 튜토리얼을 선호하는 사용자

### [CORPUS_LICENSES.md](CORPUS_LICENSES.md)
**저작권 및 라이선스 완벽 가이드**

- 한국어 말뭉치 라이선스 상세 설명
- 상업적 사용 가능 여부
- 사전 배포 시 준수사항
- 체크리스트 및 템플릿

**읽어야 할 사람:**
- 상업적으로 사용할 계획이 있는 사용자 (필수)
- 사전을 배포할 계획이 있는 사용자 (필수)
- 법적 요구사항을 이해해야 하는 사용자

### [INDEX.md](INDEX.md)
**이 파일**

- 전체 파일 구조
- 각 파일의 목적 및 사용법
- 빠른 참조

## 🛠 스크립트

### [corpus_to_dict.py](corpus_to_dict.py)
**말뭉치를 MeCab 사전으로 변환**

**기능:**
- 모두의 말뭉치 (JSON)
- 세종 말뭉치 (XML)
- CoNLL-U 형식
- 자동 빈도 계산 및 비용 할당
- 통계 분석

**사용 예:**
```bash
./corpus_to_dict.py -f modu -i corpus/ -o dict.csv --min-freq 2
```

**출력:**
- MeCab CSV 사전 파일
- 통계 로그 (표준 에러)

**의존성:**
- Python 3.10+
- 표준 라이브러리만 사용 (외부 패키지 불필요)

### [extract_neologisms.py](extract_neologisms.py)
**말뭉치에서 신조어 자동 추출**

**기능:**
- 패턴 기반 신조어 탐지
- 빈도 필터링
- 기존 사전 제외
- JSON/CSV 출력

**사용 예:**
```bash
./extract_neologisms.py \
  -f modu -i corpus/ -o neo.json \
  --reference-dict base.csv \
  --min-freq 3 --max-freq 100
```

**출력:**
- JSON: 메타데이터 포함, 구조화된 데이터
- CSV: 수동 검토용, 스프레드시트 호환

**의존성:**
- Python 3.10+
- 표준 라이브러리만 사용

### [merge_dictionaries.py](merge_dictionaries.py)
**여러 사전 병합 및 분석**

**기능:**
- 다중 사전 병합
- 충돌 해결 (min_cost, max_cost, first, last, avg_cost)
- 자동 중복 제거
- 사전 통계 분석

**사용 예:**
```bash
# 병합
./merge_dictionaries.py \
  -i dict1.csv dict2.csv dict3.csv \
  -o merged.csv \
  --strategy min_cost

# 분석
./merge_dictionaries.py --analyze merged.csv
```

**출력:**
- 병합된 CSV 사전
- 통계 보고서 (품사 분포, 비용 통계 등)

**의존성:**
- Python 3.10+
- 표준 라이브러리만 사용

## 🔧 유틸리티

### [example_workflow.sh](example_workflow.sh)
**완전한 워크플로우 자동화**

**기능:**
- 전체 사전 구축 프로세스 자동화
- 말뭉치 변환 → 신조어 추출 → 병합 → 분석
- 메타데이터 및 라이선스 파일 생성
- 상세한 로그 및 요약

**사용 예:**
```bash
./example_workflow.sh /path/to/corpus /path/to/output
```

**출력:**
- 최종 사전 (mecab_dict.csv)
- 메타데이터 (metadata.json)
- 라이선스 파일 (LICENSE.txt)
- 통계 파일들 (stats/)

**사용 시기:**
- 전체 프로세스를 한 번에 실행하고 싶을 때
- 일관된 결과가 필요할 때
- 배치 작업 또는 CI/CD 파이프라인

### [test_scripts.py](test_scripts.py)
**자동 테스트 스위트**

**기능:**
- 모든 스크립트 자동 테스트
- 샘플 데이터 생성
- 통과/실패 보고

**사용 예:**
```bash
./test_scripts.py
```

**사용 시기:**
- 설치 후 검증
- 개발 중 회귀 테스트
- 환경 변경 후 확인

## 📦 의존성

### [requirements.txt](requirements.txt)
**Python 패키지 의존성**

**참고:** 모든 스크립트는 Python 표준 라이브러리만으로 동작합니다.
이 파일의 패키지들은 선택적 향상 기능을 위한 것입니다.

**선택적 패키지:**
- `tqdm`: 진행률 표시
- `pandas`, `numpy`: 데이터 분석
- `lxml`: XML 파싱 성능 향상
- `pytest`: 테스트
- `mypy`, `ruff`: 코드 품질

**설치:**
```bash
pip install -r requirements.txt  # 선택사항
```

## 🚀 빠른 참조

### 일반적인 작업

#### 1. 말뭉치에서 기본 사전 만들기
```bash
./corpus_to_dict.py -f modu -i corpus/ -o dict.csv --min-freq 2
```

#### 2. 신조어 찾기
```bash
./extract_neologisms.py -f modu -i corpus/ -o neo.json \
  --reference-dict dict.csv --min-freq 3
```

#### 3. 여러 사전 병합
```bash
./merge_dictionaries.py -i dict1.csv dict2.csv -o merged.csv
```

#### 4. 사전 분석
```bash
./merge_dictionaries.py --analyze dict.csv
```

#### 5. 전체 자동화
```bash
./example_workflow.sh /data/corpus /output
```

### 문제 해결

#### Python 버전 확인
```bash
python3 --version  # 3.10 이상 필요
```

#### 도움말 보기
```bash
./corpus_to_dict.py --help
./extract_neologisms.py --help
./merge_dictionaries.py --help
```

#### 테스트 실행
```bash
./test_scripts.py
```

## 📖 읽기 순서 권장

### 초보자
1. [QUICKSTART.md](QUICKSTART.md) - 5분 시작
2. [README.md](README.md) - 전체 개요
3. 실제 데이터로 실습
4. [CORPUS_LICENSES.md](CORPUS_LICENSES.md) - 배포 전 필독

### 고급 사용자
1. [README.md](README.md) - 고급 기능 섹션
2. 개별 스크립트 소스 코드
3. [example_workflow.sh](example_workflow.sh) - 커스터마이징

### 상업적 사용자
1. [CORPUS_LICENSES.md](CORPUS_LICENSES.md) - **필수**
2. [README.md](README.md) - 사용법
3. 법무팀과 검토

## 🔗 관련 문서

### 프로젝트 문서
- `/home/mare/mecab-ko/CLAUDE.md` - 프로젝트 구조
- `/home/mare/mecab-ko/docs/PROJECT_PLAN.md` - 로드맵
- `/home/mare/mecab-ko/docs/ISSUE_BACKLOG.md` - 이슈 백로그

### 외부 리소스
- [MeCab 공식 문서](https://taku910.github.io/mecab/)
- [모두의 말뭉치](https://corpus.korean.go.kr)
- [Creative Commons Korea](https://creativecommons.or.kr)

## 📞 지원

### 문제 보고
- GitHub Issues: (프로젝트 저장소)
- 버그, 기능 요청, 문서 개선 제안

### 기여
- Pull Requests 환영
- 코딩 규칙: PEP 8, type hints, docstrings
- 테스트 작성 권장

## 📝 버전 정보

- **버전**: 1.0.0
- **작성일**: 2026-01-05
- **Python 요구사항**: 3.10+
- **테스트 상태**: ✓ 모든 테스트 통과

## 📄 라이선스

이 스크립트들은 MeCab-Ko 프로젝트의 일부로 GPL, LGPL, BSD 3중 라이선스를 따릅니다.

생성된 사전의 라이선스는 원본 말뭉치의 라이선스를 따릅니다.
자세한 내용은 [CORPUS_LICENSES.md](CORPUS_LICENSES.md)를 참조하세요.

---

**마지막 업데이트**: 2026-01-05
**관리자**: MeCab-Ko Team
