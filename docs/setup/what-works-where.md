# What works where — the honest compatibility page

Thruline works with every major coding agent, but agents differ in what they
let outside tools do. This page tells you exactly what to expect, so nothing
surprises you. Short version: **memory works everywhere; the extra polish is
Claude Code-only.**

## Works with EVERY agent (Claude Code, Cursor, Antigravity, Command Code, Codex, Gemini CLI, Copilot)

- **Project memory** — your decisions in `AGENTS.md` (via `thruline render`); every agent reads it
- **Decision locks** — agents are instructed to flag conflicts instead of silently deviating
- **Review queue** — `thruline note` from any terminal; the agent checks the queue when finishing
- **Snapshots** — `thruline snapshot` / `thruline restore` to carry context between sessions *and between agents*
- **MCP tools** — `get_decisions`, `log_decision`, `check_conflict`, queue and snapshot tools (one-time server registration per agent)
- **Secret redaction & tamper quarantine** — always on; they protect the files, not the agent

## Claude Code only — and why

| Feature | Why only there |
|---|---|
| **Enforced review-queue delivery** (notes arrive automatically the moment the agent finishes) | Requires hooks; Claude Code is the only agent with a hook system. Everywhere else, delivery is "the agent follows AGENTS.md instructions" — usually works, occasionally needs a reminder: *"check the thruline review queue"* |
| **Decision injection on every prompt** | Same hook system. Elsewhere, decisions reach the agent through AGENTS.md (re-run `thruline render` after new decisions) |
| **`/thruline:` slash commands** | Claude Code's plugin system. Other agents' slash menus are closed to outside tools — pressing `/` there will never show Thruline entries. Equivalent everywhere: ask the agent to run `thruline note "..."`, or run it yourself |
| **Context-health statusline** (context %, prompts left, reset timer) | Needs both a statusline surface and readable session data; only the Claude Code terminal CLI has both. The desktop app and all other agents: run `thruline context` in a terminal instead (Claude Code projects only — other agents don't expose usage data at all) |

## One deliberate omission

There is no `/thruline:trust` slash command, on purpose: trust approval exists
so a *human* reviews memory that may have been tampered with. If the agent
could approve it, a prompt-injected agent could bless its own poisoned memory.
`thruline trust` is terminal-only, forever.

## VS Code panel — our own UI surface

Since no agent lets outside tools draw inside their chat window, we built our own:
the **Thruline VS Code panel** is a sidebar that works alongside any agent running in VS Code.

| Feature | How |
|---|---|
| Live review queue | Auto-refreshes via file watcher |
| Threaded replies | Reply to a specific agent response (WhatsApp-style) |
| One-click resolve | Mark notes done from the sidebar |
| Decision viewer | See locks and rationale at a glance |
| Graceful pause | Command palette → "Thruline: Pause" |

Install: `ext install khubaib7.thruline-panel` — or see the [full setup guide](vscode-panel.md).

## `/thruline:pause` — graceful mid-task exit

Distinct from the emergency Esc key. When you trigger `/thruline:pause`:

1. The agent finishes the current atomic unit (completes the file, closes the function)
2. Snapshots state (decisions, queue, summary)
3. Renders AGENTS.md for other agents
4. Gives a 2-line sign-off — what's saved, how to resume

Use it before namaz, errands, end of day. Resume with `/thruline:restore`.

## Things no tool can do (not just us)

- Add commands, buttons, or panels **inside** another agent's chat interface — every agent's UI is closed to outsiders (that's why we built our own sidebar)
- Read context/token usage from Cursor, Copilot, Codex, Antigravity, or Command Code — they don't expose it
- Force an agent to obey instructions — locks and queues make good behavior easy and deviations visible, not impossible

If a vendor opens a new door (like Antigravity adding AGENTS.md support in
March 2026), we build on it — this page tracks the current truth. Found it
out of date? Open an issue.
