# 신조어 자동 수집 파이프라인 설계

## 문서 정보
- **작업 ID**: S13-08
- **작성일**: 2026-03-02
- **작성자**: PM Agent (Planner)
- **상태**: 설계 완료

---

## 1. 개요

### 1.1 목적
국립국어원 우리말샘 API를 활용하여 신조어를 주기적으로 수집하고,
MeCab-Ko 사전에 자동으로 추가하는 파이프라인을 구축합니다.

### 1.2 범위
- GitHub Actions 스케줄 워크플로우
- 신조어 수집 및 변환
- 중복/충돌 검사
- 자동 PR 생성
- 알림 시스템

---

## 2. 아키텍처

### 2.1 전체 흐름

```
┌─────────────────────────────────────────────────────────────────────┐
│                    신조어 자동 수집 파이프라인                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  [스케줄러]          [수집]            [검증]           [PR 생성]    │
│  ┌─────────┐      ┌─────────┐      ┌─────────┐      ┌─────────┐    │
│  │ GitHub  │─────>│ 우리말샘 │─────>│ 중복검사 │─────>│ 자동 PR │    │
│  │ Actions │      │ API 호출 │      │ 품사검증 │      │ 생성    │    │
│  │ (cron)  │      │         │      │ 비용계산 │      │         │    │
│  └─────────┘      └─────────┘      └─────────┘      └─────────┘    │
│       │                │                │                │         │
│       │                ▼                ▼                ▼         │
│       │          ┌─────────┐      ┌─────────┐      ┌─────────┐    │
│       │          │ CSV     │      │ 검증    │      │ 리뷰어  │    │
│       │          │ 변환    │      │ 리포트  │      │ 할당    │    │
│       │          └─────────┘      └─────────┘      └─────────┘    │
│       │                                                  │         │
│       │                                                  ▼         │
│       │                                            ┌─────────┐    │
│       └─────────────────── 실패 시 ───────────────>│ 알림    │    │
│                                                    │ (Slack) │    │
│                                                    └─────────┘    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 수집 전략

#### 2.2.1 검색 키워드

1. **신조어 카테고리**
   - "신조어" - 명시적 신조어 태그
   - "신어" - 새로 만들어진 말
   - "최근어" - 최근 등록 어휘

2. **도메인별 키워드**
   - IT/기술: "AI", "메타버스", "블록체인", "가상화폐"
   - 문화: "K-", "아이돌", "팬덤"
   - 사회: "MZ", "워라밸", "재테크"
   - 인터넷: "밈", "챌린지", "인플루언서"

3. **검색 주기**
   - 주간: 인기 검색어 기반 신조어
   - 월간: 종합 신조어 수집

#### 2.2.2 필터링 규칙

```rust
pub struct NeologismFilter {
    /// 최소 표면형 길이
    min_surface_len: usize,     // 2자 이상
    /// 최대 표면형 길이
    max_surface_len: usize,     // 20자 이하
    /// 허용 품사 목록
    allowed_pos: Vec<String>,   // NNG, NNP, VV, VA, MAG
    /// 제외 패턴 (정규식)
    exclude_patterns: Vec<Regex>,
}
```

---

## 3. 기술 구현

### 3.1 신조어 수집 CLI 확장

#### 3.1.1 새로운 서브커맨드

```rust
/// 신조어 자동 수집 명령
#[derive(Args, Debug)]
pub struct CollectArgs {
    /// 수집 모드 (weekly/monthly/custom)
    #[arg(short, long, default_value = "weekly")]
    pub mode: CollectMode,

    /// 커스텀 키워드 목록 (쉼표 구분)
    #[arg(short, long)]
    pub keywords: Option<String>,

    /// 최대 수집 개수
    #[arg(short = 'n', long, default_value = "100")]
    pub max_entries: u32,

    /// 출력 파일 경로
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// 기존 파일과 병합
    #[arg(long)]
    pub merge: bool,

    /// 중복 검사 대상 CSV
    #[arg(long)]
    pub check_duplicates: Option<PathBuf>,

