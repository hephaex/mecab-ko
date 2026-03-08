# Session Log: Sprint 30 - npm v0.4.0 배포

## 날짜: 2026-03-08

## 세션 개요
Sprint 30 진행 중 npm mecab-ko-wasm v0.4.0 배포 완료 및 정확도 개선 한계 도달 확인

## 주요 작업

### 1. npm mecab-ko-wasm v0.4.0 배포 ✅
- **이전 버전**: 0.3.0 (2026-03-02)
- **새 버전**: 0.4.0 (2026-03-08)
- **URL**: https://www.npmjs.com/package/mecab-ko-wasm

**배포 과정**:
1. npm 토큰 만료 임박 확인 (3/10 만료)
2. 새 토큰 생성 (`github-actions-publish-v3`) with Bypass 2FA
3. wasm-pack build 실행
4. npm publish 성공

**GitHub Secret 업데이트**:
- `NPM_TOKEN` 값을 새 토큰으로 교체 완료

### 2. 정확도 개선 시도 및 한계 확인
- **현재 정확도**: Token 52.9%, Sentence 36.0%
- **목표**: 55%

**시도한 접근**:
- 사용자 사전에 숫자(일/SN, 이/SN) 추가 → 실패
- 감탄사(아/IC, 예/IC, 네/IC) 추가 → 실패
- 결과: 52.9% → 37.4%로 급락 (동음이의어 충돌)
- 롤백하여 52.9% 복구

**결론**: MeCab 사전 자체 수정 없이는 추가 개선 불가

### 3. PyPI 계정 복구
- GitHub Issue: https://github.com/pypi/support/issues/9540
- support@pypi.org 이메일 발송 완료
- 응답 대기 중

### 4. GitHub Contributors 문제
- "claude Claude"가 Contributors에 표시되는 문제 발견
- API에서는 보이지 않음 (UI만 표시)
- GitHub Support 문의 필요

## 기술적 세부사항

### 적용된 보정 (40차까지)
```
18차: NP + "의X/NNG" → "의/JKG + X/NNG" 분리
19차: "X/NNG + 님의/NNP" → "X님/NNG + 의/JKG" 병합
20차: MAJ → MAG (또한, 따라서, 그러므로)
21차: VCP 삽입 - NNG + "이/EP" → "이/VCP"
22차: 시간표현 분리 - "열시/NNG" → "열/NR + 시/NNB"
...
36차: 문장 끝 "아요/EC" → "어요/EF"
37차: 문장 중간 "고/EF" → "고/EC"
38차: NNG + "하고/JC" + VX → "하/XSV + 고/EC" 분리
39차: NNG + "하/IC" + "면서/EF" → "하/XSV + 면서/EC"
40차: 동사형 관형사 분리 ("오는/MM" → "오/VV + 는/ETM")
```

### MeCab 사전 한계 (블로커)
- 숫자 인식: "일 이 삼" → "일이/MM 삼/EF" (오류)
- 감탄사 인식: "아 예 네" → "아예/EF 네/EF" (오류)
- 복합명사: 영화, 회의, 발표 등 미인식

## 파일 변경 내역
- `PROGRESS.md` - Sprint 30 상태 업데이트
- `rust/crates/benchmarks/accuracy_history.json` - v0.5.9 추가
- `data/user-dict/verb-inflections.csv` - 숫자/감탄사 추가 시도 후 롤백

## Git 커밋
- `d9732d7` - docs: Update Sprint 30 progress - npm v0.4.0 deployed

## 다음 단계
1. PyPI 계정 복구 완료 대기
2. GitHub Support 문의 (claude 기여자 제거)
3. MeCab 사전 개선 방안 검토 (Sprint 31)

## 학습 포인트
1. npm Granular Access Token은 Bypass 2FA 옵션이 필요함
2. 사용자 사전으로 단일 품사 강제 할당 시 동음이의어 충돌 발생
3. 코드 레벨 후처리의 한계 - 사전 자체 개선이 근본 해결책
