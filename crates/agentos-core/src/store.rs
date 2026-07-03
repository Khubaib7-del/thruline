use crate::model::{Decision, NoteStatus, ReviewNote};
use crate::trust::{self, TrustStatus};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

const STATE_DIR: &str = ".agentos";
const DECISIONS_FILE: &str = "decisions.json";
const QUEUE_FILE: &str = "review-queue.json";

/// File-backed state for one project. Plain JSON with a rendered markdown
/// mirror, so the user can always read and version the state without agentos.
pub struct Store {
    root: PathBuf,
    trust_db: Option<PathBuf>,
}

impl Store {
    pub fn init(project_root: &Path) -> Result<Self> {
        let root = project_root.join(STATE_DIR);
        if root.exists() {
            bail!(
                "{} already exists — this project is initialized",
                root.display()
            );
        }
        fs::create_dir_all(&root).with_context(|| format!("creating {}", root.display()))?;
        let store = Self {
            root,
            trust_db: trust::default_db_path(),
        };
        store.write_json(DECISIONS_FILE, &Vec::<Decision>::new())?;
        store.write_json(QUEUE_FILE, &Vec::<ReviewNote>::new())?;
        store.render_decisions_md(&[])?;
        store.approve_trust()?;
        Ok(store)
    }

    pub fn open(project_root: &Path) -> Result<Self> {
        let root = project_root.join(STATE_DIR);
        if !root.is_dir() {
            bail!(
                "no {STATE_DIR} directory in {} — run `agentos init` first",
                project_root.display()
            );
        }
        Ok(Self {
            root,
            trust_db: trust::default_db_path(),
        })
    }

    /// Compare the decisions file against the fingerprint approved on this
    /// machine. `Changed` means it was edited outside agentos (e.g. a git
    /// pull) and must be re-approved before agents see it.
    pub fn trust_status(&self) -> TrustStatus {
        match &self.trust_db {
            Some(db) => trust::status(db, &self.root, &self.root.join(DECISIONS_FILE)),
            // No resolvable home dir: cannot track trust — treat as untrusted.
            None => TrustStatus::Untrusted,
        }
    }

    /// Record the current decisions file as approved (the user reviewed it,
    /// or the change came through agentos itself).
    pub fn approve_trust(&self) -> Result<()> {
        match &self.trust_db {
            Some(db) => trust::approve(db, &self.root, &self.root.join(DECISIONS_FILE)),
            None => Ok(()),
        }
    }

    pub fn add_decision(&self, text: &str, why: Option<&str>, locked: bool) -> Result<Decision> {
        if self.trust_status() == TrustStatus::Changed {
            bail!(
                "decisions were modified outside agentos — review them with \
                 `agentos list`, then run `agentos trust` before recording new ones"
            );
        }
        let mut all: Vec<Decision> = self.read_json(DECISIONS_FILE)?;
        let decision = Decision {
            id: all.last().map_or(1, |d| d.id + 1),
            text: crate::redact(text),
            why: why.map(crate::redact),
            locked,
            made_at: chrono::Utc::now(),
        };
        all.push(decision.clone());
        self.write_json(DECISIONS_FILE, &all)?;
        self.render_decisions_md(&all)?;
        self.approve_trust()?;
        Ok(decision)
    }

    pub fn decisions(&self) -> Result<Vec<Decision>> {
        self.read_json(DECISIONS_FILE)
    }

    pub fn add_note(&self, text: &str) -> Result<ReviewNote> {
        let mut all: Vec<ReviewNote> = self.read_json(QUEUE_FILE)?;
        let note = ReviewNote {
            id: all.last().map_or(1, |n| n.id + 1),
            text: crate::redact(text),
            status: NoteStatus::Pending,
            created_at: chrono::Utc::now(),
        };
        all.push(note.clone());
        self.write_json(QUEUE_FILE, &all)?;
        Ok(note)
    }

