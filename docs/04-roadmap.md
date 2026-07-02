# Roadmap

Strategy: **vertical first (Claude Code), horizontal later.** Every milestone ships something a real user can install and feel.

## Milestone 0 — Foundation (docs done, then scaffold)
- [x] Vision, architecture, security threat model, decision log
- [x] Rust workspace scaffold (`agentos-core`, `agentos-cli`) — builds on stable 1.96, 4 tests passing
- [x] `.agentos/` state model: decisions + review queue, atomic JSON writes + rendered decisions.md (integrity hashes → M1)
- [x] CLI: `agentos init` (state dir), `agentos decide --why --lock`, `agentos note`, `agentos list` (agent-config registration + dry-run diff flow → M1)
- [ ] CI: Windows + macOS + Linux, cargo audit/deny

## Milestone 1 — Claude Code vertical (the "install this" moment)
- [ ] `agentos hook stop`: review-queue delivery with `stop_hook_active` loop guard
- [ ] `agentos hook prompt`: locked-decision injection (framed as data, length-capped)
- [ ] `agentos mcp`: rmcp stdio server — `get_decisions`, `log_decision`, `check_conflict`, `get_review_queue`, `resolve_review_note`
- [ ] Secret redaction on all state writes (security Finding 3)
- [ ] Package as a Claude Code plugin (one-command install)
- [ ] Record the demo GIF (the review-queue moment — see docs/05, it's the core marketing asset)
- **Demo:** queue three notes while Claude Code builds a feature; watch it address them as review comments when it finishes.

## Milestone 2 — Context health + snapshots
- [ ] `agentos statusline`: context %, estimated prompts remaining, reset timer (labeled as estimates)
- [ ] `agentos snapshot` / `get_latest_snapshot` MCP tool
- [ ] Snapshot → fresh-session restore flow ("Ctrl+Shift+S" experience from the vision doc)

## Milestone 3 — Cross-agent horizontal
- [ ] `agentos render`: managed regions in AGENTS.md / CLAUDE.md / .cursor/rules / GEMINI.md
- [ ] MCP registration + tested integration for Cursor, Copilot (VS Code), Codex, Gemini CLI
- [ ] Trust-on-first-use + change-detection flow (security Finding 1)
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
