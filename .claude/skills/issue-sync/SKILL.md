---
name: issue-sync
description: "Sync GitHub issues, enhance with technical analysis, and update PLAN.md. Use periodically to keep project aligned with GitHub issues."
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash, Edit, Write
---

# Issue Sync Skill

Synchronize GitHub issues with project planning.

## Current issues
!`gh issue list --state open --json number,title,body,labels,assignees,createdAt,updatedAt 2>/dev/null || echo "[]"`

## Protocol

1. Parse the issue list above
2. For each issue without technical analysis comment:
   a. Read the issue body (Korean)
   b. Analyze which crates/files are affected
   c. Add a technical analysis comment via `gh issue comment`
   d. Add appropriate labels via `gh issue edit`
3. Update `PLAN.md` with new tasks derived from issues
4. Update `PROGRESS.md` with sync timestamp
5. Preserve original issue content (Korean) - only ADD comments, never edit the original

## Label Creation (if needed)

```bash
gh label create "priority:P0" --color "d73a4a" --description "Critical" 2>/dev/null || true
gh label create "priority:P1" --color "e99695" --description "High" 2>/dev/null || true
gh label create "priority:P2" --color "fbca04" --description "Medium" 2>/dev/null || true
gh label create "priority:P3" --color "0e8a16" --description "Low" 2>/dev/null || true
gh label create "status:ready" --color "0075ca" 2>/dev/null || true
gh label create "status:in-progress" --color "cfd3d7" 2>/dev/null || true
gh label create "type:feat" --color "a2eeef" 2>/dev/null || true
gh label create "type:fix" --color "d876e3" 2>/dev/null || true
```
