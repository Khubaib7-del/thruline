# Roadmap

Strategy: **vertical first (Claude Code), horizontal later.** Every milestone ships something a real user can install and feel.

## Milestone 0 — Foundation (docs done, then scaffold)
- [x] Vision, architecture, security threat model, decision log
- [x] Rust workspace scaffold (`agentos-core`, `agentos-cli`) — builds on stable 1.96, 4 tests passing
- [x] `.agentos/` state model: decisions + review queue, atomic JSON writes + rendered decisions.md (integrity hashes → M1)
- [x] CLI: `agentos init` (state dir), `agentos decide --why --lock`, `agentos note`, `agentos list` (agent-config registration + dry-run diff flow → M1)
- [x] CI: Windows + macOS + Linux (fmt + clippy + test) plus security audit job

## Milestone 1 — Claude Code vertical (the "install this" moment)
- [x] `agentos hook stop`: review-queue delivery with `stop_hook_active` loop guard — verified end-to-end with simulated hook input
- [x] `agentos hook prompt`: locked-decision injection (framed as data, length-capped)
- [x] `agentos setup claude-code`: dry-run by default, `--apply` to write, idempotent, absolute exe path (security findings 5 & 8)
- [x] `agentos mcp`: stdio MCP server — `get_decisions`, `log_decision`, `get_review_queue`, `resolve_review_note` (hand-rolled JSON-RPC instead of rmcp: smaller dependency tree, full input control; `check_conflict` + snapshot tools → M2)
- [x] Secret redaction on all state writes (security Finding 3) — 11 credential formats (API keys, tokens, JWTs, URL passwords, private keys, generic assignments), tested against false positives
- [x] Package as a Claude Code plugin (repo = marketplace; hooks + MCP + /agentos:note, /agentos:decide, /agentos:status slash commands) — ✅ verified 2026-07-03: owner installed via CLI (`/plugin marketplace add` → user scope), slash commands registered
- [ ] Record the demo GIF (the review-queue moment — see docs/05, it's the core marketing asset)
- **Demo:** queue notes while Claude Code builds a feature; watch it address them as review comments when it finishes.
  ✅ **Verified live 2026-07-03** — owner ran the dogfood test: both queued notes delivered and addressed by the Stop hook, locked decision cited unprompted via UserPromptSubmit. The wedge feature works in production.

## Milestone 2 — Context health + snapshots
- [x] `agentos statusline`: context % + tokens, estimated prompts remaining, usage-window reset estimate — all labeled as estimates; declines to guess rather than fabricate (unknown model limit → no %; unknowable window boundary → no reset time); `setup claude-code --statusline` registers it
  - Verified 2026-07-03: **terminal CLI only.** The Claude Code desktop app does not render custom statuslines (tested: correct project folder, updated app, statusLine in settings.local.json — nothing shown). Documented in the feasibility matrix.
- [x] `agentos context` command: same context-health info printed on demand — covers desktop-app users the statusline can't reach (verified live against a real session transcript)
- [x] `agentos snapshot` / `agentos restore` CLI + `save_snapshot` / `get_latest_snapshot` MCP tools — snapshots bundle summary, TODOs, open questions, decisions, and open notes; redacted; clock-derived filenames (no path traversal)
- [x] `check_conflict` MCP tool — keyword heuristic highlights related locked decisions, always returns the full locked list for the model to judge
- [x] Snapshot → fresh-session restore flow: `agentos restore` prints the latest snapshot for pasting into any agent; MCP-connected agents call `get_latest_snapshot` themselves

## Milestone 3 — Cross-agent horizontal
- [x] `agentos render`: managed region in AGENTS.md (the emerging cross-agent standard, read by Cursor/Codex/Copilot/Gemini CLI) — user content preserved, region replaced idempotently; per-agent files (.cursor/rules, GEMINI.md) only if demand appears
- [x] `/agentos:steer` slash command: interrupt correction processed with review semantics — log it, audit work done so far against it, fix conflicts, then continue (owner's idea, 2026-07-03)
- [ ] MCP registration + tested integration for Cursor, Copilot (VS Code), Codex, Gemini CLI
- [x] Trust-on-first-use + change-detection flow (security Finding 1) — fingerprints in `~/.agentos/trust.json` (outside the repo, so repo tampering can't self-bless); external edits quarantine memory from the prompt hook and MCP read tools until `agentos trust`; concurrent-writer race fixed with lock file + atomic replace
- [ ] Public release: signed binaries, checksums, npm wrapper, SECURITY.md
- [ ] Landing page + docs site live, soft launch → Product Hunt (full sequence in docs/05)
- **This is the launch milestone** (name decided before this ships).

## Milestone 4 — Owned UI (differentiator)
- [ ] VS Code extension (TS shell over the Rust binary): review-queue panel, decision timeline
- [ ] **Threaded replies**: select agent output → quoted-reply prompt composition
- [ ] Fuzzing of parsers before tagging 1.0

## Post-1.0 (exploratory, unscheduled)
- Team-shared memory (requires the auth/signing work in security Finding 9)
- Smart context packing (choose most relevant files/memory for the next session)
- Architecture map generation from memory + repo
- Session analytics (local, opt-in only)

## Publicity
Silent build — no milestone posts (owner's decision, see docs/05). One polished public reveal after the owner personally verifies the product. The repo itself dogfoods AgentOS from Milestone 1 onward — its own decisions live in `.agentos/`.
