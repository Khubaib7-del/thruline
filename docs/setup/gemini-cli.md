# Thruline + Gemini CLI

Gemini CLI reads `GEMINI.md` for project context and supports MCP servers in its settings.

## 1. Install and initialize

```
npm install -g thruline
cd your-project
thruline init
thruline decide "your first decision" --lock
thruline render
```

`render` writes to `AGENTS.md`. Point Gemini at it by adding one line to your project's `GEMINI.md`:

```
Read AGENTS.md for recorded project decisions and the review-queue workflow.
```

## 2. Register the MCP server

Add to `~/.gemini/settings.json`:

```json
{
  "mcpServers": {
    "thruline": { "command": "thruline", "args": ["mcp"] }
  }
}
```

## 3. Verify

Ask: *"what are this project's recorded decisions?"*

## Expectations

Memory, locks, conflict checks, and snapshots work fully. Review queue is best-effort. Gemini CLI shows its own context percentage natively, so Thruline adds no gauge there. Details: [what works where](what-works-where.md).
