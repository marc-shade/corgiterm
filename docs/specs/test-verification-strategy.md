# Test And Verification Strategy

Status date: 2026-06-03

## Goal

The goal is not "100% code coverage" as a vanity metric. The goal is confidence that every public feature works in the product path a user will actually exercise.

## Current Baseline

Current automated coverage is strongest in core/config/AI unit logic and weakest in GTK UI workflows. `corgiterm-ui` currently has only one trivial test, so the product needs an end-to-end and UI-model test layer.

## Test Layers

### 1. Unit Tests

Purpose: deterministic logic.

Examples:

- Safe Mode risk classification.
- Snippet variable parsing and substitution.
- SSH config parsing.
- Terminal engine VT behavior.
- Theme contrast calculation.
- AI provider request formatting.

Gate:

```bash
cargo test --workspace
```

### 2. Integration Tests

Purpose: module interactions without full GUI automation.

Examples:

- PTY spawn, write, read, resize.
- Recording tap around PTY input/output.
- Snippet execute route into Safe Mode.
- AI mocked provider route into Safe Mode.
- MCP real backend route.

Target location:

- `crates/<crate>/tests/*.rs` for crate-level integration.
- `tests/*.rs` for workspace-level integration if shared setup is needed.

### 3. UI Model Tests

Purpose: test GTK-adjacent state and workflow controllers without depending on screenshot automation.

Examples:

- Tab/split tree operations.
- Safe Mode preview state transitions.
- Recording panel state transitions.
- Snippet dialog filtering/sorting.
- SSH manager add/edit/delete state.

Implementation guidance:

- Extract pure state machines or small controllers from GTK widgets.
- Keep GTK object construction minimal in tests.
- Avoid testing libadwaita internals.

### 4. End-To-End UI Tests

Purpose: verify the real app path.

Examples:

- Launch app bundle.
- Confirm shell prompt appears.
- Type or inject known PTY output.
- Capture window.
- Compare screenshot against stable expectations.
- Exercise Safe Mode, AI panel, snippets, SSH manager, tabs, splits.

Tooling options:

- macOS: Quartz screenshot capture plus process/TTY inspection for smoke tests.
- Linux: Xvfb or Wayland test session plus screenshot capture.
- UI automation: evaluate `gtk4` test harnesses, Dogtail/LDTP on Linux, or platform-specific accessibility automation where reliable.

### 5. Manual Acceptance

Purpose: cover behaviors that are hard to automate until harnesses mature.

Manual verification is acceptable only when:

- The exact steps are documented.
- The expected result is explicit.
- A screenshot or log artifact is captured.
- The gap is tracked for future automation.

## Feature Coverage Matrix

| Feature | Unit | Integration | UI model | E2E/manual | Required before "complete" |
|---|---|---|---|---|---|
| Terminal engine | Yes | Yes | No | Yes | Unit + integration + screenshot/manual |
| Terminal renderer | Limited | No | No | Yes | Screenshot regression or documented manual capture |
| PTY lifecycle | Yes | Needed | No | Yes | Controlled PTY integration |
| Safe Mode analyzer | Yes | Needed | Needed | Yes | Analyzer + preview workflow |
| AI panel | Partial | Needed | Needed | Yes | Mock providers + UI states |
| Natural-language input | Needed | Needed | Needed | Yes | Quick translation + AI fallback + Safe Mode handoff |
| Snippets | Partial | Needed | Needed | Yes | CRUD + variables + execution |
| SSH manager | Partial | Needed | Needed | Yes | CRUD + import + quick connect |
| Tabs/splits | No | Needed | Needed | Yes | Pane isolation and keyboard workflows |
| Recording/playback | Partial | Needed | Needed | Yes | PTY tap + playback |
| Themes/preferences | Partial | Needed | Needed | Yes | Save/apply/reload |
| Plugins | Partial | Needed | No | Manual | Sample plugin execution |
| MCP | Partial | Needed | No | Manual | Real backend or experimental label |
| App bundle/install | No | Needed | No | Yes | Build/sign/launch/render smoke |

## Required CI Gates

Minimum CI before release:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release --workspace
```

Pragmatic note: the current codebase has existing clippy warnings. Until those are fixed, CI may need a transition gate that records current warnings without adding new ones. The end state should be `-D warnings`.

## Terminal Renderer Regression Plan

1. Keep `scripts/terminal-torture.sh` as the canonical visual smoke test.
2. Add a test runner that launches CorgiTerm, locates the child TTY, writes the torture output, captures the app window, and saves an artifact.
3. Add a pixel/structure check:
   - At least N non-background colors.
   - ANSI color bands visible.
   - Box drawing rows have stable horizontal/vertical alignment.
   - Wide-character sentinel `|END` remains aligned.
4. Store screenshots as CI artifacts.
5. Require manual review until image diff is stable enough to fail builds.

## Beginner Workflow Regression Plan

High-priority scenarios:

1. First launch shows a shell prompt and usable terminal.
2. Natural-language command request produces a command.
3. Generated command opens Safe Mode.
4. Dangerous command can be canceled.
5. Safe command can be executed.
6. Snippet with variables can be inserted.
7. AI provider missing state is clear.
8. Split pane creates an independent terminal.
9. Search finds visible text.
10. App bundle launches after install.

## Test Data And Fixtures

Create fixtures under `tests/fixtures/`:

- `terminal/torture.expected.md` for visual expectations.
- `ssh/config.sample` for SSH parser and manager tests.
- `snippets/sample-snippets.toml` for snippet import/export.
- `ai/mock-provider-responses.json` for AI panel tests.
- `recordings/sample-recording.json` for playback tests.

## Coverage Reporting

Use coverage to find blind spots, not as the only quality gate.

Candidate command:

```bash
cargo llvm-cov --workspace --html
```

Targets:

- Core/config deterministic logic: high coverage expected.
- UI workflow logic: coverage through extracted controllers and E2E smoke tests.
- Renderer: coverage through engine tests plus screenshot regression, not line coverage alone.

## Release Verification Artifacts

Every release candidate should produce:

- Test command transcript.
- App bundle signature verification.
- Installed-app launch proof.
- Terminal torture screenshot.
- Safe Mode workflow screenshot or log.
- AI no-provider and mocked-provider verification.
- Known gaps list.
