# Feature Completion Spec

Status date: 2026-06-03

This spec defines the work required to make CorgiTerm feature-complete against the current product promise.

## Feature Definition Of Done

A feature is complete only when:

- It is implemented in the product path users interact with.
- User-facing docs accurately describe its current behavior.
- It has deterministic tests for core logic.
- It has workflow tests for the real UI path or a stable UI model.
- It handles missing dependencies, offline mode, and failure states.
- It has an acceptance checklist in the release process.

## Terminal

Scope:

- PTY lifecycle.
- Terminal model and renderer.
- Scrollback, selection, copy/paste, search, hints.
- Resize/reflow.
- Alternate screen.
- Bell, title, cursor visibility, bracketed paste, and PTY replies.

Acceptance criteria:

- Shell prompt appears after app launch.
- `scripts/terminal-torture.sh` renders aligned text, box drawing, CJK, emoji, ANSI colors, 256 colors, truecolor, and attributes.
- `vim`, `less`, and `htop` render in alternate screen and return cleanly.
- Resize preserves readable output without ghost columns or broken cursor state.
- Copy selected text and copy visible screen work.
- Search highlights matches and navigates between them.
- URL/path hints detect and act on links and local paths.

Required tests:

- Engine unit tests for VT behavior.
- PTY integration test using a controlled shell command.
- Renderer screenshot regression test.
- Manual app verification until screenshot tests are stable.

## Safe Mode

Scope:

- Command risk analysis.
- Preview UI.
- Execute/cancel routing.
- Plain-English explanations.
- Safer alternatives and undo hints.

Acceptance criteria:

- Common read-only commands are safe.
- Destructive commands are caution or danger.
- Unknown commands are visibly unknown and still reviewable.
- Generated AI commands route through Safe Mode.
- Snippet execution routes through Safe Mode.
- Cancel never writes to the PTY.

Required tests:

- Unit tests for risk classification.
- UI tests for preview states.
- Integration tests for command routing.

## AI Assistant

Scope:

- Chat mode.
- Explain mode.
- Command mode.
- Provider detection.
- Local, CLI, and API providers.
- Learning context.

Acceptance criteria:

- No-provider state is useful and non-blocking.
- Mocked providers can exercise all AI panel modes.
- Provider timeouts and failures do not freeze the UI.
- Secrets are not logged.
- AI-generated commands are previewed before execution.
- Learning can be disabled and cleared.

Required tests:

- Provider mock tests.
- UI tests for mode switching, loading, success, and error states.
- Persistence tests for learning controls.

## Natural-Language Command Input

Scope:

- Deterministic quick translations.
- AI fallback.
- Display of suggested command.
- Safe Mode handoff.

Acceptance criteria:

- Common requests work without AI.
- AI fallback works with a mocked provider.
- Pressing Enter does not run a command directly unless Safe Mode approval occurs.
- User-visible errors are clear.

Required tests:

- Unit tests for quick translation.
- UI integration tests for suggested command and execution handoff.

## Snippets

Scope:

- CRUD.
- Variables, defaults, hints, tags, categories, pinned state.
- Insert and execute.
- Import/export.

Acceptance criteria:

- Users can create, edit, delete, search, filter, pin, insert, and execute snippets.
- Variables prompt before insert/execute.
- Executing a snippet uses Safe Mode.
- Import/export round-trips.

Required tests:

- Config/model unit tests.
- Dialog model tests.
- UI workflow tests.

## SSH Manager

Scope:

- Import from `~/.ssh/config`.
- Add/edit/delete hosts.
- Search/filter/favorites.
- Quick connect.
- Key display.
- Port forwarding presets.

Acceptance criteria:

- Parsed SSH config hosts appear in the manager.
- Add/edit/delete persists to config.
- Quick connect sends the expected SSH command to the active terminal through Safe Mode where appropriate.
- Keys are listed without exposing private key material.

Required tests:

- Parser unit tests.
- UI model tests for CRUD.
- Command-generation tests.

## Tabs, Splits, And Broadcast

Scope:

- New/close/switch tabs.
- Horizontal/vertical splits.
- Focus movement.
- Per-pane commands.
- Broadcast mode and targets.

Acceptance criteria:

- Each pane has its own PTY and terminal engine.
- Split panes resize correctly.
- Closing a pane does not close unrelated panes.
- Broadcast sends input only to selected target panes.

Required tests:

- Split tree model tests.
- UI keyboard workflow tests.
- PTY isolation tests.

## Recording And Playback

Scope:

- Start/stop recording.
- Capture PTY output, input, resize, and markers.
- Save/load recordings.
- Playback controls.

Acceptance criteria:

- Starting recording captures subsequent terminal I/O.
- Stopping recording saves a loadable file.
- Playback replays output in timestamp order at selected speed.
- UI reflects recording and playback state.

Required tests:

- Recording model tests.
- PTY tap integration tests.
- Playback event timing tests.
- Recording panel UI tests.

## Themes And Preferences

Scope:

- Built-in themes.
- Custom theme creator.
- Font, cursor, colors, shortcuts.
- Accessibility settings.

Acceptance criteria:

- Changing theme updates terminal colors.
- Font fallback remains monospace.
- Custom theme save/load works.
- Contrast warnings appear for low-contrast themes.
- Keyboard shortcuts persist and reload.

Required tests:

- Config serialization tests.
- Theme contrast tests.
- UI apply/reload tests.

## Plugins

Scope:

- Lua runtime.
- WASM runtime.
- Stable CorgiTerm API.
- Plugin loading and metadata.

Acceptance criteria:

- Sample Lua plugin executes against the real API.
- Sample WASM plugin executes against the real API.
- Plugin failures are isolated and logged.
- Plugin permissions are documented.

Required tests:

- Runtime execution tests.
- Sample plugin integration tests.
- Permission/error tests.

## MCP

Scope:

- Tool list.
- Tool call routing.
- Real terminal backend.
- Resources.

Acceptance criteria:

- MCP terminal tools operate on a real terminal backend or are clearly marked experimental.
- Placeholder responses are removed from production paths.
- Tool errors are structured and useful.

Required tests:

- Backend mock tests.
- JSON-RPC request/response tests.
- Real-backend smoke test.

## Distribution

Scope:

- macOS app bundle.
- Install script.
- Release artifacts.
- Linux build docs.

Acceptance criteria:

- Fresh checkout can build release.
- App bundle launches and spawns a shell.
- Installed app renders terminal torture output.
- Code signing status is documented.
- Release notes identify verified features and known gaps.

Required tests:

- Build script test.
- Bundle metadata/signature verification.
- Installed-app smoke test.
