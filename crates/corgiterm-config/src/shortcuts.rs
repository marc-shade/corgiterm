//! Keyboard shortcut parsing utilities
//!
//! Parses shortcut strings like "Ctrl+Shift+A" into GTK modifier types and key values.

use gtk4::gdk::{Key, ModifierType};

/// Parsed keyboard shortcut
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedShortcut {
    pub modifiers: ModifierType,
    pub key: Key,
}

/// Parse a shortcut string like "Ctrl+Shift+A" into modifiers and key
///
/// # Examples
///
/// ```
/// use corgiterm_config::shortcuts::parse_shortcut;
///
/// let shortcut = parse_shortcut("Ctrl+Shift+A").unwrap();
/// assert!(shortcut.modifiers.contains(gtk4::gdk::ModifierType::CONTROL_MASK));
/// assert!(shortcut.modifiers.contains(gtk4::gdk::ModifierType::SHIFT_MASK));
/// ```
pub fn parse_shortcut(shortcut: &str) -> Result<ParsedShortcut, String> {
    if shortcut.trim().is_empty() {
        return Err("Empty shortcut string".to_string());
    }

    let parts: Vec<&str> = shortcut.split('+').map(|s| s.trim()).collect();

    if parts.is_empty() {
        return Err("Invalid shortcut format".to_string());
    }

    let mut modifiers = ModifierType::empty();
    let key_str = parts.last().unwrap();

    // Parse modifiers
    for part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers.insert(ModifierType::CONTROL_MASK),
            "shift" => modifiers.insert(ModifierType::SHIFT_MASK),
            "alt" | "meta" => modifiers.insert(ModifierType::ALT_MASK),
            "super" | "win" | "cmd" => modifiers.insert(ModifierType::SUPER_MASK),
            _ => return Err(format!("Unknown modifier: {}", part)),
        }
    }

    // Parse key
    let key = parse_key(key_str)?;

    Ok(ParsedShortcut { modifiers, key })
}

/// Parse a key string into a GTK Key
fn parse_key(key_str: &str) -> Result<Key, String> {
    match key_str.to_lowercase().as_str() {
        // Letters (case-insensitive)
        "a" => Ok(Key::a),
        "b" => Ok(Key::b),
        "c" => Ok(Key::c),
        "d" => Ok(Key::d),
        "e" => Ok(Key::e),
        "f" => Ok(Key::f),
        "g" => Ok(Key::g),
        "h" => Ok(Key::h),
        "i" => Ok(Key::i),
        "j" => Ok(Key::j),
        "k" => Ok(Key::k),
        "l" => Ok(Key::l),
        "m" => Ok(Key::m),
        "n" => Ok(Key::n),
        "o" => Ok(Key::o),
        "p" => Ok(Key::p),
        "q" => Ok(Key::q),
        "r" => Ok(Key::r),
        "s" => Ok(Key::s),
        "t" => Ok(Key::t),
        "u" => Ok(Key::u),
        "v" => Ok(Key::v),
        "w" => Ok(Key::w),
        "x" => Ok(Key::x),
        "y" => Ok(Key::y),
        "z" => Ok(Key::z),

        // Numbers
        "0" => Ok(Key::_0),
        "1" => Ok(Key::_1),
        "2" => Ok(Key::_2),
        "3" => Ok(Key::_3),
        "4" => Ok(Key::_4),
        "5" => Ok(Key::_5),
        "6" => Ok(Key::_6),
        "7" => Ok(Key::_7),
        "8" => Ok(Key::_8),
        "9" => Ok(Key::_9),

        // Function keys
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),

        // Special keys
        "tab" => Ok(Key::Tab),
        "space" => Ok(Key::space),
        "enter" | "return" => Ok(Key::Return),
        "escape" | "esc" => Ok(Key::Escape),
        "backspace" => Ok(Key::BackSpace),
        "delete" | "del" => Ok(Key::Delete),
        "insert" | "ins" => Ok(Key::Insert),
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "pageup" | "pgup" => Ok(Key::Page_Up),
        "pagedown" | "pgdn" => Ok(Key::Page_Down),

        // Arrow keys
        "up" => Ok(Key::Up),
        "down" => Ok(Key::Down),
        "left" => Ok(Key::Left),
        "right" => Ok(Key::Right),

        // Symbols
        "[" => Ok(Key::bracketleft),
        "]" => Ok(Key::bracketright),
        "bracketleft" => Ok(Key::bracketleft),
        "bracketright" => Ok(Key::bracketright),
        "+" | "plus" => Ok(Key::plus),
        "-" | "minus" => Ok(Key::minus),
        "=" | "equal" => Ok(Key::equal),
        "/" | "slash" => Ok(Key::slash),
        "\\" | "backslash" => Ok(Key::backslash),
        "," | "comma" => Ok(Key::comma),
        "." | "period" => Ok(Key::period),
        ";" | "semicolon" => Ok(Key::semicolon),
        "'" | "apostrophe" => Ok(Key::apostrophe),
        "`" | "grave" => Ok(Key::grave),

        _ => Err(format!("Unknown key: {}", key_str)),
    }
}

