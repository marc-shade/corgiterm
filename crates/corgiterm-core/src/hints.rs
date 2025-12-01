//! URL and path hint detection for keyboard-driven navigation
//!
//! Inspired by foot terminal's URL hints feature - scan terminal buffer
//! for clickable items and assign keyboard shortcuts for quick access.

use regex::Regex;
use std::sync::LazyLock;

/// Types of hints that can be detected
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HintType {
    /// HTTP/HTTPS URL
    Url,
    /// File path (absolute or relative)
    FilePath,
    /// IP address (v4 or v6)
    IpAddress,
    /// Email address
    Email,
    /// Git hash (short or full)
    GitHash,
    /// Port number (e.g., :8080)
    Port,
}

impl HintType {
    /// Get the action description for this hint type
    pub fn action_description(&self) -> &'static str {
        match self {
            HintType::Url => "Open in browser",
            HintType::FilePath => "Open/Navigate",
            HintType::IpAddress => "Copy to clipboard",
            HintType::Email => "Copy to clipboard",
            HintType::GitHash => "Copy to clipboard",
            HintType::Port => "Copy to clipboard",
        }
    }

    /// Get the icon for this hint type (for UI display)
    pub fn icon(&self) -> &'static str {
        match self {
            HintType::Url => "",
            HintType::FilePath => "",
            HintType::IpAddress => "",
            HintType::Email => "",
            HintType::GitHash => "",
            HintType::Port => "",
        }
    }
}

/// A detected hint in the terminal buffer
#[derive(Debug, Clone)]
pub struct Hint {
    /// The hint label (a, b, c... aa, ab, etc.)
    pub label: String,
    /// The matched text
    pub text: String,
    /// Type of hint
    pub hint_type: HintType,
    /// Row in terminal buffer (0-indexed from visible top)
    pub row: usize,
    /// Start column (0-indexed)
    pub col_start: usize,
    /// End column (exclusive)
    pub col_end: usize,
}

impl Hint {
    /// Check if a position is within this hint
    pub fn contains(&self, row: usize, col: usize) -> bool {
        self.row == row && col >= self.col_start && col < self.col_end
    }
}

/// Hint detector - scans text for various patterns
pub struct HintDetector {
    /// Enable URL detection
    pub detect_urls: bool,
    /// Enable file path detection
    pub detect_paths: bool,
    /// Enable IP address detection
    pub detect_ips: bool,
    /// Enable email detection
    pub detect_emails: bool,
    /// Enable git hash detection
    pub detect_git_hashes: bool,
}

impl Default for HintDetector {
    fn default() -> Self {
        Self {
            detect_urls: true,
            detect_paths: true,
            detect_ips: true,
            detect_emails: true,
            detect_git_hashes: true,
        }
    }
}

// Compiled regex patterns (lazy static for performance)
static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"https?://[^\s<>"'`\]\)]+"#).expect("Invalid URL regex")
});

static FILE_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Match absolute paths and relative paths with extensions
    Regex::new(r"(?:^|[\s:])(/[^\s:]+|\.{1,2}/[^\s:]+|[a-zA-Z0-9_-]+\.[a-zA-Z0-9]+)(?::\d+)?")
        .expect("Invalid file path regex")
});

static IP_V4_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}(?::\d+)?\b").expect("Invalid IPv4 regex")
});

static IP_V6_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[?[0-9a-fA-F:]+:[0-9a-fA-F:]+\]?(?::\d+)?").expect("Invalid IPv6 regex")
});

static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}\b").expect("Invalid email regex")
});

static GIT_HASH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Match 7-40 character hex strings that look like git hashes
    Regex::new(r"\b[0-9a-f]{7,40}\b").expect("Invalid git hash regex")
});

