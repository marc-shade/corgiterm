//! Terminal engine abstraction.
//!
//! Defines [`TerminalEngine`], a neutral contract for a terminal model (parse PTY
//! bytes, maintain a grid + scrollback, expose renderable cells + cursor), and
//! [`AlacrittyEngine`], an adapter over the battle-tested `alacritty_terminal`
//! crate (grid, VT parser, scrollback, wide-char flags, reflow).
//!
//! Why this exists: the original hand-rolled [`crate::terminal::Terminal`] only
//! implemented a handful of CSI sequences (no alternate screen, scroll regions,
//! insert/delete line, wide characters, etc.), and the UI rendered each cell at
//! its natural glyph advance against an averaged grid pitch. Both produced the
//! garbled / overlapping text. `alacritty_terminal` fixes the model layer; the
//! renderer is corrected separately to snap to a fixed grid and honor wide cells.
//! See `docs/plans/terminal-rebuild-plan.md`.
//!
//! Threading: the UI marshals PTY bytes onto the GTK main thread (a reader thread
//! ships chunks over a channel, a `glib` timeout drains them), so the engine is
//! used single-threaded inside `Rc<RefCell<..>>`. No `FairMutex`/`Send` is needed.

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::term::{Config, Term};
use alacritty_terminal::vte::ansi::{Color, CursorShape, NamedColor, Processor};

use crate::terminal::{ClipboardAction, TerminalEvent, TerminalSize};

/// A logical color for a rendered cell. Resolution to concrete RGBA is the
/// renderer's job (it owns the active theme palette), so the model layer never
/// hardcodes colors. `DefaultFg`/`DefaultBg` mean "inherit the theme default";
/// a cell whose `bg` is `DefaultBg` should not have its background painted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellColor {
    DefaultFg,
    DefaultBg,
    /// Palette index: 0-15 are the themed ANSI colors, 16-231 the 6x6x6 cube,
    /// 232-255 the grayscale ramp.
    Indexed(u8),
    /// True color.
    Rgb([u8; 3]),
}

/// Style flags for a rendered cell (engine-agnostic subset).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RenderFlags {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub dim: bool,
    pub inverse: bool,
    pub hidden: bool,
}

/// One renderable cell, positioned on the visible grid (row 0 = top of the
/// visible viewport, accounting for any scrollback offset). Wide cells report
/// `width == 2` and the renderer must span two columns; the trailing spacer
/// cell is not emitted.
#[derive(Debug, Clone)]
pub struct RenderCell {
    pub row: usize,
    pub col: usize,
    /// Grapheme to draw (base char plus any combining marks). Empty for a blank
    /// cell (the renderer draws only the background).
    pub text: String,
    pub fg: CellColor,
    pub bg: CellColor,
    pub flags: RenderFlags,
    /// Column span: 1 for normal cells, 2 for wide (CJK / emoji) cells.
    pub width: u8,
}

/// Cursor shape, neutral over engines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineCursorShape {
    Block,
    Underline,
    Bar,
    Hidden,
}

/// Cursor render state in viewport coordinates.
#[derive(Debug, Clone, Copy)]
pub struct EngineCursor {
    pub row: usize,
    pub col: usize,
    /// False when the cursor is hidden (DECTCEM off) or scrolled out of view.
    pub visible: bool,
    pub shape: EngineCursorShape,
}

/// The neutral terminal-model contract the renderer and feature code use.
pub trait TerminalEngine {
    /// Feed raw bytes read from the PTY.
    fn feed(&mut self, bytes: &[u8]);
    /// Resize the grid (in cells).
    fn resize(&mut self, size: TerminalSize);
    /// Current grid size (in cells).
    fn size(&self) -> TerminalSize;
    /// All visible cells at the current scroll position (row-major, spacers
    /// elided). Blank cells carry an empty `text`.
    fn render_cells(&self) -> Vec<RenderCell>;
    /// The visible rows as plain strings (blanks as spaces), for URL/hint/search
    /// detection and "read the screen" features.
    fn rows_text(&self) -> Vec<String>;
    /// The entire buffer (scrollback history plus the visible screen) as plain
    /// strings, oldest first. Used by select-all / copy-all.
    fn all_text(&self) -> Vec<String>;
    /// Cursor render state.
    fn cursor(&self) -> EngineCursor;
    /// How many lines the view is currently scrolled up into history (0 = bottom).
    fn display_offset(&self) -> usize;
    /// Number of lines currently in scrollback history.
    fn scrollback_len(&self) -> usize;
    /// Scroll the display by `delta` lines (positive = up into history).
    fn scroll_lines(&mut self, delta: i32);
    /// Jump back to the live bottom of the buffer.
    fn scroll_to_bottom(&mut self);
    /// Currently selected text, if any.
    fn selection_text(&self) -> Option<String>;
    /// Adjust the maximum scrollback (in lines).
    fn set_max_scrollback(&mut self, lines: usize);
}