/// Check if a key event matches a parsed shortcut
pub fn matches_shortcut(
    parsed: &ParsedShortcut,
    key: Key,
    modifiers: ModifierType,
) -> bool {
    // For letter keys, we need to match both uppercase and lowercase
    let key_matches = if parsed.key == key {
        true
    } else {
        // Try to match case-insensitive for letters
        match parsed.key {
            Key::a => key == Key::a || key == Key::A,
            Key::b => key == Key::b || key == Key::B,
            Key::c => key == Key::c || key == Key::C,
            Key::d => key == Key::d || key == Key::D,
            Key::e => key == Key::e || key == Key::E,
            Key::f => key == Key::f || key == Key::F,
            Key::g => key == Key::g || key == Key::G,
            Key::h => key == Key::h || key == Key::H,
            Key::i => key == Key::i || key == Key::I,
            Key::j => key == Key::j || key == Key::J,
            Key::k => key == Key::k || key == Key::K,
            Key::l => key == Key::l || key == Key::L,
            Key::m => key == Key::m || key == Key::M,
            Key::n => key == Key::n || key == Key::N,
            Key::o => key == Key::o || key == Key::O,
            Key::p => key == Key::p || key == Key::P,
            Key::q => key == Key::q || key == Key::Q,
            Key::r => key == Key::r || key == Key::R,
            Key::s => key == Key::s || key == Key::S,
            Key::t => key == Key::t || key == Key::T,
            Key::u => key == Key::u || key == Key::U,
            Key::v => key == Key::v || key == Key::V,
            Key::w => key == Key::w || key == Key::W,
            Key::x => key == Key::x || key == Key::X,
            Key::y => key == Key::y || key == Key::Y,
            Key::z => key == Key::z || key == Key::Z,
            _ => false,
        }
    };

    // For Ctrl+Shift+Tab, we need special handling for ISO_Left_Tab
    let key_matches = key_matches || (parsed.key == Key::Tab && key == Key::ISO_Left_Tab);

    key_matches && modifiers == parsed.modifiers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_shortcut() {
        let shortcut = parse_shortcut("Ctrl+T").unwrap();
        assert!(shortcut.modifiers.contains(ModifierType::CONTROL_MASK));
        assert_eq!(shortcut.key, Key::t);
    }

    #[test]
    fn test_parse_compound_shortcut() {
        let shortcut = parse_shortcut("Ctrl+Shift+A").unwrap();
        assert!(shortcut.modifiers.contains(ModifierType::CONTROL_MASK));
        assert!(shortcut.modifiers.contains(ModifierType::SHIFT_MASK));
        assert_eq!(shortcut.key, Key::a);
    }

    #[test]
    fn test_parse_function_key() {
        let shortcut = parse_shortcut("Ctrl+F5").unwrap();
        assert!(shortcut.modifiers.contains(ModifierType::CONTROL_MASK));
        assert_eq!(shortcut.key, Key::F5);
    }

    #[test]
    fn test_parse_number_key() {
        let shortcut = parse_shortcut("Ctrl+1").unwrap();
        assert!(shortcut.modifiers.contains(ModifierType::CONTROL_MASK));
        assert_eq!(shortcut.key, Key::_1);
    }

    #[test]
    fn test_parse_brackets() {
        let shortcut = parse_shortcut("Ctrl+Shift+]").unwrap();
        assert!(shortcut.modifiers.contains(ModifierType::CONTROL_MASK));
        assert!(shortcut.modifiers.contains(ModifierType::SHIFT_MASK));
        assert_eq!(shortcut.key, Key::bracketright);
    }

    #[test]
    fn test_invalid_modifier() {
        let result = parse_shortcut("Invalid+T");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_key() {
        let result = parse_shortcut("Ctrl+InvalidKey");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_shortcut() {
        let result = parse_shortcut("");
        assert!(result.is_err());
    }
}
