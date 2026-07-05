import * as vscode from "vscode";
import { ThrulinePanel } from "./panel";
import { ThrulineState } from "./state";

export function activate(context: vscode.ExtensionContext) {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (!workspaceRoot) return;

  const state = new ThrulineState(workspaceRoot);
  const provider = new ThrulinePanel(context.extensionUri, state);

  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider("thruline.panel", provider),

    vscode.commands.registerCommand("thruline.addNote", async () => {
      const text = await vscode.window.showInputBox({
        prompt: "Review note for the agent",
        placeHolder: "e.g. check error handling in auth flow",
      });
      if (text) {
        await state.addNote(text);
        provider.refresh();
      }
    }),

    vscode.commands.registerCommand("thruline.refresh", () => {
      provider.refresh();
    }),

    vscode.commands.registerCommand("thruline.pause", async () => {
      const terminal = vscode.window.createTerminal("thruline");
      terminal.sendText('thruline snapshot "paused — user stepping away"');
      terminal.show();
    })
  );

  const watcher = vscode.workspace.createFileSystemWatcher(
    new vscode.RelativePattern(workspaceRoot, ".thruline/**")
  );
  watcher.onDidChange(() => provider.refresh());
  watcher.onDidCreate(() => provider.refresh());
  watcher.onDidDelete(() => provider.refresh());
  context.subscriptions.push(watcher);
}

export function deactivate() {}
