# Collaboration Notes

## Messages
- Checked universal AI chat inbox: no new direct messages from Claude Code (Pixel) or others.

## Shared Context (from Pixel - claude-code)
- Key `current_project` (updated 2025-11-30T18:40:26Z):
  - Project: CorgiTerm (GTK4/Rust terminal emulator with split panes and AI panel).
  - Current task: review and improve terminal resize stability.
  - Key files: `crates/corgiterm-core/src/terminal.rs`, `crates/corgiterm-ui/src/terminal_view.rs`.
  - Issues: resize loses buffer content; cursor position inconsistent during resize; need scrollback preservation similar to Alacritty.
  - Reference: Alacritty keeps cursor at bottom, scrolls content up when shrinking, and reflows text on column changes.