/// Minimal [`Dimensions`] for constructing / resizing a `Term`. History is
/// tracked separately via [`Config::scrolling_history`], so `total_lines`
/// equals `screen_lines` here.
struct Dims {
    columns: usize,
    screen_lines: usize,
}

impl Dimensions for Dims {
    fn total_lines(&self) -> usize {
        self.screen_lines
    }
    fn screen_lines(&self) -> usize {
        self.screen_lines
    }
    fn columns(&self) -> usize {
        self.columns
    }
}

/// Translates `alacritty_terminal` events onto CorgiTerm's existing
/// [`TerminalEvent`] channel. Crucially forwards `PtyWrite` so terminal replies
/// (DSR, bracketed-paste acks, device attributes) reach the child process.
#[derive(Clone)]
pub struct EventProxy {
    tx: crossbeam_channel::Sender<TerminalEvent>,
}

impl EventListener for EventProxy {
    fn send_event(&self, event: Event) {
        let mapped = match event {
            Event::Title(title) => Some(TerminalEvent::TitleChanged(title)),
            Event::ResetTitle => Some(TerminalEvent::TitleChanged(String::new())),
            Event::Bell => Some(TerminalEvent::Bell),
            Event::PtyWrite(text) => Some(TerminalEvent::PtyWrite(text.into_bytes())),
            Event::ClipboardStore(_, text) => {
                Some(TerminalEvent::Clipboard(ClipboardAction::Copy(text)))
            }
            // ClipboardLoad / ColorRequest / TextAreaSizeRequest carry reply
            // closures; Wakeup / MouseCursorDirty / CursorBlinkingChange / Exit /
            // ChildExit are not surfaced through this channel.
            _ => None,
        };
        if let Some(ev) = mapped {
            let _ = self.tx.send(ev);
        }
    }
}

/// Terminal model backed by `alacritty_terminal`.
pub struct AlacrittyEngine {
    term: Term<EventProxy>,
    processor: Processor,
    size: TerminalSize,
}

impl AlacrittyEngine {
    /// Create an engine of the given size, emitting events on `events`, with
    /// `max_scrollback` lines of history.
    pub fn new(
        size: TerminalSize,
        events: crossbeam_channel::Sender<TerminalEvent>,
        max_scrollback: usize,
    ) -> Self {
        let dims = Dims {
            columns: size.cols.max(1),
            screen_lines: size.rows.max(1),
        };
        let config = Config {
            scrolling_history: max_scrollback,
            ..Config::default()
        };
        let term = Term::new(config, &dims, EventProxy { tx: events });
        Self {
            term,
            processor: Processor::new(),
            size,
        }
    }
}

/// Map an `alacritty_terminal` cell color to a neutral [`CellColor`].
fn map_color(color: Color) -> CellColor {
    match color {
        Color::Spec(rgb) => CellColor::Rgb([rgb.r, rgb.g, rgb.b]),
        Color::Indexed(i) => CellColor::Indexed(i),
        Color::Named(named) => match named {
            NamedColor::Foreground
            | NamedColor::BrightForeground
            | NamedColor::DimForeground
            | NamedColor::Cursor => CellColor::DefaultFg,
            NamedColor::Background => CellColor::DefaultBg,
            NamedColor::Black => CellColor::Indexed(0),
            NamedColor::Red => CellColor::Indexed(1),
            NamedColor::Green => CellColor::Indexed(2),
            NamedColor::Yellow => CellColor::Indexed(3),
            NamedColor::Blue => CellColor::Indexed(4),
            NamedColor::Magenta => CellColor::Indexed(5),
            NamedColor::Cyan => CellColor::Indexed(6),
            NamedColor::White => CellColor::Indexed(7),
            NamedColor::BrightBlack => CellColor::Indexed(8),
            NamedColor::BrightRed => CellColor::Indexed(9),
            NamedColor::BrightGreen => CellColor::Indexed(10),
            NamedColor::BrightYellow => CellColor::Indexed(11),
            NamedColor::BrightBlue => CellColor::Indexed(12),
            NamedColor::BrightMagenta => CellColor::Indexed(13),
            NamedColor::BrightCyan => CellColor::Indexed(14),
            NamedColor::BrightWhite => CellColor::Indexed(15),
            NamedColor::DimBlack => CellColor::Indexed(0),
            NamedColor::DimRed => CellColor::Indexed(1),
            NamedColor::DimGreen => CellColor::Indexed(2),
            NamedColor::DimYellow => CellColor::Indexed(3),
            NamedColor::DimBlue => CellColor::Indexed(4),
            NamedColor::DimMagenta => CellColor::Indexed(5),
            NamedColor::DimCyan => CellColor::Indexed(6),
            NamedColor::DimWhite => CellColor::Indexed(7),
        },
    }
}

