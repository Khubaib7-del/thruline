# Thruline + Cursor

Cursor reads Thruline's memory through `AGENTS.md` and can call its tools over MCP.

## 1. Install and initialize

```
npm install -g thruline
cd your-project
thruline init
thruline decide "your first decision" --lock
thruline render
```

`render` writes the decisions into `AGENTS.md`, which Cursor picks up as project rules. Re-run it after recording new decisions.

## 2. Register the MCP server

Create `.cursor/mcp.json` in the project (or add via Settings → MCP):

```json
{
  "mcpServers": {
    "thruline": { "command": "thruline", "args": ["mcp"] }
  }
}
```

If the server fails to start, replace `"thruline"` with the absolute path from `where thruline` (Windows) or `which thruline`.

## 3. Verify

Ask Cursor: *"what are this project's recorded decisions?"* It should answer from `AGENTS.md` or by calling `get_decisions`.

## Expectations

Memory, decision locks, conflict checks, and snapshots work fully. The review queue is best-effort: `AGENTS.md` instructs the agent to check pending notes when finishing — remind it with *"check the thruline review queue"* if needed. Cursor does not expose context usage, so there is no context gauge. Details: [what works where](what-works-where.md).
