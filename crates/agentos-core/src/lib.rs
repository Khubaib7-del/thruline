pub mod model;
pub mod redact;
pub mod store;
pub mod trust;

pub use model::{Decision, NoteStatus, ReviewNote};
pub use redact::redact;
pub use store::Store;
pub use trust::TrustStatus;

#[cfg(test)]
mod tests {
    use super::{Store, TrustStatus};

    /// Point the trust db at a temp file so tests never touch the real
    /// ~/.agentos/trust.json. Once per process; project keys stay unique
    /// because every test uses its own temp dir.
    fn isolate_trust() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let p = std::env::temp_dir()
                .join(format!("agentos-trust-test-{}.json", std::process::id()));
            let _ = std::fs::remove_file(&p);
            std::env::set_var("AGENTOS_TRUST_DB", p);
        });
    }

    #[test]
    fn trust_flags_external_edits_and_recovers_on_approval() {
        isolate_trust();
        isolate_trust();
        let dir = tempfile::tempdir().unwrap();
        let store = Store::init(dir.path()).unwrap();
        store.add_decision("DB: PostgreSQL", None, true).unwrap();
        assert_eq!(store.trust_status(), TrustStatus::Trusted);

        // Simulate a git pull / tampering: edit the file directly.
        let file = dir.path().join(".agentos/decisions.json");
        let mut raw = std::fs::read_to_string(&file).unwrap();
        raw = raw.replace("PostgreSQL", "MongoDB");
        std::fs::write(&file, raw).unwrap();
        assert_eq!(store.trust_status(), TrustStatus::Changed);

        // Writes are blocked until the user reviews.
        assert!(store.add_decision("more", None, false).is_err());

        // Approval (agentos trust) restores normal operation.
        store.approve_trust().unwrap();
        assert_eq!(store.trust_status(), TrustStatus::Trusted);
        assert!(store.add_decision("more", None, false).is_ok());
    }

    #[test]
    fn init_then_reopen_roundtrip() {
        isolate_trust();
        isolate_trust();
        let dir = tempfile::tempdir().unwrap();
        let store = Store::init(dir.path()).unwrap();
        store
            .add_decision("DB: PostgreSQL", Some("relational model fits"), true)
            .unwrap();
        store.add_note("use debounce on the search input").unwrap();

        let store = Store::open(dir.path()).unwrap();
        let decisions = store.decisions().unwrap();
        assert_eq!(decisions.len(), 1);
        assert!(decisions[0].locked);
        assert_eq!(decisions[0].why.as_deref(), Some("relational model fits"));
        assert_eq!(store.pending_notes().unwrap().len(), 1);
    }

    #[test]
    fn secrets_never_reach_disk() {
        isolate_trust();
        let dir = tempfile::tempdir().unwrap();
        let store = Store::init(dir.path()).unwrap();
        store
            .add_decision(
                "use key sk-abcdefghijklmnop1234 for the API",
                Some("connect with postgres://app:supersecretpw@db/x"),
                false,
            )
            .unwrap();
        store.add_note("set API_KEY=abc123def456xyz first").unwrap();

        let raw = std::fs::read_to_string(dir.path().join(".agentos/decisions.json")).unwrap()
            + &std::fs::read_to_string(dir.path().join(".agentos/review-queue.json")).unwrap()
            + &std::fs::read_to_string(dir.path().join(".agentos/decisions.md")).unwrap();
        assert!(!raw.contains("sk-abcdefghijklmnop1234"));
        assert!(!raw.contains("supersecretpw"));
        assert!(!raw.contains("abc123def456xyz"));
        assert!(raw.contains("[redacted:api-key]"));
    }

    #[test]
    fn mark_delivered_removes_from_pending() {
        isolate_trust();
        let dir = tempfile::tempdir().unwrap();
        let store = Store::init(dir.path()).unwrap();
        let a = store.add_note("first").unwrap();
        store.add_note("second").unwrap();
        store.mark_delivered(&[a.id]).unwrap();
        let pending = store.pending_notes().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].text, "second");
    }

    #[test]
    fn ids_increment() {
        isolate_trust();
        let dir = tempfile::tempdir().unwrap();
        let store = Store::init(dir.path()).unwrap();
        let a = store.add_decision("first", None, false).unwrap();
        let b = store.add_decision("second", None, false).unwrap();
        assert_eq!(a.id, 1);
        assert_eq!(b.id, 2);
    }

    #[test]
    fn render_preserves_user_content_and_updates_managed_region() {
        isolate_trust();
        let dir = tempfile::tempdir().unwrap();
        let store = Store::init(dir.path()).unwrap();
        store.add_decision("DB: PostgreSQL", None, true).unwrap();

        // Fresh render creates the file.
        let path = store.render_agents_md(dir.path()).unwrap();
        let first = std::fs::read_to_string(&path).unwrap();
        assert!(first.contains("DB: PostgreSQL"));
        assert!(first.contains("**[locked]**"));

        // User adds their own content around the managed region.
        let user_content =
            format!("# My rules\n\nAlways use tabs.\n\n{first}\n## Footer\nkeep me\n");
        std::fs::write(&path, &user_content).unwrap();

        // A new decision re-renders only the managed region.
        store.add_decision("Auth: Clerk", None, false).unwrap();
        store.render_agents_md(dir.path()).unwrap();
        let second = std::fs::read_to_string(&path).unwrap();
        assert!(second.contains("Always use tabs."));
        assert!(second.contains("keep me"));
        assert!(second.contains("Auth: Clerk"));
        assert_eq!(
            second.matches("agentos:begin").count(),
            1,
            "no duplicate regions"
        );
    }

    #[test]
    fn snapshot_bundles_state_and_latest_wins() {
        isolate_trust();
        let dir = tempfile::tempdir().unwrap();
        let store = Store::init(dir.path()).unwrap();
        store.add_decision("DB: PostgreSQL", None, true).unwrap();
        store.add_note("try caching").unwrap();

        store.save_snapshot("first pass", &[], &[]).unwrap();
        let path = store
            .save_snapshot("second pass", &["finish tests".into()], &[])
            .unwrap();
        assert!(path.exists());

        let (latest_path, content) = store.latest_snapshot().unwrap().unwrap();
        assert_eq!(latest_path, path);
        assert!(content.contains("second pass"));
        assert!(content.contains("DB: PostgreSQL"));
        assert!(content.contains("try caching"));
        assert!(content.contains("finish tests"));
    }

    #[test]
    fn init_twice_fails() {
        isolate_trust();
        let dir = tempfile::tempdir().unwrap();
        Store::init(dir.path()).unwrap();
        assert!(Store::init(dir.path()).is_err());
    }

    #[test]
    fn open_without_init_fails() {
        isolate_trust();
        let dir = tempfile::tempdir().unwrap();
        assert!(Store::open(dir.path()).is_err());
    }
}
