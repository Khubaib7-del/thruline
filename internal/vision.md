# Vision & Product Definition

## The problem

AI coding agents are powerful but collaborate badly over long projects:

1. **Context loss.** Decisions made in session 1 ("we use PostgreSQL, Prisma, OAuth") are forgotten by session 5. The user repeats themselves or the agent drifts architecturally.
2. **No mid-implementation steering.** While the agent works, the user thinks of improvements ("debounce that search"). Their only options are to interrupt (derailing the current task) or forget the idea.
3. **Invisible context health.** Users don't know when the context window is nearly full, when output quality is degrading, or when their usage limit resets.
4. **No cross-agent continuity.** Moving a project from Claude Code to Cursor to Codex means starting from zero each time.
5. **Linear conversations.** There's no way to reply to a *specific* agent statement (like a WhatsApp reply or a code-review comment) — every message addresses the whole conversation, so the agent guesses what you're referring to.

## The insight

Everyone is competing on "generate better code." Nobody owns "collaborate better with AI over a long project." Browsers got extensions, IDEs got plugins — coding agents have no companion ecosystem yet.

## What we build

Not another agent. A **local companion binary** that plugs into existing agents through the surfaces they already expose:

- **MCP** (universal — all major agents support it): tools for reading/writing decisions, review notes, snapshots.
- **Instruction files** (universal): `AGENTS.md` / `CLAUDE.md` / `.cursor/rules` rendered from a single memory source.
- **Claude Code hooks** (deep integration): review-queue delivery on Stop, decision injection on every prompt.
- **Claude Code statusline** (deep integration): live context %, reset timer.

## Feature definitions

### Review Queue (the wedge feature)
While the agent works, the user queues notes (`thruline note "use debounce here"`). When the agent finishes its current task, a Stop hook delivers the queue as review comments, one by one. The agent addresses them like PR review feedback. The user never interrupts, the agent never derails.

### Decision Log & Decision Lock
Decisions are recorded (`thruline decide "DB: PostgreSQL" --why "..."`) with timestamp and rationale. Locked decisions are injected into every prompt. If the agent proposes something conflicting ("let's use MongoDB"), the conflict is flagged against the specific locked decision. A decision timeline shows the project's engineering history.

### Context Health
Dashboard in the statusline (Claude Code): context used %, estimated prompts remaining, usage-limit reset time, recommendation to snapshot + start a new session when degradation risk is high. **Explicitly not available for Cursor/Copilot** — they expose no context data; we don't fake it.

### Session Snapshots
`thruline snapshot` captures: summary of session, architecture state, pending TODOs, open decisions, queued review notes. Restorable into any supported agent — the snapshot renders into that agent's instruction-file format.

### Cross-Agent Memory
All state lives in `.thruline/` as plain markdown + JSON. A render step keeps `AGENTS.md` (and per-agent files) in sync, so even agents with zero Thruline integration still read the project memory.

### Pause Flag (Claude Code, post-v1)
Real-life interruptions (prayer, family, errands) are expensive mid-agent-run: users either wait for completion or hope for a permission prompt to leave the session at. `thruline pause` sets a flag; a PreToolUse hook sees it and holds the agent safely before its next action until the user returns (`thruline resume`). Native `Esc`-to-interrupt already exists in Claude Code — this feature is for walking away *without* cutting the agent off mid-thought. Not implementable for other agents (no hook surface); documented honestly as Claude Code-only.

### Threaded Replies (post-MVP, differentiator)
Reply to a *specific* agent response, WhatsApp-style. Requires a UI surface we own (VS Code panel or dashboard) — we cannot inject UI into other agents' chat panels. The panel composes a quoted-reply prompt: "Re: *your suggestion to use Prisma* → use Drizzle instead, because we need raw SQL control."

## Differentiation & competitive honesty

- Memory-focused MCP servers already exist; context-meter tools for Claude Code already exist (ccusage, statusline projects). **Memory alone is not the wedge.**
- Our wedge is the **collaboration layer**: review queue + decision locks + (later) threaded replies. No existing tool does non-interrupting mid-task steering across agents.
- The product must stay useful even if the user switches agents — we design around the workflow, not any single model.

## Naming

"Thruline" is an internal codename only. Public name decided before first release (candidates: ContextKit, DevContext, AgentCompanion — availability check pending, see DECISIONS.md).
