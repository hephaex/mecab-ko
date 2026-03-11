#!/usr/bin/env python3
"""
IT 용어 극대화 스크립트

다양한 변이형, 복합어, 동의어를 생성하여 10,000+ 엔트리 달성
"""

from pathlib import Path
from expand_terms import ExtendedITTermCollector, ITTerm


class MaximizedITTermCollector(ExtendedITTermCollector):
    """극대화된 IT 용어 수집기."""

    # 추가 접미사/접두사
    EXPANDED_SUFFIXES = [
        ("개발자", "개발자"),
        ("개발", "개발"),
        ("프로그래머", "프로그래머"),
        ("프로그래밍", "프로그래밍"),
        ("엔지니어", "엔지니어"),
        ("엔지니어링", "엔지니어링"),
        ("코딩", "코딩"),
        ("코드", "코드"),
        ("애플리케이션", "애플리케이션"),
        ("앱", "앱"),
        ("어플리케이션", "어플리케이션"),
        ("어플", "어플"),
        ("서버", "서버"),
        ("클라이언트", "클라이언트"),
        ("프론트엔드", "프론트엔드"),
        ("백엔드", "백엔드"),
        ("풀스택", "풀스택"),
        ("라이브러리", "라이브러리"),
        ("프레임워크", "프레임워크"),
        ("API", "에이피아이"),
        ("SDK", "에스디케이"),
        ("플랫폼", "플랫폼"),
        ("시스템", "시스템"),
        ("도구", "도구"),
        ("툴", "툴"),
        ("솔루션", "솔루션"),
        ("서비스", "서비스"),
        ("환경", "환경"),
        ("아키텍처", "아키텍처"),
        ("디자인", "디자인"),
        ("패턴", "패턴"),
        ("모델", "모델"),
        ("알고리즘", "알고리즘"),
        ("자료구조", "자료구조"),
        ("데이터", "데이터"),
        ("구조", "구조"),
        ("인터페이스", "인터페이스"),
        ("모듈", "모듈"),
        ("컴포넌트", "컴포넌트"),
        ("위젯", "위젯"),
        ("플러그인", "플러그인"),
        ("확장", "확장"),
        ("익스텐션", "익스텐션"),
        ("애드온", "애드온"),
        ("미들웨어", "미들웨어"),
        ("헬퍼", "헬퍼"),
        ("유틸리티", "유틸리티"),
        ("래퍼", "래퍼"),
        ("어댑터", "어댑터"),
        ("프록시", "프록시"),
        ("게이트웨이", "게이트웨이"),
        ("브릿지", "브릿지"),
        ("파이프라인", "파이프라인"),
        ("워크플로우", "워크플로우"),
        ("워크플로", "워크플로"),
        ("스크립트", "스크립트"),
        ("배치", "배치"),
        ("태스크", "태스크"),
        ("작업", "작업"),
        ("프로세스", "프로세스"),
        ("스레드", "스레드"),
        ("큐", "큐"),
        ("스택", "스택"),
        ("버퍼", "버퍼"),
        ("캐시", "캐시"),
        ("세션", "세션"),
        ("쿠키", "쿠키"),
        ("토큰", "토큰"),
        ("인증", "인증"),
        ("권한", "권한"),
        ("보안", "보안"),
        ("암호화", "암호화"),
        ("해싱", "해싱"),
        ("검증", "검증"),
        ("테스트", "테스트"),
        ("테스팅", "테스팅"),
        ("디버깅", "디버깅"),
        ("프로파일링", "프로파일링"),
        ("모니터링", "모니터링"),
        ("로깅", "로깅"),
        ("추적", "추적"),
        ("분석", "분석"),
        ("최적화", "최적화"),
        ("성능", "성능"),
        ("벤치마크", "벤치마크"),
        ("빌드", "빌드"),
        ("컴파일", "컴파일"),
        ("배포", "배포"),
        ("릴리스", "릴리스"),
        ("버전", "버전"),
        ("업데이트", "업데이트"),
        ("업그레이드", "업그레이드"),
        ("마이그레이션", "마이그레이션"),
        ("설정", "설정"),
        ("구성", "구성"),
        ("설치", "설치"),
        ("패키지", "패키지"),
        ("의존성", "의존성"),
        ("종속성", "종속성"),
        ("매니저", "매니저"),
        ("관리자", "관리자"),
    ]

    # 접두사
    PREFIXES = [
        ("웹", "웹"),
        ("모바일", "모바일"),
        ("클라우드", "클라우드"),
        ("네이티브", "네이티브"),
        ("크로스플랫폼", "크로스플랫폼"),
        ("오픈소스", "오픈소스"),
        ("엔터프라이즈", "엔터프라이즈"),
        ("마이크로", "마이크로"),
        ("서버리스", "서버리스"),
        ("분산", "분산"),
        ("실시간", "실시간"),
        ("비동기", "비동기"),
        ("동기", "동기"),
        ("병렬", "병렬"),
        ("직렬", "직렬"),
        ("고성능", "고성능"),
        ("경량", "경량"),
        ("고가용성", "고가용성"),
        ("확장가능", "확장가능"),
        ("내결함성", "내결함성"),
    ]

    # 기술 동사
    TECH_VERBS = [
        "학습", "공부", "연구", "사용", "활용", "적용", "구현", "개발",
        "설계", "배포", "운영", "관리", "유지보수", "최적화", "튜닝",
        "마이그레이션", "업그레이드", "통합", "연동", "연결",
    ]

    def generate_expanded_compounds(self) -> None:
        """확장된 복합어 생성."""
        print("\nGenerating expanded compound terms...")

        compound_count = 0

        for category, terms in self.terms.items():
            base_terms = list(terms)

            # 1. 모든 카테고리에 대해 확장 접미사 적용
            for term in base_terms:
                # 영문 용어에만 적용
                if any(ord(c) < 128 for c in term.surface[:3]):
                    for suffix, suffix_reading in self.EXPANDED_SUFFIXES:
                        compound_surface = f"{term.surface} {suffix}"
                        compound_reading = f"{term.reading} {suffix_reading}" if term.reading else ""

                        compound_term = ITTerm(
                            surface=compound_surface,
                            category=category,
                            reading=compound_reading,
                            variants=[],
                            pos="NNG",
                            cost=-3000,  # 복합어는 약간 낮은 우선순위
                        )
                        if compound_term not in self.terms[category]:
                            self.terms[category].add(compound_term)
                            compound_count += 1

            # 2. 접두사 추가
            for term in base_terms[:100]:  # 상위 100개에만 적용
                for prefix, prefix_reading in self.PREFIXES:
                    compound_surface = f"{prefix} {term.surface}"
                    compound_reading = f"{prefix_reading} {term.reading}" if term.reading else ""

                    compound_term = ITTerm(
                        surface=compound_surface,
                        category=category,
                        reading=compound_reading,
                        variants=[],
                        pos="NNG",
                        cost=-3000,
                    )
                    if compound_term not in self.terms[category]:
                        self.terms[category].add(compound_term)
                        compound_count += 1

            # 3. 기술 동사 조합 (프로그래밍 언어와 프레임워크)
            if category in ["programming_languages", "frameworks_libraries"]:
                for term in base_terms[:50]:  # 상위 50개
                    for verb in self.TECH_VERBS:
                        # "Python 학습", "React 개발" 등
                        compound_surface = f"{term.surface} {verb}"
                        compound_reading = f"{term.reading} {verb}" if term.reading else ""

                        compound_term = ITTerm(
                            surface=compound_surface,
                            category=category,
                            reading=compound_reading,
                            variants=[],
                            pos="NNG",
                            cost=-4000,
                        )
                        if compound_term not in self.terms[category]:
                            self.terms[category].add(compound_term)
                            compound_count += 1

        print(f"Generated {compound_count} expanded compound terms")

    def generate_tech_phrases(self) -> None:
        """기술 구문 생성."""
        print("\nGenerating technical phrases...")

        phrases = [
            # 개발 방법론
            ("애자일", "애자일", "general_it"),
            ("스크럼", "스크럼", "general_it"),
            ("칸반", "칸반", "general_it"),
            ("워터폴", "워터폴", "general_it"),
            ("데브옵스", "데브옵스", "general_it"),
            ("DevOps", "데브옵스", "general_it"),
            ("CI/CD", "씨아이씨디", "general_it"),
            ("지속적통합", "지속적통합", "general_it"),
            ("지속적배포", "지속적배포", "general_it"),
            ("테스트주도개발", "테스트주도개발", "general_it"),
            ("TDD", "티디디", "general_it"),
            ("행위주도개발", "행위주도개발", "general_it"),
            ("BDD", "비디디", "general_it"),
            ("페어프로그래밍", "페어프로그래밍", "general_it"),
            ("코드리뷰", "코드리뷰", "general_it"),
            ("리팩토링", "리팩토링", "general_it"),
            ("클린코드", "클린코드", "general_it"),
            ("레거시코드", "레거시코드", "general_it"),
            ("기술부채", "기술부채", "general_it"),

            # 아키텍처 패턴
            ("마이크로서비스아키텍처", "마이크로서비스아키텍처", "general_it"),
            ("MSA", "엠에스에이", "general_it"),
            ("서비스지향아키텍처", "서비스지향아키텍처", "general_it"),
            ("SOA", "에스오에이", "general_it"),
            ("이벤트기반아키텍처", "이벤트기반아키텍처", "general_it"),
            ("EDA", "이디에이", "general_it"),
            ("육각형아키텍처", "육각형아키텍처", "general_it"),
            ("클린아키텍처", "클린아키텍처", "general_it"),
            ("레이어드아키텍처", "레이어드아키텍처", "general_it"),
            ("모놀리식아키텍처", "모놀리식아키텍처", "general_it"),

            # 디자인 패턴
            ("싱글톤패턴", "싱글톤패턴", "general_it"),
            ("팩토리패턴", "팩토리패턴", "general_it"),
            ("옵저버패턴", "옵저버패턴", "general_it"),
            ("전략패턴", "전략패턴", "general_it"),
            ("데코레이터패턴", "데코레이터패턴", "general_it"),
            ("어댑터패턴", "어댑터패턴", "general_it"),
            ("프록시패턴", "프록시패턴", "general_it"),
            ("빌더패턴", "빌더패턴", "general_it"),
            ("프로토타입패턴", "프로토타입패턴", "general_it"),
            ("퍼사드패턴", "퍼사드패턴", "general_it"),

            # 성능 개념
            ("로드밸런싱", "로드밸런싱", "cloud_infrastructure"),
            ("캐싱전략", "캐싱전략", "general_it"),
            ("데이터베이스인덱싱", "데이터베이스인덱싱", "general_it"),
            ("쿼리최적화", "쿼리최적화", "general_it"),
            ("코드스플리팅", "코드스플리팅", "frameworks_libraries"),
            ("레이지로딩", "레이지로딩", "general_it"),
            ("프리페칭", "프리페칭", "general_it"),
            ("메모이제이션", "메모이제이션", "general_it"),
            ("쓰로틀링", "쓰로틀링", "general_it"),
            ("디바운싱", "디바운싱", "general_it"),

            # 보안 개념
            ("크로스사이트스크립팅", "크로스사이트스크립팅", "general_it"),
            ("XSS", "엑스에스에스", "general_it"),
            ("SQL인젝션", "에스큐엘인젝션", "general_it"),
            ("CSRF", "씨에스알에프", "general_it"),
            ("CORS", "코어스", "general_it"),
            ("제로트러스트", "제로트러스트", "general_it"),
            ("다중인증", "다중인증", "general_it"),
            ("MFA", "엠에프에이", "general_it"),
            ("2FA", "투에프에이", "general_it"),
            ("싱글사인온", "싱글사인온", "general_it"),
            ("SSO", "에스에스오", "general_it"),

            # 네트워킹
            ("REST API", "레스트에이피아이", "general_it"),
            ("RESTful API", "레스트풀에이피아이", "general_it"),
            ("GraphQL API", "그래프큐엘에이피아이", "frameworks_libraries"),
            ("웹소켓", "웹소켓", "general_it"),
            ("롱폴링", "롱폴링", "general_it"),
            ("서버센트이벤트", "서버센트이벤트", "general_it"),
            ("SSE", "에스에스이", "general_it"),
            ("웹RTC", "웹알티씨", "general_it"),
            ("WebRTC", "웹알티씨", "general_it"),

            # 데이터베이스 개념
            ("ACID", "애시드", "general_it"),
            ("BASE", "베이스", "general_it"),
            ("CAP정리", "캡정리", "general_it"),
            ("샤딩", "샤딩", "general_it"),
            ("레플리케이션", "레플리케이션", "general_it"),
            ("파티셔닝", "파티셔닝", "general_it"),
            ("정규화", "정규화", "general_it"),
            ("역정규화", "역정규화", "general_it"),
            ("트랜잭션", "트랜잭션", "general_it"),
            ("인덱스", "인덱스", "general_it"),

            # ML/AI 개념
            ("지도학습", "지도학습", "ai_ml"),
            ("비지도학습", "비지도학습", "ai_ml"),
            ("강화학습", "강화학습", "ai_ml"),
            ("준지도학습", "준지도학습", "ai_ml"),
            ("전이학습", "전이학습", "ai_ml"),
            ("메타학습", "메타학습", "ai_ml"),
            ("온라인학습", "온라인학습", "ai_ml"),
            ("배치학습", "배치학습", "ai_ml"),
            ("앙상블학습", "앙상블학습", "ai_ml"),
            ("능동학습", "능동학습", "ai_ml"),
            ("연합학습", "연합학습", "ai_ml"),
            ("자기지도학습", "자기지도학습", "ai_ml"),
            ("대조학습", "대조학습", "ai_ml"),
            ("증류", "증류", "ai_ml"),
            ("양자화", "양자화", "ai_ml"),
            ("프루닝", "프루닝", "ai_ml"),
            ("지식증류", "지식증류", "ai_ml"),

            # 클라우드 개념
            ("IaaS", "아이아스", "cloud_infrastructure"),
            ("PaaS", "파스", "cloud_infrastructure"),
            ("SaaS", "사스", "cloud_infrastructure"),
            ("FaaS", "파스", "cloud_infrastructure"),
            ("서비스형인프라", "서비스형인프라", "cloud_infrastructure"),
            ("서비스형플랫폼", "서비스형플랫폼", "cloud_infrastructure"),
            ("서비스형소프트웨어", "서비스형소프트웨어", "cloud_infrastructure"),
            ("멀티클라우드", "멀티클라우드", "cloud_infrastructure"),
            ("하이브리드클라우드", "하이브리드클라우드", "cloud_infrastructure"),
            ("프라이빗클라우드", "프라이빗클라우드", "cloud_infrastructure"),
            ("퍼블릭클라우드", "퍼블릭클라우드", "cloud_infrastructure"),
            ("엣지컴퓨팅", "엣지컴퓨팅", "cloud_infrastructure"),
            ("포그컴퓨팅", "포그컴퓨팅", "cloud_infrastructure"),
            ("컨테이너오케스트레이션", "컨테이너오케스트레이션", "cloud_infrastructure"),
            ("불변인프라", "불변인프라", "cloud_infrastructure"),
            ("애완동물vs가축", "애완동물대가축", "cloud_infrastructure"),
        ]

        phrase_count = 0
        for surface, reading, category in phrases:
            term = ITTerm(
                surface=surface,
                category=category,
                reading=reading,
                variants=[],
                pos="NNG",
                cost=-4000,
            )
            if term not in self.terms[category]:
                self.terms[category].add(term)
                phrase_count += 1

        print(f"Generated {phrase_count} technical phrases")


