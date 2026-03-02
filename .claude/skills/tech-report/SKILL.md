---
name: tech-report
description: "Generate a technical report (LessonLearn) for a completed PR. Includes learning points and reference links."
user-invocable: true
argument-hint: <PR number>
allowed-tools: Read, Grep, Glob, Bash, Write, WebSearch, WebFetch
---

# Tech Report Skill

Generate a technical report for a completed PR.

## PR Context
!`gh pr view $ARGUMENTS --json number,title,body,files,commits,labels,closedAt 2>/dev/null || echo "No PR found"`

## Protocol

1. Parse PR details from above
2. Read all changed files to understand implementation
3. Identify key technical decisions
4. Search web for 3 relevant reference links
5. Write report to `docs/LessonLearn/PR-<NUMBER>-<slug>.md`

## Report Template

```markdown
# PR #<N>: <Title>

**날짜**: <date>
**관련 이슈**: #<issues>

## 요약
<2-3 sentences>

## 기술적 변경사항
<What changed and why>

## 학습 포인트 (Learning Points)
1. **<Topic>**: <1-line lesson>
2. **<Topic>**: <1-line lesson>
3. **<Topic>**: <1-line lesson>

## 참고 자료 (References)
- [<Title>](<URL>) - <1-line why useful>
- [<Title>](<URL>) - <1-line why useful>
- [<Title>](<URL>) - <1-line why useful>
```
