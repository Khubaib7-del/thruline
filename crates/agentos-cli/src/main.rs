mod hooks;
mod setup;
mod statusline;

use agentos_core::Store;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::env;

#[derive(Parser)]
#[command(
    name = "agentos",
    version,
    about = "Companion layer for AI coding agents"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create the .agentos state directory in the current project
    Init,
    /// Record a project decision
    Decide {
        /// The decision itself, e.g. "DB: PostgreSQL"
        text: String,
        /// Rationale stored alongside the decision
        #[arg(long)]
        why: Option<String>,
        /// Lock it — agents will be warned on conflicting proposals
        #[arg(long)]
        lock: bool,
    },
    /// Queue a review note for the agent without interrupting it
    Note {
        /// The idea to deliver when the agent finishes its current task
        text: String,
    },
    /// Show recorded decisions and pending review notes
    List {
        /// Output as JSON instead of plain text
        #[arg(long)]
        json: bool,
    },
    /// Review and approve project memory after it changed outside agentos
    Trust,
    /// Render project memory into AGENTS.md for other agents (Cursor, Codex, ...)
    Render,
    /// Show context health for this project's latest Claude Code session
    Context,
    /// Save a session snapshot (summary + decisions + open notes)
    Snapshot {
        /// One-paragraph summary of where the work stands
        summary: String,
        /// Pending TODO (repeatable)
        #[arg(long = "todo")]
        todos: Vec<String>,
    },
    /// Print the latest snapshot — paste it into any agent to restore context
    Restore,
    /// Agent hook entry points (called by the agent, not by hand)
    #[command(subcommand)]
    Hook(HookEvent),
    /// Run the MCP stdio server (spawned by agents, not by hand)
    Mcp,
    /// Render the Claude Code statusline (called by the agent, not by hand)
    Statusline,
    /// Wire agentos into an agent's configuration (dry run unless --apply)
    #[command(subcommand)]
    Setup(SetupTarget),
}

#[derive(Subcommand)]
enum HookEvent {
    /// Claude Code Stop hook: deliver queued review notes
    Stop,
    /// Claude Code UserPromptSubmit hook: inject locked decisions
    Prompt,
}

#[derive(Subcommand)]
enum SetupTarget {
    /// Configure Claude Code hooks in .claude/settings.local.json
    ClaudeCode {
        /// Actually write the file (default is a dry run)
        #[arg(long)]
        apply: bool,
        /// Also register the context-health statusline
        #[arg(long)]
        statusline: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = env::current_dir()?;

    match cli.command {
        Command::Init => {
            Store::init(&cwd)?;
            println!("initialized .agentos in {}", cwd.display());
        }
        Command::Decide { text, why, lock } => {
            let store = Store::open(&cwd)?;
            let d = store.add_decision(&text, why.as_deref(), lock)?;
            let suffix = if d.locked { " (locked)" } else { "" };
            println!("decision #{} recorded{suffix}", d.id);
        }
        Command::Note { text } => {
            let store = Store::open(&cwd)?;
            let n = store.add_note(&text)?;
            println!("note #{} queued for the agent's next review pass", n.id);
        }
        Command::List { json } => {
            let store = Store::open(&cwd)?;
            let decisions = store.decisions()?;
            let pending = store.pending_notes()?;
            if json {
                let out = serde_json::json!({
                    "decisions": decisions,
                    "pending_notes": pending,
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("decisions ({}):", decisions.len());
                for d in &decisions {
                    let lock = if d.locked { " [locked]" } else { "" };
                    println!("  #{} {}{lock}", d.id, d.text);
                }
                println!("pending review notes ({}):", pending.len());
                for n in &pending {
                    println!("  #{} {}", n.id, n.text);
                }
            }
        }
        Command::Trust => {
            let store = Store::open(&cwd)?;
            let decisions = store.decisions()?;
            println!("you are approving these decisions for injection into agents:");
            for d in &decisions {
                let lock = if d.locked { " [locked]" } else { "" };
                println!("  #{} {}{lock}", d.id, d.text);
            }
            if decisions.is_empty() {
                println!("  (none)");
            }
            store.approve_trust()?;
            println!("trusted — agents on this machine will now receive this project's memory");
        }
        Command::Render => {
            let store = Store::open(&cwd)?;
            let path = store.render_agents_md(&cwd)?;
            println!("rendered project memory into {}", path.display());
            println!(
                "agents that read AGENTS.md (Cursor, Codex, Copilot, ...) now see your decisions"
            );
        }
        Command::Context => statusline::run_context(),
        Command::Snapshot { summary, todos } => {
            let store = Store::open(&cwd)?;
            let path = store.save_snapshot(&summary, &todos, &[])?;
            println!("snapshot saved: {}", path.display());
        }
        Command::Restore => {
            let store = Store::open(&cwd)?;
            match store.latest_snapshot()? {
                Some((path, content)) => {
                    eprintln!("latest snapshot: {}\n", path.display());
                    println!("{content}");
                }
                None => println!("no snapshots yet — run `agentos snapshot \"<summary>\"` first"),
            }
        }
        Command::Hook(HookEvent::Stop) => hooks::run_stop(),
        Command::Hook(HookEvent::Prompt) => hooks::run_prompt(),
        Command::Mcp => agentos_mcp::serve(&cwd)?,
        Command::Statusline => statusline::run(),
        Command::Setup(SetupTarget::ClaudeCode { apply, statusline }) => {
            setup::claude_code(&cwd, apply, statusline)?
        }
    }
    Ok(())
}
