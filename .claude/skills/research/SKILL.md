---
name: research
description: "Run parallel research on a technical topic. Searches web, gathers findings, saves to docs/research/. Use when investigating algorithms, libraries, or ecosystem."
user-invocable: true
argument-hint: <topic>
allowed-tools: Read, Grep, Glob, Task, WebSearch, WebFetch, Write
---

# Research Skill

You are conducting technical research for the MeCab-Ko project.

## Instructions

1. Parse the research topic from: $ARGUMENTS
2. Break the topic into 2-3 independent sub-topics
3. Launch parallel research agents (run_in_background: true) for each sub-topic
4. Wait for all agents to complete
5. Synthesize results into a single report
6. Save to `docs/research/<category>/<topic>.md`
7. Update PROGRESS.md with research completion

## Report Structure

```markdown
# <Topic> 조사 보고서

**날짜**: <date>
**카테고리**: algorithms|ecosystem|rust-crates|dictionary|benchmarks

## 요약 (3줄)
1. <Key finding 1>
2. <Key finding 2>
3. <Key finding 3>

## 상세 내용
### <Subtopic 1>
<Details with source links>

### <Subtopic 2>
<Details with source links>

## 학습 포인트
1. <Lesson 1>
2. <Lesson 2>
3. <Lesson 3>

## 참고 자료
- [Title](URL) - Description
- [Title](URL) - Description
- [Title](URL) - Description

## 프로젝트 적용 방안
<How to apply these findings to MeCab-Ko>
```

## Categories

- `algorithms` - Viterbi, DA Trie, CRF, lattice
- `ecosystem` - Kiwi, Nori, Lindera, mecab-ko
- `rust-crates` - Rust libraries and tools
- `dictionary` - Dictionary formats, data sources
- `benchmarks` - Performance comparisons
