# MeCab-Ko 도메인 사전

도메인별 전문 용어 사전입니다. IT 용어, 뉴스 고유명사, 정부 기관명 등 분야별로 구성되어 있습니다.

## 개요

| 사전 | 엔트리 | 형식 | 갱신 |
|------|--------|------|------|
| IT 용어 | 283,670 | 13필드 CSV | 수동 큐레이션 |
| 뉴스 NNP | ~3,000/주 | 12필드 CSV | 자동 (주간) |
| 관보 기관명 | 503 | 12필드 CSV | 수동 (일회성) |

- **포맷**: MeCab CSV 형식
- **인코딩**: UTF-8
- **목적**: 한국어 텍스트의 도메인별 형태소 분석 정확도 향상

## 사전 구조

```
data/domain-dic/
├── it-terms/                          # IT 용어 사전 (283K)
│   ├── programming_languages.csv      # 프로그래밍 언어 (49,096개)
│   ├── frameworks_libraries.csv       # 프레임워크/라이브러리 (170,899개)
│   ├── cloud_infrastructure.csv       # 클라우드/인프라 (18,493개)
│   ├── ai_ml.csv                      # AI/ML (19,642개)
│   └── general_it.csv                 # 일반 IT 용어 (25,540개)
├── news/                              # 뉴스 고유명사 (주간 자동 갱신)
│   ├── news-nnp.csv                   # 뉴스 NNP 사전
│   └── README.md
├── government/                        # 관보 기관명 (503건)
│   ├── agencies.csv                   # 정부 기관명 NNP 사전
│   └── README.md
├── sources/                           # 원본 데이터 (예약)
├── statistics.json                    # 사전 통계
└── README.md                          # 이 문서
```

## 카테고리별 상세

### 1. 프로그래밍 언어 (programming_languages.csv)
**49,096 엔트리**

주요 포함 내용:
- **메이저 언어**: Python, JavaScript, Java, C++, Rust, Go, TypeScript 등
- **스크립트 언어**: Ruby, PHP, Perl, Lua, Bash 등
- **함수형 언어**: Haskell, Clojure, Erlang, Elixir, Scala 등
- **시스템 언어**: C, C++, Rust, Zig, Nim 등
- **웹 언어**: TypeScript, Dart, Elm 등
- **데이터 과학**: R, Julia, MATLAB 등
- **블록체인**: Solidity, Move, Cairo 등

용어 형태:
- 영문 표기: `Python`, `JavaScript`
- 한글 표기: `파이썬`, `자바스크립트`
- 변이형: `자바스크립트`, `자스`
- 복합어: `Python 개발자`, `JavaScript 프로그래밍`

### 2. 프레임워크/라이브러리 (frameworks_libraries.csv)
**170,899 엔트리** (최대 카테고리)

주요 포함 내용:
- **웹 프레임워크**: React, Angular, Vue, Django, Flask, Spring 등
- **백엔드**: Express, FastAPI, NestJS, Phoenix 등
- **모바일**: React Native, Flutter, Ionic 등
- **데스크톱**: Electron, Tauri, Qt 등
- **ML/AI**: TensorFlow, PyTorch, Keras, Hugging Face 등
- **데이터 처리**: NumPy, Pandas, Apache Arrow 등
- **데이터 시각화**: Matplotlib, D3.js, Plotly 등
- **테스팅**: Jest, Pytest, Selenium, Cypress 등
- **빌드 도구**: Webpack, Vite, Babel, esbuild 등

### 3. 클라우드/인프라 (cloud_infrastructure.csv)
**18,493 엔트리**

주요 포함 내용:
- **클라우드 제공자**: AWS, Azure, GCP, DigitalOcean 등
- **컨테이너**: Docker, Kubernetes, Podman 등
- **CI/CD**: Jenkins, GitHub Actions, GitLab CI, ArgoCD 등
- **IaC**: Terraform, Ansible, Pulumi 등
- **모니터링**: Prometheus, Grafana, Datadog, Elastic 등
- **메시징**: Kafka, RabbitMQ, Redis, NATS 등
- **서비스 메시**: Istio, Linkerd, Envoy 등
- **API 게이트웨이**: Kong, Traefik, NGINX 등

### 4. AI/ML (ai_ml.csv)
**19,642 엔트리**