impl HintDetector {
    /// Create a new hint detector with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Scan a single line for hints
    fn scan_line(&self, line: &str, _row: usize, hints: &mut Vec<(HintType, String, usize, usize)>) {
        // URLs (highest priority - check first)
        if self.detect_urls {
            for mat in URL_REGEX.find_iter(line) {
                hints.push((HintType::Url, mat.as_str().to_string(), mat.start(), mat.end()));
            }
        }

        // Emails
        if self.detect_emails {
            for mat in EMAIL_REGEX.find_iter(line) {
                // Skip if overlaps with URL
                let start = mat.start();
                let end = mat.end();
                if !hints.iter().any(|(_, _, s, e)| start < *e && end > *s) {
                    hints.push((HintType::Email, mat.as_str().to_string(), start, end));
                }
            }
        }

        // IP addresses
        if self.detect_ips {
            for mat in IP_V4_REGEX.find_iter(line) {
                let start = mat.start();
                let end = mat.end();
                // Validate IP components are <= 255
                let ip_text = mat.as_str().split(':').next().unwrap_or("");
                let valid = ip_text.split('.').all(|part| {
                    part.parse::<u16>().map(|n| n <= 255).unwrap_or(false)
                });
                if valid && !hints.iter().any(|(_, _, s, e)| start < *e && end > *s) {
                    hints.push((HintType::IpAddress, mat.as_str().to_string(), start, end));
                }
            }

            for mat in IP_V6_REGEX.find_iter(line) {
                let start = mat.start();
                let end = mat.end();
                if !hints.iter().any(|(_, _, s, e)| start < *e && end > *s) {
                    hints.push((HintType::IpAddress, mat.as_str().to_string(), start, end));
                }
            }
        }

        // File paths
        if self.detect_paths {
            for mat in FILE_PATH_REGEX.find_iter(line) {
                let start = mat.start();
                let end = mat.end();
                let text = mat.as_str().trim_start_matches(|c: char| c.is_whitespace() || c == ':');
                let actual_start = start + (mat.as_str().len() - text.len());

                // Skip if overlaps with existing hints
                if !hints.iter().any(|(_, _, s, e)| actual_start < *e && end > *s) {
                    // Skip common false positives
                    if !text.starts_with("http") && text.len() > 2 {
                        hints.push((HintType::FilePath, text.to_string(), actual_start, end));
                    }
                }
            }
        }

        // Git hashes (lower priority - many false positives possible)
        if self.detect_git_hashes {
            for mat in GIT_HASH_REGEX.find_iter(line) {
                let start = mat.start();
                let end = mat.end();
                let text = mat.as_str();

                // Only match if it looks like a git hash context
                // (preceded by certain keywords or in a specific format)
                let before = if start > 0 { &line[..start] } else { "" };
                let is_git_context = before.ends_with("commit ")
                    || before.ends_with("hash ")
                    || before.ends_with("ref ")
                    || before.contains("git ")
                    || text.len() >= 40; // Full SHA is almost certainly a hash

                if is_git_context && !hints.iter().any(|(_, _, s, e)| start < *e && end > *s) {
                    hints.push((HintType::GitHash, text.to_string(), start, end));
                }
            }
        }
    }

    /// Scan multiple lines and return labeled hints
    pub fn scan(&self, lines: &[String]) -> Vec<Hint> {
        let mut raw_hints: Vec<(usize, HintType, String, usize, usize)> = Vec::new();

        // Collect all hints from all lines
        for (row, line) in lines.iter().enumerate() {
            let mut line_hints = Vec::new();
            self.scan_line(line, row, &mut line_hints);

            for (hint_type, text, col_start, col_end) in line_hints {
                raw_hints.push((row, hint_type, text, col_start, col_end));
            }
        }

        // Sort by row, then by column (top-to-bottom, left-to-right)
        raw_hints.sort_by(|a, b| {
            a.0.cmp(&b.0).then(a.3.cmp(&b.3))
        });

        // Assign labels
        let mut hints = Vec::with_capacity(raw_hints.len());
        for (idx, (row, hint_type, text, col_start, col_end)) in raw_hints.into_iter().enumerate() {
            hints.push(Hint {
                label: index_to_label(idx),
                text,
                hint_type,
                row,
                col_start,
                col_end,
            });
        }

        hints
    }

    /// Find a hint by its label
    pub fn find_by_label<'a>(&self, hints: &'a [Hint], label: &str) -> Option<&'a Hint> {
        hints.iter().find(|h| h.label.eq_ignore_ascii_case(label))
    }
}

/// Convert an index to a hint label (a, b, c... z, aa, ab, etc.)
fn index_to_label(idx: usize) -> String {
    let mut result = String::new();
    let mut n = idx;

    loop {
        result.insert(0, (b'a' + (n % 26) as u8) as char);
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }

    result
}

/// Convert a label back to an index
pub fn label_to_index(label: &str) -> Option<usize> {
    if label.is_empty() {
        return None;
    }

    let mut result = 0usize;
    for (i, c) in label.chars().rev().enumerate() {
        if !c.is_ascii_lowercase() {
            return None;
        }
        let digit = (c as usize) - ('a' as usize);
        if i == 0 {
            result = digit;
        } else {
            result += (digit + 1) * 26usize.pow(i as u32);
        }
    }

    Some(result)
}

