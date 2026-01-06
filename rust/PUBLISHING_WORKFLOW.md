# Publishing Workflow Visualization

## 배포 의존성 그래프

```
                    mecab-ko-hangul (0.1.0)
                            │
                ┌───────────┴───────────┐
                │                       │
        mecab-ko-dict (0.1.0)          │
                │                       │
        ┌───────┴────────┐              │
        │                │              │
mecab-ko-core      mecab-ko-dict-builder│
   (0.1.0)              (0.1.0)         │
        │                                │
        │                                │
   mecab-ko-cli                          │
     (0.1.0)                             │
                                         │
                    ┌────────────────────┴────────────────────┐
                    │                                          │
                    │                    mecab-ko (0.1.0)      │
                    │                      [facade]            │
                    └──────────────────────────────────────────┘
```

## 배포 프로세스 플로우

```
┌─────────────────────────────────────────────────────────────┐
│ 1. 사전 준비                                                  │
├─────────────────────────────────────────────────────────────┤
│ □ Git working directory 정리                                 │
│ □ 모든 테스트 통과 (cargo test --workspace)                  │
│ □ Clippy 경고 제거 (cargo clippy --workspace -- -D warnings)│
│ □ 코드 포맷팅 (cargo fmt --all)                              │
│ □ 문서 생성 확인 (cargo doc --workspace --no-deps)          │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. 의존성 전환                                                │
├─────────────────────────────────────────────────────────────┤
│ $ ./scripts/toggle-deps.sh version 0.1.0                    │
│                                                              │
│ path 의존성 → version 의존성 변환:                           │
│   mecab-ko-hangul = { path = ".." }                         │
│   ↓                                                          │
│   mecab-ko-hangul = "0.1.0"                                 │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. 빌드 검증                                                  │
├─────────────────────────────────────────────────────────────┤
│ $ cargo build --workspace                                   │
│ $ cargo test --workspace                                    │
│                                                              │
│ version 의존성으로 빌드 가능한지 확인                         │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. Dry-run 테스트                                            │
├─────────────────────────────────────────────────────────────┤
│ $ ./scripts/publish.sh --dry-run --version 0.1.0            │
│                                                              │
│ 각 크레이트별로:                                              │
│   □ 버전 확인                                                │
│   □ 의존성 검증 (path 사용 시 에러)                          │
│   □ 테스트 실행                                               │
│   □ 문서 빌드                                                 │
│   □ 패키지 검증                                               │
│   □ Dry-run publish                                          │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 5. Git 커밋 및 태그                                           │
├─────────────────────────────────────────────────────────────┤
│ $ git add .                                                 │
│ $ git commit -m "chore: prepare for v0.1.0 release"        │
│ $ git tag -a v0.1.0 -m "Release v0.1.0"                    │
│ $ git push origin main                                      │
│ $ git push origin v0.1.0                                    │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 6. 실제 배포                                                  │
├─────────────────────────────────────────────────────────────┤
│ $ ./scripts/publish.sh --version 0.1.0                      │
│                                                              │
│ 순차적으로 배포:                                              │
│   1. mecab-ko-hangul                                        │
│      └─ cargo publish                                       │
│      └─ sleep 30 (인덱스 업데이트 대기)                      │
│                                                              │
│   2. mecab-ko-dict                                          │
│      └─ cargo publish                                       │
│      └─ sleep 30                                            │
│                                                              │
│   3. mecab-ko-core                                          │
│      └─ cargo publish                                       │
│      └─ sleep 30                                            │
│                                                              │
│   4. mecab-ko-dict-builder                                  │
│      └─ cargo publish                                       │
│      └─ sleep 30                                            │
│                                                              │
│   5. mecab-ko-cli                                           │
│      └─ cargo publish                                       │
│      └─ sleep 30                                            │
│                                                              │
│   6. mecab-ko                                               │
│      └─ cargo publish                                       │
│      └─ 완료!                                                │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 7. 배포 확인                                                  │
├─────────────────────────────────────────────────────────────┤
│ crates.io 확인:                                              │
│   □ https://crates.io/crates/mecab-ko-hangul               │
│   □ https://crates.io/crates/mecab-ko-dict                 │
│   □ https://crates.io/crates/mecab-ko-core                 │
│   □ https://crates.io/crates/mecab-ko-dict-builder         │
│   □ https://crates.io/crates/mecab-ko-cli                  │
│   □ https://crates.io/crates/mecab-ko                      │
│                                                              │
│ docs.rs 확인:                                                │
│   □ https://docs.rs/mecab-ko-hangul                        │
│   □ https://docs.rs/mecab-ko-dict                          │
│   □ https://docs.rs/mecab-ko-core                          │
│   □ https://docs.rs/mecab-ko-dict-builder                  │
│   □ https://docs.rs/mecab-ko-cli                           │
│   □ https://docs.rs/mecab-ko                               │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 8. 설치 테스트                                                │
├─────────────────────────────────────────────────────────────┤
│ $ cargo new test-mecab-ko                                   │
│ $ cd test-mecab-ko                                          │
│ $ cargo add mecab-ko                                        │
│ $ cargo build                                               │
│ $ cargo test                                                │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 9. 개발 모드로 복귀                                           │
├─────────────────────────────────────────────────────────────┤
│ $ ./scripts/toggle-deps.sh path                             │
│                                                              │
│ version 의존성 → path 의존성 변환:                           │
│   mecab-ko-hangul = "0.1.0"                                 │
│   ↓                                                          │
│   mecab-ko-hangul = { path = "../mecab-ko-hangul" }        │
│                                                              │
│ $ git add .                                                 │
│ $ git commit -m "chore: restore path dependencies"         │
│ $ git push                                                  │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 10. GitHub Release 생성 (선택사항)                            │
├─────────────────────────────────────────────────────────────┤
│ $ gh release create v0.1.0 \                                │
│     --title "v0.1.0" \                                      │
│     --notes "Initial release of MeCab-Ko Rust implementation"│
└─────────────────────────────────────────────────────────────┘
```

