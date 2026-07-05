import * as fs from "fs";
import * as path from "path";

export interface ReviewNote {
  id: string;
  text: string;
  ts: string;
  thread?: string;
  resolved?: boolean;
}

export interface Decision {
  id: string;
  text: string;
  why?: string;
  locked: boolean;
  ts: string;
}

export class ThrulineState {
  private root: string;

  constructor(workspaceRoot: string) {
    this.root = path.join(workspaceRoot, ".thruline");
  }

  get exists(): boolean {
    return fs.existsSync(this.root);
  }

  getQueue(): ReviewNote[] {
    const file = path.join(this.root, "queue.json");
    if (!fs.existsSync(file)) return [];
    try {
      const data = JSON.parse(fs.readFileSync(file, "utf-8"));
      return Array.isArray(data) ? data : data.notes || [];
    } catch {
      return [];
    }
  }

  getDecisions(): Decision[] {
    const file = path.join(this.root, "decisions.json");
    if (!fs.existsSync(file)) return [];
    try {
      const data = JSON.parse(fs.readFileSync(file, "utf-8"));
      return Array.isArray(data) ? data : data.decisions || [];
    } catch {
      return [];
    }
  }

  async addNote(text: string, thread?: string): Promise<void> {
    const file = path.join(this.root, "queue.json");
    const queue = this.getQueue();
    const note: ReviewNote = {
      id: this.genId(),
      text,
      ts: new Date().toISOString(),
      thread,
    };
    queue.push(note);
    fs.mkdirSync(this.root, { recursive: true });
    fs.writeFileSync(file, JSON.stringify({ notes: queue }, null, 2), "utf-8");
  }

  async resolveNote(id: string): Promise<void> {
    const file = path.join(this.root, "queue.json");
    const queue = this.getQueue().map((n) =>
      n.id === id ? { ...n, resolved: true } : n
    );
    fs.writeFileSync(file, JSON.stringify({ notes: queue }, null, 2), "utf-8");
  }

  private genId(): string {
    return Date.now().toString(36) + Math.random().toString(36).slice(2, 6);
  }
}