    /// 리포트 출력
    #[arg(long)]
    pub report: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CollectMode {
    /// 주간 수집 (인기 키워드 기반)
    Weekly,
    /// 월간 수집 (종합)
    Monthly,
    /// 커스텀 키워드 수집
    Custom,
}
```

#### 3.1.2 수집 로직

```rust
pub async fn collect_neologisms(
    client: &OpenDictClient,
    mode: CollectMode,
    keywords: Option<&[String]>,
    max_entries: u32,
) -> Result<Vec<ConverterEntry>> {
    let search_keywords = match mode {
        CollectMode::Weekly => get_weekly_keywords(),
        CollectMode::Monthly => get_monthly_keywords(),
        CollectMode::Custom => keywords
            .ok_or(Error::MissingKeywords)?
            .to_vec(),
    };

    let mut all_entries = Vec::new();

    for keyword in &search_keywords {
        let entries = client.search(keyword).await?;
        all_entries.extend(entries);

        // Rate limiting
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // 중복 제거 및 필터링
    let filtered = filter_and_dedupe(all_entries, max_entries);

    Ok(filtered)
}

fn get_weekly_keywords() -> Vec<String> {
    vec![
        "신조어".into(),
        "AI".into(),
        "메타버스".into(),
        "MZ".into(),
    ]
}

fn get_monthly_keywords() -> Vec<String> {
    vec![
        "신조어".into(),
        "신어".into(),
        "AI".into(),
        "블록체인".into(),
        "가상화폐".into(),
        "메타버스".into(),
        "MZ세대".into(),
        "워라밸".into(),
        "인플루언서".into(),
        "밈".into(),
        "K-팝".into(),
        "K-드라마".into(),
    ]
}
```

### 3.2 검증 시스템

#### 3.2.1 중복 검사

```rust
pub struct DuplicateChecker {
    existing_entries: HashSet<String>,
    system_dict_surfaces: HashSet<String>,
}

impl DuplicateChecker {
    pub fn from_csv(path: &Path) -> Result<Self> {
        let mut existing = HashSet::new();

        if path.exists() {
            let reader = csv::Reader::from_path(path)?;
            for record in reader.records() {
                if let Ok(r) = record {
                    existing.insert(r[0].to_string());
                }
            }
        }

        Ok(Self {
            existing_entries: existing,
            system_dict_surfaces: HashSet::new(), // 로드 필요
        })
    }

    pub fn check(&self, entry: &ConverterEntry) -> DuplicateResult {
        if self.existing_entries.contains(&entry.surface) {
            DuplicateResult::ExistingEntry
        } else if self.system_dict_surfaces.contains(&entry.surface) {
            DuplicateResult::SystemDictConflict
        } else {
            DuplicateResult::New
        }
    }
}

pub enum DuplicateResult {
    New,
    ExistingEntry,
    SystemDictConflict,
}
```

#### 3.2.2 품사 태그 검증

```rust
pub fn validate_pos_tag(pos: &str) -> ValidationResult {
    const VALID_TAGS: &[&str] = &[
        "NNG", "NNP", "NNB", "NP", "NR",  // 명사류
        "VV", "VA", "VX", "VCP", "VCN",   // 동사류
        "MAG", "MAJ", "MM", "IC",         // 부사/관형사/감탄사
        "JKS", "JKC", "JKG", "JKO", "JKB", "JKV", "JKQ", "JX", "JC", // 조사
        "EP", "EF", "EC", "ETN", "ETM",   // 어미
        "XPN", "XSN", "XSV", "XSA", "XR", // 접사/어근
        "SF", "SE", "SS", "SP", "SO", "SW", "SH", "SL", "SN", // 기호
    ];

    if VALID_TAGS.contains(&pos) {
        ValidationResult::Valid
    } else {
        ValidationResult::Invalid(format!("Unknown POS tag: {}", pos))
    }
}
```

#### 3.2.3 비용 자동 계산

```rust
pub fn calculate_cost(entry: &ConverterEntry) -> i16 {
    let mut cost: i16 = 0;

    // 빈도 기반 비용
    match entry.frequency {
        Some(f) if f >= 1000 => cost = 0,
        Some(f) if f >= 100 => cost = 500,
        Some(_) => cost = 1000,
        None => cost = 500,
    }

    // 단어 길이 보정 (긴 단어 선호)
    let char_count = entry.surface.chars().count();
    if char_count > 5 {
        cost -= 100;
    }

    // 고유명사 보정
    if entry.pos == "NNP" || entry.pos == "고유명사" {
        cost -= 200;
    }

    cost.max(-10000)
}
```

### 3.3 리포트 생성

```rust
pub struct CollectionReport {
    pub total_collected: usize,
    pub new_entries: usize,
    pub duplicates_skipped: usize,
    pub validation_errors: usize,
    pub entries: Vec<ConverterEntry>,
    pub errors: Vec<String>,
}

impl CollectionReport {
    pub fn to_markdown(&self) -> String {
        format!(r#"
## 신조어 수집 리포트

### 요약
| 항목 | 개수 |
|------|------|
| 총 수집 | {} |
| 신규 추가 | {} |
| 중복 스킵 | {} |
| 검증 실패 | {} |

### 신규 엔트리 (상위 20개)
| 표면형 | 품사 | 비용 | 읽기 |
|--------|------|------|------|
{}

### 에러 로그
{}
"#,
            self.total_collected,
            self.new_entries,
            self.duplicates_skipped,
            self.validation_errors,
            self.format_entries_table(),
            self.format_errors(),
        )
    }
}
```

---

## 4. GitHub Actions 워크플로우

### 4.1 워크플로우 파일

**파일 경로**: `.github/workflows/neologism-sync.yml`

```yaml
name: Neologism Sync

on:
  schedule:
    # 매주 월요일 09:00 KST (00:00 UTC)
    - cron: '0 0 * * 1'
  workflow_dispatch:
    inputs:
      mode:
        description: 'Collection mode'
        required: true
        default: 'weekly'
        type: choice
        options:
          - weekly
          - monthly
          - custom
      keywords:
        description: 'Custom keywords (comma-separated)'
        required: false
        type: string
      max_entries:
        description: 'Maximum entries to collect'
        required: false
        default: '100'
        type: string
      dry_run:
        description: 'Dry run (no PR creation)'
        required: false
        default: false
        type: boolean

env:
  CARGO_TERM_COLOR: always
  NEOLOGISMS_CSV: data/user-dict/neologisms.csv

jobs:
  collect-neologisms:
    name: Collect Neologisms
    runs-on: ubuntu-latest
    permissions:
      contents: write
      pull-requests: write
    outputs:
      new_entries: ${{ steps.collect.outputs.new_entries }}
      has_changes: ${{ steps.check.outputs.has_changes }}

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo build
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            rust/target
          key: ${{ runner.os }}-cargo-neologism-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-neologism-

      - name: Build CLI
        run: cargo build --release --manifest-path rust/Cargo.toml -p mecab-ko-cli

      - name: Collect neologisms
        id: collect
        env:
          OPENDICT_API_KEY: ${{ secrets.OPENDICT_API_KEY }}
        run: |
          MODE="${{ github.event.inputs.mode || 'weekly' }}"
          KEYWORDS="${{ github.event.inputs.keywords }}"
          MAX_ENTRIES="${{ github.event.inputs.max_entries || '100' }}"

          echo "Mode: $MODE"
          echo "Max entries: $MAX_ENTRIES"

          # Run collection
          ./rust/target/release/mecab-ko collect \
            --mode "$MODE" \
            ${KEYWORDS:+--keywords "$KEYWORDS"} \
            --max-entries "$MAX_ENTRIES" \
            --output collected.csv \
            --check-duplicates "$NEOLOGISMS_CSV" \
            --report > report.md

          # Count new entries
          NEW_COUNT=$(wc -l < collected.csv | tr -d ' ')
          echo "new_entries=$NEW_COUNT" >> $GITHUB_OUTPUT

          # Output report
          cat report.md >> $GITHUB_STEP_SUMMARY

      - name: Merge with existing dictionary
        if: steps.collect.outputs.new_entries != '0'
        run: |
          # Backup original
          cp "$NEOLOGISMS_CSV" neologisms_backup.csv

          # Merge new entries
          cat collected.csv >> "$NEOLOGISMS_CSV"

          # Sort and remove duplicates (keep first occurrence)
          head -n 6 neologisms_backup.csv > temp_header.csv
          tail -n +7 "$NEOLOGISMS_CSV" | sort -t, -k1,1 -u >> temp_sorted.csv
          cat temp_header.csv temp_sorted.csv > "$NEOLOGISMS_CSV"
          rm temp_header.csv temp_sorted.csv

      - name: Check for changes
        id: check
        run: |
          if git diff --quiet "$NEOLOGISMS_CSV"; then
            echo "has_changes=false" >> $GITHUB_OUTPUT
          else
            echo "has_changes=true" >> $GITHUB_OUTPUT
          fi

      - name: Upload collection report
        uses: actions/upload-artifact@v4
        with:
          name: neologism-report
          path: |
            collected.csv
            report.md
          retention-days: 30

      - name: Create Pull Request
        if: steps.check.outputs.has_changes == 'true' && github.event.inputs.dry_run != 'true'
        uses: peter-evans/create-pull-request@v6
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          commit-message: |
            feat(dict): add new neologisms from ${{ github.event.inputs.mode || 'weekly' }} sync

            - Added ${{ steps.collect.outputs.new_entries }} new entries
            - Source: 국립국어원 우리말샘 API
          branch: neologism-sync/${{ github.run_number }}
          delete-branch: true
          title: "[자동] 신조어 사전 업데이트 (${{ steps.collect.outputs.new_entries }}개)"
          body: |
            ## 신조어 자동 수집 결과

            ### 수집 정보
            - **모드**: ${{ github.event.inputs.mode || 'weekly' }}
            - **신규 엔트리**: ${{ steps.collect.outputs.new_entries }}개
            - **소스**: 국립국어원 우리말샘 API
            - **실행 시각**: ${{ github.event.repository.updated_at }}

            ### 변경 내용
            신조어 사전(`data/user-dict/neologisms.csv`)에 새로운 엔트리가 추가되었습니다.

            ### 검토 체크리스트
            - [ ] 추가된 단어가 적절한가
            - [ ] 품사 태그가 올바른가
            - [ ] 비용 값이 합리적인가
            - [ ] 중복 엔트리가 없는가

            ### 상세 리포트
            상세 내용은 [Actions Artifacts](https://github.com/${{ github.repository }}/actions/runs/${{ github.run_id }})를 참조하세요.

            ---
            _이 PR은 GitHub Actions에 의해 자동 생성되었습니다._
          labels: |
            automated
            dictionary
            neologism
          reviewers: |
            hephaex
          draft: false

  notify-on-failure:
    name: Notify on Failure
    runs-on: ubuntu-latest
    needs: collect-neologisms
    if: failure()
    steps:
      - name: Send Slack notification
        if: secrets.SLACK_WEBHOOK_URL != ''
        uses: slackapi/slack-github-action@v1.25.0
        with:
          payload: |
            {
              "text": "신조어 수집 워크플로우 실패",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": "*신조어 수집 워크플로우 실패*\n<${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}|워크플로우 확인>"
                  }
                }
              ]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}
          SLACK_WEBHOOK_TYPE: INCOMING_WEBHOOK

      - name: Create failure issue
        uses: actions/github-script@v7
        with:
          script: |
            const issues = await github.rest.issues.listForRepo({
              owner: context.repo.owner,
              repo: context.repo.repo,
              labels: 'automated,neologism-sync-failure',
              state: 'open'
            });

            if (issues.data.length === 0) {
              await github.rest.issues.create({
                owner: context.repo.owner,
                repo: context.repo.repo,
                title: '[자동] 신조어 수집 실패',
                body: `## 신조어 수집 워크플로우 실패

            **실행 시각**: ${new Date().toISOString()}
            **워크플로우**: [Run #${{ github.run_id }}](${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }})

            ### 확인 사항
            - [ ] API 키 유효성 확인 (\`OPENDICT_API_KEY\` secret)
            - [ ] API 서버 상태 확인
            - [ ] Rate limit 확인

            ---
            _이 이슈는 자동 생성되었습니다._`,
                labels: ['automated', 'neologism-sync-failure', 'bug']
              });
            }
