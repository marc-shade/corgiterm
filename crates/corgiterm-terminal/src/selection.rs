//! Text selection handling
//!
//! Manages rectangular and stream selections in the terminal grid.

use serde::{Deserialize, Serialize};

/// A point in the terminal grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Point {
    /// Column (0-indexed)
    pub col: usize,
    /// Row (0-indexed, can be negative for scrollback)
    pub row: isize,
}

impl Point {
    pub fn new(col: usize, row: isize) -> Self {
        Self { col, row }
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.row.cmp(&other.row) {
            std::cmp::Ordering::Equal => self.col.cmp(&other.col),
            ord => ord,
        }
    }
}

/// Selection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SelectionType {
    /// Character-by-character selection
    #[default]
    Simple,
    /// Word selection (double-click)
    Semantic,
    /// Line selection (triple-click)
    Lines,
    /// Block/rectangular selection
    Block,
}

/// A selection range in the terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionRange {
    /// Start point (may be after end point if selecting backwards)
    pub start: Point,
    /// End point
    pub end: Point,
    /// Selection type
    pub selection_type: SelectionType,
}

impl SelectionRange {
    /// Create a new selection range
    pub fn new(start: Point, end: Point, selection_type: SelectionType) -> Self {
        Self {
            start,
            end,
            selection_type,
        }
    }

    /// Get normalized range (start before end)
    pub fn normalized(&self) -> (Point, Point) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    /// Check if a point is within this selection
    pub fn contains(&self, point: Point) -> bool {
        let (start, end) = self.normalized();

        match self.selection_type {
            SelectionType::Block => {
                // Rectangular selection
                let min_col = start.col.min(end.col);
                let max_col = start.col.max(end.col);
                point.row >= start.row
                    && point.row <= end.row
                    && point.col >= min_col
                    && point.col <= max_col
            }
            _ => {
                // Stream selection
                if point.row < start.row || point.row > end.row {
                    return false;
                }
                if point.row == start.row && point.col < start.col {
                    return false;
                }
                if point.row == end.row && point.col > end.col {
                    return false;
                }
                true
            }
        }
    }

    /// Check if selection spans multiple lines
    pub fn is_multiline(&self) -> bool {
        self.start.row != self.end.row
    }

    /// Check if selection is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl Default for SelectionRange {
    fn default() -> Self {
        Self {
            start: Point::default(),
            end: Point::default(),
            selection_type: SelectionType::Simple,
        }
    }
}

/// Selection state machine
#[derive(Debug, Clone, Default)]
pub struct Selection {
    /// Current selection range (if any)
    range: Option<SelectionRange>,
    /// Selection in progress (mouse button held)
    in_progress: bool,
    /// Click count for multi-click detection
    click_count: u8,
    /// Last click position for multi-click detection
    last_click: Option<Point>,
    /// Last click time (for double/triple click detection)
    last_click_time: Option<std::time::Instant>,
}

impl Selection {
    /// Create a new selection state
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new selection
    pub fn start(&mut self, point: Point) {
        let now = std::time::Instant::now();
        let double_click_threshold = std::time::Duration::from_millis(500);

        // Detect multi-click
        let click_count =
            if let (Some(last_point), Some(last_time)) = (self.last_click, self.last_click_time) {
                if last_point == point && now.duration_since(last_time) < double_click_threshold {
                    (self.click_count % 3) + 1
                } else {
                    1
                }
            } else {
                1
            };

        self.click_count = click_count;
        self.last_click = Some(point);
        self.last_click_time = Some(now);

        let selection_type = match click_count {
            1 => SelectionType::Simple,
            2 => SelectionType::Semantic,
            3 => SelectionType::Lines,
            _ => SelectionType::Simple,
        };

        self.range = Some(SelectionRange::new(point, point, selection_type));
        self.in_progress = true;
    }

    /// Update selection as mouse moves
    pub fn update(&mut self, point: Point) {
        if !self.in_progress {
            return;
        }

        if let Some(ref mut range) = self.range {
            range.end = point;
        }
    }

    /// Finish selection
    pub fn finish(&mut self) {
        self.in_progress = false;

        // Remove empty selections
        if let Some(ref range) = self.range {
            if range.is_empty() {
                self.range = None;
            }
        }
    }

    /// Clear selection
    pub fn clear(&mut self) {
        self.range = None;
        self.in_progress = false;
    }

    /// Get current selection range
    pub fn range(&self) -> Option<&SelectionRange> {
        self.range.as_ref()
    }

    /// Check if selection is in progress
    pub fn in_progress(&self) -> bool {
        self.in_progress
    }

    /// Check if there is an active selection
    pub fn is_active(&self) -> bool {
        self.range.is_some()
    }

    /// Check if a point is selected
    pub fn is_selected(&self, point: Point) -> bool {
        self.range
            .as_ref()
            .map(|r| r.contains(point))
            .unwrap_or(false)
    }

    /// Set block selection mode (shift+click typically)
    pub fn set_block_mode(&mut self, enabled: bool) {
        if let Some(ref mut range) = self.range {
            range.selection_type = if enabled {
                SelectionType::Block
            } else {
                SelectionType::Simple
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_ordering() {
        let p1 = Point::new(5, 10);
        let p2 = Point::new(10, 10);
        let p3 = Point::new(5, 11);

        assert!(p1 < p2);
        assert!(p1 < p3);
        assert!(p2 < p3);
    }

    #[test]
    fn test_selection_contains() {
        let range = SelectionRange::new(Point::new(5, 0), Point::new(10, 2), SelectionType::Simple);

        // On start line
        assert!(range.contains(Point::new(5, 0)));
        assert!(range.contains(Point::new(10, 0)));
        assert!(!range.contains(Point::new(4, 0)));

        // Middle line
        assert!(range.contains(Point::new(0, 1)));
        assert!(range.contains(Point::new(100, 1)));

        // End line
        assert!(range.contains(Point::new(0, 2)));
        assert!(range.contains(Point::new(10, 2)));
        assert!(!range.contains(Point::new(11, 2)));
    }

    #[test]
    fn test_block_selection() {
        let range = SelectionRange::new(Point::new(5, 0), Point::new(10, 2), SelectionType::Block);

        assert!(range.contains(Point::new(7, 1)));
        assert!(!range.contains(Point::new(3, 1)));
        assert!(!range.contains(Point::new(12, 1)));
    }

    #[test]
    fn test_selection_lifecycle() {
        let mut selection = Selection::new();

        assert!(!selection.is_active());

        selection.start(Point::new(5, 0));
        assert!(selection.in_progress());
        assert!(selection.is_active());

        selection.update(Point::new(10, 2));
        selection.finish();

        assert!(!selection.in_progress());
        assert!(selection.is_active());
        assert!(selection.is_selected(Point::new(7, 1)));

        selection.clear();
        assert!(!selection.is_active());
    }
}
