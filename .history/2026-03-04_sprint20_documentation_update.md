# Sprint 20 Session Log: Documentation Update (2026-03-04)

## 세션 개요
Sprint 20 S20-08 작업: 문서 사이트 업데이트 완료

## 완료된 작업

### S20-08: 문서 사이트 업데이트 ✅

#### introduction.md 업데이트
- 버전: v0.3.0 → v0.3.1
- 새 기능 추가:
  - 세종 코퍼스 호환 모드 (SejongConverter)
  - 복합 태그 분리 (VV+EF → VV, EF)
  - CLI `--sejong` 옵션
- 크레이트 버전 테이블 v0.3.1로 업데이트
- v0.3.1 주요 변경사항 섹션 추가

#### changelog.md 업데이트
- v0.3.1 섹션 추가:
  - 세종 코퍼스 호환 모드 상세 설명
  - EndingRule: VV+EF, VA+EF, EC, ETM 지원
  - CLI `evaluate --sejong` 옵션
  - 정확도 측정 결과 테이블
- v0.3.0 섹션 추가 (K-best, 분석 모드, 시각화, 캐싱, 스트리밍)
- 로드맵 업데이트:
  - v0.4.0: mecab-ko-dic v3.0, 정확도 50%+, 실시간 어미 분리
  - v0.5.0: 정확도 70%+, OpenSearch
  - v1.0.0: 정확도 90%+, PyPI 배포

#### cli-usage.md 업데이트
- `evaluate` 서브커맨드 문서화:
  - `--input`, `--dicdir`, `--verbose` 옵션
  - `--sejong` 옵션 (세종 호환 모드)
  - TSV 테스트 데이터 형식 설명
  - 출력 지표 설명 (Token/Sentence/POS Accuracy, F1)
- 버전 예시 0.1.0 → 0.3.1로 업데이트

## 파일 변경

### 수정
- `docs/book/src/introduction.md`
- `docs/book/src/changelog.md`
- `docs/book/src/cli-usage.md`
- `PROGRESS.md`

## 커밋

```
9f50f93 docs(book): update documentation site for v0.3.1 (S20-08)
```

## Sprint 20 최종 상태

### 완료
- S20-02: 세종 코퍼스 호환 모드 ✅
- S20-03: mecab-ko-dic v3.0 현대화 계획 ✅
- S20-05: v0.3.1 릴리스 ✅
- S20-06: 정확도 개선 측정 ✅
- S20-08: 문서 사이트 업데이트 ✅

### BLOCKED
- S20-01: PyPI 배포 (토큰 필요)
- S20-04: 신조어 자동 수집 실행 (OPENDICT_API_KEY 필요)

### 미완료
- S20-07: 커뮤니티 피드백 통합 (P3, 낮은 우선순위)

## 학습 포인트

1. **문서 버전 관리**: 버전 업데이트 시 introduction, changelog, cli-usage 모두 동기화 필요
2. **세종 호환 모드 문서화**: CLI 옵션과 함께 사용 예시 제공이 중요
3. **로드맵 현실화**: 정확도 목표를 50% → 70% → 90%로 단계적 설정

## 다음 스프린트 제안

### Sprint 21 목표
1. **P0**: mecab-ko-dic v3.0 Phase 1 시작
   - 국립국어원 API 연동 테스트
   - 신조어 수집 파일럿
2. **P1**: 실시간 어미 분리 알고리즘 연구
   - 한국어 용언 활용 패턴 분석
   - Sejong 모듈 확장
3. **P2**: GitHub Actions 시크릿 문서화
   - OPENDICT_API_KEY 설정 가이드
   - PYPI_TOKEN 설정 가이드
