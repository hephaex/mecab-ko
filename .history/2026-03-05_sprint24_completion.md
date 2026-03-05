# Sprint 24 완료 세션 로그

## 날짜: 2026-03-05

## 세션 개요
Sprint 24 (v0.4.0 릴리스 & 품질 개선)의 주요 작업들을 완료하고 스프린트를 마무리함.

## 완료한 작업

### S24-04: 평가 데이터셋 확장 ✅
- 160 → **300문장** 확장
- 추가 카테고리:
  - 일상 대화 (30): 오늘 점심 뭐 먹을까, 피곤해서 일찍 자야겠다
  - 뉴스/공식 문체 (30): 정부는 새로운 정책을 발표했다
  - 기술/전문 용어 (25): 인공지능이 데이터를 분석한다
  - 신조어/인터넷 용어 (25): 요즘 틱톡이 대세야, 갓생 살고 있어
  - 불규칙 활용 (20): 걷다 걸어 걸으면, 덥다 더워 더우면
  - 복합 구조 (10): 다중 절 문장
- 커밋: 4c1b9b7

### S24-03 보완: CI 기준선 업데이트 ✅
- 세종 모드 기준선: 23.8% (300문장)
- dict-build.yml 기준선 업데이트: 29.6% → 23.8%
- 커밋: 7e58ddd

### S24-06: 문서 사이트 v0.4.0 업데이트 ✅
- docs/book/src/changelog.md: v0.4.0 릴리스 노트 추가
- docs/book/src/introduction.md: 버전 정보, 크레이트 테이블 업데이트
- docs/book/src/installation.md: cargo add 명령어 업데이트
- 커밋: de04f4e

### S24-07: PyPI 배포 준비 ✅
- pyproject.toml 버전 0.4.0 업데이트
- mecab-ko-python 빌드 확인 (cargo check 통과)
- maturin 빌드 시스템 준비 완료
- 커밋: 792c8a7
- **Note**: 실제 배포는 PyPI 토큰 확보 후 진행

### S24-08: 커뮤니티 기능 요청 검토 ✅
- Issue #6 확인: 프로젝트 방향성 질문 (이미 답변됨)
- 현재 열린 이슈: 1개 (question, answered 라벨)

## Sprint 24 최종 현황

| 작업 | 상태 |
|------|------|
| S24-01: crates.io 배포 | ✅ 완료 (6개 크레이트) |
| S24-02: 품질 분석 | ✅ 완료 |
| S24-03: CI 자동 정확도 | ✅ 완료 |
| S24-04: 평가 데이터셋 확장 | ✅ 완료 (300문장) |
| S24-05: 비용값 튜닝 | → Sprint 25 이월 |
| S24-06: 문서 사이트 업데이트 | ✅ 완료 |
| S24-07: PyPI 배포 준비 | ✅ 완료 |
| S24-08: 커뮤니티 검토 | ✅ 완료 |

**완료율: 7/8 (87.5%)**

## 정확도 측정 결과 (300문장, 세종 모드)

```
Token Accuracy: 23.8%
Sentence Accuracy: 7.3%
F1 Score: 0.229

품사별 정확도:
  NNG (357개): 37.3%
  EF (243개): 22.2%
  VA (87개): 40.2%
  VV (226개): 17.3%
  EC (117개): 14.5%
  JKB (43개): 32.6%
  NP (26개): 34.6%
  JKO (52개): 26.9%
  EP (80개): 20.0%
  MAG (31개): 19.4%
  NNB (23개): 21.7%
  VX (35개): 11.4%
  JKS (52개): 7.7%
  XSV (64개): 4.7%
  ETM (31개): 0.0%
```

## Git 커밋 이력 (이 세션)

1. `4c1b9b7` - feat(eval): Expand evaluation dataset from 160 to 300 sentences (S24-04)
2. `46a89d7` - docs: Mark S24-04 evaluation dataset expansion as complete
3. `9d20736` - docs: Update error analysis with 300-sentence dataset metrics
4. `7e58ddd` - ci: Update accuracy baseline to 23.8% for 300-sentence dataset
5. `de04f4e` - docs(book): Update documentation site for v0.4.0 release (S24-06)
6. `11445bd` - docs: Mark S24-06 documentation update as complete
7. `792c8a7` - feat(python): Update pyproject.toml version to 0.4.0 (S24-07)
8. `bd53a3d` - docs: Mark S24-07 PyPI deployment preparation as complete

## 기술적 세부사항

### 평가 데이터셋 구조
```
data/eval/sample.tsv
- 형식: 원문\t정답 (surface/pos 쌍)
- 총 300문장
- 카테고리별 균형 있는 분포
```

### 세종 모드 평가 명령어
```bash
cargo run --release -p mecab-ko-cli -- evaluate \
  -i data/eval/sample.tsv \
  -d data/dict-output \
  --sejong
```

### CI 정확도 측정 (dict-build.yml)
- accuracy-test job 추가
- 기준선: 23.8% (300문장, 세종 모드)
- 회귀 탐지: 기준선 이하 시 경고
- 아티팩트: accuracy-history.json (90일 보관)

## 다음 스프린트 (Sprint 25) 예정

1. S24-05 이월: mecab-ko-dic 비용값 튜닝 (사전 레벨 수정 필요)
2. 정확도 추가 개선 (목표: 30%+)
3. PyPI 실제 배포 (토큰 확보 후)
4. npm WASM 패키지 v0.4.0 업데이트
5. 신조어 자동 수집 파이프라인 구축

## 학습 포인트

1. **데이터셋 확장의 중요성**: 160 → 300문장으로 확장하니 더 현실적인 정확도 측정 가능
2. **세종 모드 필수**: 후처리 없이는 품사 매칭이 어려움
3. **사전 한계**: 후처리로는 VV/EC/XSV 정확도 개선에 한계가 있음
4. **CI 자동화**: 정확도 회귀 방지를 위한 자동 측정 필수
