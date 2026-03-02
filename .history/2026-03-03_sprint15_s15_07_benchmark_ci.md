# Sprint 15 - S15-07: 성능 벤치마크 CI 통합

**날짜**: 2026-03-03
**상태**: ✅ 완료
**커밋**: 88ddc781, 85bd1315

## 목표

PR별 성능 비교를 자동화하고 회귀 알림을 강화하여 성능 퇴행을 조기에 감지하는 CI/CD 시스템 구축.

## 주요 개선사항

### 1. 회귀 감지 3단계 체계 구축

```
Performance Change    │ Symbol │ Action              │ Status
──────────────────────┼────────┼─────────────────────┼──────────
Improvement (< 0%)    │ 🟢 ✅  │ Approved            │ Pass
Stable (0% - 5%)      │ ✅    │ Approved            │ Pass
Warning (5% - 10%)    │ ⚠️    │ Review required     │ Pass
Critical (> 10%)      │ ❌    │ Prevent merge       │ Fail
```

**의의**: 성능 변화의 심각도를 즉시 파악 가능

### 2. JSON 기반 결과 저장

#### benchmark-results.json (단일 실행)
```json
{
  "version": "main",
  "commit": "a4b3a6f",
  "timestamp": "2026-03-03T15:30:45Z",
  "platform": "ubuntu-latest",
  "rustc": "stable",
  "results": {
    "tokenize::tokenize_short": {
      "time_ns": 5200,
      "time_us": 5.2,
      "time_ms": 0.0052
    }
  }
}
```

#### benchmark-comparison.json (PR 비교)
```json
{
  "pr_number": 123,
  "base_branch": "main",
  "pr_branch": "feature/opt",
  "timestamp": "2026-03-03T15:30:45Z",
  "comparison": {
    "tokenize::tokenize_short": {
      "base_ns": 5200,
      "pr_ns": 5100,
      "diff_ns": -100,
      "diff_pct": -1.9,
      "status": "pass"
    }
  }
}
```

**의의**: 히스토리 추적 및 자동화된 분석 가능

### 3. 개선된 PR 코멘트 포맷

**이전**:
```
| tokenize_short | 5200.0µs | 5100.0µs | -1.9% |
```

**개선**:
```
| tokenize_short | 5.2µs | 5.1µs | -1.9% | 🟢 |
```

**개선점**:
- 시간 단위 자동 변환 (ns → µs → ms)
- 상태 기호 추가 (✅/⚠️/❌/🟢)
- 명확한 의도 표시

### 4. [skip bench] 커밋 메시지 지원

문서나 설정만 수정할 때:
```bash
git commit -m "docs: update README [skip bench]"
```

벤치마크가 자동으로 스킵되어 CI 시간 단축.

### 5. Concurrency 제어

동일 이벤트로 중복 실행되는 벤치마크 워크플로우 자동 취소.

## 파일 변경 사항

### 수정된 파일

**`.github/workflows/benchmark.yml`** (376 → 453 줄)

주요 변경:
- `concurrency` 추가: 중복 실행 방지
- `check-skip` 스텝: [skip bench] 감지
- `Convert results to JSON` 스텝: 결과 JSON 변환
- 개선된 회귀 감지 로직:
  ```javascript
  if (diffNum > 10) {
    status = '❌';  // Critical
  } else if (diffNum > 5) {
    status = '⚠️';   // Warning
  } else if (diffNum < 0) {
    status = '🟢';   // Improvement
  } else {
    status = '✅';   // Pass
  }
  ```
- `formatTime()` 헬퍼 함수: 단위 자동 변환
- JSON 변환 스텝 (Python):
  - benchmark-results.json 생성
  - benchmark-comparison.json 생성
- 비교 결과 JSON화

### 신규 파일

1. **`docs/BENCHMARK_CI_GUIDE.md`** (370 줄)
   - 워크플로우 사용법
   - JSON 포맷 설명
   - 예제 및 트러블슈팅
   - 파일 다운로드 방법

2. **`docs/PERFORMANCE_BASELINES.md`** (280 줄)
   - 기준선 성능 지표
   - 회귀 임계값 정의
   - 측정 환경 명시
   - 최적화 우선순위

3. **`scripts/benchmark-parser.py`** (260 줄)
   - 로컬 벤치마크 파싱 도구
   - Bencher 형식 파서
   - JSON 결과 생성
   - 마크다운 테이블 생성
   - CLI 인터페이스

## 기술 구현 상세

### Bencher 형식 파싱

```python
# 정규식 패턴
r'test\s+(\S+)\s+.*bench:\s+([\d,]+)\s+ns/iter'

# 예제 입력
"test tokenize::tokenize_short ... bench: 5,200 ns/iter"

# 파싱 결과
{
  "tokenize::tokenize_short": 5200
}
```