/// State for hint mode interaction
#[derive(Debug, Default)]
pub struct HintModeState {
    /// Currently active hints
    pub hints: Vec<Hint>,
    /// Current input buffer for multi-character labels
    pub input_buffer: String,
    /// Whether hint mode is active
    pub active: bool,
}

impl HintModeState {
    /// Create a new hint mode state
    pub fn new() -> Self {
        Self::default()
    }

    /// Activate hint mode with the given hints
    pub fn activate(&mut self, hints: Vec<Hint>) {
        self.hints = hints;
        self.input_buffer.clear();
        self.active = true;
    }

    /// Deactivate hint mode
    pub fn deactivate(&mut self) {
        self.hints.clear();
        self.input_buffer.clear();
        self.active = false;
    }

    /// Handle a key press, returns the matched hint if found
    pub fn handle_key(&mut self, c: char) -> Option<Hint> {
        if !self.active {
            return None;
        }

        if c.is_ascii_lowercase() {
            self.input_buffer.push(c);

            // Check for exact match
            if let Some(hint) = self.hints.iter().find(|h| h.label == self.input_buffer) {
                let result = hint.clone();
                self.deactivate();
                return Some(result);
            }

            // Check if any hints start with current buffer
            let has_prefix = self.hints.iter().any(|h| h.label.starts_with(&self.input_buffer));
            if !has_prefix {
                // No matches possible, clear buffer
                self.input_buffer.clear();
            }
        }

        None
    }

    /// Get hints that match the current input buffer
    pub fn matching_hints(&self) -> Vec<&Hint> {
        if self.input_buffer.is_empty() {
            self.hints.iter().collect()
        } else {
            self.hints
                .iter()
                .filter(|h| h.label.starts_with(&self.input_buffer))
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_to_label() {
        assert_eq!(index_to_label(0), "a");
        assert_eq!(index_to_label(1), "b");
        assert_eq!(index_to_label(25), "z");
        assert_eq!(index_to_label(26), "aa");
        assert_eq!(index_to_label(27), "ab");
        assert_eq!(index_to_label(51), "az");
        assert_eq!(index_to_label(52), "ba");
    }

    #[test]
    fn test_label_to_index() {
        assert_eq!(label_to_index("a"), Some(0));
        assert_eq!(label_to_index("b"), Some(1));
        assert_eq!(label_to_index("z"), Some(25));
        assert_eq!(label_to_index("aa"), Some(26));
        assert_eq!(label_to_index("ab"), Some(27));
    }

    #[test]
    fn test_url_detection() {
        let detector = HintDetector::new();
        let lines = vec![
            "Check out https://github.com/user/repo for more".to_string(),
            "Also see http://example.com/path?query=1".to_string(),
        ];

        let hints = detector.scan(&lines);
        assert_eq!(hints.len(), 2);
        assert_eq!(hints[0].hint_type, HintType::Url);
        assert_eq!(hints[0].text, "https://github.com/user/repo");
        assert_eq!(hints[1].hint_type, HintType::Url);
    }

    #[test]
    fn test_email_detection() {
        let detector = HintDetector::new();
        let lines = vec!["Contact user@example.com for help".to_string()];

        let hints = detector.scan(&lines);
        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0].hint_type, HintType::Email);
        assert_eq!(hints[0].text, "user@example.com");
    }

    #[test]
    fn test_ip_detection() {
        let detector = HintDetector::new();
        let lines = vec!["Server at 192.168.1.100:8080".to_string()];

        let hints = detector.scan(&lines);
        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0].hint_type, HintType::IpAddress);
    }

    #[test]
    fn test_path_detection() {
        let detector = HintDetector::new();
        let lines = vec![
            "Edit /home/user/config.toml".to_string(),
            "See ./src/main.rs:42".to_string(),
        ];

        let hints = detector.scan(&lines);
        assert!(hints.iter().any(|h| h.hint_type == HintType::FilePath));
    }

    #[test]
    fn test_hint_mode_state() {
        let mut state = HintModeState::new();
        let hints = vec![
            Hint {
                label: "a".to_string(),
                text: "https://example.com".to_string(),
                hint_type: HintType::Url,
                row: 0,
                col_start: 0,
                col_end: 19,
            },
            Hint {
                label: "b".to_string(),
                text: "https://test.com".to_string(),
                hint_type: HintType::Url,
                row: 1,
                col_start: 0,
                col_end: 16,
            },
        ];

        state.activate(hints);
        assert!(state.active);

        let result = state.handle_key('a');
        assert!(result.is_some());
        assert_eq!(result.unwrap().text, "https://example.com");
        assert!(!state.active);
    }
}
