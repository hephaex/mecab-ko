Read PLAN.md and PROGRESS.md, then give a concise status summary including:

1. Current sprint name and goal
2. Tasks completed vs remaining
3. Any blockers
4. Next task to execute
5. Overall project health (build/test/clippy status)

Run `cd rust && cargo test --workspace 2>&1 | tail -5` and `cd rust && cargo clippy --workspace 2>&1 | tail -3` to verify current state.
