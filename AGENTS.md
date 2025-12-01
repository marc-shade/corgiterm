# Repository Guidelines

## Project Structure & Module Organization
- Root binary entry: `src/main.rs` launches the app.
- Workspace crates in `crates/`:
  - `corgiterm-core` (PTY, sessions, history, safe mode)
  - `corgiterm-ui` (GTK4/libadwaita UI, panes, dialogs, themes)
  - `corgiterm-ai` (providers, MCP integration, history/learning)
  - `corgiterm-config` (schemas, shortcuts, themes)
  - `corgiterm-plugins` (WASM/Lua runtime and API)
- Docs in `docs/` (features, shortcuts, AI learning); examples in `examples/`; packaging in `flatpak/`; helper scripts in `scripts/`.

## Build, Test, and Development Commands
- `cargo build --workspace` – compile all crates.
- `cargo run` – launch the GTK app from the workspace root.
- `cargo watch -x run` – iterative dev with auto-rebuild (requires `cargo-watch`).
- `cargo test --workspace` – run unit/integration tests for every crate.
- `cargo fmt --all` and `cargo clippy --workspace` – format and lint; run before commits.
- `scripts/build.sh` – convenience build script mirroring the standard commands.

## Coding Style & Naming Conventions
- Rust 1.75+; follow `rustfmt` defaults and resolve `clippy` warnings.
- Prefer descriptive module/file names mirroring types (e.g., `split_pane.rs` for split-pane widgets).
- Use `snake_case` for modules/functions, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for consts, and `kebab-case` for feature branches (`feature/*`, `fix/*`).
- Keep public APIs documented with `///` examples when behavior is non-obvious.

## Testing Guidelines
- Framework: built-in Rust test harness; integration tests live alongside crates.
- Name tests with intent (e.g., `handles_nested_panes`, `persists_safe_mode_flag`).
- Run `cargo test --workspace` plus focused crates (`-p corgiterm-core`, `-p corgiterm-ui`) after changes touching those areas.
- For UI logic, favor small pure functions where possible; gate GTK-dependent pieces behind feature flags or mocks when practical.

## Commit & Pull Request Guidelines
- Commit messages use Conventional Commits (`feat(ui): ...`, `fix(core): ...`, `docs: ...`).
- Keep commits scoped; include relevant `fmt`/`clippy` changes in the same commit.
- PRs should state scope, user-visible changes, and testing done; link issues and include screenshots/GIFs for UI changes.
- Ensure CI parity locally: `cargo fmt --all`, `cargo clippy --workspace`, `cargo test --workspace` before requesting review.

## Security & Configuration Tips
- Avoid hardcoding secrets; configuration belongs in `~/.config/corgiterm/config.toml`.
- When adding providers or plugins, validate inputs and favor existing safe-mode patterns in `corgiterm-core`.
