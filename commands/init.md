---
description: Set up thruline in this project (state dir + first decisions + AGENTS.md)
allowed-tools: Bash(thruline init:*), Bash(thruline decide:*), Bash(thruline render:*), Bash(thruline list:*)
---

Set up thruline in this project:

1. Run `thruline init` (if it says already initialized, that's fine — continue).
2. If the user stated decisions to record, record each with `thruline decide "<decision>" --why "<reason>" --lock` (lock only what they marked final): $ARGUMENTS
3. Run `thruline render` so other agents see the decisions too.
4. Run `thruline list` and show the user the resulting state in a compact summary.