```

### 4.2 월간 수집 워크플로우 (추가)

```yaml
# 월간 종합 수집을 위한 별도 job 추가
  monthly-comprehensive:
    name: Monthly Comprehensive Collection
    runs-on: ubuntu-latest
    if: github.event.schedule == '0 0 1 * *'  # 매월 1일
    # ... (동일한 steps, mode: monthly)
```

---

## 5. 환경 설정

### 5.1 필요한 Secrets

| Secret | 설명 | 필수 |
|--------|------|------|
| `OPENDICT_API_KEY` | 국립국어원 우리말샘 API 키 | O |
| `SLACK_WEBHOOK_URL` | Slack 알림 웹훅 | X |

### 5.2 API 키 발급

1. [공공데이터포털](https://www.data.go.kr/) 회원가입
2. [우리말샘 API](https://www.data.go.kr/data/15019347/openapi.do) 활용 신청
3. 발급받은 API 키를 GitHub Secrets에 등록

### 5.3 Slack 웹훅 설정 (선택)

1. Slack App 생성 또는 Incoming Webhook 설정
2. 웹훅 URL을 `SLACK_WEBHOOK_URL` secret에 등록

---

## 6. 구현 체크리스트

### Phase 1: CLI 확장 (P0)
- [ ] `mecab-ko collect` 서브커맨드 추가
- [ ] `CollectMode` enum 구현 (weekly/monthly/custom)
- [ ] 키워드 기반 수집 로직 구현
- [ ] 중복 검사 로직 구현
- [ ] 리포트 생성 기능 구현
- [ ] 테스트 작성 (unit + integration)

### Phase 2: 검증 시스템 (P1)
- [ ] `DuplicateChecker` 구현
- [ ] 시스템 사전 충돌 검사
- [ ] 품사 태그 검증 강화
- [ ] 비용 자동 계산 최적화
- [ ] 검증 리포트 포맷 개선

### Phase 3: GitHub Actions (P1)
- [ ] `neologism-sync.yml` 워크플로우 생성
- [ ] 스케줄 설정 (주간/월간)
- [ ] PR 자동 생성 구현
- [ ] 리뷰어 자동 할당 설정
- [ ] Artifact 업로드 설정

### Phase 4: 알림 시스템 (P2)
- [ ] Slack 알림 설정
- [ ] 실패 시 이슈 자동 생성
- [ ] Step Summary 리포트 개선

### Phase 5: 문서화 (P2)
- [ ] README 업데이트
- [ ] API 키 발급 가이드
- [ ] 트러블슈팅 가이드

---

## 7. 테스트 계획

### 7.1 단위 테스트

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weekly_keywords() {
        let keywords = get_weekly_keywords();
        assert!(keywords.contains(&"신조어".to_string()));
    }

    #[test]
    fn test_duplicate_checker() {
        let checker = DuplicateChecker::new();
        checker.add_existing("메타버스");

        let entry = ConverterEntry {
            surface: "메타버스".to_string(),
            pos: "명사".to_string(),
            ..Default::default()
        };

        assert_eq!(checker.check(&entry), DuplicateResult::ExistingEntry);
    }

    #[test]
    fn test_cost_calculation() {
        let entry = ConverterEntry {
            surface: "챗GPT".to_string(),
            pos: "고유명사".to_string(),
            frequency: Some(5000),
            ..Default::default()
        };

        let cost = calculate_cost(&entry);
        assert!(cost < 0); // 고빈도 고유명사는 음수 비용
    }
}
```

