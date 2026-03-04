# mecab-ko-dic v3.0 현대화 계획

## 작성일: 2026-03-04
## 작성: PM Agent (Sprint 20 - S20-03)

---

## 1. 목표

### 1.1 핵심 목표
- **엔트리 수**: 816K → 1M+ (22% 증가)
- **커버리지**: 2018-2026 신조어/외래어 추가
- **정확도**: Token Accuracy 50%+ 달성

### 1.2 비목표
- 품사 태그 체계 변경 (세종 코퍼스 호환 유지)
- 기존 엔트리 수정 (추가만)
- 비지도 학습 도입 (사전 기반 유지)

---

## 2. 데이터 소스

### 2.1 국립국어원 API (Primary)

| 소스 | API | 엔트리 수 | 갱신 주기 |
|------|-----|----------|----------|
| 우리말샘 | opendict API | 100K+ | 실시간 |
| 한국어기초사전 | krdict API | 50K+ | 월별 |
| 신어 자료집 | 연간 발행 | 500-1000/년 | 연간 |

### 2.2 커뮤니티 기여 (Secondary)

```
data/user-dict/
├── neologisms.csv      # 신조어 (현재 123개)
├── brands.csv          # 브랜드명/상품명
├── tech-terms.csv      # 기술 용어
├── internet-slang.csv  # 인터넷 용어
└── foreign-words.csv   # 외래어
```

### 2.3 자동 수집 (Tertiary)

- neologism-sync.yml 워크플로우
- 월간 자동 실행
- 품질 검증 후 병합

---

## 3. 품사 태그 체계

### 3.1 현재 체계 (유지)

세종 코퍼스 기반 품사 태그:

| 대분류 | 태그 | 설명 |
|--------|------|------|
| 체언 | NNG, NNP, NNB, NR, NP | 명사, 대명사, 수사 |
| 용언 | VV, VA, VX, VCP, VCN | 동사, 형용사, 보조용언 |
| 관형사 | MM | 관형사 |
| 부사 | MAG, MAJ | 일반부사, 접속부사 |
| 조사 | JKS, JKC, JKG, JKO, JKB, JKV, JKQ, JX, JC | 격조사, 보조사 |
| 어미 | EP, EF, EC, ETN, ETM | 선어말, 종결, 연결, 전성어미 |
| 접사 | XPN, XSN, XSV, XSA | 접두사, 접미사 |
| 기호 | SF, SE, SS, SP, SO, SW | 문장부호, 특수기호 |

### 3.2 신조어 품사 추정 규칙

```rust
// 품사 자동 추정 로직 (이미 구현됨)
fn estimate_pos(surface: &str) -> String {
    if is_proper_noun(surface) {
        "NNP"  // 고유명사: 브랜드, 인명, 지명
    } else if is_foreign_word(surface) {
        "NNP"  // 외래어 → 고유명사
    } else if ends_with_하다(surface) {
        "VV"   // 동사성 명사
    } else {
        "NNG"  // 기본값: 일반명사
    }
}
```

---

## 4. 빌드 프로세스

### 4.1 현재 파이프라인

```
[mecab-ko-dic CSV] → [dict-builder] → [바이너리 사전]
                          ↓
                    sys.dic (Trie)
                    matrix.bin (연접비용)
                    entries.bin (엔트리 데이터)
```

### 4.2 v3.0 개선 파이프라인

```
┌───────────────────────────────────────────────────────┐
│  v3.0 Dictionary Build Pipeline                       │
├───────────────────────────────────────────────────────┤
│                                                       │
│  [국립국어원 API] ──┐                                   │
│  [커뮤니티 CSV]  ──┼──▶ [병합/검증] ──▶ [dict-builder] │
│  [자동 수집]     ──┘        │              │           │
│                            ▼              ▼           │
│                      [중복 제거]    [바이너리 사전]     │
│                      [품사 검증]    [GitHub Release]   │
│                      [비용 최적화]                     │
│                                                       │
└───────────────────────────────────────────────────────┘
```

### 4.3 GitHub Actions 자동화

