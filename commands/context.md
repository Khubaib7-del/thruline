---
description: Show context health for this project's latest Claude Code session
allowed-tools: Bash(thruline context:*)
---

Run `thruline context` and relay the output compactly: context used, estimated prompts left, and the usage-window reset estimate if shown. If context usage is high (over ~70%), suggest saving a snapshot (`/thruline:snapshot`) and starting a fresh session.
