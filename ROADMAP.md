# CorgiTerm Roadmap

Status date: 2026-06-03

CorgiTerm's product goal is to make the terminal approachable for users who know a computer well but do not yet feel confident with command-line workflows. The roadmap below separates what is verified, what exists but needs product hardening, and what is not complete.

## Current Position

The terminal rendering rebuild is complete and verified in the local app bundle. The broader product is not yet feature-complete and the current automated test suite does not verify all product features end to end.

### Verified

- Terminal model now uses `alacritty_terminal` through `corgiterm-core::engine`.
- GTK terminal view renders text on a fixed grid with correct spacing, ANSI colors, 256-color, truecolor, attributes, wide CJK characters, emoji, and cursor rendering.
- `cargo check --workspace`, `cargo build --workspace`, `cargo build --release --workspace`, `cargo test --workspace`, and `cargo clippy --workspace` have passed after the renderer rebuild.
- The installed macOS app bundle was rebuilt, re-signed, launched, and visually verified.

### Implemented But Needs Product Verification

- Safe Mode analyzer and preview UI.
- AI panel modes and provider detection.
- Natural-language command translation path.
- Snippets with variables.
- SSH manager UI.
- Tabs, split panes, URL/path hints, search, copy/paste, and broadcast mode.
- Theme creator and preferences.
- Session recording data model and recording UI.
- Lua/WASM plugin runtimes.

### Partial Or Not Complete

- CLI `--execute` mode is a logged TODO.
- MCP terminal tools use placeholder behavior unless a backend is wired.
- Session recording is not yet proven as an end-to-end PTY I/O tap and playback loop.
- Plugin marketplace/repository is roadmap, not complete.
- AI-powered command history search is roadmap, not complete.
- Windows support, collaborative terminals, tmux/screen integration, and custom keybinding profiles are roadmap.
- GPU text rendering is not the current app renderer. The current verified path is GTK/Pango/Cairo rendering backed by the alacritty terminal model.

## Milestones

### M0: Truth And Test Baseline

Goal: make the public docs, roadmap, and automated gates match the actual product.

Deliverables:

- Public README status corrected.
- Current feature audit maintained under `docs/specs/current-feature-audit.md`.
- Test strategy maintained under `docs/specs/test-verification-strategy.md`.
- Release checklist maintained under `docs/specs/release-readiness-checklist.md`.
- CI jobs for format, clippy, unit tests, release build, and docs lint.

Exit criteria:

- No README "done" claim exists without an implementation owner and acceptance test plan.
- Every public feature has a status: verified, implemented-needs-verification, partial, or roadmap.
- Any release candidate has a completed release checklist.

### M1: Terminal Foundation

Goal: make terminal behavior production reliable before adding more surface area.

Deliverables:

- Automated terminal engine tests for wide chars, combining marks, alternate screen, cursor modes, scrollback, resize/reflow, bracketed paste, OSC title, hyperlink detection, and PTY replies.
- Automated renderer screenshot regression tests for macOS and Linux where feasible.
- Integrated recording hooks for PTY input, output, resize, and command markers.
- Clear decision on dead or parked terminal-related crates and image parsers.

Exit criteria:

- The terminal torture script passes visually on macOS and Linux.
- Renderer regressions can be detected without relying only on manual screenshots.
- Alternate-screen apps such as `vim`, `less`, and `htop` pass a manual or automated acceptance checklist.

### M2: Beginner Safety Experience

Goal: make CorgiTerm genuinely safer and clearer for novice users.

Deliverables:

- Safe Mode command preview covers destructive filesystem commands, package managers, network operations, privilege escalation, process killing, chmod/chown, git destructive commands, and shell redirection.
- Safe Mode can explain command impact in plain English with deterministic fallback explanations.
- Execute/cancel flows are covered by end-to-end tests.
- Risk copy and visual states are consistent across safe, caution, danger, and unknown cases.

Exit criteria:

- A beginner can review and cancel a risky command without needing terminal knowledge.
- Dangerous commands cannot bypass Safe Mode through the beginner execution paths.
- The Safe Mode spec has unit, integration, and UI acceptance coverage.

### M3: AI Assistant And Learning

Goal: make AI assistance helpful without making the terminal dependent on cloud services.

Deliverables:

- Mocked-provider tests for Chat, Explain, and Command modes.
- Local-first provider path documented and tested with Ollama where available.
- CLI-provider path documented and tested with Claude/Gemini command mocks.
- API-key provider path validated without logging secrets.
- Learning context bounded, inspectable, and user-controllable.

Exit criteria:

- AI features gracefully degrade when no provider exists.
- Generated commands always pass through Safe Mode before execution.
- Provider failures and slow responses do not freeze the GTK UI.

### M4: Workflow Features

Goal: harden the features that make CorgiTerm useful beyond the first command.

Deliverables:

- Snippets CRUD, variables, defaults, hints, insert, execute, import, and export covered by tests.
- SSH manager import, add/edit/delete, quick connect, key display, and config persistence covered by tests.
- Tabs, split panes, focus, closing, broadcast mode, and per-pane working directories covered by UI tests.
- History search and command history learning behavior specified and tested.
- Theme creator save/load/apply/contrast behavior specified and tested.

Exit criteria:

- Each workflow has an acceptance test that exercises the real UI or a stable UI model.
- The README only marks a workflow "done" after its acceptance tests pass.

### M5: Distribution And Release

Goal: make installation and releases repeatable.

Deliverables:

- macOS app bundle build script verified.
- DMG creation script or install script verified against real artifacts.
- Code signing and quarantine guidance documented.
- Linux package path documented.
- Release notes template and changelog workflow.

Exit criteria:

- A release can be built from a clean checkout using documented commands.
- A tester can install the app, launch it, run the terminal torture script, and verify the beginner workflows using the release checklist.

## Definition Of Done For A Feature

A feature is complete only when all of these are true:

- The behavior is implemented in the product, not only in a model or placeholder.
- User-facing copy and docs match the behavior.
- The feature has unit coverage for deterministic logic.
- The feature has integration or UI coverage for the real workflow.
- Failure and offline states are handled.
- The release checklist identifies how to verify it.

## Primary Specs

- `docs/specs/current-feature-audit.md`
- `docs/specs/beginner-experience-spec.md`
- `docs/specs/feature-completion-spec.md`
- `docs/specs/test-verification-strategy.md`
- `docs/specs/release-readiness-checklist.md`
- `docs/roadmap/implementation-backlog.md`