```yaml
# .github/workflows/dict-v3-build.yml
name: Build mecab-ko-dic v3.0

on:
  schedule:
    - cron: '0 0 1 * *'  # 매월 1일
  workflow_dispatch:
    inputs:
      include_neologisms:
        description: 'Include neologism sync'
        type: boolean
        default: true

jobs:
  sync-sources:
    runs-on: ubuntu-latest
    steps:
      - name: Sync 국립국어원 API
        run: ./scripts/sync-nikl-api.sh

      - name: Validate user dictionaries
        run: cargo run -p mecab-ko-cli -- validate-dict data/user-dict/

  build-dictionary:
    needs: sync-sources
    runs-on: ubuntu-latest
    steps:
      - name: Merge all sources
        run: ./scripts/merge-dictionaries.sh

      - name: Build binary dictionary
        run: cargo run -p mecab-ko-dict-builder -- build

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
```

---

## 5. 품질 관리

### 5.1 검증 규칙

1. **포맷 검증**: CSV 필드 수, 인코딩
2. **품사 검증**: 유효한 품사 태그만 허용
3. **중복 검증**: 동일 표면형 중복 방지
4. **비용 검증**: 이상치 탐지 (3σ 기준)

### 5.2 테스트 데이터셋

| 데이터셋 | 문장 수 | 용도 |
|---------|--------|------|
| sample.tsv | 160 | 기본 벤치마크 |
| sejong-subset.tsv | 1,000 | 정확도 측정 |
| news-2024.tsv | 500 | 신조어 테스트 |

### 5.3 KPI 목표

| 지표 | v2.1.1 | v3.0 목표 |
|------|--------|----------|
| 엔트리 수 | 816K | 1M+ |
| Token Accuracy | 15.2% | 50%+ |
| 신조어 커버리지 | 0% | 80%+ |
| 미등록어 비율 | 15%+ | 5% 이하 |

---

## 6. 구현 로드맵

### Phase 1: 인프라 구축 (Sprint 20-21)

- [x] S20-02: 세종 코퍼스 호환 모드
- [x] S20-06: 정확도 측정 인프라
- [ ] 국립국어원 API 동기화 스크립트
- [ ] 사전 병합 도구 개선

### Phase 2: 데이터 수집 (Sprint 22-23)

- [ ] 우리말샘 API 연동
- [ ] 2018-2024 신조어 자료집 수집
- [ ] 커뮤니티 기여 CSV 확장

### Phase 3: 빌드 및 테스트 (Sprint 24-25)

- [ ] v3.0 통합 빌드
- [ ] 정확도 벤치마크 (목표: 50%+)
- [ ] 성능 회귀 테스트

### Phase 4: 배포 (Sprint 26)

- [ ] GitHub Release v3.0
- [ ] crates.io 업데이트
- [ ] 문서 업데이트

---

## 7. 리스크 및 대응

### 7.1 기술적 리스크

| 리스크 | 영향 | 대응 |
|--------|------|------|
| 빌드 시간 증가 | 중간 | 병렬 빌드, 캐싱 |
| 메모리 사용량 증가 | 높음 | 지연 로딩 최적화 |
| API Rate Limit | 낮음 | 캐싱, 배치 처리 |

### 7.2 데이터 리스크

| 리스크 | 영향 | 대응 |
|--------|------|------|
| 라이선스 충돌 | 높음 | CC-BY-SA 호환만 사용 |
| 품질 저하 | 중간 | 자동 검증, 리뷰 |
| 중복 데이터 | 낮음 | 중복 제거 파이프라인 |

---

## 8. 결론

mecab-ko-dic v3.0은 다음을 달성합니다:

1. **규모 확대**: 816K → 1M+ 엔트리
2. **현대화**: 2018-2026 신조어/외래어 추가
3. **자동화**: 월간 자동 업데이트 파이프라인
4. **품질**: Token Accuracy 50%+ 목표

구현은 Phase 1-4로 나누어 점진적으로 진행하며, 기존 사전 호환성을 유지합니다.