주요 포함 내용:
- **LLM 모델**: GPT, Claude, Llama, Mistral, Gemini 등
- **비전 모델**: CLIP, DALL-E, SAM, YOLO, ResNet 등
- **오디오 모델**: Whisper, Wav2Vec 등
- **ML 기법**: Transformer, GAN, CNN, RNN, LSTM 등
- **강화학습**: PPO, DQN, AlphaGo 등
- **앙상블**: XGBoost, LightGBM, Random Forest 등
- **최적화**: Adam, SGD, AdaGrad 등
- **개념**: 딥러닝, 지도학습, 강화학습, 전이학습 등
- **벡터 DB**: ChromaDB, Pinecone, Weaviate, Milvus 등

### 5. 일반 IT 용어 (general_it.csv)
**25,540 엔트리**

주요 포함 내용:
- **개발 도구**: Git, VS Code, IntelliJ, Vim 등
- **데이터베이스**: PostgreSQL, MySQL, MongoDB, Redis 등
- **프로토콜**: HTTP, WebSocket, GraphQL, gRPC 등
- **보안**: OAuth, JWT, SSL/TLS 등
- **패키지 관리자**: npm, pip, Maven, Cargo 등
- **방법론**: 애자일, 스크럼, DevOps, TDD 등
- **아키텍처 패턴**: 마이크로서비스, MSA, DDD 등
- **디자인 패턴**: 싱글톤, 팩토리, 옵저버 등
- **IDE**: WebStorm, PyCharm, GoLand 등
- **협업 도구**: Jira, Notion, Figma 등

## 사전 특징

### 1. 다양한 표기 지원
- **영문 원어**: `TensorFlow`, `Kubernetes`
- **한글 발음**: `텐서플로`, `쿠버네티스`
- **약어**: `K8s`, `ML`, `AI`
- **변이형**: `쿠버네티스`/`쿠베르네테스`

### 2. 복합어 포함
- 직업: `Python 개발자`, `React 프로그래머`
- 기술: `Kubernetes 환경`, `Docker 컨테이너`
- 활동: `머신러닝 학습`, `클라우드 배포`

### 3. 품사 태그
- **NNP** (고유명사): 프로그래밍 언어, 제품명 (Python, AWS)
- **NNG** (일반명사): 복합어, 기술 개념 (마이크로서비스, 딥러닝)
- **SL** (외국어): 원어 표기

### 4. 비용 설정
- **-5000**: 핵심 용어 (높은 우선순위)
- **-4000**: 기술 구문
- **-3000**: 복합어

## 사용 방법

### 1. MeCab 사전 컴파일

```bash
# 사전 디렉토리로 이동
cd /home/mare/mecab-ko/data/domain-dic/it-terms

# MeCab 사전 컴파일 (각 CSV 파일)
/usr/local/libexec/mecab/mecab-dict-index \
  -d /usr/local/lib/mecab/dic/mecab-ko-dic \
  -u programming_languages.dic \
  -f utf-8 \
  -t utf-8 \
  programming_languages.csv

# 다른 파일들도 동일하게 컴파일
```

### 2. MeCab 설정 파일에 추가

`mecabrc` 파일에 사용자 사전 추가:

```
# 기본 사전
dicdir = /usr/local/lib/mecab/dic/mecab-ko-dic

# 사용자 사전 (IT 용어)
userdic = /home/mare/mecab-ko/data/domain-dic/it-terms/programming_languages.dic,/home/mare/mecab-ko/data/domain-dic/it-terms/frameworks_libraries.dic
```

### 3. Python에서 사용

```python
import MeCab

# MeCab 초기화 (사용자 사전 포함)
tagger = MeCab.Tagger('-d /usr/local/lib/mecab/dic/mecab-ko-dic')

# 테스트
text = "Python과 TensorFlow를 사용한 딥러닝 개발"
result = tagger.parse(text)
print(result)
```

### 4. Rust 구현에서 사용

```rust
use mecab_ko_dict::DictBuilder;

let dict = DictBuilder::new()
    .add_user_dict("data/domain-dic/it-terms/programming_languages.csv")
    .add_user_dict("data/domain-dic/it-terms/frameworks_libraries.csv")
    .build()?;
```

## 데이터 수집 방법

### 1. 시드 데이터
수작업으로 큐레이션된 핵심 IT 용어:
- 주요 프로그래밍 언어 (30개)
- 인기 프레임워크/라이브러리 (200개)
- 주요 클라우드 서비스 (100개)
- 대표 AI/ML 모델 및 기법 (150개)
- 필수 IT 개념 (100개)

### 2. 자동 확장
- **변이형 생성**: 외래어 표기법 변이 (쿠버네티스/쿠베르네테스)
- **복합어 생성**: 기술 용어 + 접미사/접두사 조합
- **기술 구문**: 개발 방법론, 아키텍처 패턴, 보안 개념 등