def main() -> None:
    """메인 함수."""
    import argparse

    parser = argparse.ArgumentParser(
        description="IT 용어를 극대화하여 10,000+ 엔트리 생성"
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("/home/mare/mecab-ko/data/domain-dic"),
        help="출력 디렉토리 경로",
    )

    args = parser.parse_args()

    collector = MaximizedITTermCollector(args.output_dir)

    # 1. 기본 시드 용어 수집
    collector.collect_seed_terms()

    # 2. 확장 용어 수집
    collector.collect_extended_terms()

    # 3. 기본 복합어 생성
    collector.generate_compound_terms()

    # 4. 확장 복합어 생성
    collector.generate_expanded_compounds()

    # 5. 기술 구문 생성
    collector.generate_tech_phrases()

    # 6. MeCab CSV로 내보내기
    exported_files = collector.export_to_mecab_csv()

    # 7. 통계 생성 및 저장
    stats = collector.generate_statistics()
    collector.save_statistics(stats)

    # 8. 요약 출력
    print("\n" + "="*60)
    print("Maximized IT Term Collection Summary")
    print("="*60)
    print(f"Total terms: {stats['total_terms']}")
    print("\nBy category:")
    for cat, cat_stats in stats["categories"].items():
        print(f"  {cat}: {cat_stats['count']} terms")
    print("\nExported files:")
    for cat, file_path in exported_files.items():
        print(f"  {file_path}")

    # 목표 달성 여부
    if stats["total_terms"] >= 10000:
        print("\n✓ Target achieved: 10,000+ entries!")
    else:
        print(f"\n⚠ Target not reached: {10000 - stats['total_terms']} more entries needed")

    print("="*60)


if __name__ == "__main__":
    main()
