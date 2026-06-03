# Implementation Backlog

Status date: 2026-06-03

This backlog converts the roadmap and specs into work packages. It is ordered by dependency, not by excitement.

## Epic 0: Documentation And Truth Baseline

Purpose: ensure public claims, specs, and release gates are aligned.

Tasks:

- [ ] Keep README roadmap aligned with `ROADMAP.md`.
- [ ] Add doc links to release notes and contribution docs.
- [ ] Add docs linting or markdown validation to CI.
- [ ] Create a known-gaps section for each release.

Dependencies:

- None.

Exit criteria:

- All public "done" claims map to a verified feature and test plan.

## Epic 1: Test Harness Foundation

Purpose: make feature completion measurable.

Tasks:

- [ ] Add workspace integration test folder and fixture layout.
- [ ] Add test fixtures for SSH config, snippets, AI responses, and recordings.
- [ ] Add a terminal screenshot smoke-test runner for macOS.
- [ ] Add a Linux screenshot smoke-test runner or document why it is manual.
- [ ] Add CI jobs for format, clippy, tests, release build, and docs.
- [ ] Decide on clippy transition gate if current warnings block `-D warnings`.

Dependencies:

- Current docs/spec baseline.

Exit criteria:

- Test strategy can be run by a contributor from a clean checkout.

## Epic 2: Terminal Reliability

Purpose: lock down the fixed renderer and terminal behavior.

Tasks:

- [ ] Add more engine tests for alternate screen, cursor visibility, bracketed paste, title events, PTY replies, scrollback, and resize.
- [ ] Add PTY integration tests using a controlled shell.
- [ ] Add automated runner for `scripts/terminal-torture.sh`.
- [ ] Add screenshot artifact capture.
- [ ] Test `vim`, `less`, and `htop` manually or through automation.
- [ ] Decide whether to remove, park, or integrate legacy `corgiterm-terminal` and `images.rs`.

Dependencies:

- Epic 1 harness.

Exit criteria:

- Terminal rendering regressions are caught before release.

## Epic 3: Safe Mode Completion

Purpose: make the safety promise real for beginner command paths.

Tasks:

- [ ] Expand command classifier coverage.
- [ ] Add deterministic explanation templates.
- [ ] Add safer-alternative generation for common risky commands.
- [ ] Add UI state tests for safe, caution, danger, and unknown.
- [ ] Ensure AI-generated and snippet-generated commands always pass through Safe Mode.
- [ ] Add bypass tests for generated commands.

Dependencies:

- Epic 1 harness.
- Terminal command injection path stabilized.

Exit criteria:

- A risky generated command can be previewed, canceled, and verified not executed.

## Epic 4: AI Assistant Completion

Purpose: make AI help reliable, local-first, and safe.

Tasks:

- [ ] Add mock provider interface for tests.
- [ ] Test Chat mode success/error/timeout.
- [ ] Test Explain mode with command output.
- [ ] Test Command mode and Safe Mode handoff.
- [ ] Test no-provider state.
- [ ] Test provider detection without network dependency.
- [ ] Add learning privacy controls and tests.
- [ ] Add secret redaction tests.

Dependencies:

- Epic 3 Safe Mode gate.

Exit criteria:

- AI never executes generated commands without Safe Mode approval.

## Epic 5: Snippets Completion

Purpose: make command reuse safe and useful.

Tasks:

- [ ] Add tests for snippet variable parsing, defaults, hints, and unresolved values.
- [ ] Add UI model tests for search, filters, sort, pinned state.
- [ ] Add create/edit/delete workflow.
- [ ] Add import/export round-trip.
- [ ] Route execute through Safe Mode.
- [ ] Add release checklist scenario.

Dependencies:

- Epic 3 Safe Mode gate.

Exit criteria:

- A parameterized snippet can be filled and executed safely.

## Epic 6: SSH Manager Completion

Purpose: make visual SSH workflows reliable.

Tasks:

