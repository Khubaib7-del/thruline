# Thruline + Codex CLI

Codex reads `AGENTS.md` natively and supports MCP servers via its config file.

## 1. Install and initialize

```
npm install -g thruline
cd your-project
thruline init
thruline decide "your first decision" --lock
thruline render
```

## 2. Register the MCP server

Add to `~/.codex/config.toml`:

```toml
[mcp_servers.thruline]
command = "thruline"
args = ["mcp"]
```

Use the absolute binary path if `thruline` alone fails to resolve.

## 3. Verify

Ask Codex: *"what are this project's recorded decisions?"*

## Expectations

Memory, locks, conflict checks, and snapshots work fully. Review queue is best-effort via the `AGENTS.md` instruction; no context gauge (usage not exposed). Details: [what works where](what-works-where.md).