fn map_flags(flags: Flags) -> RenderFlags {
    RenderFlags {
        bold: flags.contains(Flags::BOLD),
        italic: flags.contains(Flags::ITALIC),
        underline: flags.intersects(Flags::ALL_UNDERLINES),
        strikethrough: flags.contains(Flags::STRIKEOUT),
        dim: flags.contains(Flags::DIM),
        inverse: flags.contains(Flags::INVERSE),
        hidden: flags.contains(Flags::HIDDEN),
    }
}

fn map_shape(shape: CursorShape) -> EngineCursorShape {
    match shape {
        CursorShape::Block | CursorShape::HollowBlock => EngineCursorShape::Block,
        CursorShape::Underline => EngineCursorShape::Underline,
        CursorShape::Beam => EngineCursorShape::Bar,
        CursorShape::Hidden => EngineCursorShape::Hidden,
    }
}

/// Build a cell's text (base char + combining marks). Returns empty for a blank
/// cell so the renderer can skip the glyph pass.
fn cell_text(cell: &Cell) -> String {
    let zerowidth = cell.zerowidth();
    let blank = cell.c == ' ' && zerowidth.is_none_or(|z| z.is_empty());
    if blank {
        return String::new();
    }
    let mut s = String::with_capacity(1 + zerowidth.map_or(0, <[char]>::len));
    s.push(cell.c);
    if let Some(zw) = zerowidth {
        s.extend(zw.iter().copied());
    }
    s
}

/// True for spacer cells that must not be emitted (the wide glyph spans them).
fn is_spacer(flags: Flags) -> bool {
    flags.contains(Flags::WIDE_CHAR_SPACER) || flags.contains(Flags::LEADING_WIDE_CHAR_SPACER)
}

impl TerminalEngine for AlacrittyEngine {
    fn feed(&mut self, bytes: &[u8]) {
        self.processor.advance(&mut self.term, bytes);
    }

    fn resize(&mut self, size: TerminalSize) {
        let dims = Dims {
            columns: size.cols.max(1),
            screen_lines: size.rows.max(1),
        };
        self.term.resize(dims);
        self.size = size;
    }

    fn size(&self) -> TerminalSize {
        self.size
    }

    fn render_cells(&self) -> Vec<RenderCell> {
        let content = self.term.renderable_content();
        let offset = content.display_offset as i32;
        let mut cells = Vec::new();
        for indexed in content.display_iter {
            let cell = indexed.cell;
            let flags = cell.flags;
            if is_spacer(flags) {
                continue;
            }
            let row = indexed.point.line.0 + offset;
            if row < 0 {
                continue;
            }
            cells.push(RenderCell {
                row: row as usize,
                col: indexed.point.column.0,
                text: cell_text(cell),
                fg: map_color(cell.fg),
                bg: map_color(cell.bg),
                flags: map_flags(flags),
                width: if flags.contains(Flags::WIDE_CHAR) {
                    2
                } else {
                    1
                },
            });
        }
        cells
    }

    fn rows_text(&self) -> Vec<String> {
        let rows = self.size.rows;
        let cols = self.size.cols;
        let mut grid = vec![vec![String::from(" "); cols]; rows];
        let content = self.term.renderable_content();
        let offset = content.display_offset as i32;
        for indexed in content.display_iter {
            let cell = indexed.cell;
            if is_spacer(cell.flags) {
                continue;
            }
            let row = indexed.point.line.0 + offset;
            let col = indexed.point.column.0;
            if row < 0 || row as usize >= rows || col >= cols {
                continue;
            }
            let text = cell_text(cell);
            if !text.is_empty() {
                grid[row as usize][col] = text;
            }
        }
        grid.into_iter()
            .map(|r| r.concat().trim_end().to_string())
            .collect()
    }

