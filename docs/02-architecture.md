# Architecture

## Overview: one Rust binary, three integration surfaces

```
                      ┌──────────────────────────────┐
                      │        agentos (Rust)        │
                      │                              │
  Claude Code ──MCP──▶│  agentos mcp        (stdio)  │
  Cursor      ──MCP──▶│                              │
  Copilot     ──MCP──▶│  agentos hook <event>        │◀── Claude Code hooks
  Codex       ──MCP──▶│                              │
  Gemini CLI  ──MCP──▶│  agentos statusline          │◀── Claude Code statusline
                      │                              │
                      │  agentos note / decide /     │◀── user, from any terminal
                      │  snapshot / init / render    │
                      └──────────────┬───────────────┘
                                     │
                             ┌───────▼────────┐
                             │   .agentos/    │  plain markdown + JSON
                             │  (per project) │  the single source of truth
                             └───────┬────────┘
                                     │ render
                     ┌───────────────┼───────────────┐
                     ▼               ▼               ▼
                 AGENTS.md      CLAUDE.md      .cursor/rules
```

**Why Rust:** single static binary (no runtime for users to install), ~instant startup (hooks fire constantly — a Node hook pays interpreter startup every time), official MCP SDK exists (`rmcp`), and the same binary serves every role via subcommands.

**The one non-Rust component (post-MVP):** the VS Code panel for threaded replies must be TypeScript (VS Code API requirement). Pattern: thin TS shell rendering UI, all logic in the Rust binary (same model as rust-analyzer, Biome, Ruff).

## How the MCP server runs (there is no "deployment")

For coding agents, MCP servers are **local child processes**, not deployed services:

1. User installs the binary (cargo install / GitHub Releases / npm wrapper that downloads the platform binary).
2. `agentos init` registers it in each agent's config (with per-file consent — see security doc):
   - Claude Code: `.mcp.json` or `claude mcp add agentos -- agentos mcp`
   - Cursor: `.cursor/mcp.json`
   - VS Code / Copilot: `.vscode/mcp.json`
   - Codex: `~/.codex/config.toml`
   - Gemini CLI: `~/.gemini/settings.json`
3. On startup the agent spawns `agentos mcp` and speaks JSON-RPC over stdin/stdout: `initialize` handshake → `tools/list` discovery → the model calls tools mid-conversation.

A remote MCP transport (HTTP) only enters the picture for team-shared memory, post-v1.

## MCP tool surface (v1)

| Tool | Purpose |
|---|---|
| `get_decisions` | Return locked + active decisions (agent reads before proposing architecture) |
| `log_decision` | Record a decision with rationale; optionally lock |
| `check_conflict` | Given a proposal, return any conflicting locked decision |
| `get_review_queue` | Return pending review notes |
| `resolve_review_note` | Mark a note addressed (with what was done) |
| `save_snapshot` | Persist session state (summary, TODOs, architecture, open questions) |
| `get_latest_snapshot` | Restore context in a fresh session / different agent |

## Claude Code deep integration

- **`agentos hook stop`** (Stop hook): if the review queue is non-empty, return `{"decision": "block", "reason": "<queued notes as review comments>"}` so the agent addresses them before finishing. Must honor `stop_hook_active` in the hook input to prevent infinite loops (a Stop hook that blocks re-triggers Stop when the agent finishes again).
- **`agentos hook prompt`** (UserPromptSubmit hook): inject locked decisions as additional context on every user prompt.
- **`agentos statusline`**: receives session JSON on stdin (includes transcript path); parses the transcript JSONL for token usage → renders context %, estimated prompts remaining, usage reset time.
- **Packaging**: also ship as a Claude Code **plugin** (bundles hook config + MCP registration into one install command) while raw configs keep other agents supported.

## State layout (`.agentos/` in the project root)

```
.agentos/
├── decisions.md        # human-readable decision log (append-only, timestamped)
├── decisions.json      # structured mirror for tooling (lock status, ids)
├── review-queue.json   # pending notes: text, timestamp, status
├── snapshots/
│   └── 2026-07-02T14-30.md
└── config.toml         # per-project settings (which files to render, redaction on/off)
```

Plain text on purpose: the user can read, edit, and version everything without our tool (see design principle 4). `agentos render` regenerates the agent-facing files (`AGENTS.md` section, etc.) from this source of truth — marked regions only, never clobbering user content.

## Per-agent feasibility matrix

| Capability | Claude Code | Cursor | Copilot (VS Code) | Codex | Gemini CLI |
|---|---|---|---|---|---|
| Read project memory (files) | ✅ CLAUDE.md/AGENTS.md | ✅ rules | ✅ instructions | ✅ AGENTS.md | ✅ GEMINI.md |
| MCP tools | ✅ | ✅ | ✅ | ✅ | ✅ |
| Review queue, non-interrupting | ✅ Stop hook | ⚠️ rules-file instruction to poll MCP (best effort) | ⚠️ same | ⚠️ same | ⚠️ same |
| Decision injection every prompt | ✅ UserPromptSubmit hook | ⚠️ via rules file (static) | ⚠️ same | ⚠️ same | ⚠️ same |
| Context % / reset timer | ✅ CLI statusline (⚠️ desktop app doesn't render statuslines — `agentos context` on demand instead) | ❌ not exposed | ❌ not exposed | ❌ | ⚠️ partial |
| Threaded replies | ❌ needs own UI | ❌ needs own UI | ❌ needs own UI | ❌ | ❌ |

Legend: ✅ full · ⚠️ degraded/best-effort · ❌ not technically possible today. This matrix is the honest basis for marketing claims.

## Crate layout

```
agentos/
├── Cargo.toml            # workspace
├── crates/
│   ├── agentos-core/     # state model, decisions, queue, snapshots, render
│   ├── agentos-mcp/      # rmcp server exposing core as tools
│   ├── agentos-hooks/    # Claude Code hook I/O (serde over stdin/stdout)
│   └── agentos-cli/      # the `agentos` binary: subcommand dispatch
└── docs/
```

## Key technical risks (non-security)

1. **Context-meter accuracy.** Token counts from transcript JSONL are post-hoc; estimates of "prompts remaining" are heuristic. Ship as estimate, label as estimate.
2. **Stop-hook loops.** Blocking on Stop re-fires Stop. Mitigation: respect `stop_hook_active`, cap deliveries per session.
3. **Agent behavior drift.** Hook/config schemas of third-party agents change without notice. Mitigation: integration tests per agent, versioned adapters, graceful degradation to file-based memory.
4. **Windows quirks.** Paths with spaces (this very machine: `C:\Users\T L S`), CRLF, PowerShell vs bash quoting in hook commands. CI must test Windows + macOS + Linux.
5. **Rules-file duplication.** Rendering into multiple agent files risks drift. Mitigation: single source of truth + marked managed regions + `agentos render` as the only writer.
