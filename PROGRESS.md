# PROGRESS — mecab-ko Sprint 165 (Track B Step 2: 학습 데이터 준비)

> 마지막 업데이트: 2026-05-27

## Sprint 165 — Track B Step 2: 학습 데이터 준비

| Task | 상태 | 결과 |
|------|------|------|
| S165-B1: raw data 인벤토리 확인 | ✅ 완료 | train split 부재 → dev만 사용 |
| S165-B2: tools/to_mecab_tagged.py 작성 | ✅ 완료 | CoNLL-U → .tagged 변환 |
| S165-B3: UD Kaist dev 변환 | ✅ 완료 | 2066 sentences |
| S165-B4: UD GSD dev 변환 | ✅ 완료 | 950 sentences |
| S165-B5: 통합 corpus 생성 | ✅ 완료 | corpus_dev.tagged (3016 sent, ~73K morph) |
| S165-B6: .gitignore 보호 | ✅ 완료 | data/train/ |
| S165-B7: seed dictdir 결정 | ✅ 완료 | mecab-ko-dic 2.1.1 그대로 사용 |

## 핵심 발견

### Train data 부재 → dev split 활용

| Dataset | Train | Dev | Test | 사용 |
|---------|-------|-----|------|------|
| KLUE | ❌ | val (1995, 평가용) | — | 평가 (leakage 방지) |
| UD Kaist | ❌ | 2066 ✓ | 1638 (평가용) | 학습 (dev) |
| UD GSD | ❌ | 950 ✓ | 971 (평가용) | 학습 (dev) |

**학습 데이터**: UD Kaist dev + UD GSD dev = **3016 sentences (~73K morphemes)**.

작지만 첫 시도로 충분. test data는 평가용 그대로 보존 (leakage 방지).

### 변환 스크립트 (`tools/to_mecab_tagged.py`)

CoNLL-U → `.tagged`:
```
<surface>\t<POS>,*,*,*,*,*,*,*,*
...
EOS
```

- UD Kaist xpos (lowercase): `KAIST_TO_SEJONG` 매핑 (Sprint 139 patterns 재사용)
- UD GSD xpos (Sejong 직접): 대문자 변환만
- mecab features 9 fields, POS만 정확하고 나머지 `*` (optional features 처리)

### Corpus 통계

```
ud_kaist_dev.tagged: 52450 lines
ud_gsd_dev.tagged:   23525 lines
corpus_dev.tagged:   75975 lines (~73K morphemes + 3016 EOS markers)
```

## 검증

- 변환 스크립트 정상 동작 (Unknown POS 1건 minor)
- 통합 corpus 형식 검증 (mecab .tagged 호환)
- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 변경 없음 (411 pass)
- 5-gate sample.tsv: 영향 없음 (학습 미실행)

## 변경 파일

- `tools/to_mecab_tagged.py` (신규)
- `data/train/` (gitignore 추가)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 166 — Track B Step 3: 1차 학습

### 작업

1. mecab-cost-train 실행:
   ```bash
   cd legacy
   DYLD_LIBRARY_PATH=src/.libs src/.libs/mecab-cost-train \
     -d ../data/mecab-ko-dic-2.1.1-20180720 \
     -p 4 -f 1 \
     ../data/train/corpus_dev.tagged
   ```
2. 학습 시간 측정 (예상: 수 분)
3. 결과 model.def 생성 확인
4. mecab-dict-gen으로 새 matrix.def + sys.dic 변환

### 위험

- 학습 코퍼스가 작음 (3016 sentences) → 정확도 lift 작을 수 있음
- 작은 corpus로 학습한 model이 기존 mecab-ko-dic baseline 대비 회귀 가능
- 새 dict는 별도 디렉토리로 격리 (점진적 검증)
