# NIKL 모두의말뭉치 형태분석 (Modu) — Silver Dataset 설정

> Sprint 159 F: NIKL Modu corpus를 5번째 silver dataset으로 통합.

## 개요

NIKL Modu 형태분석 말뭉치는 국립국어원이 제공하는 한국어 형태소 분석 corpus입니다.
- **규모**: 371,571 sentences (training portion)
- **POS scheme**: Sejong-compatible (mecab-ko와 직접 호환)
- **도메인**: 신문, 웹, 구어, 문어 등 multi-domain (KLUE의 Airbnb/뉴스 도메인보다 광범위)
- **License**: **학술 사용 전용** — 재배포 금지

mecab-ko의 silver dataset 평가에 NIKL Modu을 추가하면:
- KLUE DP (Airbnb 후기 위주)
- UD Korean-Kaist (학술/뉴스)
- UD Korean-GSD (Google news/web)
- **+ NIKL Modu (구어/SNS/문어 multi-domain)** ← 신규
- sample.tsv (synthetic baseline)

→ 도메인 coverage 가장 광범위한 한국어 정확도 측정.

## 다운로드 방법

### 1. 학술 등록

1. https://kli.korean.go.kr 접속 (한국어만 지원)
2. 회원가입 (학술 기관 affiliation 필요)
3. 승인 대기 (보통 1-3일)

### 2. 데이터 다운로드

1. 로그인 후 '모두의말뭉치' → '형태분석 말뭉치' 선택
2. 다운로드 신청 (사용 목적 작성: "정확도 평가용")
3. 승인 후 JSON 파일 다운로드 (예: `NXMP1902008051.json`)

### 3. 파일 배치

다운로드한 JSON 파일을 임의 경로에 배치 (예시):
```bash
mkdir -p ~/Korpora/NIKL_MP/
mv ~/Downloads/NXMP1902008051.json ~/Korpora/NIKL_MP/
```

## 원샷 스크립트 (권장)

다운로드 완료 후 변환 + 평가를 한 번에:

```bash
./tools/nikl_modu_setup.sh ~/Korpora/NIKL_MP/NXMP1902008051.json
```

스크립트 동작:
1. JSON 파일 유효성 확인
2. `convert_nikl_modu.py` 실행 → `data/eval/nikl_modu_sample.tsv`
3. `cargo test test_nikl_modu_dual_metric` 실행 → 정확도 측정
4. 결과 요약 출력

옵션:
```bash
# 더 큰 sample
MAX_SENTENCES=10000 ./tools/nikl_modu_setup.sh ~/Korpora/NIKL_MP/NXMP*.json
```

## TSV 변환 (수동)

원샷 스크립트 대신 단계별 실행:

```bash
cd /path/to/mecab-ko

# 5000 sentences sample (기본)
python3 tools/convert_nikl_modu.py \
  ~/Korpora/NIKL_MP/NXMP1902008051.json \
  data/eval/nikl_modu_sample.tsv

# 더 큰 sample (10000 sentences)
python3 tools/convert_nikl_modu.py \
  ~/Korpora/NIKL_MP/NXMP1902008051.json \
  data/eval/nikl_modu_sample.tsv \
  --max-sentences 10000
```

스크립트가 처리하는 NIKL Modu JSON 구조:

```json
{
  "document": [{
    "sentence": [{
      "id": "...",
      "form": "원문 문장",
      "morpheme": [
        {"id": 1, "form": "표면형", "label": "NNG", "position": 0}
      ]
    }]
  }]
}
```

출력 TSV 형식 (다른 silver datasets와 동일):
```
text<TAB>surface1/POS1 surface2/POS2 ...<TAB>eojeol_counts
```

## 평가 실행

```bash
cd rust
cargo test --package mecab-ko-core --test accuracy_eval \
  -- test_nikl_modu_dual_metric --nocapture --ignored
```

출력 예시:
```
=== NIKL Modu (silver, multi-domain — 구어/SNS/문어) ===
Dataset: 5000 sentences

--- Strict ---
  Morpheme: 6X.X%
  Eojeol:   X.X% (XXXX / XXXXX)

--- Practical ---
  Morpheme: 7X.X% [Δ +X.Xpp vs strict]
  Eojeol:   X.X% (XXXX / XXXXX) [Δ +X.Xpp vs strict]
```

## CI 통합 상태

- **로컬 only**: license 제약으로 CI에는 포함되지 않음
- **5-gate CI**: 그대로 유지 (sample.tsv, KLUE morph, surface_only, UD Kaist, UD GSD)
- 6번째 gate (NIKL Modu) 추가는 향후 사용자 환경에서만 활성

`accuracy_eval.rs:test_nikl_modu_dual_metric`는 파일 미존재 시 자동 skip됩니다.

## License 안내

- NIKL Modu corpus는 **학술 사용 전용**
- 변환된 TSV (`data/eval/nikl_modu_sample.tsv`)도 **저장소에 commit 금지**
- `.gitignore`에 `data/eval/nikl_modu_*.tsv` 추가 권장:
  ```bash
  echo "data/eval/nikl_modu_*.tsv" >> .gitignore
  ```

## 트러블슈팅

### Q: kli.korean.go.kr 회원가입이 안 됨

- 학술 기관 affiliation 정보가 정확한지 확인 (이메일/소속)
- 관리자 승인은 보통 1-3일 소요
- 승인 안 되면 corpus.korean.go.kr (구버전) 시도

### Q: 변환 시 "Unknown tags" 경고가 많이 뜸

- NIKL Modu의 최신 버전은 SE/SO/SW 같은 확장 태그 사용
- `convert_nikl_modu.py`의 `KNOWN_TAGS`를 필요 시 추가
- 경고만 출력되고 변환은 정상 진행됨

### Q: TSV 변환 후 cargo test가 dataset not found 경고

- 파일 경로 확인: `data/eval/nikl_modu_sample.tsv` 정확히 일치해야 함
- 권한: `chmod 644 data/eval/nikl_modu_sample.tsv`
- 5-gate test는 자동 skip (없으면 PASS)

### Q: Korpora 라이브러리로 다운로드 가능?

- 가능하지만 동일하게 사용자 인증 필요
- `pip install korpora` → `Korpora.fetch('modu_mp')` (인증 토큰 필요)
- 수동 다운로드와 동일한 절차

### Q: license 위반 우려

- TSV 파일은 `.gitignore`로 보호 (Sprint 159 F)
- 평가 결과 (정확도 수치)만 PROGRESS.md 등에 commit OK
- 원본 JSON 또는 변환된 TSV commit 금지

## 참고 자료

- NIKL 모두의말뭉치 포털: https://kli.korean.go.kr
- Korpora docs (한국어 corpus 라이브러리): https://ko-nlp.github.io/Korpora/
- Sprint 159 F 연구 문서: `docs/research/accuracy/2026-05-21_sprint159_nikl_modu_infrastructure.md`

---

*작성: 2026-05-21 (Sprint 159 F)*