    fn all_text(&self) -> Vec<String> {
        let grid = self.term.grid();
        let cols = grid.columns();
        let top = grid.topmost_line().0;
        let bottom = grid.bottommost_line().0;
        let mut out = Vec::with_capacity((bottom - top + 1).max(0) as usize);
        for line in top..=bottom {
            let row = &grid[Line(line)];
            let mut s = String::new();
            for c in 0..cols {
                let cell = &row[Column(c)];
                if !is_spacer(cell.flags) {
                    let text = cell_text(cell);
                    if text.is_empty() {
                        s.push(' ');
                    } else {
                        s.push_str(&text);
                    }
                }
            }
            out.push(s.trim_end().to_string());
        }
        out
    }

    fn cursor(&self) -> EngineCursor {
        let content = self.term.renderable_content();
        let offset = content.display_offset as i32;
        let cursor = content.cursor;
        let shape = map_shape(cursor.shape);
        let row = cursor.point.line.0 + offset;
        let in_view = row >= 0 && (row as usize) < self.size.rows;
        EngineCursor {
            row: row.max(0) as usize,
            col: cursor.point.column.0,
            visible: in_view && shape != EngineCursorShape::Hidden,
            shape,
        }
    }

    fn display_offset(&self) -> usize {
        self.term.grid().display_offset()
    }

    fn scrollback_len(&self) -> usize {
        self.term
            .grid()
            .total_lines()
            .saturating_sub(self.term.grid().screen_lines())
    }

    fn scroll_lines(&mut self, delta: i32) {
        self.term.scroll_display(Scroll::Delta(delta));
    }

    fn scroll_to_bottom(&mut self) {
        let offset = self.term.grid().display_offset() as i32;
        if offset != 0 {
            self.term.scroll_display(Scroll::Delta(-offset));
        }
    }

    fn selection_text(&self) -> Option<String> {
        self.term.selection_to_string()
    }

