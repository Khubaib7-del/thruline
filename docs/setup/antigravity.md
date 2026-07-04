# Thruline + Antigravity

Antigravity (IDE and CLI) reads `AGENTS.md` natively (v1.20.3+) and supports MCP through the shared configuration used across the Antigravity suite.

## 1. Install and initialize

```
npm install -g thruline
cd your-project
thruline init
thruline decide "your first decision" --lock
thruline render
```

Tip: you can also paste the setup into the agent chat and let it run the commands itself — Antigravity handles this well.

## 2. Register the MCP server

In Antigravity's MCP settings (see antigravity.google/docs/mcp), add a stdio server:

- name: `thruline`
- command: `thruline` (absolute path from `where thruline` if it fails to start)
- args: `mcp`

One config covers the IDE and the CLI.

## 3. Verify

Ask the agent: *"what are this project's recorded decisions?"* Then try proposing something that contradicts a locked decision — it should flag the conflict.

## Expectations

Memory, locks, conflict checks, and snapshots work fully. In live testing, Antigravity respected locked decisions unprompted and saved a session snapshot on its own initiative. The review queue is best-effort (no hook system); context usage is not exposed. Details: [what works where](what-works-where.md).
