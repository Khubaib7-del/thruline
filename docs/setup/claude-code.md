# Thruline + Claude Code

Claude Code is Thruline's deepest integration: enforced review-queue delivery, decision injection on every prompt, and a context-health statusline. Two ways to set up — the plugin does everything in one command.

## Option A: plugin (recommended, terminal CLI)

```
npm install -g thruline
```

Then inside a Claude Code session:

```
/plugin marketplace add Khubaib7-del/thruline
/plugin install thruline@thruline
```

Choose "Install for you (user scope)". Restart the session. Done — hooks, MCP tools, and the `/thruline:note`, `/thruline:decide`, `/thruline:status`, `/thruline:steer` commands are live in every project.

## Option B: manual (works in the desktop app too)

```
npm install -g thruline
cd your-project
thruline init
thruline setup claude-code            (dry run — shows exactly what it will write)
thruline setup claude-code --apply
```

Add `--statusline` before `--apply` if you also want the context-health line (terminal CLI only — the desktop app doesn't render statuslines; use `thruline context` there instead).

## Turn it on per project

```
thruline init
thruline decide "your first decision" --lock
```

## The core loop to try

1. Give Claude Code a task.
2. While it works, from a second terminal: `thruline note "an idea you just had"`.
3. Don't interrupt. When it finishes, it receives your notes as review comments and addresses them — automatically, enforced by the Stop hook.

Locked decisions ride along with every prompt you send; if memory files were edited outside Thruline (e.g. a `git pull`), they're quarantined until you review and run `thruline trust` — that's a security feature.

## Troubleshooting

- **`thruline` not recognized** — fresh terminal; check `npm config get prefix` is on PATH.
- **Hooks don't fire** — hooks load at session start: start a *new* session after installing. First fire may show a one-time permission prompt — allow it.
- **Notes delivered twice** — you probably have both the plugin and a manual `setup` config in one project. Remove the hooks block from `.claude/settings.local.json`.
- **"not trusted on this machine" warning** — memory changed outside Thruline. Review with `thruline list`, approve with `thruline trust`.
- **Plugin update doesn't pick up new commands** — the marketplace caches a local copy of the repo; refresh it first: `/plugin marketplace update thruline`, then `/plugin update thruline@thruline`, then start a new session. Last resort: uninstall plugin → remove marketplace → re-add → reinstall.
