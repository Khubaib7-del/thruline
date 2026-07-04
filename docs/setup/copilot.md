# Thruline + GitHub Copilot (VS Code)

Copilot's agent mode reads workspace instruction files and supports MCP servers configured per workspace.

## 1. Install and initialize

```
npm install -g thruline
cd your-project
thruline init
thruline decide "your first decision" --lock
thruline render
```

Copilot picks up `AGENTS.md` as workspace context.

## 2. Register the MCP server

Create `.vscode/mcp.json`:

```json
{
  "servers": {
    "thruline": { "type": "stdio", "command": "thruline", "args": ["mcp"] }
  }
}
```

Use the absolute binary path if the server fails to start.

## 3. Verify

In agent mode, ask: *"what are this project's recorded decisions?"*

## Expectations

Memory, locks, conflict checks, and snapshots work fully. Review queue is best-effort; context usage is not exposed. Details: [what works where](what-works-where.md).