- [ ] Expand SSH config parser tests.
- [ ] Add host add/edit/delete persistence tests.
- [ ] Add favorites/search/filter tests.
- [ ] Add key listing tests that prove private key material is not exposed.
- [ ] Add quick-connect command generation.
- [ ] Route quick-connect through Safe Mode or a dedicated trusted SSH preview.
- [ ] Add port-forward command generation tests.

Dependencies:

- Epic 3 Safe Mode gate.

Exit criteria:

- Quick connect produces the expected command and does not expose secrets.

## Epic 7: Tabs, Splits, And Broadcast Completion

Purpose: make multi-terminal workflows dependable.

Tasks:

- [ ] Extract split tree state into testable model if needed.
- [ ] Test horizontal and vertical split creation.
- [ ] Test close focused pane.
- [ ] Test focus next/previous pane.
- [ ] Test independent PTY per pane.
- [ ] Test broadcast all panes.
- [ ] Test targeted broadcast.

Dependencies:

- Epic 1 harness.
- Epic 2 terminal reliability.

Exit criteria:

- Multi-pane workflows do not corrupt terminal state or send input to the wrong pane.

## Epic 8: Recording And Playback Completion

Purpose: make session recording a real feature, not only a model.

Tasks:

- [ ] Wire terminal PTY output into recording state.
- [ ] Wire terminal input into recording state.
- [ ] Wire resize events into recording state.
- [ ] Add command marker support.
- [ ] Test save/load recordings.
- [ ] Test playback timing.
- [ ] Test playback UI controls.

Dependencies:

- Epic 2 PTY integration.

Exit criteria:

- A recorded command session can be saved, reloaded, and replayed.

## Epic 9: Themes And Preferences Completion

Purpose: make customization reliable and accessible.

Tasks:

- [ ] Add theme save/load tests.
- [ ] Add theme apply/reload tests for terminal colors.
- [ ] Add contrast warning tests.
- [ ] Add font fallback tests.
- [ ] Add shortcut persistence tests.
- [ ] Add accessibility setting tests.

Dependencies:

- Epic 1 harness.

Exit criteria:

- A custom theme can be created, saved, applied, and reloaded without breaking terminal readability.

## Epic 10: Plugins And MCP Completion

Purpose: make extensibility honest and stable.

Tasks:

- [ ] Define stable plugin API.
- [ ] Add sample Lua plugin.
- [ ] Add sample WASM plugin.
- [ ] Execute sample plugins in integration tests.
- [ ] Define plugin permissions model.
- [ ] Wire MCP tools to a real terminal backend or mark MCP experimental.
- [ ] Add MCP JSON-RPC backend tests.

Dependencies:

- Epic 2 terminal backend stability.

Exit criteria:

- Extensibility claims are backed by sample plugins and real backend tests.

## Epic 11: Distribution And Release Automation

Purpose: make releases repeatable and trustworthy.

Tasks:

- [ ] Verify macOS app bundle build script.
- [ ] Verify DMG or install script artifact flow.
- [ ] Add app bundle signature verification.
- [ ] Add installed-app launch smoke test.
- [ ] Add release notes template.
- [ ] Add known limitations template.
- [ ] Document Linux build and package expectations.

Dependencies:

- Epic 1 CI.
- Epic 2 terminal smoke test.

Exit criteria:

- A release candidate can be built, installed, launched, verified, and documented from a clean checkout.

## Suggested Execution Order

1. Epic 0: Documentation and truth baseline.
2. Epic 1: Test harness foundation.
3. Epic 2: Terminal reliability.
4. Epic 3: Safe Mode completion.
5. Epic 4: AI assistant completion.
6. Epics 5-9: workflow hardening.
7. Epic 10: plugins and MCP.
8. Epic 11: release automation.

## Issue Template Shape

Use this shape when turning backlog items into GitHub issues:

```text
Title:
Epic:
Problem:
Scope:
Acceptance criteria:
Required tests:
Out of scope:
Verification command/artifact:
```