    pub fn notes(&self) -> Result<Vec<ReviewNote>> {
        self.read_json(QUEUE_FILE)
    }

    /// Flip pending notes to delivered once they've been handed to an agent,
    /// so a note is never delivered twice.
    pub fn mark_delivered(&self, ids: &[u64]) -> Result<()> {
        let mut all: Vec<ReviewNote> = self.read_json(QUEUE_FILE)?;
        for n in all.iter_mut() {
            if ids.contains(&n.id) && n.status == NoteStatus::Pending {
                n.status = NoteStatus::Delivered;
            }
        }
        self.write_json(QUEUE_FILE, &all)
    }

    /// Mark a note resolved (the agent addressed it). Returns false if the
    /// id doesn't exist.
    pub fn resolve_note(&self, id: u64) -> Result<bool> {
        let mut all: Vec<ReviewNote> = self.read_json(QUEUE_FILE)?;
        let mut found = false;
        for n in all.iter_mut() {
            if n.id == id {
                n.status = NoteStatus::Resolved;
                found = true;
            }
        }
        if found {
            self.write_json(QUEUE_FILE, &all)?;
        }
        Ok(found)
    }

    pub fn pending_notes(&self) -> Result<Vec<ReviewNote>> {
        Ok(self
            .notes()?
            .into_iter()
            .filter(|n| n.status == NoteStatus::Pending)
            .collect())
    }

    /// Render project memory into AGENTS.md (the cross-agent instruction file
    /// read by Cursor, Codex, Copilot, and others). Only the marked managed
    /// region is ever touched — user content above/below is preserved.
    pub fn render_agents_md(&self, project_root: &Path) -> Result<PathBuf> {
        const BEGIN: &str = "<!-- agentos:begin — managed region, edit with `agentos` commands -->";
        const END: &str = "<!-- agentos:end -->";

        let decisions = self.decisions()?;
        let mut block = String::from(BEGIN);
        block.push_str("\n## Project decisions (recorded with agentos)\n\n");
        block.push_str(
            "Locked decisions are commitments: if your plan conflicts with one, \
             surface the conflict to the user instead of silently deviating.\n\n",
        );
        if decisions.is_empty() {
            block.push_str("- none recorded yet\n");
        }
        for d in &decisions {
            let lock = if d.locked { " **[locked]**" } else { "" };
            block.push_str(&format!("- {}{}", d.text, lock));
            if let Some(why) = &d.why {
                block.push_str(&format!(" — why: {why}"));
            }
            block.push('\n');
        }
        block.push_str(
            "\nWhen you finish a task, check `.agentos/review-queue.json` for pending \
             user notes and address them like code-review comments.\n",
        );
        block.push_str(END);

        let path = project_root.join("AGENTS.md");
        let existing = if path.exists() {
            fs::read_to_string(&path)?
        } else {
            String::new()
        };
        let updated = match (existing.find(BEGIN), existing.find(END)) {
            (Some(start), Some(end)) if end >= start => {
                format!(
                    "{}{}{}",
                    &existing[..start],
                    block,
                    &existing[end + END.len()..]
                )
            }
            _ if existing.trim().is_empty() => format!("# Agent instructions\n\n{block}\n"),
            _ => format!("{}\n\n{block}\n", existing.trim_end()),
        };
        fs::write(&path, updated)?;
        Ok(path)
    }

