#!/usr/bin/env python3
"""
IT/기술 용어 수집 스크립트

Wikipedia, GitHub, 그리고 정적 데이터 소스에서 IT 용어를 수집합니다.
"""

import json
import re
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any
import unicodedata


@dataclass
class ITTerm:
    """IT 용어 데이터 클래스."""

    surface: str  # 표면형
    category: str  # 카테고리
    reading: str = ""  # 읽기 (한글 발음)
    variants: list[str] = field(default_factory=list)  # 변이형
    pos: str = "NNP"  # 품사 (기본: 고유명사)
    cost: int = -5000  # 비용 (높은 우선순위)

    def __hash__(self) -> int:
        return hash(self.surface)


class KoreanRomanization:
    """한글-로마자 변환 및 외래어 표기 변이형 생성."""

    # 외래어 표기법 변이형 패턴
    VARIANT_PATTERNS = {
        # 자음 변이
        'ㅋ': ['ㅋ', 'ㄱ'],
        'ㄱ': ['ㄱ', 'ㅋ'],
        'ㅍ': ['ㅍ', 'ㅂ'],
        'ㅂ': ['ㅂ', 'ㅍ'],
        'ㅌ': ['ㅌ', 'ㄷ'],
        'ㄷ': ['ㄷ', 'ㅌ'],
        # 모음 변이
        'ㅓ': ['ㅓ', 'ㅔ', 'ㅐ'],
        'ㅔ': ['ㅔ', 'ㅓ', 'ㅐ'],
        'ㅐ': ['ㅐ', 'ㅔ', 'ㅓ'],
        'ㅗ': ['ㅗ', 'ㅜ'],
        'ㅜ': ['ㅜ', 'ㅗ'],
    }

    @staticmethod
    def is_hangul(char: str) -> bool:
        """한글 음절인지 확인."""
        code = ord(char)
        return 0xAC00 <= code <= 0xD7A3

    @staticmethod
    def decompose_hangul(syllable: str) -> tuple[str, str, str]:
        """한글 음절을 초성, 중성, 종성으로 분해."""
        if not KoreanRomanization.is_hangul(syllable):
            return "", "", ""

        code = ord(syllable) - 0xAC00
        jong = code % 28
        jung = ((code - jong) // 28) % 21
        cho = ((code - jong) // 28) // 21

        CHO = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ"
        JUNG = "ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ"
        JONG = [""] + list("ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ")

        return CHO[cho], JUNG[jung], JONG[jong]

    @classmethod
    def generate_variants(cls, term: str, max_variants: int = 5) -> list[str]:
        """외래어 표기 변이형 생성."""
        if not any(cls.is_hangul(c) for c in term):
            return []

        variants = set()

        # 간단한 자모 대체 기반 변이형 생성
        for i, char in enumerate(term):
            if not cls.is_hangul(char):
                continue

            cho, jung, jong = cls.decompose_hangul(char)

            # 초성 변이
            if cho in cls.VARIANT_PATTERNS:
                for alt_cho in cls.VARIANT_PATTERNS[cho]:
                    if alt_cho != cho:
                        # 변이형 생성 로직 (간단화)
                        variant = term[:i] + char + term[i+1:]
                        if variant != term:
                            variants.add(variant)

        return sorted(variants)[:max_variants]


class ITTermCollector:
    """IT 용어 수집기."""

    # 카테고리별 핵심 용어 (시드 데이터)
    SEED_TERMS = {
        "programming_languages": [
            # 메이저 언어
            ("Python", "파이썬", ["파이선"]),
            ("JavaScript", "자바스크립트", ["자바스크립트", "자스"]),
            ("Java", "자바", []),
            ("C++", "씨플러스플러스", ["C++", "씨쁠쁠"]),
            ("C#", "씨샵", ["C#", "씨샤프"]),
            ("Rust", "러스트", ["Rust", "러스트"]),
            ("Go", "고", ["고", "Go", "Golang", "고랭"]),
            ("Swift", "스위프트", []),
            ("Kotlin", "코틀린", ["코틀린", "Kotlin"]),
            ("TypeScript", "타입스크립트", ["타입스크립트", "TS"]),
            ("Ruby", "루비", []),
            ("PHP", "피에이치피", ["PHP"]),
            ("Scala", "스칼라", []),
            ("R", "알", ["R"]),
            ("Dart", "다트", []),
            ("Elixir", "엘릭서", []),
            ("Haskell", "하스켈", []),
            ("Clojure", "클로저", ["클로져"]),
            ("Erlang", "얼랭", []),
            ("Perl", "펄", []),
            ("Lua", "루아", []),
            ("Julia", "줄리아", []),
            ("MATLAB", "매트랩", []),
            ("Assembly", "어셈블리", []),
            ("Fortran", "포트란", []),
            ("COBOL", "코볼", []),
            ("Objective-C", "오브젝티브씨", ["오브젝티브-C"]),
            ("F#", "에프샵", ["F#"]),
            ("OCaml", "오캐멀", []),
            ("Groovy", "그루비", []),
            ("Ada", "에이다", []),
        ],
        "frameworks_libraries": [
            # 웹 프레임워크
            ("React", "리액트", ["React"]),
            ("Angular", "앵귤러", ["앵글러"]),
            ("Vue", "뷰", ["Vue.js", "뷰제이에스"]),
            ("Django", "장고", ["Django", "쟝고"]),
            ("Flask", "플라스크", []),
            ("FastAPI", "패스트API", ["패스트에이피아이"]),
            ("Spring", "스프링", ["Spring Boot", "스프링부트"]),
            ("Express", "익스프레스", ["Express.js"]),
            ("Next.js", "넥스트", ["넥스트제이에스"]),
            ("Nuxt.js", "넉스트", ["넉스트제이에스"]),
            ("Svelte", "스벨트", []),
            ("Solid", "솔리드", []),
            ("Remix", "리믹스", []),
            ("Astro", "아스트로", []),

            # 백엔드/API
            ("GraphQL", "그래프큐엘", ["GraphQL"]),
            ("gRPC", "지알피씨", ["gRPC"]),
            ("REST", "레스트", ["REST", "RESTful", "레스트풀"]),
            ("Node.js", "노드", ["노드제이에스"]),
            ("Deno", "데노", []),
            ("Bun", "번", []),

            # ML/AI 프레임워크
            ("TensorFlow", "텐서플로", ["텐서플로우"]),
            ("PyTorch", "파이토치", []),
            ("Keras", "케라스", []),
            ("Scikit-learn", "사이킷런", ["싸이킷런"]),
            ("Hugging Face", "허깅페이스", ["허깅 페이스"]),
            ("LangChain", "랭체인", []),
            ("OpenAI", "오픈AI", ["오픈에이아이"]),
            ("Anthropic", "앤트로픽", []),

            # 데이터베이스 ORM
            ("SQLAlchemy", "에스큐엘알케미", []),
            ("Prisma", "프리즈마", []),
            ("TypeORM", "타입오알엠", []),
            ("Mongoose", "몽구스", []),
            ("Sequelize", "시퀄라이즈", []),

            # 테스팅
            ("Jest", "제스트", []),
            ("Pytest", "파이테스트", []),
            ("Mocha", "모카", []),
            ("Cypress", "사이프레스", []),
            ("Selenium", "셀레니움", []),
            ("Playwright", "플레이라이트", []),

            # UI 라이브러리
            ("Bootstrap", "부트스트랩", []),
            ("Tailwind", "테일윈드", ["Tailwind CSS"]),
            ("Material-UI", "머티리얼UI", ["MUI"]),
            ("Chakra UI", "차크라UI", []),
            ("Ant Design", "앤트디자인", []),

            # 상태관리
            ("Redux", "리덕스", []),
            ("MobX", "몹엑스", []),
            ("Zustand", "주스탠드", []),
            ("Recoil", "리코일", []),
            ("Jotai", "조타이", []),
        ],
        "cloud_infrastructure": [
            # 클라우드 제공자
            ("AWS", "에이더블유에스", ["Amazon Web Services", "아마존웹서비스"]),
            ("Azure", "애저", ["애져"]),
            ("GCP", "지씨피", ["Google Cloud Platform", "구글클라우드플랫폼"]),
            ("Alibaba Cloud", "알리바바클라우드", []),
            ("Oracle Cloud", "오라클클라우드", []),

            # 컨테이너/오케스트레이션
            ("Docker", "도커", []),
            ("Kubernetes", "쿠버네티스", ["쿠베르네테스", "K8s", "케이에잇에스"]),
            ("Podman", "팟맨", []),
            ("containerd", "컨테이너디", []),
            ("Nomad", "노마드", []),

            # CI/CD
            ("Jenkins", "젠킨스", []),
            ("GitLab CI", "깃랩CI", ["깃랩씨아이"]),
            ("GitHub Actions", "깃허브액션스", ["깃허브 액션"]),
            ("CircleCI", "서클CI", ["서클씨아이"]),
            ("Travis CI", "트래비스CI", []),
            ("ArgoCD", "아르고CD", ["아르고씨디"]),

            # IaC
            ("Terraform", "테라폼", []),
            ("Ansible", "앤서블", []),
            ("Puppet", "퍼핏", []),
            ("Chef", "셰프", []),
            ("Pulumi", "풀루미", []),

            # 모니터링/관찰성
            ("Prometheus", "프로메테우스", []),
            ("Grafana", "그라파나", []),
            ("Datadog", "데이터독", []),
            ("New Relic", "뉴렐릭", []),
            ("Elastic", "일래스틱", ["Elasticsearch", "일래스틱서치"]),
            ("Kibana", "키바나", []),
            ("Jaeger", "예거", []),

            # 메시징/스트리밍
            ("Kafka", "카프카", []),
            ("RabbitMQ", "래빗엠큐", []),
            ("Redis", "레디스", []),
            ("NATS", "내츠", []),
            ("Pulsar", "펄서", []),

            # 서비스 메시
            ("Istio", "이스티오", []),
            ("Linkerd", "링커드", []),
            ("Consul", "컨설", []),

            # 스토리지
            ("MinIO", "미니오", []),
            ("Ceph", "셰프", []),
            ("GlusterFS", "글러스터FS", []),
        ],
        "ai_ml": [
            # AI 모델/기법
            ("GPT", "지피티", ["GPT-3", "GPT-4", "지피티3", "지피티4"]),
            ("BERT", "버트", []),
            ("Transformer", "트랜스포머", []),
            ("GAN", "갠", ["Generative Adversarial Network", "생성적적대신경망"]),
            ("CNN", "씨엔엔", ["Convolutional Neural Network", "합성곱신경망"]),
            ("RNN", "알엔엔", ["Recurrent Neural Network", "순환신경망"]),
            ("LSTM", "엘에스티엠", ["Long Short-Term Memory"]),
            ("Attention", "어텐션", ["어텐션메커니즘"]),
            ("Diffusion", "디퓨전", ["Stable Diffusion", "스테이블디퓨전"]),
            ("LoRA", "로라", ["Low-Rank Adaptation"]),
            ("RAG", "래그", ["Retrieval Augmented Generation"]),
            ("Fine-tuning", "파인튜닝", ["파인튜닝", "미세조정"]),
            ("Zero-shot", "제로샷", []),
            ("Few-shot", "퓨샷", []),
            ("Prompt Engineering", "프롬프트엔지니어링", []),

            # ML 개념
            ("Deep Learning", "딥러닝", ["딥 러닝", "심층학습"]),
            ("Machine Learning", "머신러닝", ["머신 러닝", "기계학습"]),
            ("Neural Network", "신경망", ["뉴럴네트워크"]),
            ("Backpropagation", "역전파", ["백프로퍼게이션"]),
            ("Gradient Descent", "경사하강법", ["그래디언트디센트"]),
            ("Overfitting", "과적합", ["오버피팅"]),
            ("Underfitting", "과소적합", ["언더피팅"]),
            ("Regularization", "정규화", ["레귤러라이제이션"]),
            ("Normalization", "정규화", ["노멀라이제이션"]),
            ("Dropout", "드롭아웃", []),
            ("Batch Normalization", "배치정규화", ["배치노멀라이제이션"]),
            ("Cross-validation", "교차검증", ["크로스밸리데이션"]),

            # NLP
            ("Tokenization", "토큰화", ["토크나이제이션"]),
            ("Embedding", "임베딩", []),
            ("Word2Vec", "워드투벡", []),
            ("FastText", "패스트텍스트", []),
            ("BLEU", "블루", []),
            ("ROUGE", "루즈", []),

            # CV
            ("Object Detection", "객체탐지", ["객체검출", "오브젝트디텍션"]),
            ("Image Segmentation", "이미지세그멘테이션", ["이미지분할"]),
            ("OCR", "오씨알", ["Optical Character Recognition", "광학문자인식"]),

            # 데이터
            ("Dataset", "데이터셋", ["데이터세트"]),
            ("Training", "트레이닝", ["훈련"]),
            ("Inference", "추론", ["인퍼런스"]),
            ("Hyperparameter", "하이퍼파라미터", []),
            ("Feature Engineering", "피처엔지니어링", ["특성공학"]),
        ],
        "general_it": [
            # 개발 도구
            ("Git", "깃", []),
            ("GitHub", "깃허브", ["깃헙"]),
            ("GitLab", "깃랩", []),
            ("Visual Studio Code", "비주얼스튜디오코드", ["VS Code", "VSCode", "브이에스코드"]),
            ("IntelliJ", "인텔리제이", []),
            ("PyCharm", "파이참", []),
            ("Vim", "빔", []),
            ("Emacs", "이맥스", []),
            ("Sublime Text", "서브라임텍스트", []),

            # 개념
            ("API", "에이피아이", ["Application Programming Interface"]),
            ("SDK", "에스디케이", ["Software Development Kit"]),
            ("IDE", "아이디이", ["Integrated Development Environment", "통합개발환경"]),
            ("CLI", "씨엘아이", ["Command Line Interface", "명령줄인터페이스"]),
            ("GUI", "지유아이", ["Graphical User Interface", "그래픽사용자인터페이스"]),
            ("REPL", "레플", ["Read-Eval-Print Loop"]),
            ("CRUD", "크러드", ["Create Read Update Delete"]),
            ("MVC", "엠브이씨", ["Model-View-Controller"]),
            ("MVP", "엠브이피", ["Model-View-Presenter"]),
            ("MVVM", "엠브이브이엠", ["Model-View-ViewModel"]),

            # 아키텍처
            ("Microservices", "마이크로서비스", []),
            ("Monolithic", "모놀리식", ["모놀리틱"]),
            ("Serverless", "서버리스", []),
            ("Event-driven", "이벤트드리븐", ["이벤트기반"]),
            ("CQRS", "씨큐알에스", ["Command Query Responsibility Segregation"]),
            ("DDD", "디디디", ["Domain-Driven Design", "도메인주도설계"]),

            # 데이터베이스
            ("PostgreSQL", "포스트그레SQL", ["포스트그레에스큐엘", "Postgres", "포스트그레스"]),
            ("MySQL", "마이SQL", ["마이에스큐엘"]),
            ("MongoDB", "몽고DB", ["몽고디비"]),
            ("SQLite", "에스큐엘라이트", []),
            ("MariaDB", "마리아DB", ["마리아디비"]),
            ("Oracle", "오라클", []),
            ("SQL Server", "SQL서버", ["에스큐엘서버"]),
            ("Cassandra", "카산드라", []),
            ("DynamoDB", "다이나모DB", ["다이나모디비"]),
            ("Neo4j", "네오포제이", []),

            # 프로토콜/포맷
            ("HTTP", "에이치티티피", ["Hypertext Transfer Protocol"]),
            ("HTTPS", "에이치티티피에스", []),
            ("WebSocket", "웹소켓", []),
            ("JSON", "제이슨", ["JavaScript Object Notation"]),
            ("XML", "엑스엠엘", ["Extensible Markup Language"]),
            ("YAML", "야믈", ["야멜"]),
            ("TOML", "토믈", []),
            ("Protocol Buffer", "프로토콜버퍼", ["프로토버프", "Protobuf"]),

            # 보안
            ("OAuth", "오어스", ["OAuth 2.0", "오어스2.0"]),
            ("JWT", "제이더블유티", ["JSON Web Token"]),
            ("SSL", "에스에스엘", ["Secure Sockets Layer"]),
            ("TLS", "티엘에스", ["Transport Layer Security"]),
            ("HTTPS", "에이치티티피에스", []),
            ("Encryption", "암호화", ["인크립션"]),
            ("Hashing", "해싱", []),

            # 버전 관리
            ("Semantic Versioning", "시맨틱버저닝", ["유의적버전"]),
            ("Commit", "커밋", []),
            ("Branch", "브랜치", []),
            ("Merge", "머지", ["병합"]),
            ("Rebase", "리베이스", []),
            ("Pull Request", "풀리퀘스트", ["풀 리퀘스트", "PR", "피알"]),

            # 패키지 관리자
            ("npm", "엔피엠", ["Node Package Manager"]),
            ("pip", "핍", []),
            ("Maven", "메이븐", []),
            ("Gradle", "그래들", []),
            ("Cargo", "카고", []),
            ("Composer", "컴포저", []),
            ("NuGet", "뉴겟", []),

            # 빌드 도구
            ("Webpack", "웹팩", []),
            ("Vite", "비트", []),
            ("Rollup", "롤업", []),
            ("Parcel", "파셀", []),
            ("esbuild", "이에스빌드", []),
            ("Babel", "바벨", []),
            ("TSC", "티에스씨", ["TypeScript Compiler"]),

            # 린터/포매터
            ("ESLint", "이에스린트", []),
            ("Prettier", "프리티어", []),
            ("Black", "블랙", []),
            ("Pylint", "파이린트", []),
            ("Clippy", "클리피", []),
            ("rustfmt", "러스트에프엠티", []),
        ],
    }

    def __init__(self, output_dir: Path):
        self.output_dir = Path(output_dir)
        self.terms: dict[str, set[ITTerm]] = defaultdict(set)
        self.romanizer = KoreanRomanization()

    def collect_seed_terms(self) -> None:
        """시드 데이터에서 용어 수집."""
        print("Collecting seed terms...")

        for category, terms in self.SEED_TERMS.items():
            for term_data in terms:
                if len(term_data) == 3:
                    surface, reading, variants = term_data
                else:
                    surface, reading = term_data
                    variants = []

                # 메인 용어 추가
                it_term = ITTerm(
                    surface=surface,
                    category=category,
                    reading=reading,
                    variants=variants,
                )
                self.terms[category].add(it_term)

                # 한글 용어가 있으면 추가
                if reading and reading != surface:
                    hangul_term = ITTerm(
                        surface=reading,
                        category=category,
                        reading=reading,
                        variants=[],
                    )
                    self.terms[category].add(hangul_term)

                # 변이형 추가
                for variant in variants:
                    if variant and variant != surface:
                        variant_term = ITTerm(
                            surface=variant,
                            category=category,
                            reading=reading,
                            variants=[],
                        )
                        self.terms[category].add(variant_term)

        # 통계 출력
        total = sum(len(terms) for terms in self.terms.values())
        print(f"Collected {total} seed terms across {len(self.terms)} categories")
        for cat, terms in self.terms.items():
            print(f"  {cat}: {len(terms)} terms")

    def generate_compound_terms(self) -> None:
        """복합 용어 생성 (예: Python 개발자, React 앱)."""
        print("\nGenerating compound terms...")

        # 일반적인 접미사/접두사
        suffixes = [
            ("개발자", "개발자"),
            ("프로그래머", "프로그래머"),
            ("엔지니어", "엔지니어"),
            ("개발", "개발"),
            ("프로그래밍", "프로그래밍"),
            ("애플리케이션", "애플리케이션"),
            ("앱", "앱"),
            ("서버", "서버"),
            ("클라이언트", "클라이언트"),
            ("라이브러리", "라이브러리"),
            ("프레임워크", "프레임워크"),
            ("API", "에이피아이"),
            ("SDK", "에스디케이"),
        ]

        compound_count = 0
        for category, terms in self.terms.items():
            # 프로그래밍 언어와 프레임워크에 대해 복합어 생성
            if category in ["programming_languages", "frameworks_libraries"]:
                for term in list(terms):  # 리스트로 변환하여 순회 중 수정 방지
                    for suffix, suffix_reading in suffixes[:6]:  # 상위 6개만 사용
                        compound_surface = f"{term.surface}{suffix}"
                        compound_reading = f"{term.reading}{suffix_reading}" if term.reading else ""

                        compound_term = ITTerm(
                            surface=compound_surface,
                            category=category,
                            reading=compound_reading,
                            variants=[],
                            pos="NNG",  # 일반 명사
                        )
                        if compound_term not in self.terms[category]:
                            self.terms[category].add(compound_term)
                            compound_count += 1

        print(f"Generated {compound_count} compound terms")

    def export_to_mecab_csv(self) -> dict[str, Path]:
        """MeCab CSV 포맷으로 내보내기."""
        print("\nExporting to MeCab CSV format...")

        exported_files = {}

        for category, terms in self.terms.items():
            output_file = self.output_dir / "it-terms" / f"{category}.csv"
            output_file.parent.mkdir(parents=True, exist_ok=True)

            # 표면형으로 정렬
            sorted_terms = sorted(terms, key=lambda t: t.surface)

            with output_file.open("w", encoding="utf-8") as f:
                for term in sorted_terms:
                    # MeCab CSV 포맷:
                    # 표면형,좌문맥ID,우문맥ID,비용,품사,품사세분류1,품사세분류2,품사세분류3,활용형,활용,원형,읽기,발음

                    # 간단한 포맷 (좌/우문맥ID는 mecab-dict-index가 생성)
                    # 표면형,0,0,비용,품사,*,*,*,*,*,*,읽기,읽기
                    reading = term.reading if term.reading else term.surface

                    line = (
                        f"{term.surface},"  # 표면형
                        f"0,0,"  # 좌문맥ID, 우문맥ID (나중에 생성)
                        f"{term.cost},"  # 비용
                        f"{term.pos},"  # 품사
                        f"*,*,*,*,*,"  # 품사세분류들
                        f"{term.surface},"  # 원형
                        f"{reading},"  # 읽기
                        f"{reading}"  # 발음
                    )
                    f.write(line + "\n")

            exported_files[category] = output_file
            print(f"  Exported {len(sorted_terms)} terms to {output_file}")

        return exported_files

    def generate_statistics(self) -> dict[str, Any]:
        """통계 생성."""
        stats = {
            "total_terms": sum(len(terms) for terms in self.terms.values()),
            "categories": {},
        }

        for category, terms in self.terms.items():
            stats["categories"][category] = {
                "count": len(terms),
                "has_reading": sum(1 for t in terms if t.reading),
                "has_variants": sum(1 for t in terms if t.variants),
            }

        return stats

    def save_statistics(self, stats: dict[str, Any]) -> None:
        """통계를 JSON 파일로 저장."""
        stats_file = self.output_dir / "statistics.json"
        with stats_file.open("w", encoding="utf-8") as f:
            json.dump(stats, f, ensure_ascii=False, indent=2)
        print(f"\nStatistics saved to {stats_file}")


def main() -> None:
    """메인 함수."""
    import argparse

    parser = argparse.ArgumentParser(
        description="IT/기술 용어를 수집하여 MeCab 사전 형식으로 변환"
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("/home/mare/mecab-ko/data/domain-dic"),
        help="출력 디렉토리 경로",
    )
    parser.add_argument(
        "--no-compounds",
        action="store_true",
        help="복합어 생성 비활성화",
    )

    args = parser.parse_args()

    collector = ITTermCollector(args.output_dir)

    # 1. 시드 용어 수집
    collector.collect_seed_terms()

    # 2. 복합어 생성
    if not args.no_compounds:
        collector.generate_compound_terms()

    # 3. MeCab CSV로 내보내기
    exported_files = collector.export_to_mecab_csv()

    # 4. 통계 생성 및 저장
    stats = collector.generate_statistics()
    collector.save_statistics(stats)

    # 5. 요약 출력
    print("\n" + "="*60)
    print("IT Term Collection Summary")
    print("="*60)
    print(f"Total terms: {stats['total_terms']}")
    print("\nBy category:")
    for cat, cat_stats in stats["categories"].items():
        print(f"  {cat}: {cat_stats['count']} terms")
    print("\nExported files:")
    for cat, file_path in exported_files.items():
        print(f"  {file_path}")
    print("="*60)


if __name__ == "__main__":
    main()