## 에러 처리 플로우

```
배포 중 에러 발생
        │
        ├─ 테스트 실패?
        │   └─ 코드 수정 → 테스트 → 재시도
        │
        ├─ Clippy 경고?
        │   └─ 코드 수정 → Clippy → 재시도
        │
        ├─ 문서 빌드 실패?
        │   └─ Rustdoc 수정 → 재시도
        │
        ├─ 패키징 실패?
        │   └─ Cargo.toml 확인 → 재시도
        │
        ├─ 의존성 버전 불일치?
        │   └─ 이전 크레이트 배포 확인 → 재시도
        │
        └─ 네트워크 오류?
            └─ 재시도 (cargo publish는 멱등성)
```

## 타임라인 예상

```
단계                    소요 시간
────────────────────────────────────
1. 사전 준비            5-10분
2. 의존성 전환          1분
3. 빌드 검증            2-5분
4. Dry-run 테스트       5-10분
5. Git 커밋 및 태그     2분
6. 실제 배포            10-15분
   (6개 크레이트 × ~2분 + 대기시간)
7. 배포 확인            5분
8. 설치 테스트          3분
9. 개발 모드 복귀       2분
10. GitHub Release      2분
────────────────────────────────────
총 소요 시간            ~40-60분
```

## 체크포인트

### ✅ 배포 전
- [ ] 모든 크레이트의 구현 완료
- [ ] 통합 테스트 작성 및 통과
- [ ] 벤치마크 작성
- [ ] 모든 public API에 rustdoc
- [ ] README.md 검토
- [ ] CHANGELOG.md 업데이트

### ✅ 배포 중
- [ ] mecab-ko-hangul 배포 성공
- [ ] mecab-ko-dict 배포 성공
- [ ] mecab-ko-core 배포 성공
- [ ] mecab-ko-dict-builder 배포 성공
- [ ] mecab-ko-cli 배포 성공
- [ ] mecab-ko 배포 성공

### ✅ 배포 후
- [ ] 모든 크레이트 crates.io에서 확인
- [ ] 모든 크레이트 docs.rs에서 확인
- [ ] 설치 테스트 성공
- [ ] GitHub Release 생성
- [ ] 블로그/SNS 공지 (선택사항)

## 참고 문서

- [QUICK_PUBLISH.md](QUICK_PUBLISH.md) - 빠른 참조
- [PUBLISHING.md](PUBLISHING.md) - 상세 가이드
- [PUBLISHING_CHECKLIST.md](PUBLISHING_CHECKLIST.md) - 체크리스트
