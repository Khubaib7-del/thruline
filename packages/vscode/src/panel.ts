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
