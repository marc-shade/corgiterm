# Release Readiness Checklist

Status date: 2026-06-03

Use this checklist before tagging a release or telling users a feature is complete.

## 1. Repository State

- [ ] `git status --short --branch` is clean.
- [ ] Release branch is up to date with `origin/main`.
- [ ] Version number is updated where needed.
- [ ] README status and roadmap are accurate.
- [ ] Known gaps are documented.

## 2. Build And Static Gates

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo build --release --workspace`

If the clippy `-D warnings` gate is not yet achievable, record the exact warnings and use a no-new-warnings transition gate.

## 3. Terminal Verification

- [ ] Launch release binary.
- [ ] Confirm shell prompt appears.
- [ ] Run or inject `scripts/terminal-torture.sh`.
- [ ] Verify ASCII alignment.
- [ ] Verify box drawing alignment.
- [ ] Verify CJK and emoji do not overrun following text.
- [ ] Verify ANSI 16 colors.
- [ ] Verify 256-color ramp.
- [ ] Verify truecolor gradient.
- [ ] Verify bold, dim, italic, underline, strike, and inverse attributes.
- [ ] Verify cursor position and blink behavior.
- [ ] Verify resize/reflow.
- [ ] Verify alternate-screen apps: `vim`, `less`, and `htop` if available.

Artifact:

- [ ] Save screenshot path.
- [ ] Save command transcript or verification log.

## 4. Beginner Workflow Verification

- [ ] First launch creates a usable terminal.
- [ ] Natural-language quick translation works without AI.
- [ ] No-provider AI state is clear and non-blocking.
- [ ] Mocked or available provider can return a suggested command.
- [ ] Suggested command opens Safe Mode before execution.
- [ ] Dangerous command preview shows danger/caution state.
- [ ] Cancel does not execute a command.
- [ ] Execute writes the approved command to the active terminal.
- [ ] AI Explain mode handles pasted command/output.

## 5. Workflow Feature Verification

- [ ] Create, switch, and close tabs.
- [ ] Create horizontal and vertical splits.
- [ ] Confirm split panes have independent terminal sessions.
- [ ] Search visible terminal text.
- [ ] Copy and paste terminal text.
- [ ] Activate URL/path hints.
- [ ] Create and insert a snippet.
- [ ] Execute a snippet through Safe Mode.
- [ ] Open SSH manager.
- [ ] Import or display SSH config hosts.
- [ ] Quick connect generates the expected SSH command.
- [ ] Open theme creator.
- [ ] Save/apply a theme.
- [ ] Start and stop a recording.
- [ ] Load and play back a recording.

## 6. App Bundle Verification (macOS)

- [ ] Build release binary.
- [ ] Build or update `/Applications/CorgiTerm.app`.
- [ ] Re-sign app bundle if modified:

```bash
codesign --force --deep --sign - /Applications/CorgiTerm.app
```

- [ ] Verify signature:

```bash
codesign --verify --deep --strict --verbose=2 /Applications/CorgiTerm.app
```

- [ ] Launch from Finder or:

```bash
open /Applications/CorgiTerm.app
```

- [ ] Confirm the launched process is the installed app bundle.
- [ ] Run terminal verification inside the installed app.

## 7. Install Script Verification

- [ ] macOS install script downloads or builds the intended artifact.
- [ ] Dependencies are checked clearly.
- [ ] Quarantine guidance is correct.
- [ ] Install does not overwrite without clear user intent.
- [ ] Launch after install works.

## 8. Privacy And Safety

- [ ] No API keys are logged.
- [ ] AI requests do not include unnecessary terminal history.
- [ ] Learning/history can be disabled.
- [ ] Local history location is documented.
- [ ] Safe Mode remains enabled by default for beginner paths.

## 9. Release Notes

Release notes must include:

- [ ] User-visible changes.
- [ ] Verification performed.
- [ ] Known limitations.
- [ ] Upgrade/install instructions.
- [ ] Screenshots or GIFs for UI changes.

## 10. Final Release Decision

- [ ] All required checks passed.
- [ ] Any skipped checks have a documented reason and owner.
- [ ] Public docs do not overclaim incomplete features.
- [ ] Tag/release/announcement is approved.
