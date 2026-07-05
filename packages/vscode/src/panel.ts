import * as vscode from "vscode";
import { ThrulineState } from "./state";

export class ThrulinePanel implements vscode.WebviewViewProvider {
  private view?: vscode.WebviewView;

  constructor(
    private readonly extensionUri: vscode.Uri,
    private readonly state: ThrulineState
  ) {}

  resolveWebviewView(webviewView: vscode.WebviewView) {
    this.view = webviewView;
    webviewView.webview.options = { enableScripts: true };
    webviewView.webview.html = this.getHtml();

    webviewView.webview.onDidReceiveMessage((msg) => {
      switch (msg.type) {
        case "addNote":
          this.state.addNote(msg.text, msg.thread).then(() => this.refresh());
          break;
        case "resolve":
          this.state.resolveNote(msg.id).then(() => this.refresh());
          break;
        case "refresh":
          this.refresh();
          break;
      }
    });
  }

  refresh() {
    if (this.view) {
      this.view.webview.html = this.getHtml();
    }
  }

  private getHtml(): string {
    const queue = this.state.getQueue();
    const decisions = this.state.getDecisions();
    const pending = queue.filter((n) => !n.resolved);
    const resolved = queue.filter((n) => n.resolved);

    return `<!DOCTYPE html>
<html>
<head>
<style>
:root {
  --bg: var(--vscode-editor-background);
  --fg: var(--vscode-editor-foreground);
  --dim: var(--vscode-descriptionForeground);
  --vio: #7c6ffb;
  --border: var(--vscode-panel-border);
  --input-bg: var(--vscode-input-background);
  --input-fg: var(--vscode-input-foreground);
  --btn-bg: var(--vscode-button-background);
  --btn-fg: var(--vscode-button-foreground);
}
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: var(--vscode-font-family); font-size: 13px; color: var(--fg); padding: 12px; }
h2 { font-size: 11px; text-transform: uppercase; letter-spacing: 0.08em; color: var(--dim); margin: 16px 0 8px; }
h2:first-child { margin-top: 0; }
.badge { display: inline-block; background: var(--vio); color: #fff; border-radius: 10px; padding: 1px 7px; font-size: 11px; margin-left: 6px; }

.note { padding: 8px 10px; border-left: 2px solid var(--vio); margin: 6px 0; border-radius: 0 4px 4px 0; background: color-mix(in srgb, var(--vio) 8%, transparent); }
.note .text { margin-bottom: 4px; }
.note .meta { font-size: 11px; color: var(--dim); display: flex; justify-content: space-between; align-items: center; }
.note .thread-tag { font-size: 10px; background: var(--vio); color: #fff; padding: 1px 6px; border-radius: 8px; }
.note.resolved { opacity: 0.5; border-left-color: var(--dim); }
.note .resolve-btn { cursor: pointer; color: var(--vio); border: none; background: none; font-size: 11px; }
.note .resolve-btn:hover { text-decoration: underline; }

.decision { padding: 6px 10px; margin: 4px 0; border-radius: 4px; border: 1px solid var(--border); }
.decision .lock { color: #e8b04b; margin-right: 4px; }
.decision .why { font-size: 11px; color: var(--dim); margin-top: 2px; }

.input-row { display: flex; gap: 6px; margin-top: 12px; }
.input-row input { flex: 1; background: var(--input-bg); color: var(--input-fg); border: 1px solid var(--border); border-radius: 4px; padding: 6px 8px; font-size: 12px; }
.input-row button { background: var(--btn-bg); color: var(--btn-fg); border: none; border-radius: 4px; padding: 6px 12px; cursor: pointer; font-size: 12px; white-space: nowrap; }

.thread-input { margin-top: 6px; }
.thread-input input { width: 100%; background: var(--input-bg); color: var(--input-fg); border: 1px solid var(--border); border-radius: 4px; padding: 4px 8px; font-size: 11px; }
.thread-input label { font-size: 10px; color: var(--dim); }

.empty { color: var(--dim); font-style: italic; padding: 8px 0; }

.prompts { margin-top: 16px; border: 1px solid var(--border); border-radius: 6px; overflow: hidden; }
.prompts summary { cursor: pointer; padding: 8px 10px; font-size: 11px; text-transform: uppercase; letter-spacing: 0.08em; color: var(--dim); list-style: none; }
.prompts summary::before { content: "▸ "; color: var(--vio); }
.prompts[open] summary::before { content: "▾ "; }
.prompts .agent-btn { display: block; width: 100%; text-align: left; background: none; border: none; border-top: 1px solid var(--border); padding: 8px 10px; color: var(--fg); cursor: pointer; font-size: 12px; font-family: inherit; }
.prompts .agent-btn:hover { background: color-mix(in srgb, var(--vio) 10%, transparent); }
.prompts .agent-btn .tag { color: var(--vio); font-size: 10px; }
.prompt-box { margin-top: 8px; padding: 10px; border: 1px solid var(--vio); border-radius: 6px; background: color-mix(in srgb, var(--vio) 6%, transparent); }
.prompt-box .prompt-label { font-size: 10px; color: var(--vio); text-transform: uppercase; letter-spacing: 0.06em; margin-bottom: 6px; }
.prompt-box pre { font-size: 11px; white-space: pre-wrap; word-break: break-word; color: var(--fg); line-height: 1.5; margin: 0; }
.prompt-box .copy-btn { margin-top: 8px; background: var(--vio); color: #fff; border: none; border-radius: 4px; padding: 4px 12px; cursor: pointer; font-size: 11px; }
.prompt-box .copy-btn:hover { opacity: 0.9; }
</style>
</head>
<body>

<h2>Review Queue${pending.length ? `<span class="badge">${pending.length}</span>` : ""}</h2>
${pending.length === 0 ? '<div class="empty">No pending notes — the agent will work uninterrupted.</div>' : ""}
${pending.map((n) => `
<div class="note">
  <div class="text">${esc(n.text)}</div>
  <div class="meta">
    <span>${timeAgo(n.ts)}</span>
    ${n.thread ? `<span class="thread-tag">↩ ${esc(n.thread)}</span>` : ""}
    <button class="resolve-btn" onclick="resolve('${n.id}')">resolve</button>
  </div>
</div>`).join("")}

<div class="input-row">
  <input id="noteInput" placeholder="Add a note for the agent..." onkeydown="if(event.key==='Enter')addNote()" />
  <button onclick="addNote()">+ Note</button>
</div>
<div class="thread-input">
  <label>Thread (optional — reply to a specific response):</label>
  <input id="threadInput" placeholder="e.g. 'the auth refactor commit'" />
</div>

${decisions.length ? `<h2>Decisions</h2>` : ""}
${decisions.map((d) => `
<div class="decision">
  ${d.locked ? '<span class="lock">🔒</span>' : ""}${esc(d.text)}
  ${d.why ? `<div class="why">${esc(d.why)}</div>` : ""}
</div>`).join("")}

${resolved.length ? `<h2>Resolved (${resolved.length})</h2>` : ""}
${resolved.map((n) => `<div class="note resolved"><div class="text">${esc(n.text)}</div></div>`).join("")}

<details class="prompts">
  <summary>Tell your agent about Thruline</summary>
  <button class="agent-btn" onclick="showPrompt('copilot')">GitHub Copilot <span class="tag">paste into chat</span></button>
  <button class="agent-btn" onclick="showPrompt('cursor')">Cursor <span class="tag">paste into chat</span></button>
  <button class="agent-btn" onclick="showPrompt('claude')">Claude Code <span class="tag">already wired via hooks</span></button>
  <button class="agent-btn" onclick="showPrompt('generic')">Other agent <span class="tag">universal prompt</span></button>
</details>
<div id="promptBox"></div>

<script>
const vscode = acquireVsCodeApi();
function addNote() {
  const input = document.getElementById('noteInput');
  const thread = document.getElementById('threadInput');
  const text = input.value.trim();
  if (!text) return;
  vscode.postMessage({ type: 'addNote', text, thread: thread.value.trim() || undefined });
  input.value = '';
  thread.value = '';
}
function resolve(id) {
  vscode.postMessage({ type: 'resolve', id });
}

const PROMPTS = {
  copilot: {
    label: 'GitHub Copilot',
    text: \`Before finishing any task, check the file .thruline/queue.json in this project. If it has notes with "resolved": false or null, treat each one as a code-review comment and address it. After addressing a note, mark it resolved by setting "resolved": true in the JSON.

Also read .thruline/decisions.json — these are locked project decisions. Do not contradict them without flagging the conflict first.

The Thruline sidebar panel in VS Code shows these notes and decisions. The user adds review notes there while you work — this is how they steer you without interrupting.\`
  },
  cursor: {
    label: 'Cursor',
    text: \`This project uses Thruline for review notes and decisions. Before finishing any task:

1. Read .thruline/queue.json — any notes with resolved: false are review comments from the user. Address each one.
2. Read .thruline/decisions.json — these are locked architectural decisions. Respect them.
3. Read AGENTS.md if it exists — it contains rendered decisions for cross-agent consistency.

The user adds notes via the Thruline VS Code sidebar while you work. This lets them steer without interrupting your flow.\`
  },
  claude: {
    label: 'Claude Code',
    text: \`Claude Code already has Thruline wired via hooks — the review queue is delivered automatically when you finish a task, and decisions are injected into every prompt.

If you installed the plugin (/plugin marketplace add Khubaib7-del/thruline), all 12 slash commands are available: /thruline:note, :decide, :status, :pause, :snapshot, :restore, :wrap, and more.

The VS Code panel is an additional UI — you can use it alongside the terminal.\`
  },
  generic: {
    label: 'Any agent',
    text: \`This project uses Thruline (.thruline/ directory) for persistent review notes and decisions.

Before finishing any task:
- Check .thruline/queue.json for pending notes (resolved: false). Treat each as a code-review comment and address it.
- Check .thruline/decisions.json for locked decisions. Do not contradict them.
- If AGENTS.md exists, read it for project context.

The user queues notes from the Thruline VS Code sidebar while you work — this is how they give feedback without interrupting.\`
  }
};

function showPrompt(key) {
  const p = PROMPTS[key];
  document.getElementById('promptBox').innerHTML =
    '<div class="prompt-box">' +
    '<div class="prompt-label">Paste this into ' + p.label + '</div>' +
    '<pre>' + p.text + '</pre>' +
    '<button class="copy-btn" onclick="copyPrompt(this)">Copy to clipboard</button>' +
    '</div>';
}

function copyPrompt(btn) {
  const text = btn.parentElement.querySelector('pre').textContent;
  navigator.clipboard.writeText(text);
  btn.textContent = 'Copied!';
  setTimeout(() => btn.textContent = 'Copy to clipboard', 1500);
}
</script>
</body>
</html>`;
  }
}

function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

function timeAgo(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  return `${Math.floor(hrs / 24)}d ago`;
}