### 3. 품질 관리
- 중복 제거
- 품사 태그 검증
- MeCab CSV 포맷 검증
- 읽기 필드 완성도 100%

## 통계

### 전체 통계
```json
{
  "total_terms": 283670,
  "categories": 5,
  "unique_surfaces": 283670,
  "reading_coverage": 100.0,
  "variant_terms": 147
}
```

### 카테고리별 통계
| 카테고리 | 엔트리 수 | 읽기 포함 | 변이형 포함 |
|---------|----------|----------|------------|
| programming_languages | 49,096 | 100% | 17 |
| frameworks_libraries | 170,899 | 100% | 24 |
| cloud_infrastructure | 18,493 | 100% | 22 |
| ai_ml | 19,642 | 100% | 45 |
| general_it | 25,540 | 100% | 39 |

## 유지보수

### 용어 추가

새로운 IT 용어를 추가하려면:

```bash
# 수집 스크립트 실행
python3 /home/mare/mecab-ko/tools/dict-expander/maximize_terms.py

# 검증
python3 /home/mare/mecab-ko/tools/dict-expander/validate_dict.py \
  data/domain-dic/it-terms/

# 중복 제거
python3 /home/mare/mecab-ko/tools/dict-expander/validate_dict.py \
  data/domain-dic/it-terms/ \
  --remove-duplicates
```

### 수동 편집

CSV 파일 포맷:
```
표면형,0,0,비용,품사,*,*,*,*,*,원형,읽기,발음
Python,0,0,-5000,NNP,*,*,*,*,*,Python,파이썬,파이썬
파이썬,0,0,-5000,NNP,*,*,*,*,*,파이썬,파이썬,파이썬
```

필드 설명:
1. **표면형**: 사전에 등록될 단어
2. **좌문맥ID**: 0 (mecab-dict-index가 생성)
3. **우문맥ID**: 0 (mecab-dict-index가 생성)
4. **비용**: 음수 (낮을수록 높은 우선순위)
5. **품사**: NNP/NNG/SL 등
6-10. **품사세분류**: * (미사용)
11. **원형**: 보통 표면형과 동일
12. **읽기**: 한글 발음
13. **발음**: 읽기와 동일

## 검증 도구

### validate_dict.py

```bash
# 기본 검증
python3 tools/dict-expander/validate_dict.py data/domain-dic/it-terms/

# 엄격 모드 (IT 용어 품사 제한)
python3 tools/dict-expander/validate_dict.py \
  data/domain-dic/it-terms/ \
  --strict

# JSON 리포트 생성
python3 tools/dict-expander/validate_dict.py \
  data/domain-dic/it-terms/ \
  --output-report validation_report.json

# 중복 제거
python3 tools/dict-expander/validate_dict.py \
  data/domain-dic/it-terms/ \
  --remove-duplicates
```

검증 항목:
- ✓ CSV 포맷 유효성
- ✓ 품사 태그 유효성
- ✓ 중복 항목 검사
- ✓ 읽기 필드 완성도
- ✓ 비용 값 적절성

## 업데이트 이력

### 2026-01-27 - v1.0.0 (초기 릴리스)
- 283,670개 IT 용어 수집 (중복 제거 완료)
- 5개 카테고리 구성
- 프로그래밍 언어, 프레임워크, 클라우드, AI/ML, 일반 IT 용어 포함
- 외래어 변이형 지원
- 복합어 자동 생성
- 품질 검증 도구 제공
- 100% 읽기 필드 커버리지

## 기여 가이드

### 새로운 용어 제안

1. GitHub Issue 생성
2. 용어 정보 제공:
   - 표면형 (영문/한글)
   - 읽기 (한글 발음)
   - 카테고리
   - 변이형 (있는 경우)
3. Pull Request 제출

### 용어 수정

1. 해당 CSV 파일 직접 수정
2. 검증 도구 실행
3. Pull Request 제출

## 라이선스

이 사전은 MeCab-Ko 프로젝트의 일부이며, 프로젝트 라이선스를 따릅니다.

## 참고 자료

- [MeCab 공식 문서](https://taku910.github.io/mecab/)
- [MeCab-Ko 프로젝트](/home/mare/mecab-ko)
- [한글 외래어 표기법](https://kornorms.korean.go.kr/)

## 문의

- GitHub Issues: [MeCab-Ko Issues](https://github.com/your-repo/mecab-ko/issues)
- 프로젝트 메인테이너: MeCab-Ko Team

---

**생성일**: 2026-01-27
**버전**: 1.0.0
**상태**: 운영 준비 완료
