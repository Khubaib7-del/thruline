# Distribution & Launch Plan

The code is one layer of the product. This doc covers the other three: how people install it, the web properties we own, and how people find out it exists.

## Layer 1 — You build: the GitHub repo

The repo (code, docs, issues, discussions) is the home base. Everything below points back to it. Public from day one; build-in-public posts link to real commits.

## Layer 2 — Installation channels (other people's platforms)

We publish artifacts to platforms developers already use — we build no infrastructure here.

| Channel | Audience | What we ship |
|---|---|---|
| **GitHub Releases** | anyone | Compiled binaries per OS (Windows/macOS/Linux, x64 + ARM), each with SHA-256 checksum + signature (security Finding 4) |
| **npm wrapper** (`npm install -g <name>`) | JS/web devs (the majority of agent users) | Tiny package whose install step downloads the matching signed binary and verifies its checksum — no compilation, no Rust needed |
| **cargo install** | Rust devs | Source build from crates.io |
| **Claude Code plugin** | Claude Code users | One-command install bundling MCP registration + hooks + statusline |

Later candidates (post-launch, by demand): Homebrew (macOS), winget/Scoop (Windows).

**Release discipline:** binaries are built and signed by CI only — never from a laptop. Versioned releases (semver); no silent auto-update channel.

## Layer 3 — Web properties we own and build

### Landing page (built ~Milestone 2–3, before launch)
- One page. Job: convince a developer to try it within 10 seconds of arriving.
- Structure: headline (the one-line pitch) → **demo GIF** → 3 feature blocks (review queue, decision locks, context health) → install command → GitHub link.
- Hosted free (Vercel or GitHub Pages) on the product domain (bought when the public name is chosen — see DECISIONS.md naming entry).
- This is also a design showcase: motion/typography quality here signals product quality.

### The demo GIF (the single most important marketing asset)
- ~20-second screen recording of the wedge moment: user queues two notes while Claude Code builds a feature → agent finishes → agent addresses the notes as review comments.
- Recorded once, reused everywhere: landing page hero, README top, X/LinkedIn posts, Product Hunt gallery.
- Make it early (as soon as Milestone 1 works) — it's also the build-in-public content engine.

### Docs site (built by Milestone 3)
- Generated from markdown (Starlight or Docusaurus), hosted free at `docs.` subdomain.
- Minimum viable docs: install (per OS), quickstart, one page per feature, one page per agent integration (with the honest ✅/⚠️/❌ from the feasibility matrix), security page, troubleshooting.
- README stays short and links here once this exists.

## Layer 4 — Getting seen (marketing on other people's platforms)

### Publicity approach: silent build, product-first launch (owner's decision)
- No progress/milestone posts. We build quietly; the owner verifies the product personally before anything goes public.
- The public reveal is one polished moment: finished product + demo GIF + live repo + working install command.
- Rationale: the story is the product, not the steps; keeps the owner's LinkedIn presence clean.
- Trade-off accepted: no pre-warmed audience — compensated by a stronger single launch (soft launch communities + Product Hunt do the distribution work).
- Optional later: casual progress posts on X only (culture fits), LinkedIn stays polished-only.

### Launch sequence (at Milestone 3)
1. **Soft launch:** post in communities where the pain is felt — r/ClaudeAI, r/cursor, Hacker News (Show HN), relevant Discords. Gather feedback, fix the top complaints.
2. **Product Hunt launch:** one prepared day — gallery of GIFs, first-comment explaining the story ("built with Claude Code, to fix working with Claude Code"), respond to every comment that day. Goal: first ~1k visitors and the initial user base.
3. **Post-launch:** the "built Thruline with Claude Code, here's everything I learned" long-form writeup — historically the highest-leverage content format for dev tools.

## Funding (post-launch, expectations calibrated)

Open-source support channels, enabled at public launch — all free to set up:
- **GitHub Sponsors** — the default; a "Sponsor" button on the repo, zero fees, integrates with the profile people already see.
- **Buy Me a Coffee / Ko-fi** — casual one-time tips, link in README + landing page footer.
- **Open Collective / Polar** — only if the project grows contributors/company users; transparent budgets appeal to corporate sponsors.

**Honest expectations:** sponsorship follows visibility, not quality — most OSS projects earn $0–50/month even with real users; a project with thousands of stars might earn a few hundred. Sponsors will cover the ~$15/year domain easily; they will not be income. The real monetization ladder is: sponsors (covers costs) → team-sync SaaS tier (actual revenue, post-1.0) → career value from a visible shipped project (for a student, usually the biggest payoff of the three).

## Sequencing summary

```
now ──────────────────────────────────────────────▶ launch
repo → M1 demo GIF → posts every milestone → landing page → docs site → soft launch → Product Hunt → writeup
```

Rule of thumb: nothing in Layer 3–4 blocks coding until Milestone 1 is done. The demo GIF is the trigger: once the review queue works on camera, marketing assets start compounding.