### 7.2 통합 테스트

```rust
#[tokio::test]
#[ignore] // API 키 필요
async fn test_collect_from_opendict() {
    let config = OpenDictConfig::from_env().unwrap();
    let client = OpenDictClient::new(config).unwrap();

    let entries = collect_neologisms(
        &client,
        CollectMode::Custom,
        Some(&["테스트".to_string()]),
        10,
    ).await.unwrap();

    assert!(!entries.is_empty());
}
```

### 7.3 워크플로우 테스트

1. `workflow_dispatch`로 수동 실행
2. `dry_run: true`로 PR 생성 없이 테스트
3. 결과 Artifact 확인

---

## 8. 모니터링 및 유지보수

### 8.1 모니터링 지표

- 수집 성공/실패 횟수
- 신규 엔트리 수 추이
- API 호출 횟수
- PR 병합률

### 8.2 유지보수 작업

- 월간: 키워드 목록 검토 및 업데이트
- 분기: API 변경 사항 확인
- 연간: 수집 전략 검토

---

## 9. 참고 자료

- [우리말샘 API 문서](https://opendict.korean.go.kr/service/openApiInfo)
- [공공데이터포털](https://www.data.go.kr/)
- [peter-evans/create-pull-request](https://github.com/peter-evans/create-pull-request)
- [mecab-ko-dict-sync 크레이트](../../../rust/crates/mecab-ko-dict-sync/)

---

## 10. 변경 이력

| 버전 | 날짜 | 변경 내용 |
|------|------|----------|
| 1.0 | 2026-03-02 | 초기 설계 문서 작성 |
