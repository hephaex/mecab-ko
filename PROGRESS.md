# PROGRESS — mecab-ko Sprint 163 (NIKL Modu 인프라 보강)

> 마지막 업데이트: 2026-05-21

## Sprint 163 — 다운로드 가이드 보강 + 원샷 스크립트

| Task | 상태 | 결과 |
|------|------|------|
| S163-1: NIKL Modu 다운로드 상태 확인 | ✅ 완료 | 여전히 미다운로드 |
| S163-2: 원샷 스크립트 작성 | ✅ 완료 | `tools/nikl_modu_setup.sh` |
| S163-3: 트러블슈팅 섹션 추가 | ✅ 완료 | `docs/eval/nikl_modu_setup.md` |
| S163-4: 권한 설정 | ✅ 완료 | chmod +x |

## 변경 내용

### 1. 원샷 스크립트 (`tools/nikl_modu_setup.sh`)

다운로드 완료 후 원-커맨드로 변환 + 평가:

```bash
./tools/nikl_modu_setup.sh ~/Korpora/NIKL_MP/NXMP1902008051.json
```

동작:
1. JSON 파일 유효성 확인 (size check)
2. `convert_nikl_modu.py` 실행 → TSV 생성
3. `cargo test test_nikl_modu_dual_metric` 실행 → 정확도 측정
4. 결과 요약 출력

옵션:
```bash
MAX_SENTENCES=10000 ./tools/nikl_modu_setup.sh <json>
```

### 2. 트러블슈팅 섹션 (`docs/eval/nikl_modu_setup.md`)

추가 FAQ:
- 회원가입 안 됨 → affiliation 확인, 1-3일 대기
- Unknown tags 경고 → 정상 (확장 태그)
- TSV 변환 후 dataset not found → 경로/권한 확인
- Korpora 라이브러리 사용 → 동일하게 인증 필요
- license 위반 우려 → .gitignore 보호 확인

## NIKL Modu 다운로드 상태

```
$ ls data/eval/nikl_modu_*.tsv
MISSING

$ ls ~/Korpora/NIKL_MP/
(없음)
```

여전히 사용자 수동 다운로드 필요. 자동 다운로드 불가능 (학술 인증).

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 변경 없음 (411 pass)
- 5-gate sample.tsv: 영향 없음
- 원샷 스크립트: chmod +x 적용

## 변경 파일

- `tools/nikl_modu_setup.sh` (신규, 실행 가능)
- `docs/eval/nikl_modu_setup.md`: 원샷 + 트러블슈팅 섹션 추가
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 164 — 사용자 NIKL Modu 다운로드 대기

다운로드 완료 시:
1. 사용자가 `./tools/nikl_modu_setup.sh <json>` 실행
2. 자동 변환 + 평가
3. 결과 PROGRESS.md에 기록
4. POS mismatch 분석 → 추가 동치/normalize 후보 발굴 (Sprint 164)

다운로드 보류 시:
- 다른 영역 전환 또는 유지보수 모드 (사용자 결정 필요)