    /// Write a session snapshot: caller-provided summary/todos/questions plus
    /// the current decisions and open review notes, as one markdown file any
    /// agent can read. Returns the file path.
    pub fn save_snapshot(
        &self,
        summary: &str,
        todos: &[String],
        open_questions: &[String],
    ) -> Result<PathBuf> {
        let dir = self.root.join("snapshots");
        fs::create_dir_all(&dir)?;
        let now = chrono::Utc::now();

        let mut md = format!(
            "# Session snapshot — {}\n\n## Summary\n{}\n",
            now.format("%Y-%m-%d %H:%M UTC"),
            crate::redact(summary)
        );

        md.push_str("\n## Decisions on record\n");
        let decisions = self.decisions()?;
        if decisions.is_empty() {
            md.push_str("- none\n");
        }
        for d in &decisions {
            let lock = if d.locked { " [locked]" } else { "" };
            md.push_str(&format!("- #{} {}{}", d.id, d.text, lock));
            if let Some(why) = &d.why {
                md.push_str(&format!(" — why: {why}"));
            }
            md.push('\n');
        }

        md.push_str("\n## Open review notes\n");
        let open: Vec<ReviewNote> = self
            .notes()?
            .into_iter()
            .filter(|n| n.status != NoteStatus::Resolved)
            .collect();
        if open.is_empty() {
            md.push_str("- none\n");
        }
        for n in &open {
            md.push_str(&format!("- #{} {}\n", n.id, n.text));
        }

        if !todos.is_empty() {
            md.push_str("\n## Pending TODOs\n");
            for t in todos {
                md.push_str(&format!("- {}\n", crate::redact(t)));
            }
        }
        if !open_questions.is_empty() {
            md.push_str("\n## Open questions\n");
            for q in open_questions {
                md.push_str(&format!("- {}\n", crate::redact(q)));
            }
        }
        md.push_str("\nRestore hint: read this snapshot, then continue the work it describes.\n");

        // Filename is derived from the clock only — never from input (path
        // traversal is impossible by construction). Same-second collisions
        // get an `_NN` suffix; `_` sorts after `.` so lexicographic order
        // still equals chronological order.
        let stamp = now.format("%Y-%m-%dT%H-%M-%S");
        let mut path = dir.join(format!("{stamp}.md"));
        let mut counter = 2;
        while path.exists() {
            path = dir.join(format!("{stamp}_{counter:02}.md"));
            counter += 1;
        }
        fs::write(&path, md)?;
        Ok(path)
    }

    /// Newest snapshot (path + content), if any. Filenames are sortable
    /// timestamps, so lexicographic max is the latest.
    pub fn latest_snapshot(&self) -> Result<Option<(PathBuf, String)>> {
        let dir = self.root.join("snapshots");
        if !dir.is_dir() {
            return Ok(None);
        }
        let mut files: Vec<PathBuf> = fs::read_dir(&dir)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "md"))
            .collect();
        files.sort();
        match files.pop() {
            Some(p) => {
                let content = fs::read_to_string(&p)?;
                Ok(Some((p, content)))
            }
            None => Ok(None),
        }
    }

    fn read_json<T: serde::de::DeserializeOwned>(&self, file: &str) -> Result<T> {
        let path = self.root.join(file);
        let raw =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
    }

    /// Write via temp file + rename so a crash mid-write never corrupts state.
    fn write_json<T: serde::Serialize>(&self, file: &str, value: &T) -> Result<()> {
        let path = self.root.join(file);
        let tmp = self.root.join(format!("{file}.tmp"));
        fs::write(&tmp, serde_json::to_string_pretty(value)?)
            .with_context(|| format!("writing {}", tmp.display()))?;
        fs::rename(&tmp, &path).with_context(|| format!("replacing {}", path.display()))?;
        Ok(())
    }

    fn render_decisions_md(&self, decisions: &[Decision]) -> Result<()> {
        let mut md = String::from(
            "# Decision log\n\nManaged by agentos — record entries with `agentos decide`, not by hand.\n",
        );
        for d in decisions {
            let lock = if d.locked { " [locked]" } else { "" };
            md.push_str(&format!("\n## #{} — {}{}\n", d.id, d.text, lock));
            md.push_str(&format!("*{}*\n", d.made_at.format("%Y-%m-%d %H:%M UTC")));
            if let Some(why) = &d.why {
                md.push_str(&format!("\n**Why:** {why}\n"));
            }
        }
        fs::write(self.root.join("decisions.md"), md)?;
        Ok(())
    }
}