### 시간 단위 자동 변환

```javascript
function formatTime(ns) {
  if (ns < 1000) return ns.toFixed(1) + 'ns';
  if (ns < 1_000_000) return (ns / 1000).toFixed(1) + 'µs';
  return (ns / 1_000_000).toFixed(1) + 'ms';
}

// 예제
formatTime(5200) → "5.2µs"
formatTime(130000000) → "130.0ms"
```

### PR 코멘트 생성 로직

```javascript
// 1. base/PR 결과 파싱
const baseResults = parseBenchmarks(baseBench);
const prResults = parseBenchmarks(prBench);

// 2. 차이점 계산
for (const [name, prTime] of Object.entries(prResults)) {
  const baseTime = baseResults[name];
  const diff = ((prTime - baseTime) / baseTime * 100).toFixed(1);

  // 3. 상태 결정
  if (diff > 10) status = '❌';
  else if (diff > 5) status = '⚠️';
  // ...
}

// 4. 마크다운 생성
comment += `| ${shortName} | ${formatTime(baseTime)} | ${formatTime(prTime)} | ${diff}% | ${status} |\n`;

// 5. 요약 추가
if (hasError) comment += '### ❌ Critical Performance Regression Detected!';
```

## 검증

### 로컬 테스트 가능성

```bash
# 1. 결과 파싱
python3 scripts/benchmark-parser.py parse /tmp/benchmark.txt

# 2. 두 결과 비교
python3 scripts/benchmark-parser.py compare /tmp/base.txt /tmp/pr.txt

# 3. 마크다운 테이블 생성
python3 scripts/benchmark-parser.py format-table /tmp/base.txt /tmp/pr.txt
```

### 워크플로우 검증

주요 체크 포인트:
- ✅ concurrency 설정으로 중복 실행 방지
- ✅ [skip bench] 감지 로직 동작 확인
- ✅ Python 스크립트 JSON 변환 정상
- ✅ PR 코멘트 3단계 회귀 감지 구현
- ✅ 90일 artifact 보관

## 성능 영향 분석

### CI 시간 증가
- 이전: 벤치마크 2-3분
- 현재: 벤치마크 2-3분 + JSON 변환 10초
- **총증가**: ~10초 (무시할 수 있는 수준)

### 아티팩트 크기
- benchmark-results.json: ~5-10KB (벤치마크 50-100개 기준)
- benchmark-comparison.json: ~10-15KB
- **90일 보관**: ~100-200개 PR 기준 100-200MB

## 문서화

### 개발자 가이드
- **사용자**: BENCHMARK_CI_GUIDE.md
- **성능 기준선**: PERFORMANCE_BASELINES.md
- **로컬 도구**: scripts/benchmark-parser.py

### 예제 워크플로우

1. **PR 생성**
   - 자동으로 벤치마크 실행
   - base branch(main)와 비교

2. **결과 검토**
   - PR 코멘트에서 비교 테이블 확인
   - 상태 기호로 심각도 판단
   - 회귀 시 조사/최적화

3. **머지**
   - 기준 이내면 승인
   - main에 푸시하면 대시보드 업데이트

## 다음 작업 (S15-08)

### 대기 중
- S15-08: 문서 사이트 개선
  - mdBook 구조 정리
  - API 문서 보강
  - 튜토리얼 추가
  - 벤치마크 대시보드 시각화

## 학습 포인트

1. **회귀 감지의 다단계 설계**
   - 0-5% 범위는 무시할 수 있는 수준
   - 5-10% 범위는 검토 필요
   - 10% 이상은 반드시 조사

2. **JSON 기반 메트릭 저장의 가치**
   - 자동화된 분석 가능
   - 히스토리 추적
   - 대시보드 시각화 기초

3. **PR 코멘트 자동화의 개발자 경험 개선**
   - 수동 비교 불필요
   - 즉시 피드백
   - 의도 명확성

## 참고 자료

- [GitHub Actions](https://docs.github.com/en/actions)
- [Rust Benchmarks](https://doc.rust-lang.org/1.0.0/book/benchmark-tests.html)
- [Bencher Format](https://docs.rs/criterion/latest/criterion/)

## 개선 대상 (향후)

- [ ] 역사적 추이 분석 (main branch 성능 그래프)
- [ ] 회귀 시 자동 알림 (Slack/이메일)
- [ ] 다중 플랫폼 벤치마크 (macOS, Windows)
- [ ] 메모리 프로파일링
- [ ] 성능 회귀 시 자동 PR 차단