    fn set_max_scrollback(&mut self, lines: usize) {
        self.term.grid_mut().update_history(lines);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::TerminalEvent;
    use std::time::Duration;

    fn engine(rows: usize, cols: usize) -> AlacrittyEngine {
        let (tx, _rx) = crossbeam_channel::unbounded();
        AlacrittyEngine::new(TerminalSize { rows, cols }, tx, 1000)
    }

    fn engine_with_events(
        rows: usize,
        cols: usize,
    ) -> (AlacrittyEngine, crossbeam_channel::Receiver<TerminalEvent>) {
        let (tx, rx) = crossbeam_channel::unbounded();
        (
            AlacrittyEngine::new(TerminalSize { rows, cols }, tx, 1000),
            rx,
        )
    }

    /// Find the rendered cell at a viewport coordinate.
    fn at(cells: &[RenderCell], row: usize, col: usize) -> Option<&RenderCell> {
        cells.iter().find(|c| c.row == row && c.col == col)
    }

    #[test]
    fn plain_ascii_lands_in_order() {
        let mut e = engine(24, 80);
        e.feed(b"Hello");
        let cells = e.render_cells();
        assert_eq!(at(&cells, 0, 0).unwrap().text, "H");
        assert_eq!(at(&cells, 0, 4).unwrap().text, "o");
        // Cursor advanced one column per ASCII char.
        assert_eq!(e.cursor().col, 5);
    }

    #[test]
    fn wide_char_occupies_two_columns_and_advances_cursor_by_two() {
        let mut e = engine(24, 80);
        // CJK ideograph is two columns wide.
        e.feed("世".as_bytes());
        let cells = e.render_cells();
        let wide = at(&cells, 0, 0).expect("wide cell present");
        assert_eq!(wide.text, "世");
        assert_eq!(wide.width, 2, "CJK cell must be width 2");
        // The spacer column is elided, not a second glyph.
        assert!(
            at(&cells, 0, 1).is_none(),
            "spacer cell must not be emitted"
        );
        // Cursor advanced two columns over the wide char.
        assert_eq!(e.cursor().col, 2);
    }

    #[test]
    fn emoji_occupies_two_columns_and_preserves_text() {
        let mut e = engine(24, 80);
        e.feed("😀".as_bytes());
        let cells = e.render_cells();
        let emoji = at(&cells, 0, 0).expect("emoji cell present");
        assert_eq!(emoji.text, "😀");
        assert_eq!(emoji.width, 2, "emoji cell must be width 2");
        assert!(
            at(&cells, 0, 1).is_none(),
            "emoji spacer cell must not be emitted"
        );
        assert_eq!(e.cursor().col, 2);
    }

    #[test]
    fn truecolor_sgr_resolves_to_rgb() {
        let mut e = engine(24, 80);
        // SGR 38;2;10;20;30 sets a 24-bit foreground.
        e.feed(b"\x1b[38;2;10;20;30mX");
        let cells = e.render_cells();
        let cell = at(&cells, 0, 0).expect("cell present");
        assert_eq!(cell.text, "X");
        assert_eq!(cell.fg, CellColor::Rgb([10, 20, 30]));
    }

    #[test]
    fn indexed_and_named_colors_map() {
        let mut e = engine(24, 80);
        // SGR 31 = red foreground (named -> indexed 1), 1 = bold.
        e.feed(b"\x1b[1;31mA");
        let cells = e.render_cells();
        let cell = at(&cells, 0, 0).unwrap();
        assert_eq!(cell.fg, CellColor::Indexed(1));
        assert!(cell.flags.bold);
        // Default background is not painted (inherits theme).
        assert_eq!(cell.bg, CellColor::DefaultBg);
    }

    #[test]
    fn newline_moves_to_next_row() {
        let mut e = engine(24, 80);
        e.feed(b"a\r\nb");
        let cells = e.render_cells();
        assert_eq!(at(&cells, 0, 0).unwrap().text, "a");
        assert_eq!(at(&cells, 1, 0).unwrap().text, "b");
    }

    #[test]
    fn rows_text_reads_the_screen() {
        let mut e = engine(24, 80);
        e.feed(b"echo hi");
        let rows = e.rows_text();
        assert_eq!(rows[0], "echo hi");
    }

    #[test]
    fn rows_text_preserves_combining_marks_and_wide_spacing() {
        let mut e = engine(24, 80);
        e.feed("e\u{0301}世X".as_bytes());
        let rows = e.rows_text();
        assert_eq!(rows[0], "e\u{0301}世 X");
    }

    #[test]
    fn all_text_includes_scrollback() {
        // Small screen so early lines scroll into history.
        let mut e = engine(3, 20);
        for i in 0..10 {
            e.feed(format!("line{i}\r\n").as_bytes());
        }
        let all = e.all_text();
        assert!(
            all.iter().any(|l| l == "line0"),
            "scrollback line0 missing from all_text: {all:?}"
        );
        assert!(
            all.iter().any(|l| l == "line9"),
            "recent line9 missing from all_text"
        );
    }

    #[test]
    fn cursor_visibility_follows_dectcem() {
        let mut e = engine(24, 80);
        assert!(e.cursor().visible);

        // DECTCEM private mode: hide/show cursor.
        e.feed(b"\x1b[?25l");
        assert!(!e.cursor().visible);
        e.feed(b"\x1b[?25h");
        assert!(e.cursor().visible);
    }

    #[test]
    fn alternate_screen_restores_primary_screen() {
        let mut e = engine(5, 20);
        e.feed(b"primary");
        assert_eq!(e.rows_text()[0], "primary");

        // 1049 swaps to the alternate screen and saves/restores the primary.
        e.feed(b"\x1b[?1049h\x1b[Halternate");
        assert_eq!(e.rows_text()[0], "alternate");

        e.feed(b"\x1b[?1049l");
        assert_eq!(e.rows_text()[0], "primary");
    }

    #[test]
    fn osc_title_events_are_forwarded() {
        let (mut e, rx) = engine_with_events(24, 80);
        e.feed(b"\x1b]0;CorgiTerm Test\x07");

        let event = rx
            .recv_timeout(Duration::from_millis(100))
            .expect("title event should be forwarded");
        match event {
            TerminalEvent::TitleChanged(title) => assert_eq!(title, "CorgiTerm Test"),
            other => panic!("expected TitleChanged event, got {other:?}"),
        }
    }

    #[test]
    fn device_status_report_is_forwarded_to_pty() {
        let (mut e, rx) = engine_with_events(24, 80);
        e.feed(b"\x1b[6n");

        let event = rx
            .recv_timeout(Duration::from_millis(100))
            .expect("DSR response should be forwarded to PTY");
        match event {
            TerminalEvent::PtyWrite(bytes) => {
                assert_eq!(String::from_utf8_lossy(&bytes), "\x1b[1;1R");
            }
            other => panic!("expected PtyWrite event, got {other:?}"),
        }
    }
}
