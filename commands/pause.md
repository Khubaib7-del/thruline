---
description: Graceful pause — wrap to a save point before stepping away
allowed-tools: Bash(thruline snapshot:*), Bash(thruline render:*)
---

The user is stepping away (namaz, errands, break). Do NOT start any new work. Instead:

1. Finish the current atomic unit of work (complete the file you're editing, close the function, finish the test — don't leave half-written code).
2. Call the thruline MCP tool `save_snapshot` with a summary of where things stand and what you were about to do next. Include unfinished items as todos.
3. Run `thruline render` so other agents can pick up if needed.
4. Tell the user in 2 lines: what's saved, what to do when they're back (`/thruline:restore` to resume).

Do NOT ask questions, do NOT start new tasks, do NOT continue exploratory work. Save and stop.

User context, if any: $ARGUMENTS
