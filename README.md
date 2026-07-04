# Thruline

**A companion layer for AI coding agents.** It doesn't generate code — it improves the collaboration between developers and agents like Claude Code, Cursor, GitHub Copilot, Codex, Gemini CLI, and Antigravity.

> Status: early development (July 2026). Core CLI, Claude Code hooks, and MCP server are working; see Usage below.

## The one-line pitch

Every AI coding agent forgets your decisions, can't be steered mid-task without derailing, and hides its context health. Thruline is a single local binary that fixes all three — for every agent at once.

![Thruline demo: queue ideas while the agent works, they arrive as review comments when it finishes](docs/assets/thruline-hero.gif)

*Real session, real hooks: two ideas queued from a terminal mid-task; the agent addressed both the moment it finished — no interruption. ([69-second full version](docs/assets/thruline-demo.mp4))*

## Core features (v1 scope)

| Feature | What it does |
|---|---|
| **Review Queue** | Drop ideas while the agent works; they're delivered as review comments when it finishes — no interruption, no derail |
| **Decision Log & Locks** | Architectural decisions become persistent, injected into every prompt; conflicts get flagged |
| **Context Health** | Live context-window %, degradation estimate, usage-reset timer (Claude Code) |
| **Session Snapshots** | One command captures state (decisions, TODOs, architecture) for the next session or a different agent |
| **Cross-Agent Memory** | File-based memory readable by every agent via `AGENTS.md` conventions + MCP |

## Install

```
npm install -g thruline     # any platform — downloads the release binary, SHA-256 verified
cargo install thruline      # or build from source
```

Binaries are also on [GitHub Releases](https://github.com/Khubaib7-del/thruline/releases) with checksums and Sigstore provenance.

## Set up with your agent

Pick your guide — each one is complete, honest about what works, and has troubleshooting:

| Your agent | Guide |
|---|---|
| **Claude Code** (CLI or desktop app) | [setup guide](docs/setup/claude-code.md) — deepest integration: enforced review queue, decision injection, statusline |
| **Cursor** | [setup guide](docs/setup/cursor.md) |
| **Antigravity** | [setup guide](docs/setup/antigravity.md) |
| **Command Code** | [setup guide](docs/setup/command-code.md) |
| **Codex CLI** | [setup guide](docs/setup/codex.md) |
| **Gemini CLI** | [setup guide](docs/setup/gemini-cli.md) |
| **Copilot (VS Code)** | [setup guide](docs/setup/copilot.md) |

Before assuming a feature exists in your agent, read **[what works where](docs/setup/what-works-where.md)** — the honest compatibility page (e.g. slash commands and enforced queue delivery are Claude Code-only; memory and MCP tools work everywhere).

(These guides will move to the docs site at launch; the links will keep working.)

**Versioning note:** the binary (GitHub releases, npm, crates.io) and the Claude Code plugin (`.claude-plugin/plugin.json`) are versioned separately — plugin numbers move faster because slash-command changes don't need a new binary. Both align again at `v0.1.0`.

## Usage

```
thruline init                          create the .thruline state directory in the current project
thruline decide "DB: PostgreSQL"       record a project decision
        --why "team knows it"         rationale stored alongside the decision
        --lock                        agents get warned on conflicting proposals
thruline note "check error handling"   queue a review note; delivered when the agent finishes its task
thruline list                          show recorded decisions and pending review notes
thruline list --json                   same data as JSON, for scripts (includes why/status/timestamps)
thruline render                        write decisions into AGENTS.md so Cursor/Codex/Copilot see them
thruline context                       context health of the latest Claude Code session in this project
thruline snapshot "summary" --todo t   save a session snapshot (decisions + open notes bundled in)
thruline restore                       print the latest snapshot — paste into any agent to restore context
thruline setup claude-code --apply     wire the Claude Code hooks into .claude/settings.local.json
```

The plain-text `list` output is human-oriented; scripts should use `--json`, whose schema is the stable interface.

### Install as a Claude Code plugin

With the `thruline` binary on PATH (`cargo install --path crates/thruline-cli`), the repo doubles as a plugin marketplace:

```
/plugin marketplace add Khubaib7-del/thruline
/plugin install thruline@thruline
```

This wires the Stop/UserPromptSubmit hooks, registers the MCP server, and adds `/thruline:note`, `/thruline:decide`, and `/thruline:status` slash commands. Manual alternative: `thruline setup claude-code --apply` per project.

## Documentation

- [Architecture](docs/architecture.md) — single Rust binary, MCP + hooks + statusline, per-agent feasibility matrix
- [Security & Threat Model](docs/security.md) — findings on prompt injection, secrets, supply chain, and how we mitigate them
- [DECISIONS.md](DECISIONS.md) — the project's own decision log (we dogfood our own concept)

## Design principles

1. **Local-first.** No cloud, no telemetry, no account for v1. Everything lives in the repo and on the user's machine.
2. **Agent-agnostic core, agent-specific depth.** Files + MCP work everywhere; Claude Code hooks give the deepest experience first.
3. **Honest capability claims.** If an agent doesn't expose something (e.g. Cursor's context usage), we say so instead of faking it.
4. **The user owns the memory.** Plain markdown + JSON, readable and editable without our tool.
