# VS Code Panel

The Thruline sidebar panel gives you a persistent UI for the review queue and decisions — right next to whatever agent you're using in VS Code (Cursor, Copilot, Claude Code desktop).

## What it does

| Feature | How it works |
|---|---|
| **Live review queue** | Shows pending notes with timestamps; auto-refreshes via file watcher |
| **Threaded replies** | Reply to a specific agent response — like WhatsApp reply-to-message |
| **One-click resolve** | Mark notes as addressed without touching the terminal |
| **Decision viewer** | See all locked decisions and their rationale at a glance |
| **Add notes from VS Code** | Command palette → "Thruline: Add Review Note" or use the input in the panel |
| **Pause command** | Command palette → "Thruline: Pause" — snapshots and stops gracefully |

## Install

### From the marketplace

```
ext install khubaib7.thruline-panel
```

Or search "Thruline" in the VS Code extensions tab.

### From source

```bash
cd packages/vscode
npm install
npm run compile
```

Then press F5 in VS Code to launch the Extension Development Host, or package it:

```bash
npx vsce package
# → thruline-panel-0.1.0.vsix
# Install via: Extensions → ⋯ → Install from VSIX
```

## Requirements

- The `thruline` binary on PATH (`npm install -g thruline`)
- A `.thruline/` directory in your workspace (run `thruline init` first)

The panel activates automatically when it detects `.thruline/` in the workspace root.

## How threaded replies work

When you add a note, you can optionally attach a **thread** — a short label identifying which agent response you're reacting to (e.g. "the auth refactor commit", "search feature PR").

The agent sees both your note and the thread label when the review queue is delivered, giving it context about which specific output you're responding to.

This is the "WhatsApp reply-to-message" pattern: you're pointing at something specific rather than dropping a note into the void.

## How it compares to the CLI

| | CLI (`thruline note`) | VS Code panel |
|---|---|---|
| Add notes | ✓ from any terminal | ✓ from the sidebar |
| Thread context | not yet | ✓ built-in |
| Resolve notes | via MCP tool | ✓ one click |
| View decisions | `thruline list` | ✓ always visible |
| Works outside VS Code | ✓ | ✗ |

Both write to the same `.thruline/queue.json` — they're interchangeable. Use whichever is closer to your hands.

## Troubleshooting

**Panel doesn't appear**
- Check that `.thruline/` exists in the workspace root (`thruline init`)
- Reload the window (Ctrl+Shift+P → "Reload Window")

**Notes don't refresh**
- The file watcher triggers on `.thruline/**` changes. If you're editing queue.json directly outside VS Code, the watcher may miss it — click the refresh button or run "Thruline: Refresh Panel" from the command palette.

**Extension not activating**
- The extension only activates when `workspaceContains:.thruline` is true. Open a folder that has `.thruline/` in it.
