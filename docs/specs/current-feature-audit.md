# Current Feature Audit

Status date: 2026-06-03

This audit is intentionally conservative. "Implemented" means code exists. "Verified" means the behavior was exercised through tests or live app evidence. "Complete" requires the definition in `ROADMAP.md`.

## Summary

The terminal rendering issue is fixed and verified. The product has many implemented surfaces, but most of the beginner workflows need automated end-to-end verification before they should be treated as complete.

## Test Inventory

Current `cargo test --workspace -- --list` inventory:

| Area | Listed tests | Notes |
|---|---:|---|
| Root binary | 2 | Argument parsing only. |
| `corgiterm-ai` | 30 | Provider names, MCP placeholder behavior, history, learning, conversations. |
| `corgiterm-config` | 17 | Config serialization, shortcuts, themes, SSH config parsing. |
| `corgiterm-core` | 50 | Terminal engine/model, Safe Mode analyzer, PTY size, recording model, hints, history, sessions. |
| `corgiterm-plugins` | 4 | Runtime creation and serialization basics. |
| `corgiterm-terminal` | 26 | Legacy/secondary terminal parser, grid, renderer model tests. |
| `corgiterm-ui` | 1 | App ID only. No workflow UI coverage yet. |

Current gap: most user-facing features live in `corgiterm-ui`, but UI has only one trivial test.

## Feature Status

| Feature | Current status | Evidence | Risk | Next step |
|---|---|---|---|---|
| Terminal rendering | Verified | Live macOS app capture plus engine tests and torture script. | Medium | Add automated screenshot regression and alternate-screen cases. |
| PTY spawn/read/write | Implemented, partially verified | Live app shell spawn and output capture. | Medium | Add integration tests using a controlled shell command. |
| Safe Mode analyzer | Implemented, unit tested | `corgiterm-core::safe_mode` tests. | Medium | Add UI execute/cancel flow tests and expand command patterns. |
| Safe Mode preview UI | Implemented, not automated | `safe_mode_preview.rs`. | High | Add UI tests for safe/caution/danger/unknown states. |
| Natural-language input | Implemented, not automated | `window.rs` quick translation and AI fallback path. | High | Add mocked-provider tests and Safe Mode handoff tests. |
| AI panel Chat/Explain/Command | Implemented, not automated end to end | `ai_panel.rs`, provider tests. | High | Add provider mocks, timeout/error tests, no-provider graceful state tests. |
| Local/CLI/API AI providers | Implemented, partially unit tested | Provider name tests; detection path exists. | Medium | Add deterministic tests that avoid network and secret leakage. |
| AI learning/history | Implemented, unit tested | AI and core learning tests. | Medium | Add persistence and user-control tests. |
| Snippets library | Implemented, partial tests in config | `snippets.rs`, config snippet tests. | High | Add CRUD, variable, insert, execute, import/export workflow tests. |
| SSH manager | Implemented, partial config tests | `ssh_manager.rs`, SSH config parser tests. | High | Add add/edit/delete/import/quick-connect tests with mocked terminal insertion. |
| Tabs and split panes | Implemented, not automated | `tab_bar.rs`, `split_pane.rs`. | High | Add UI model tests and app-level keyboard workflow tests. |
| URL/path hints | Implemented, unit tested detector | `hints.rs`, terminal view hint mode. | Medium | Add UI activation and action tests. |
| Search/copy/paste | Implemented, not automated | `terminal_view.rs`. | Medium | Add terminal buffer and clipboard workflow tests. |
| Broadcast mode | Implemented, limited tests | `broadcast.rs`, `split_pane.rs`. | Medium | Add per-pane broadcast tests. |
| Theme creator | Implemented, not automated | `theme_creator.rs`, theme config tests. | Medium | Add save/apply/contrast tests. |
| Session recording model | Implemented, unit tested | `recording.rs` tests. | High | Prove end-to-end PTY I/O capture and playback UI. |
| Recording panel UI | Implemented, not automated | `recording_panel.rs`. | High | Add start/stop/playback integration tests. |
| Lua/WASM plugin runtimes | Implemented, basic tests | Runtime creation tests. | High | Define plugin API contract and execute sample plugins in tests. |
| MCP terminal tools | Partial | Placeholder logic documented in `mcp.rs`. | High | Wire real backend or mark as experimental. |
| CLI `--execute` | Not complete | TODO in `src/main.rs`. | Low | Implement or remove from CLI until ready. |
| App bundle/install | Verified manually once | Rebuilt and re-signed `/Applications/CorgiTerm.app`. | Medium | Add scripted bundle verification. |
| Windows support | Roadmap | README roadmap. | High | Specify platform scope before implementation. |
| Plugin marketplace | Roadmap | README roadmap. | High | Create marketplace spec after plugin API is hardened. |
| AI-powered history search | Roadmap | README roadmap. | Medium | Define product behavior and privacy constraints. |

## Documentation Corrections Needed

The README should avoid calling a feature complete unless it passes the feature definition of done. In particular:

- Replace "GPU-accelerated rendering" with the current verified renderer architecture.
- Treat session recording/playback as partial until PTY tap and playback are proven end to end.
- Treat plugin system as experimental until sample plugins execute against a stable API.
- Link to the roadmap and feature audit for current status.
