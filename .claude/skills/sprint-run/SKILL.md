---
name: sprint-run
description: "Execute the current sprint automatically. PM agent reads PLAN.md, delegates tasks to sub-agents, auto-commits, handles errors, and updates progress. Use to start autonomous development."
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash, Edit, Write, Task, WebSearch, WebFetch
model: opus
---

# Sprint Run Skill

Execute the current sprint plan autonomously.

## Current Plan
!`cat PLAN.md 2>/dev/null || echo "No PLAN.md found"`

## Current Progress
!`cat PROGRESS.md 2>/dev/null || echo "No PROGRESS.md found"`

## Execution Protocol

### Phase 1: Initialization
1. Read PLAN.md and PROGRESS.md
2. Identify next unblocked, uncompleted task
3. Verify prerequisites are met

### Phase 2: Task Execution Loop
For each pending task:

1. **Mark in-progress** in PROGRESS.md
2. **Delegate** based on task type:
   - `research:` → Launch researcher agent (background)
   - `implement:` → Launch issue-developer agent
   - `review:` → Launch code-reviewer-kr agent
   - `docs:` → Launch tech-writer agent
   - `test:` → Run `cargo test` directly
3. **Verify** result:
   - `cargo build` must succeed
   - `cargo test` must pass
   - `cargo clippy` must be clean
4. **Auto-commit** if verification passes
5. **Update** PROGRESS.md and PLAN.md
6. **Error recovery** if any step fails (max 3 retries)

### Phase 3: Sprint Completion
1. Generate sprint summary in PROGRESS.md
2. Update PLAN.md with next sprint preview
3. Run `/issue-sync` to check for new issues

## Error Recovery

```
IF build fails:
  → Parse error message
  → Fix compilation error
  → Retry build (max 3)

IF test fails:
  → Identify failing test
  → Read test code and implementation
  → Fix implementation (not the test)
  → Retry test

IF clippy fails:
  → Run `cargo clippy --fix`
  → Verify build still passes

IF unrecoverable:
  → Log to PROGRESS.md as blocker
  → Skip task, continue to next
  → Alert: "블로커 발생: <description>"
```

## Auto-Commit Format

```
<type>(<scope>): <description>

[sprint:<sprint_name>] [task:<task_id>]
```

Types: feat, fix, refactor, docs, test, perf
