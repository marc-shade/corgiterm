//! Emoji Picker Dialog
//!
//! A searchable emoji picker with categories for easy emoji insertion.

use gtk4::prelude::*;
use gtk4::{Box, Button, FlowBox, Orientation, ScrolledWindow, SearchEntry, SelectionMode};
use libadwaita::prelude::*;
use libadwaita::{Dialog, HeaderBar, ToolbarView};
use std::cell::RefCell;
use std::rc::Rc;

/// Emoji category
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmojiCategory {
    Recent,
    Smileys,
    People,
    Animals,
    Food,
    Travel,
    Activities,
    Objects,
    Symbols,
    Flags,
}

impl EmojiCategory {
    fn label(&self) -> &'static str {
        match self {
            Self::Recent => "🕐 Recent",
            Self::Smileys => "😀 Smileys",
            Self::People => "👋 People",
            Self::Animals => "🐕 Animals",
            Self::Food => "🍕 Food",
            Self::Travel => "✈️ Travel",
            Self::Activities => "⚽ Activities",
            Self::Objects => "💡 Objects",
            Self::Symbols => "❤️ Symbols",
            Self::Flags => "🏁 Flags",
        }
    }

    fn all() -> Vec<Self> {
        vec![
            Self::Recent,
            Self::Smileys,
            Self::People,
            Self::Animals,
            Self::Food,
            Self::Travel,
            Self::Activities,
            Self::Objects,
            Self::Symbols,
            Self::Flags,
        ]
    }
}

/// Emoji data structure
#[derive(Debug, Clone)]
pub struct Emoji {
    pub emoji: &'static str,
    pub name: &'static str,
    pub category: EmojiCategory,
}

/// Get all emojis by category
fn get_emojis(category: EmojiCategory) -> Vec<Emoji> {
    match category {
        EmojiCategory::Recent => vec![], // Populated dynamically
        EmojiCategory::Smileys => vec![
            Emoji {
                emoji: "😀",
                name: "grinning face",
                category,
            },
            Emoji {
                emoji: "😃",
                name: "grinning face with big eyes",
                category,
            },
            Emoji {
                emoji: "😄",
                name: "grinning face with smiling eyes",
                category,
            },
            Emoji {
                emoji: "😁",
                name: "beaming face",
                category,
            },
            Emoji {
                emoji: "😆",
                name: "grinning squinting face",
                category,
            },
            Emoji {
                emoji: "😅",
                name: "grinning face with sweat",
                category,
            },
            Emoji {
                emoji: "🤣",
                name: "rolling on floor laughing",
                category,
            },
            Emoji {
                emoji: "😂",
                name: "face with tears of joy",
                category,
            },
            Emoji {
                emoji: "🙂",
                name: "slightly smiling face",
                category,
            },
            Emoji {
                emoji: "😊",
                name: "smiling face with smiling eyes",
                category,
            },
            Emoji {
                emoji: "😇",
                name: "smiling face with halo",
                category,
            },
            Emoji {
                emoji: "🥰",
                name: "smiling face with hearts",
                category,
            },
            Emoji {
                emoji: "😍",
                name: "heart eyes",
                category,
            },
            Emoji {
                emoji: "🤩",
                name: "star struck",
                category,
            },
            Emoji {
                emoji: "😘",
                name: "kissing face",
                category,
            },
            Emoji {
                emoji: "😗",
                name: "kissing face",
                category,
            },
            Emoji {
                emoji: "😚",
                name: "kissing closed eyes",
                category,
            },
            Emoji {
                emoji: "😋",
                name: "face savoring food",
                category,
            },
            Emoji {
                emoji: "😛",
                name: "face with tongue",
                category,
            },
            Emoji {
                emoji: "😜",
                name: "winking face with tongue",
                category,
            },
            Emoji {
                emoji: "🤪",
                name: "zany face",
                category,
            },
            Emoji {
                emoji: "😝",
                name: "squinting face with tongue",
                category,
            },
            Emoji {
                emoji: "🤑",
                name: "money mouth face",
                category,
            },
            Emoji {
                emoji: "🤗",
                name: "hugging face",
                category,
            },
            Emoji {
                emoji: "🤭",
                name: "face with hand over mouth",
                category,
            },
            Emoji {
                emoji: "🤫",
                name: "shushing face",
                category,
            },
            Emoji {
                emoji: "🤔",
                name: "thinking face",
                category,
            },
            Emoji {
                emoji: "🤐",
                name: "zipper mouth face",
                category,
            },
            Emoji {
                emoji: "🤨",
                name: "raised eyebrow",
                category,
            },
            Emoji {
                emoji: "😐",
                name: "neutral face",
                category,
            },
            Emoji {
                emoji: "😑",
                name: "expressionless face",
                category,
            },
            Emoji {
                emoji: "😶",
                name: "face without mouth",
                category,
            },
            Emoji {
                emoji: "😏",
                name: "smirking face",
                category,
            },
            Emoji {
                emoji: "😒",
                name: "unamused face",
                category,
            },
            Emoji {
                emoji: "🙄",
                name: "rolling eyes",
                category,
            },
            Emoji {
                emoji: "😬",
                name: "grimacing face",
                category,
            },
            Emoji {
                emoji: "🤥",
                name: "lying face",
                category,
            },
            Emoji {
                emoji: "😌",
                name: "relieved face",
                category,
            },
            Emoji {
                emoji: "😔",
                name: "pensive face",
                category,
            },
            Emoji {
                emoji: "😪",
                name: "sleepy face",
                category,
            },
            Emoji {
                emoji: "🤤",
                name: "drooling face",
                category,
            },
            Emoji {
                emoji: "😴",
                name: "sleeping face",
                category,
            },
            Emoji {
                emoji: "😷",
                name: "face with medical mask",
                category,
            },
            Emoji {
                emoji: "🤒",
                name: "face with thermometer",
                category,
            },
            Emoji {
                emoji: "🤕",
                name: "face with head bandage",
                category,
            },
            Emoji {
                emoji: "🤢",
                name: "nauseated face",
                category,
            },
            Emoji {
                emoji: "🤮",
                name: "vomiting face",
                category,
            },
            Emoji {
                emoji: "🤧",
                name: "sneezing face",
                category,
            },
            Emoji {
                emoji: "🥵",
                name: "hot face",
                category,
            },
            Emoji {
                emoji: "🥶",
                name: "cold face",
                category,
            },
            Emoji {
                emoji: "🥴",
                name: "woozy face",
                category,
            },
            Emoji {
                emoji: "😵",
                name: "dizzy face",
                category,
            },
            Emoji {
                emoji: "🤯",
                name: "exploding head",
                category,
            },
            Emoji {
                emoji: "🤠",
                name: "cowboy hat face",
                category,
            },
            Emoji {
                emoji: "🥳",
                name: "partying face",
                category,
            },
            Emoji {
                emoji: "🥸",
                name: "disguised face",
                category,
            },
            Emoji {
                emoji: "😎",
                name: "smiling face with sunglasses",
                category,
            },
            Emoji {
                emoji: "🤓",
                name: "nerd face",
                category,
            },
            Emoji {
                emoji: "🧐",
                name: "face with monocle",
                category,
            },
            Emoji {
                emoji: "😕",
                name: "confused face",
                category,
            },
            Emoji {
                emoji: "😟",
                name: "worried face",
                category,
            },
            Emoji {
                emoji: "🙁",
                name: "slightly frowning face",
                category,
            },
            Emoji {
                emoji: "😮",
                name: "face with open mouth",
                category,
            },
            Emoji {
                emoji: "😯",
                name: "hushed face",
                category,
            },
            Emoji {
                emoji: "😲",
                name: "astonished face",
                category,
            },
            Emoji {
                emoji: "😳",
                name: "flushed face",
                category,
            },
            Emoji {
                emoji: "🥺",
                name: "pleading face",
                category,
            },
            Emoji {
                emoji: "😦",
                name: "frowning face with open mouth",
                category,
            },
            Emoji {
                emoji: "😧",
                name: "anguished face",
                category,
            },
            Emoji {
                emoji: "😨",
                name: "fearful face",
                category,
            },
            Emoji {
                emoji: "😰",
                name: "anxious face with sweat",
                category,
            },
            Emoji {
                emoji: "😥",
                name: "sad but relieved face",
                category,
            },
            Emoji {
                emoji: "😢",
                name: "crying face",
                category,
            },
            Emoji {
                emoji: "😭",
                name: "loudly crying face",
                category,
            },
            Emoji {
                emoji: "😱",
                name: "face screaming in fear",
                category,
            },
            Emoji {
                emoji: "😖",
                name: "confounded face",
                category,
            },
            Emoji {
                emoji: "😣",
                name: "persevering face",
                category,
            },
            Emoji {
                emoji: "😞",
                name: "disappointed face",
                category,
            },
            Emoji {
                emoji: "😓",
                name: "downcast face with sweat",
                category,
            },
            Emoji {
                emoji: "😩",
                name: "weary face",
                category,
            },
            Emoji {
                emoji: "😫",
                name: "tired face",
                category,
            },
            Emoji {
                emoji: "🥱",
                name: "yawning face",
                category,
            },
            Emoji {
                emoji: "😤",
                name: "huffing face",
                category,
            },
            Emoji {
                emoji: "😡",
                name: "pouting face",
                category,
            },
            Emoji {
                emoji: "😠",
                name: "angry face",
                category,
            },
            Emoji {
                emoji: "🤬",
                name: "face with symbols on mouth",
                category,
            },
            Emoji {
                emoji: "😈",
                name: "smiling face with horns",
                category,
            },
            Emoji {
                emoji: "👿",
                name: "angry face with horns",
                category,
            },
            Emoji {
                emoji: "💀",
                name: "skull",
                category,
            },
            Emoji {
                emoji: "☠️",
                name: "skull and crossbones",
                category,
            },
            Emoji {
                emoji: "💩",
                name: "pile of poo",
                category,
            },
            Emoji {
                emoji: "🤡",
                name: "clown face",
                category,
            },
            Emoji {
                emoji: "👹",
                name: "ogre",
                category,
            },
            Emoji {
                emoji: "👺",
                name: "goblin",
                category,
            },
            Emoji {
                emoji: "👻",
                name: "ghost",
                category,
            },
            Emoji {
                emoji: "👽",
                name: "alien",
                category,
            },
            Emoji {
                emoji: "👾",
                name: "alien monster",
                category,
            },
            Emoji {
                emoji: "🤖",
                name: "robot",
                category,
            },
        ],
        EmojiCategory::People => vec![
            Emoji {
                emoji: "👋",
                name: "waving hand",
                category,
            },
            Emoji {
                emoji: "🤚",
                name: "raised back of hand",
                category,
            },
            Emoji {
                emoji: "🖐️",
                name: "hand with fingers splayed",
                category,
            },
            Emoji {
                emoji: "✋",
                name: "raised hand",
                category,
            },
            Emoji {
                emoji: "🖖",
                name: "vulcan salute",
                category,
            },
            Emoji {
                emoji: "👌",
                name: "ok hand",
                category,
            },
            Emoji {
                emoji: "🤌",
                name: "pinched fingers",
                category,
            },
            Emoji {
                emoji: "🤏",
                name: "pinching hand",
                category,
            },
            Emoji {
                emoji: "✌️",
                name: "victory hand",
                category,
            },
            Emoji {
                emoji: "🤞",
                name: "crossed fingers",
                category,
            },
            Emoji {
                emoji: "🤟",
                name: "love you gesture",
                category,
            },
            Emoji {
                emoji: "🤘",
                name: "sign of the horns",
                category,
            },
            Emoji {
                emoji: "🤙",
                name: "call me hand",
                category,
            },
            Emoji {
                emoji: "👈",
                name: "backhand index pointing left",
                category,
            },
            Emoji {
                emoji: "👉",
                name: "backhand index pointing right",
                category,
            },
            Emoji {
                emoji: "👆",
                name: "backhand index pointing up",
                category,
            },
            Emoji {
                emoji: "🖕",
                name: "middle finger",
                category,
            },
            Emoji {
                emoji: "👇",
                name: "backhand index pointing down",
                category,
            },
            Emoji {
                emoji: "☝️",
                name: "index pointing up",
                category,
            },
            Emoji {
                emoji: "👍",
                name: "thumbs up",
                category,
            },
            Emoji {
                emoji: "👎",
                name: "thumbs down",
                category,
            },
            Emoji {
                emoji: "✊",
                name: "raised fist",
                category,
            },
            Emoji {
                emoji: "👊",
                name: "oncoming fist",
                category,
            },
            Emoji {
                emoji: "🤛",
                name: "left-facing fist",
                category,
            },
            Emoji {
                emoji: "🤜",
                name: "right-facing fist",
                category,
            },
            Emoji {
                emoji: "👏",
                name: "clapping hands",
                category,
            },
            Emoji {
                emoji: "🙌",
                name: "raising hands",
                category,
            },
            Emoji {
                emoji: "👐",
                name: "open hands",
                category,
            },
            Emoji {
                emoji: "🤲",
                name: "palms up together",
                category,
            },
            Emoji {
                emoji: "🤝",
                name: "handshake",
                category,
            },
            Emoji {
                emoji: "🙏",
                name: "folded hands",
                category,
            },
            Emoji {
                emoji: "✍️",
                name: "writing hand",
                category,
            },
            Emoji {
                emoji: "💅",
                name: "nail polish",
                category,
            },
            Emoji {
                emoji: "🤳",
                name: "selfie",
                category,
            },
            Emoji {
                emoji: "💪",
                name: "flexed biceps",
                category,
            },
            Emoji {
                emoji: "🦾",
                name: "mechanical arm",
                category,
            },
            Emoji {
                emoji: "🦿",
                name: "mechanical leg",
                category,
            },
            Emoji {
                emoji: "🦵",
                name: "leg",
                category,
            },
            Emoji {
                emoji: "🦶",
                name: "foot",
                category,
            },
            Emoji {
                emoji: "👂",
                name: "ear",
                category,
            },
            Emoji {
                emoji: "🦻",
                name: "ear with hearing aid",
                category,
            },
            Emoji {
                emoji: "👃",
                name: "nose",
                category,
            },
            Emoji {
                emoji: "🧠",
                name: "brain",
                category,
            },
            Emoji {
                emoji: "👀",
                name: "eyes",
                category,
            },
            Emoji {
                emoji: "👁️",
                name: "eye",
                category,
            },
            Emoji {
                emoji: "👅",
                name: "tongue",
                category,
            },
            Emoji {
                emoji: "👄",
                name: "mouth",
                category,
            },
            Emoji {
                emoji: "👶",
                name: "baby",
                category,
            },
            Emoji {
                emoji: "🧒",
                name: "child",
                category,
            },
            Emoji {
                emoji: "👦",
                name: "boy",
                category,
            },
            Emoji {
                emoji: "👧",
                name: "girl",
                category,
            },
            Emoji {
                emoji: "🧑",
                name: "person",
                category,
            },
            Emoji {
                emoji: "👱",
                name: "person blond hair",
                category,
            },
            Emoji {
                emoji: "👨",
                name: "man",
                category,
            },
            Emoji {
                emoji: "🧔",
                name: "man beard",
                category,
            },
            Emoji {
                emoji: "👩",
                name: "woman",
                category,
            },
            Emoji {
                emoji: "🧓",
                name: "older person",
                category,
            },
            Emoji {
                emoji: "👴",
                name: "old man",
                category,
            },
            Emoji {
                emoji: "👵",
                name: "old woman",
                category,
            },
        ],
        EmojiCategory::Animals => vec![
            Emoji {
                emoji: "🐶",
                name: "dog face",
                category,
            },
            Emoji {
                emoji: "🐕",
                name: "dog",
                category,
            },
            Emoji {
                emoji: "🦮",
                name: "guide dog",
                category,
            },
            Emoji {
                emoji: "🐕‍🦺",
                name: "service dog",
                category,
            },
            Emoji {
                emoji: "🐩",
                name: "poodle",
                category,
            },
            Emoji {
                emoji: "🐺",
                name: "wolf",
                category,
            },
            Emoji {
                emoji: "🦊",
                name: "fox",
                category,
            },
            Emoji {
                emoji: "🦝",
                name: "raccoon",
                category,
            },
            Emoji {
                emoji: "🐱",
                name: "cat face",
                category,
            },
            Emoji {
                emoji: "🐈",
                name: "cat",
                category,
            },
            Emoji {
                emoji: "🐈‍⬛",
                name: "black cat",
                category,
            },
            Emoji {
                emoji: "🦁",
                name: "lion",
                category,
            },
            Emoji {
                emoji: "🐯",
                name: "tiger face",
                category,
            },
            Emoji {
                emoji: "🐅",
                name: "tiger",
                category,
            },
            Emoji {
                emoji: "🐆",
                name: "leopard",
                category,
            },
            Emoji {
                emoji: "🐴",
                name: "horse face",
                category,
            },
            Emoji {
                emoji: "🐎",
                name: "horse",
                category,
            },
            Emoji {
                emoji: "🦄",
                name: "unicorn",
                category,
            },
            Emoji {
                emoji: "🦓",
                name: "zebra",
                category,
            },
            Emoji {
                emoji: "🦌",
                name: "deer",
                category,
            },
            Emoji {
                emoji: "🦬",
                name: "bison",
                category,
            },
            Emoji {
                emoji: "🐮",
                name: "cow face",
                category,
            },
            Emoji {
                emoji: "🐂",
                name: "ox",
                category,
            },
            Emoji {
                emoji: "🐃",
                name: "water buffalo",
                category,
            },
            Emoji {
                emoji: "🐄",
                name: "cow",
                category,
            },
            Emoji {
                emoji: "🐷",
                name: "pig face",
                category,
            },
            Emoji {
                emoji: "🐖",
                name: "pig",
                category,
            },
            Emoji {
                emoji: "🐗",
                name: "boar",
                category,
            },
            Emoji {
                emoji: "🐽",
                name: "pig nose",
                category,
            },
            Emoji {
                emoji: "🐏",
                name: "ram",
                category,
            },
            Emoji {
                emoji: "🐑",
                name: "ewe",
                category,
            },
            Emoji {
                emoji: "🐐",
                name: "goat",
                category,
            },
            Emoji {
                emoji: "🐪",
                name: "camel",
                category,
            },
            Emoji {
                emoji: "🐫",
                name: "two-hump camel",
                category,
            },
            Emoji {
                emoji: "🦙",
                name: "llama",
                category,
            },
            Emoji {
                emoji: "🦒",
                name: "giraffe",
                category,
            },
            Emoji {
                emoji: "🐘",
                name: "elephant",
                category,
            },
            Emoji {
                emoji: "🦣",
                name: "mammoth",
                category,
            },
            Emoji {
                emoji: "🦏",
                name: "rhinoceros",
                category,
            },
            Emoji {
                emoji: "🦛",
                name: "hippopotamus",
                category,
            },
            Emoji {
                emoji: "🐭",
                name: "mouse face",
                category,
            },
            Emoji {
                emoji: "🐁",
                name: "mouse",
                category,
            },
            Emoji {
                emoji: "🐀",
                name: "rat",
                category,
            },
            Emoji {
                emoji: "🐹",
                name: "hamster",
                category,
            },
            Emoji {
                emoji: "🐰",
                name: "rabbit face",
                category,
            },
            Emoji {
                emoji: "🐇",
                name: "rabbit",
                category,
            },
            Emoji {
                emoji: "🐿️",
                name: "chipmunk",
                category,
            },
            Emoji {
                emoji: "🦫",
                name: "beaver",
                category,
            },
            Emoji {
                emoji: "🦔",
                name: "hedgehog",
                category,
            },
            Emoji {
                emoji: "🦇",
                name: "bat",
                category,
            },
            Emoji {
                emoji: "🐻",
                name: "bear",
                category,
            },
            Emoji {
                emoji: "🐻‍❄️",
                name: "polar bear",
                category,
            },
            Emoji {
                emoji: "🐨",
                name: "koala",
                category,
            },
            Emoji {
                emoji: "🐼",
                name: "panda",
                category,
            },
            Emoji {
                emoji: "🦥",
                name: "sloth",
                category,
            },
            Emoji {
                emoji: "🦦",
                name: "otter",
                category,
            },
            Emoji {
                emoji: "🦨",
                name: "skunk",
                category,
            },
            Emoji {
                emoji: "🦘",
                name: "kangaroo",
                category,
            },
            Emoji {
                emoji: "🦡",
                name: "badger",
                category,
            },
            Emoji {
                emoji: "🐾",
                name: "paw prints",
                category,
            },
            Emoji {
                emoji: "🦃",
                name: "turkey",
                category,
            },
            Emoji {
                emoji: "🐔",
                name: "chicken",
                category,
            },
            Emoji {
                emoji: "🐓",
                name: "rooster",
                category,
            },
            Emoji {
                emoji: "🐣",
                name: "hatching chick",
                category,
            },
            Emoji {
                emoji: "🐤",
                name: "baby chick",
                category,
            },
            Emoji {
                emoji: "🐥",
                name: "front-facing baby chick",
                category,
            },
            Emoji {
                emoji: "🐦",
                name: "bird",
                category,
            },
            Emoji {
                emoji: "🐧",
                name: "penguin",
                category,
            },
            Emoji {
                emoji: "🕊️",
                name: "dove",
                category,
            },
            Emoji {
                emoji: "🦅",
                name: "eagle",
                category,
            },
            Emoji {
                emoji: "🦆",
                name: "duck",
                category,
            },
            Emoji {
                emoji: "🦢",
                name: "swan",
                category,
            },
            Emoji {
                emoji: "🦉",
                name: "owl",
                category,
            },
            Emoji {
                emoji: "🦤",
                name: "dodo",
                category,
            },
            Emoji {
                emoji: "🪶",
                name: "feather",
                category,
            },
            Emoji {
                emoji: "🦩",
                name: "flamingo",
                category,
            },
            Emoji {
                emoji: "🦚",
                name: "peacock",
                category,
            },
            Emoji {
                emoji: "🦜",
                name: "parrot",
                category,
            },
            Emoji {
                emoji: "🐸",
                name: "frog",
                category,
            },
            Emoji {
                emoji: "🐊",
                name: "crocodile",
                category,
            },
            Emoji {
                emoji: "🐢",
                name: "turtle",
                category,
            },
            Emoji {
                emoji: "🦎",
                name: "lizard",
                category,
            },
            Emoji {
                emoji: "🐍",
                name: "snake",
                category,
            },
            Emoji {
                emoji: "🐲",
                name: "dragon face",
                category,
            },
            Emoji {
                emoji: "🐉",
                name: "dragon",
                category,
            },
            Emoji {
                emoji: "🦕",
                name: "sauropod",
                category,
            },
            Emoji {
                emoji: "🦖",
                name: "t-rex",
                category,
            },
            Emoji {
                emoji: "🐳",
                name: "spouting whale",
                category,
            },
            Emoji {
                emoji: "🐋",
                name: "whale",
                category,
            },
            Emoji {
                emoji: "🐬",
                name: "dolphin",
                category,
            },
            Emoji {
                emoji: "🦭",
                name: "seal",
                category,
            },
            Emoji {
                emoji: "🐟",
                name: "fish",
                category,
            },
            Emoji {
                emoji: "🐠",
                name: "tropical fish",
                category,
            },
            Emoji {
                emoji: "🐡",
                name: "blowfish",
                category,
            },
            Emoji {
                emoji: "🦈",
                name: "shark",
                category,
            },
            Emoji {
                emoji: "🐙",
                name: "octopus",
                category,
            },
            Emoji {
                emoji: "🐚",
                name: "spiral shell",
                category,
            },
            Emoji {
                emoji: "🐌",
                name: "snail",
                category,
            },
            Emoji {
                emoji: "🦋",
                name: "butterfly",
                category,
            },
            Emoji {
                emoji: "🐛",
                name: "bug",
                category,
            },
            Emoji {
                emoji: "🐜",
                name: "ant",
                category,
            },
            Emoji {
                emoji: "🐝",
                name: "honeybee",
                category,
            },
            Emoji {
                emoji: "🪲",
                name: "beetle",
                category,
            },
            Emoji {
                emoji: "🐞",
                name: "lady beetle",
                category,
            },
            Emoji {
                emoji: "🦗",
                name: "cricket",
                category,
            },
            Emoji {
                emoji: "🪳",
                name: "cockroach",
                category,
            },
            Emoji {
                emoji: "🕷️",
                name: "spider",
                category,
            },
            Emoji {
                emoji: "🕸️",
                name: "spider web",
                category,
            },
            Emoji {
                emoji: "🦂",
                name: "scorpion",
                category,
            },
        ],
        EmojiCategory::Food => vec![
            Emoji {
                emoji: "🍇",
                name: "grapes",
                category,
            },
            Emoji {
                emoji: "🍈",
                name: "melon",
                category,
            },
            Emoji {
                emoji: "🍉",
                name: "watermelon",
                category,
            },
            Emoji {
                emoji: "🍊",
                name: "tangerine",
                category,
            },
            Emoji {
                emoji: "🍋",
                name: "lemon",
                category,
            },
            Emoji {
                emoji: "🍌",
                name: "banana",
                category,
            },
            Emoji {
                emoji: "🍍",
                name: "pineapple",
                category,
            },
            Emoji {
                emoji: "🥭",
                name: "mango",
                category,
            },
            Emoji {
                emoji: "🍎",
                name: "red apple",
                category,
            },
            Emoji {
                emoji: "🍏",
                name: "green apple",
                category,
            },
            Emoji {
                emoji: "🍐",
                name: "pear",
                category,
            },
            Emoji {
                emoji: "🍑",
                name: "peach",
                category,
            },
            Emoji {
                emoji: "🍒",
                name: "cherries",
                category,
            },
            Emoji {
                emoji: "🍓",
                name: "strawberry",
                category,
            },
            Emoji {
                emoji: "🫐",
                name: "blueberries",
                category,
            },
            Emoji {
                emoji: "🥝",
                name: "kiwi fruit",
                category,
            },
            Emoji {
                emoji: "🍅",
                name: "tomato",
                category,
            },
            Emoji {
                emoji: "🫒",
                name: "olive",
                category,
            },
            Emoji {
                emoji: "🥥",
                name: "coconut",
                category,
            },
            Emoji {
                emoji: "🥑",
                name: "avocado",
                category,
            },
            Emoji {
                emoji: "🍆",
                name: "eggplant",
                category,
            },
            Emoji {
                emoji: "🥔",
                name: "potato",
                category,
            },
            Emoji {
                emoji: "🥕",
                name: "carrot",
                category,
            },
            Emoji {
                emoji: "🌽",
                name: "corn",
                category,
            },
            Emoji {
                emoji: "🌶️",
                name: "hot pepper",
                category,
            },
            Emoji {
                emoji: "🫑",
                name: "bell pepper",
                category,
            },
            Emoji {
                emoji: "🥒",
                name: "cucumber",
                category,
            },
            Emoji {
                emoji: "🥬",
                name: "leafy green",
                category,
            },
            Emoji {
                emoji: "🥦",
                name: "broccoli",
                category,
            },
            Emoji {
                emoji: "🧄",
                name: "garlic",
                category,
            },
            Emoji {
                emoji: "🧅",
                name: "onion",
                category,
            },
            Emoji {
                emoji: "🍄",
                name: "mushroom",
                category,
            },
            Emoji {
                emoji: "🥜",
                name: "peanuts",
                category,
            },
            Emoji {
                emoji: "🌰",
                name: "chestnut",
                category,
            },
            Emoji {
                emoji: "🍞",
                name: "bread",
                category,
            },
            Emoji {
                emoji: "🥐",
                name: "croissant",
                category,
            },
            Emoji {
                emoji: "🥖",
                name: "baguette bread",
                category,
            },
            Emoji {
                emoji: "🫓",
                name: "flatbread",
                category,
            },
            Emoji {
                emoji: "🥨",
                name: "pretzel",
                category,
            },
            Emoji {
                emoji: "🥯",
                name: "bagel",
                category,
            },
            Emoji {
                emoji: "🥞",
                name: "pancakes",
                category,
            },
            Emoji {
                emoji: "🧇",
                name: "waffle",
                category,
            },
            Emoji {
                emoji: "🧀",
                name: "cheese wedge",
                category,
            },
            Emoji {
                emoji: "🍖",
                name: "meat on bone",
                category,
            },
            Emoji {
                emoji: "🍗",
                name: "poultry leg",
                category,
            },
            Emoji {
                emoji: "🥩",
                name: "cut of meat",
                category,
            },
            Emoji {
                emoji: "🥓",
                name: "bacon",
                category,
            },
            Emoji {
                emoji: "🍔",
                name: "hamburger",
                category,
            },
            Emoji {
                emoji: "🍟",
                name: "french fries",
                category,
            },
            Emoji {
                emoji: "🍕",
                name: "pizza",
                category,
            },
            Emoji {
                emoji: "🌭",
                name: "hot dog",
                category,
            },
            Emoji {
                emoji: "🥪",
                name: "sandwich",
                category,
            },
            Emoji {
                emoji: "🌮",
                name: "taco",
                category,
            },
            Emoji {
                emoji: "🌯",
                name: "burrito",
                category,
            },
            Emoji {
                emoji: "🫔",
                name: "tamale",
                category,
            },
            Emoji {
                emoji: "🥙",
                name: "stuffed flatbread",
                category,
            },
            Emoji {
                emoji: "🧆",
                name: "falafel",
                category,
            },
            Emoji {
                emoji: "🥚",
                name: "egg",
                category,
            },
            Emoji {
                emoji: "🍳",
                name: "cooking",
                category,
            },
            Emoji {
                emoji: "🥘",
                name: "shallow pan of food",
                category,
            },
            Emoji {
                emoji: "🍲",
                name: "pot of food",
                category,
            },
            Emoji {
                emoji: "🫕",
                name: "fondue",
                category,
            },
            Emoji {
                emoji: "🥣",
                name: "bowl with spoon",
                category,
            },
            Emoji {
                emoji: "🥗",
                name: "green salad",
                category,
            },
            Emoji {
                emoji: "🍿",
                name: "popcorn",
                category,
            },
            Emoji {
                emoji: "🧈",
                name: "butter",
                category,
            },
            Emoji {
                emoji: "🧂",
                name: "salt",
                category,
            },
            Emoji {
                emoji: "🥫",
                name: "canned food",
                category,
            },
            Emoji {
                emoji: "🍱",
                name: "bento box",
                category,
            },
            Emoji {
                emoji: "🍘",
                name: "rice cracker",
                category,
            },
            Emoji {
                emoji: "🍙",
                name: "rice ball",
                category,
            },
            Emoji {
                emoji: "🍚",
                name: "cooked rice",
                category,
            },
            Emoji {
                emoji: "🍛",
                name: "curry rice",
                category,
            },
            Emoji {
                emoji: "🍜",
                name: "steaming bowl",
                category,
            },
            Emoji {
                emoji: "🍝",
                name: "spaghetti",
                category,
            },
            Emoji {
                emoji: "🍠",
                name: "roasted sweet potato",
                category,
            },
            Emoji {
                emoji: "🍢",
                name: "oden",
                category,
            },
            Emoji {
                emoji: "🍣",
                name: "sushi",
                category,
            },
            Emoji {
                emoji: "🍤",
                name: "fried shrimp",
                category,
            },
            Emoji {
                emoji: "🍥",
                name: "fish cake",
                category,
            },
            Emoji {
                emoji: "🥮",
                name: "moon cake",
                category,
            },
            Emoji {
                emoji: "🍡",
                name: "dango",
                category,
            },
            Emoji {
                emoji: "🥟",
                name: "dumpling",
                category,
            },
            Emoji {
                emoji: "🥠",
                name: "fortune cookie",
                category,
            },
            Emoji {
                emoji: "🥡",
                name: "takeout box",
                category,
            },
            Emoji {
                emoji: "🦀",
                name: "crab",
                category,
            },
            Emoji {
                emoji: "🦞",
                name: "lobster",
                category,
            },
            Emoji {
                emoji: "🦐",
                name: "shrimp",
                category,
            },
            Emoji {
                emoji: "🦑",
                name: "squid",
                category,
            },
            Emoji {
                emoji: "🦪",
                name: "oyster",
                category,
            },
            Emoji {
                emoji: "🍦",
                name: "soft ice cream",
                category,
            },
            Emoji {
                emoji: "🍧",
                name: "shaved ice",
                category,
            },
            Emoji {
                emoji: "🍨",
                name: "ice cream",
                category,
            },
            Emoji {
                emoji: "🍩",
                name: "doughnut",
                category,
            },
            Emoji {
                emoji: "🍪",
                name: "cookie",
                category,
            },
            Emoji {
                emoji: "🎂",
                name: "birthday cake",
                category,
            },
            Emoji {
                emoji: "🍰",
                name: "shortcake",
                category,
            },
            Emoji {
                emoji: "🧁",
                name: "cupcake",
                category,
            },
            Emoji {
                emoji: "🥧",
                name: "pie",
                category,
            },
            Emoji {
                emoji: "🍫",
                name: "chocolate bar",
                category,
            },
            Emoji {
                emoji: "🍬",
                name: "candy",
                category,
            },
            Emoji {
                emoji: "🍭",
                name: "lollipop",
                category,
            },
            Emoji {
                emoji: "🍮",
                name: "custard",
                category,
            },
            Emoji {
                emoji: "🍯",
                name: "honey pot",
                category,
            },
            Emoji {
                emoji: "🍼",
                name: "baby bottle",
                category,
            },
            Emoji {
                emoji: "🥛",
                name: "glass of milk",
                category,
            },
            Emoji {
                emoji: "☕",
                name: "hot beverage",
                category,
            },
            Emoji {
                emoji: "🫖",
                name: "teapot",
                category,
            },
            Emoji {
                emoji: "🍵",
                name: "teacup without handle",
                category,
            },
            Emoji {
                emoji: "🍶",
                name: "sake",
                category,
            },
            Emoji {
                emoji: "🍾",
                name: "bottle with popping cork",
                category,
            },
            Emoji {
                emoji: "🍷",
                name: "wine glass",
                category,
            },
            Emoji {
                emoji: "🍸",
                name: "cocktail glass",
                category,
            },
            Emoji {
                emoji: "🍹",
                name: "tropical drink",
                category,
            },
            Emoji {
                emoji: "🍺",
                name: "beer mug",
                category,
            },
            Emoji {
                emoji: "🍻",
                name: "clinking beer mugs",
                category,
            },
            Emoji {
                emoji: "🥂",
                name: "clinking glasses",
                category,
            },
            Emoji {
                emoji: "🥃",
                name: "tumbler glass",
                category,
            },
            Emoji {
                emoji: "🥤",
                name: "cup with straw",
                category,
            },
            Emoji {
                emoji: "🧋",
                name: "bubble tea",
                category,
            },
            Emoji {
                emoji: "🧃",
                name: "beverage box",
                category,
            },
            Emoji {
                emoji: "🧉",
                name: "mate",
                category,
            },
            Emoji {
                emoji: "🧊",
                name: "ice",
                category,
            },
        ],
        EmojiCategory::Travel => vec![
            Emoji {
                emoji: "🌍",
                name: "globe europe africa",
                category,
            },
            Emoji {
                emoji: "🌎",
                name: "globe americas",
                category,
            },
            Emoji {
                emoji: "🌏",
                name: "globe asia australia",
                category,
            },
            Emoji {
                emoji: "🌐",
                name: "globe with meridians",
                category,
            },
            Emoji {
                emoji: "🗺️",
                name: "world map",
                category,
            },
            Emoji {
                emoji: "🧭",
                name: "compass",
                category,
            },
            Emoji {
                emoji: "🏔️",
                name: "snow-capped mountain",
                category,
            },
            Emoji {
                emoji: "⛰️",
                name: "mountain",
                category,
            },
            Emoji {
                emoji: "🌋",
                name: "volcano",
                category,
            },
            Emoji {
                emoji: "🗻",
                name: "mount fuji",
                category,
            },
            Emoji {
                emoji: "🏕️",
                name: "camping",
                category,
            },
            Emoji {
                emoji: "🏖️",
                name: "beach with umbrella",
                category,
            },
            Emoji {
                emoji: "🏜️",
                name: "desert",
                category,
            },
            Emoji {
                emoji: "🏝️",
                name: "desert island",
                category,
            },
            Emoji {
                emoji: "🏞️",
                name: "national park",
                category,
            },
            Emoji {
                emoji: "🏟️",
                name: "stadium",
                category,
            },
            Emoji {
                emoji: "🏛️",
                name: "classical building",
                category,
            },
            Emoji {
                emoji: "🏗️",
                name: "building construction",
                category,
            },
            Emoji {
                emoji: "🧱",
                name: "brick",
                category,
            },
            Emoji {
                emoji: "🏘️",
                name: "houses",
                category,
            },
            Emoji {
                emoji: "🏚️",
                name: "derelict house",
                category,
            },
            Emoji {
                emoji: "🏠",
                name: "house",
                category,
            },
            Emoji {
                emoji: "🏡",
                name: "house with garden",
                category,
            },
            Emoji {
                emoji: "🏢",
                name: "office building",
                category,
            },
            Emoji {
                emoji: "🏣",
                name: "japanese post office",
                category,
            },
            Emoji {
                emoji: "🏤",
                name: "post office",
                category,
            },
            Emoji {
                emoji: "🏥",
                name: "hospital",
                category,
            },
            Emoji {
                emoji: "🏦",
                name: "bank",
                category,
            },
            Emoji {
                emoji: "🏨",
                name: "hotel",
                category,
            },
            Emoji {
                emoji: "🏩",
                name: "love hotel",
                category,
            },
            Emoji {
                emoji: "🏪",
                name: "convenience store",
                category,
            },
            Emoji {
                emoji: "🏫",
                name: "school",
                category,
            },
            Emoji {
                emoji: "🏬",
                name: "department store",
                category,
            },
            Emoji {
                emoji: "🏭",
                name: "factory",
                category,
            },
            Emoji {
                emoji: "🏯",
                name: "japanese castle",
                category,
            },
            Emoji {
                emoji: "🏰",
                name: "castle",
                category,
            },
            Emoji {
                emoji: "💒",
                name: "wedding",
                category,
            },
            Emoji {
                emoji: "🗼",
                name: "tokyo tower",
                category,
            },
            Emoji {
                emoji: "🗽",
                name: "statue of liberty",
                category,
            },
            Emoji {
                emoji: "⛪",
                name: "church",
                category,
            },
            Emoji {
                emoji: "🕌",
                name: "mosque",
                category,
            },
            Emoji {
                emoji: "🛕",
                name: "hindu temple",
                category,
            },
            Emoji {
                emoji: "🕍",
                name: "synagogue",
                category,
            },
            Emoji {
                emoji: "⛩️",
                name: "shinto shrine",
                category,
            },
            Emoji {
                emoji: "🕋",
                name: "kaaba",
                category,
            },
            Emoji {
                emoji: "⛲",
                name: "fountain",
                category,
            },
            Emoji {
                emoji: "⛺",
                name: "tent",
                category,
            },
            Emoji {
                emoji: "🌁",
                name: "foggy",
                category,
            },
            Emoji {
                emoji: "🌃",
                name: "night with stars",
                category,
            },
            Emoji {
                emoji: "🏙️",
                name: "cityscape",
                category,
            },
            Emoji {
                emoji: "🌄",
                name: "sunrise over mountains",
                category,
            },
            Emoji {
                emoji: "🌅",
                name: "sunrise",
                category,
            },
            Emoji {
                emoji: "🌆",
                name: "cityscape at dusk",
                category,
            },
            Emoji {
                emoji: "🌇",
                name: "sunset",
                category,
            },
            Emoji {
                emoji: "🌉",
                name: "bridge at night",
                category,
            },
            Emoji {
                emoji: "♨️",
                name: "hot springs",
                category,
            },
            Emoji {
                emoji: "🎠",
                name: "carousel horse",
                category,
            },
            Emoji {
                emoji: "🎡",
                name: "ferris wheel",
                category,
            },
            Emoji {
                emoji: "🎢",
                name: "roller coaster",
                category,
            },
            Emoji {
                emoji: "💈",
                name: "barber pole",
                category,
            },
            Emoji {
                emoji: "🎪",
                name: "circus tent",
                category,
            },
            Emoji {
                emoji: "🚂",
                name: "locomotive",
                category,
            },
            Emoji {
                emoji: "🚃",
                name: "railway car",
                category,
            },
            Emoji {
                emoji: "🚄",
                name: "high-speed train",
                category,
            },
            Emoji {
                emoji: "🚅",
                name: "bullet train",
                category,
            },
            Emoji {
                emoji: "🚆",
                name: "train",
                category,
            },
            Emoji {
                emoji: "🚇",
                name: "metro",
                category,
            },
            Emoji {
                emoji: "🚈",
                name: "light rail",
                category,
            },
            Emoji {
                emoji: "🚉",
                name: "station",
                category,
            },
            Emoji {
                emoji: "🚊",
                name: "tram",
                category,
            },
            Emoji {
                emoji: "🚝",
                name: "monorail",
                category,
            },
            Emoji {
                emoji: "🚞",
                name: "mountain railway",
                category,
            },
            Emoji {
                emoji: "🚋",
                name: "tram car",
                category,
            },
            Emoji {
                emoji: "🚌",
                name: "bus",
                category,
            },
            Emoji {
                emoji: "🚍",
                name: "oncoming bus",
                category,
            },
            Emoji {
                emoji: "🚎",
                name: "trolleybus",
                category,
            },
            Emoji {
                emoji: "🚐",
                name: "minibus",
                category,
            },
            Emoji {
                emoji: "🚑",
                name: "ambulance",
                category,
            },
            Emoji {
                emoji: "🚒",
                name: "fire engine",
                category,
            },
            Emoji {
                emoji: "🚓",
                name: "police car",
                category,
            },
            Emoji {
                emoji: "🚔",
                name: "oncoming police car",
                category,
            },
            Emoji {
                emoji: "🚕",
                name: "taxi",
                category,
            },
            Emoji {
                emoji: "🚖",
                name: "oncoming taxi",
                category,
            },
            Emoji {
                emoji: "🚗",
                name: "automobile",
                category,
            },
            Emoji {
                emoji: "🚘",
                name: "oncoming automobile",
                category,
            },
            Emoji {
                emoji: "🚙",
                name: "sport utility vehicle",
                category,
            },
            Emoji {
                emoji: "🛻",
                name: "pickup truck",
                category,
            },
            Emoji {
                emoji: "🚚",
                name: "delivery truck",
                category,
            },
            Emoji {
                emoji: "🚛",
                name: "articulated lorry",
                category,
            },
            Emoji {
                emoji: "🚜",
                name: "tractor",
                category,
            },
            Emoji {
                emoji: "🏎️",
                name: "racing car",
                category,
            },
            Emoji {
                emoji: "🏍️",
                name: "motorcycle",
                category,
            },
            Emoji {
                emoji: "🛵",
                name: "motor scooter",
                category,
            },
            Emoji {
                emoji: "🦽",
                name: "manual wheelchair",
                category,
            },
            Emoji {
                emoji: "🦼",
                name: "motorized wheelchair",
                category,
            },
            Emoji {
                emoji: "🛺",
                name: "auto rickshaw",
                category,
            },
            Emoji {
                emoji: "🚲",
                name: "bicycle",
                category,
            },
            Emoji {
                emoji: "🛴",
                name: "kick scooter",
                category,
            },
            Emoji {
                emoji: "🛹",
                name: "skateboard",
                category,
            },
            Emoji {
                emoji: "🛼",
                name: "roller skate",
                category,
            },
            Emoji {
                emoji: "🚏",
                name: "bus stop",
                category,
            },
            Emoji {
                emoji: "🛣️",
                name: "motorway",
                category,
            },
            Emoji {
                emoji: "🛤️",
                name: "railway track",
                category,
            },
            Emoji {
                emoji: "🛢️",
                name: "oil drum",
                category,
            },
            Emoji {
                emoji: "⛽",
                name: "fuel pump",
                category,
            },
            Emoji {
                emoji: "🚨",
                name: "police car light",
                category,
            },
            Emoji {
                emoji: "🚥",
                name: "horizontal traffic light",
                category,
            },
            Emoji {
                emoji: "🚦",
                name: "vertical traffic light",
                category,
            },
            Emoji {
                emoji: "🛑",
                name: "stop sign",
                category,
            },
            Emoji {
                emoji: "🚧",
                name: "construction",
                category,
            },
            Emoji {
                emoji: "⚓",
                name: "anchor",
                category,
            },
            Emoji {
                emoji: "⛵",
                name: "sailboat",
                category,
            },
            Emoji {
                emoji: "🛶",
                name: "canoe",
                category,
            },
            Emoji {
                emoji: "🚤",
                name: "speedboat",
                category,
            },
            Emoji {
                emoji: "🛳️",
                name: "passenger ship",
                category,
            },
            Emoji {
                emoji: "⛴️",
                name: "ferry",
                category,
            },
            Emoji {
                emoji: "🛥️",
                name: "motor boat",
                category,
            },
            Emoji {
                emoji: "🚢",
                name: "ship",
                category,
            },
            Emoji {
                emoji: "✈️",
                name: "airplane",
                category,
            },
            Emoji {
                emoji: "🛩️",
                name: "small airplane",
                category,
            },
            Emoji {
                emoji: "🛫",
                name: "airplane departure",
                category,
            },
            Emoji {
                emoji: "🛬",
                name: "airplane arrival",
                category,
            },
            Emoji {
                emoji: "🪂",
                name: "parachute",
                category,
            },
            Emoji {
                emoji: "💺",
                name: "seat",
                category,
            },
            Emoji {
                emoji: "🚁",
                name: "helicopter",
                category,
            },
            Emoji {
                emoji: "🚟",
                name: "suspension railway",
                category,
            },
            Emoji {
                emoji: "🚠",
                name: "mountain cableway",
                category,
            },
            Emoji {
                emoji: "🚡",
                name: "aerial tramway",
                category,
            },
            Emoji {
                emoji: "🛰️",
                name: "satellite",
                category,
            },
            Emoji {
                emoji: "🚀",
                name: "rocket",
                category,
            },
            Emoji {
                emoji: "🛸",
                name: "flying saucer",
                category,
            },
        ],
        EmojiCategory::Activities => vec![
            Emoji {
                emoji: "⚽",
                name: "soccer ball",
                category,
            },
            Emoji {
                emoji: "🏀",
                name: "basketball",
                category,
            },
            Emoji {
                emoji: "🏈",
                name: "american football",
                category,
            },
            Emoji {
                emoji: "⚾",
                name: "baseball",
                category,
            },
            Emoji {
                emoji: "🥎",
                name: "softball",
                category,
            },
            Emoji {
                emoji: "🎾",
                name: "tennis",
                category,
            },
            Emoji {
                emoji: "🏐",
                name: "volleyball",
                category,
            },
            Emoji {
                emoji: "🏉",
                name: "rugby football",
                category,
            },
            Emoji {
                emoji: "🥏",
                name: "flying disc",
                category,
            },
            Emoji {
                emoji: "🎱",
                name: "pool 8 ball",
                category,
            },
            Emoji {
                emoji: "🪀",
                name: "yo-yo",
                category,
            },
            Emoji {
                emoji: "🏓",
                name: "ping pong",
                category,
            },
            Emoji {
                emoji: "🏸",
                name: "badminton",
                category,
            },
            Emoji {
                emoji: "🏒",
                name: "ice hockey",
                category,
            },
            Emoji {
                emoji: "🏑",
                name: "field hockey",
                category,
            },
            Emoji {
                emoji: "🥍",
                name: "lacrosse",
                category,
            },
            Emoji {
                emoji: "🏏",
                name: "cricket game",
                category,
            },
            Emoji {
                emoji: "🪃",
                name: "boomerang",
                category,
            },
            Emoji {
                emoji: "🥅",
                name: "goal net",
                category,
            },
            Emoji {
                emoji: "⛳",
                name: "flag in hole",
                category,
            },
            Emoji {
                emoji: "🪁",
                name: "kite",
                category,
            },
            Emoji {
                emoji: "🏹",
                name: "bow and arrow",
                category,
            },
            Emoji {
                emoji: "🎣",
                name: "fishing pole",
                category,
            },
            Emoji {
                emoji: "🤿",
                name: "diving mask",
                category,
            },
            Emoji {
                emoji: "🥊",
                name: "boxing glove",
                category,
            },
            Emoji {
                emoji: "🥋",
                name: "martial arts uniform",
                category,
            },
            Emoji {
                emoji: "🎽",
                name: "running shirt",
                category,
            },
            Emoji {
                emoji: "🛹",
                name: "skateboard",
                category,
            },
            Emoji {
                emoji: "🛷",
                name: "sled",
                category,
            },
            Emoji {
                emoji: "⛸️",
                name: "ice skate",
                category,
            },
            Emoji {
                emoji: "🥌",
                name: "curling stone",
                category,
            },
            Emoji {
                emoji: "🎿",
                name: "skis",
                category,
            },
            Emoji {
                emoji: "⛷️",
                name: "skier",
                category,
            },
            Emoji {
                emoji: "🏂",
                name: "snowboarder",
                category,
            },
            Emoji {
                emoji: "🪂",
                name: "parachute",
                category,
            },
            Emoji {
                emoji: "🏋️",
                name: "person lifting weights",
                category,
            },
            Emoji {
                emoji: "🤼",
                name: "people wrestling",
                category,
            },
            Emoji {
                emoji: "🤸",
                name: "person cartwheeling",
                category,
            },
            Emoji {
                emoji: "🤺",
                name: "person fencing",
                category,
            },
            Emoji {
                emoji: "🤾",
                name: "person playing handball",
                category,
            },
            Emoji {
                emoji: "🏌️",
                name: "person golfing",
                category,
            },
            Emoji {
                emoji: "🏇",
                name: "horse racing",
                category,
            },
            Emoji {
                emoji: "🧘",
                name: "person in lotus position",
                category,
            },
            Emoji {
                emoji: "🏄",
                name: "person surfing",
                category,
            },
            Emoji {
                emoji: "🏊",
                name: "person swimming",
                category,
            },
            Emoji {
                emoji: "🤽",
                name: "person playing water polo",
                category,
            },
            Emoji {
                emoji: "🚣",
                name: "person rowing boat",
                category,
            },
            Emoji {
                emoji: "🧗",
                name: "person climbing",
                category,
            },
            Emoji {
                emoji: "🚵",
                name: "person mountain biking",
                category,
            },
            Emoji {
                emoji: "🚴",
                name: "person biking",
                category,
            },
            Emoji {
                emoji: "🏆",
                name: "trophy",
                category,
            },
            Emoji {
                emoji: "🥇",
                name: "1st place medal",
                category,
            },
            Emoji {
                emoji: "🥈",
                name: "2nd place medal",
                category,
            },
            Emoji {
                emoji: "🥉",
                name: "3rd place medal",
                category,
            },
            Emoji {
                emoji: "🏅",
                name: "sports medal",
                category,
            },
            Emoji {
                emoji: "🎖️",
                name: "military medal",
                category,
            },
            Emoji {
                emoji: "🏵️",
                name: "rosette",
                category,
            },
            Emoji {
                emoji: "🎗️",
                name: "reminder ribbon",
                category,
            },
            Emoji {
                emoji: "🎫",
                name: "ticket",
                category,
            },
            Emoji {
                emoji: "🎟️",
                name: "admission tickets",
                category,
            },
            Emoji {
                emoji: "🎪",
                name: "circus tent",
                category,
            },
            Emoji {
                emoji: "🤹",
                name: "person juggling",
                category,
            },
            Emoji {
                emoji: "🎭",
                name: "performing arts",
                category,
            },
            Emoji {
                emoji: "🩰",
                name: "ballet shoes",
                category,
            },
            Emoji {
                emoji: "🎨",
                name: "artist palette",
                category,
            },
            Emoji {
                emoji: "🎬",
                name: "clapper board",
                category,
            },
            Emoji {
                emoji: "🎤",
                name: "microphone",
                category,
            },
            Emoji {
                emoji: "🎧",
                name: "headphone",
                category,
            },
            Emoji {
                emoji: "🎼",
                name: "musical score",
                category,
            },
            Emoji {
                emoji: "🎹",
                name: "musical keyboard",
                category,
            },
            Emoji {
                emoji: "🥁",
                name: "drum",
                category,
            },
            Emoji {
                emoji: "🪘",
                name: "long drum",
                category,
            },
            Emoji {
                emoji: "🎷",
                name: "saxophone",
                category,
            },
            Emoji {
                emoji: "🎺",
                name: "trumpet",
                category,
            },
            Emoji {
                emoji: "🎸",
                name: "guitar",
                category,
            },
            Emoji {
                emoji: "🪕",
                name: "banjo",
                category,
            },
            Emoji {
                emoji: "🎻",
                name: "violin",
                category,
            },
            Emoji {
                emoji: "🎲",
                name: "game die",
                category,
            },
            Emoji {
                emoji: "♟️",
                name: "chess pawn",
                category,
            },
            Emoji {
                emoji: "🎯",
                name: "direct hit",
                category,
            },
            Emoji {
                emoji: "🎳",
                name: "bowling",
                category,
            },
            Emoji {
                emoji: "🎮",
                name: "video game",
                category,
            },
            Emoji {
                emoji: "🎰",
                name: "slot machine",
                category,
            },
            Emoji {
                emoji: "🧩",
                name: "puzzle piece",
                category,
            },
        ],
        EmojiCategory::Objects => vec![
            Emoji {
                emoji: "⌚",
                name: "watch",
                category,
            },
            Emoji {
                emoji: "📱",
                name: "mobile phone",
                category,
            },
            Emoji {
                emoji: "📲",
                name: "mobile phone with arrow",
                category,
            },
            Emoji {
                emoji: "💻",
                name: "laptop",
                category,
            },
            Emoji {
                emoji: "⌨️",
                name: "keyboard",
                category,
            },
            Emoji {
                emoji: "🖥️",
                name: "desktop computer",
                category,
            },
            Emoji {
                emoji: "🖨️",
                name: "printer",
                category,
            },
            Emoji {
                emoji: "🖱️",
                name: "computer mouse",
                category,
            },
            Emoji {
                emoji: "🖲️",
                name: "trackball",
                category,
            },
            Emoji {
                emoji: "💽",
                name: "computer disk",
                category,
            },
            Emoji {
                emoji: "💾",
                name: "floppy disk",
                category,
            },
            Emoji {
                emoji: "💿",
                name: "optical disk",
                category,
            },
            Emoji {
                emoji: "📀",
                name: "dvd",
                category,
            },
            Emoji {
                emoji: "🧮",
                name: "abacus",
                category,
            },
            Emoji {
                emoji: "🎥",
                name: "movie camera",
                category,
            },
            Emoji {
                emoji: "🎞️",
                name: "film frames",
                category,
            },
            Emoji {
                emoji: "📽️",
                name: "film projector",
                category,
            },
            Emoji {
                emoji: "🎬",
                name: "clapper board",
                category,
            },
            Emoji {
                emoji: "📺",
                name: "television",
                category,
            },
            Emoji {
                emoji: "📷",
                name: "camera",
                category,
            },
            Emoji {
                emoji: "📸",
                name: "camera with flash",
                category,
            },
            Emoji {
                emoji: "📹",
                name: "video camera",
                category,
            },
            Emoji {
                emoji: "📼",
                name: "videocassette",
                category,
            },
            Emoji {
                emoji: "🔍",
                name: "magnifying glass tilted left",
                category,
            },
            Emoji {
                emoji: "🔎",
                name: "magnifying glass tilted right",
                category,
            },
            Emoji {
                emoji: "🕯️",
                name: "candle",
                category,
            },
            Emoji {
                emoji: "💡",
                name: "light bulb",
                category,
            },
            Emoji {
                emoji: "🔦",
                name: "flashlight",
                category,
            },
            Emoji {
                emoji: "🏮",
                name: "red paper lantern",
                category,
            },
            Emoji {
                emoji: "🪔",
                name: "diya lamp",
                category,
            },
            Emoji {
                emoji: "📔",
                name: "notebook with decorative cover",
                category,
            },
            Emoji {
                emoji: "📕",
                name: "closed book",
                category,
            },
            Emoji {
                emoji: "📖",
                name: "open book",
                category,
            },
            Emoji {
                emoji: "📗",
                name: "green book",
                category,
            },
            Emoji {
                emoji: "📘",
                name: "blue book",
                category,
            },
            Emoji {
                emoji: "📙",
                name: "orange book",
                category,
            },
            Emoji {
                emoji: "📚",
                name: "books",
                category,
            },
            Emoji {
                emoji: "📓",
                name: "notebook",
                category,
            },
            Emoji {
                emoji: "📒",
                name: "ledger",
                category,
            },
            Emoji {
                emoji: "📃",
                name: "page with curl",
                category,
            },
            Emoji {
                emoji: "📜",
                name: "scroll",
                category,
            },
            Emoji {
                emoji: "📄",
                name: "page facing up",
                category,
            },
            Emoji {
                emoji: "📰",
                name: "newspaper",
                category,
            },
            Emoji {
                emoji: "🗞️",
                name: "rolled-up newspaper",
                category,
            },
            Emoji {
                emoji: "📑",
                name: "bookmark tabs",
                category,
            },
            Emoji {
                emoji: "🔖",
                name: "bookmark",
                category,
            },
            Emoji {
                emoji: "🏷️",
                name: "label",
                category,
            },
            Emoji {
                emoji: "💰",
                name: "money bag",
                category,
            },
            Emoji {
                emoji: "🪙",
                name: "coin",
                category,
            },
            Emoji {
                emoji: "💴",
                name: "yen banknote",
                category,
            },
            Emoji {
                emoji: "💵",
                name: "dollar banknote",
                category,
            },
            Emoji {
                emoji: "💶",
                name: "euro banknote",
                category,
            },
            Emoji {
                emoji: "💷",
                name: "pound banknote",
                category,
            },
            Emoji {
                emoji: "💸",
                name: "money with wings",
                category,
            },
            Emoji {
                emoji: "💳",
                name: "credit card",
                category,
            },
            Emoji {
                emoji: "🧾",
                name: "receipt",
                category,
            },
            Emoji {
                emoji: "💹",
                name: "chart increasing with yen",
                category,
            },
            Emoji {
                emoji: "✉️",
                name: "envelope",
                category,
            },
            Emoji {
                emoji: "📧",
                name: "e-mail",
                category,
            },
            Emoji {
                emoji: "📨",
                name: "incoming envelope",
                category,
            },
            Emoji {
                emoji: "📩",
                name: "envelope with arrow",
                category,
            },
            Emoji {
                emoji: "📤",
                name: "outbox tray",
                category,
            },
            Emoji {
                emoji: "📥",
                name: "inbox tray",
                category,
            },
            Emoji {
                emoji: "📦",
                name: "package",
                category,
            },
            Emoji {
                emoji: "📫",
                name: "closed mailbox with raised flag",
                category,
            },
            Emoji {
                emoji: "📪",
                name: "closed mailbox with lowered flag",
                category,
            },
            Emoji {
                emoji: "📬",
                name: "open mailbox with raised flag",
                category,
            },
            Emoji {
                emoji: "📭",
                name: "open mailbox with lowered flag",
                category,
            },
            Emoji {
                emoji: "📮",
                name: "postbox",
                category,
            },
            Emoji {
                emoji: "🗳️",
                name: "ballot box with ballot",
                category,
            },
            Emoji {
                emoji: "✏️",
                name: "pencil",
                category,
            },
            Emoji {
                emoji: "✒️",
                name: "black nib",
                category,
            },
            Emoji {
                emoji: "🖋️",
                name: "fountain pen",
                category,
            },
            Emoji {
                emoji: "🖊️",
                name: "pen",
                category,
            },
            Emoji {
                emoji: "🖌️",
                name: "paintbrush",
                category,
            },
            Emoji {
                emoji: "🖍️",
                name: "crayon",
                category,
            },
            Emoji {
                emoji: "📝",
                name: "memo",
                category,
            },
            Emoji {
                emoji: "💼",
                name: "briefcase",
                category,
            },
            Emoji {
                emoji: "📁",
                name: "file folder",
                category,
            },
            Emoji {
                emoji: "📂",
                name: "open file folder",
                category,
            },
            Emoji {
                emoji: "🗂️",
                name: "card index dividers",
                category,
            },
            Emoji {
                emoji: "📅",
                name: "calendar",
                category,
            },
            Emoji {
                emoji: "📆",
                name: "tear-off calendar",
                category,
            },
            Emoji {
                emoji: "🗒️",
                name: "spiral notepad",
                category,
            },
            Emoji {
                emoji: "🗓️",
                name: "spiral calendar",
                category,
            },
            Emoji {
                emoji: "📇",
                name: "card index",
                category,
            },
            Emoji {
                emoji: "📈",
                name: "chart increasing",
                category,
            },
            Emoji {
                emoji: "📉",
                name: "chart decreasing",
                category,
            },
            Emoji {
                emoji: "📊",
                name: "bar chart",
                category,
            },
            Emoji {
                emoji: "📋",
                name: "clipboard",
                category,
            },
            Emoji {
                emoji: "📌",
                name: "pushpin",
                category,
            },
            Emoji {
                emoji: "📍",
                name: "round pushpin",
                category,
            },
            Emoji {
                emoji: "📎",
                name: "paperclip",
                category,
            },
            Emoji {
                emoji: "🖇️",
                name: "linked paperclips",
                category,
            },
            Emoji {
                emoji: "📏",
                name: "straight ruler",
                category,
            },
            Emoji {
                emoji: "📐",
                name: "triangular ruler",
                category,
            },
            Emoji {
                emoji: "✂️",
                name: "scissors",
                category,
            },
            Emoji {
                emoji: "🗃️",
                name: "card file box",
                category,
            },
            Emoji {
                emoji: "🗄️",
                name: "file cabinet",
                category,
            },
            Emoji {
                emoji: "🗑️",
                name: "wastebasket",
                category,
            },
            Emoji {
                emoji: "🔒",
                name: "locked",
                category,
            },
            Emoji {
                emoji: "🔓",
                name: "unlocked",
                category,
            },
            Emoji {
                emoji: "🔏",
                name: "locked with pen",
                category,
            },
            Emoji {
                emoji: "🔐",
                name: "locked with key",
                category,
            },
            Emoji {
                emoji: "🔑",
                name: "key",
                category,
            },
            Emoji {
                emoji: "🗝️",
                name: "old key",
                category,
            },
            Emoji {
                emoji: "🔨",
                name: "hammer",
                category,
            },
            Emoji {
                emoji: "🪓",
                name: "axe",
                category,
            },
            Emoji {
                emoji: "⛏️",
                name: "pick",
                category,
            },
            Emoji {
                emoji: "⚒️",
                name: "hammer and pick",
                category,
            },
            Emoji {
                emoji: "🛠️",
                name: "hammer and wrench",
                category,
            },
            Emoji {
                emoji: "🗡️",
                name: "dagger",
                category,
            },
            Emoji {
                emoji: "⚔️",
                name: "crossed swords",
                category,
            },
            Emoji {
                emoji: "🔫",
                name: "pistol",
                category,
            },
            Emoji {
                emoji: "🪃",
                name: "boomerang",
                category,
            },
            Emoji {
                emoji: "🏹",
                name: "bow and arrow",
                category,
            },
            Emoji {
                emoji: "🛡️",
                name: "shield",
                category,
            },
            Emoji {
                emoji: "🪚",
                name: "carpentry saw",
                category,
            },
            Emoji {
                emoji: "🔧",
                name: "wrench",
                category,
            },
            Emoji {
                emoji: "🪛",
                name: "screwdriver",
                category,
            },
            Emoji {
                emoji: "🔩",
                name: "nut and bolt",
                category,
            },
            Emoji {
                emoji: "⚙️",
                name: "gear",
                category,
            },
            Emoji {
                emoji: "🗜️",
                name: "clamp",
                category,
            },
            Emoji {
                emoji: "⚖️",
                name: "balance scale",
                category,
            },
            Emoji {
                emoji: "🦯",
                name: "white cane",
                category,
            },
            Emoji {
                emoji: "🔗",
                name: "link",
                category,
            },
            Emoji {
                emoji: "⛓️",
                name: "chains",
                category,
            },
            Emoji {
                emoji: "🪝",
                name: "hook",
                category,
            },
            Emoji {
                emoji: "🧰",
                name: "toolbox",
                category,
            },
            Emoji {
                emoji: "🧲",
                name: "magnet",
                category,
            },
            Emoji {
                emoji: "🪜",
                name: "ladder",
                category,
            },
        ],
        EmojiCategory::Symbols => vec![
            Emoji {
                emoji: "❤️",
                name: "red heart",
                category,
            },
            Emoji {
                emoji: "🧡",
                name: "orange heart",
                category,
            },
            Emoji {
                emoji: "💛",
                name: "yellow heart",
                category,
            },
            Emoji {
                emoji: "💚",
                name: "green heart",
                category,
            },
            Emoji {
                emoji: "💙",
                name: "blue heart",
                category,
            },
            Emoji {
                emoji: "💜",
                name: "purple heart",
                category,
            },
            Emoji {
                emoji: "🖤",
                name: "black heart",
                category,
            },
            Emoji {
                emoji: "🤍",
                name: "white heart",
                category,
            },
            Emoji {
                emoji: "🤎",
                name: "brown heart",
                category,
            },
            Emoji {
                emoji: "💔",
                name: "broken heart",
                category,
            },
            Emoji {
                emoji: "❣️",
                name: "heart exclamation",
                category,
            },
            Emoji {
                emoji: "💕",
                name: "two hearts",
                category,
            },
            Emoji {
                emoji: "💞",
                name: "revolving hearts",
                category,
            },
            Emoji {
                emoji: "💓",
                name: "beating heart",
                category,
            },
            Emoji {
                emoji: "💗",
                name: "growing heart",
                category,
            },
            Emoji {
                emoji: "💖",
                name: "sparkling heart",
                category,
            },
            Emoji {
                emoji: "💘",
                name: "heart with arrow",
                category,
            },
            Emoji {
                emoji: "💝",
                name: "heart with ribbon",
                category,
            },
            Emoji {
                emoji: "💟",
                name: "heart decoration",
                category,
            },
            Emoji {
                emoji: "☮️",
                name: "peace symbol",
                category,
            },
            Emoji {
                emoji: "✝️",
                name: "latin cross",
                category,
            },
            Emoji {
                emoji: "☪️",
                name: "star and crescent",
                category,
            },
            Emoji {
                emoji: "🕉️",
                name: "om",
                category,
            },
            Emoji {
                emoji: "☸️",
                name: "wheel of dharma",
                category,
            },
            Emoji {
                emoji: "✡️",
                name: "star of david",
                category,
            },
            Emoji {
                emoji: "🔯",
                name: "dotted six-pointed star",
                category,
            },
            Emoji {
                emoji: "🕎",
                name: "menorah",
                category,
            },
            Emoji {
                emoji: "☯️",
                name: "yin yang",
                category,
            },
            Emoji {
                emoji: "☦️",
                name: "orthodox cross",
                category,
            },
            Emoji {
                emoji: "🛐",
                name: "place of worship",
                category,
            },
            Emoji {
                emoji: "⛎",
                name: "ophiuchus",
                category,
            },
            Emoji {
                emoji: "♈",
                name: "aries",
                category,
            },
            Emoji {
                emoji: "♉",
                name: "taurus",
                category,
            },
            Emoji {
                emoji: "♊",
                name: "gemini",
                category,
            },
            Emoji {
                emoji: "♋",
                name: "cancer",
                category,
            },
            Emoji {
                emoji: "♌",
                name: "leo",
                category,
            },
            Emoji {
                emoji: "♍",
                name: "virgo",
                category,
            },
            Emoji {
                emoji: "♎",
                name: "libra",
                category,
            },
            Emoji {
                emoji: "♏",
                name: "scorpio",
                category,
            },
            Emoji {
                emoji: "♐",
                name: "sagittarius",
                category,
            },
            Emoji {
                emoji: "♑",
                name: "capricorn",
                category,
            },
            Emoji {
                emoji: "♒",
                name: "aquarius",
                category,
            },
            Emoji {
                emoji: "♓",
                name: "pisces",
                category,
            },
            Emoji {
                emoji: "🆔",
                name: "id button",
                category,
            },
            Emoji {
                emoji: "⚛️",
                name: "atom symbol",
                category,
            },
            Emoji {
                emoji: "🉑",
                name: "japanese acceptable button",
                category,
            },
            Emoji {
                emoji: "☢️",
                name: "radioactive",
                category,
            },
            Emoji {
                emoji: "☣️",
                name: "biohazard",
                category,
            },
            Emoji {
                emoji: "📴",
                name: "mobile phone off",
                category,
            },
            Emoji {
                emoji: "📳",
                name: "vibration mode",
                category,
            },
            Emoji {
                emoji: "🈶",
                name: "japanese not free of charge button",
                category,
            },
            Emoji {
                emoji: "🈚",
                name: "japanese free of charge button",
                category,
            },
            Emoji {
                emoji: "🈸",
                name: "japanese application button",
                category,
            },
            Emoji {
                emoji: "🈺",
                name: "japanese open for business button",
                category,
            },
            Emoji {
                emoji: "🈷️",
                name: "japanese monthly amount button",
                category,
            },
            Emoji {
                emoji: "✴️",
                name: "eight-pointed star",
                category,
            },
            Emoji {
                emoji: "🆚",
                name: "vs button",
                category,
            },
            Emoji {
                emoji: "💮",
                name: "white flower",
                category,
            },
            Emoji {
                emoji: "🉐",
                name: "japanese bargain button",
                category,
            },
            Emoji {
                emoji: "㊙️",
                name: "japanese secret button",
                category,
            },
            Emoji {
                emoji: "㊗️",
                name: "japanese congratulations button",
                category,
            },
            Emoji {
                emoji: "🈴",
                name: "japanese passing grade button",
                category,
            },
            Emoji {
                emoji: "🈵",
                name: "japanese no vacancy button",
                category,
            },
            Emoji {
                emoji: "🈹",
                name: "japanese discount button",
                category,
            },
            Emoji {
                emoji: "🈲",
                name: "japanese prohibited button",
                category,
            },
            Emoji {
                emoji: "🅰️",
                name: "a button",
                category,
            },
            Emoji {
                emoji: "🅱️",
                name: "b button",
                category,
            },
            Emoji {
                emoji: "🆎",
                name: "ab button",
                category,
            },
            Emoji {
                emoji: "🆑",
                name: "cl button",
                category,
            },
            Emoji {
                emoji: "🅾️",
                name: "o button",
                category,
            },
            Emoji {
                emoji: "🆘",
                name: "sos button",
                category,
            },
            Emoji {
                emoji: "❌",
                name: "cross mark",
                category,
            },
            Emoji {
                emoji: "⭕",
                name: "hollow red circle",
                category,
            },
            Emoji {
                emoji: "🛑",
                name: "stop sign",
                category,
            },
            Emoji {
                emoji: "⛔",
                name: "no entry",
                category,
            },
            Emoji {
                emoji: "📛",
                name: "name badge",
                category,
            },
            Emoji {
                emoji: "🚫",
                name: "prohibited",
                category,
            },
            Emoji {
                emoji: "💯",
                name: "hundred points",
                category,
            },
            Emoji {
                emoji: "💢",
                name: "anger symbol",
                category,
            },
            Emoji {
                emoji: "♨️",
                name: "hot springs",
                category,
            },
            Emoji {
                emoji: "🚷",
                name: "no pedestrians",
                category,
            },
            Emoji {
                emoji: "🚯",
                name: "no littering",
                category,
            },
            Emoji {
                emoji: "🚳",
                name: "no bicycles",
                category,
            },
            Emoji {
                emoji: "🚱",
                name: "non-potable water",
                category,
            },
            Emoji {
                emoji: "🔞",
                name: "no one under eighteen",
                category,
            },
            Emoji {
                emoji: "📵",
                name: "no mobile phones",
                category,
            },
            Emoji {
                emoji: "🚭",
                name: "no smoking",
                category,
            },
            Emoji {
                emoji: "❗",
                name: "exclamation mark",
                category,
            },
            Emoji {
                emoji: "❕",
                name: "white exclamation mark",
                category,
            },
            Emoji {
                emoji: "❓",
                name: "question mark",
                category,
            },
            Emoji {
                emoji: "❔",
                name: "white question mark",
                category,
            },
            Emoji {
                emoji: "‼️",
                name: "double exclamation mark",
                category,
            },
            Emoji {
                emoji: "⁉️",
                name: "exclamation question mark",
                category,
            },
            Emoji {
                emoji: "🔅",
                name: "dim button",
                category,
            },
            Emoji {
                emoji: "🔆",
                name: "bright button",
                category,
            },
            Emoji {
                emoji: "〽️",
                name: "part alternation mark",
                category,
            },
            Emoji {
                emoji: "⚠️",
                name: "warning",
                category,
            },
            Emoji {
                emoji: "🚸",
                name: "children crossing",
                category,
            },
            Emoji {
                emoji: "🔱",
                name: "trident emblem",
                category,
            },
            Emoji {
                emoji: "⚜️",
                name: "fleur-de-lis",
                category,
            },
            Emoji {
                emoji: "🔰",
                name: "japanese symbol for beginner",
                category,
            },
            Emoji {
                emoji: "♻️",
                name: "recycling symbol",
                category,
            },
            Emoji {
                emoji: "✅",
                name: "check mark button",
                category,
            },
            Emoji {
                emoji: "🈯",
                name: "japanese reserved button",
                category,
            },
            Emoji {
                emoji: "💹",
                name: "chart increasing with yen",
                category,
            },
            Emoji {
                emoji: "❇️",
                name: "sparkle",
                category,
            },
            Emoji {
                emoji: "✳️",
                name: "eight-spoked asterisk",
                category,
            },
            Emoji {
                emoji: "❎",
                name: "cross mark button",
                category,
            },
            Emoji {
                emoji: "🌐",
                name: "globe with meridians",
                category,
            },
            Emoji {
                emoji: "💠",
                name: "diamond with a dot",
                category,
            },
            Emoji {
                emoji: "Ⓜ️",
                name: "circled m",
                category,
            },
            Emoji {
                emoji: "🌀",
                name: "cyclone",
                category,
            },
            Emoji {
                emoji: "💤",
                name: "zzz",
                category,
            },
            Emoji {
                emoji: "🏧",
                name: "atm sign",
                category,
            },
            Emoji {
                emoji: "🚾",
                name: "water closet",
                category,
            },
            Emoji {
                emoji: "♿",
                name: "wheelchair symbol",
                category,
            },
            Emoji {
                emoji: "🅿️",
                name: "p button",
                category,
            },
            Emoji {
                emoji: "🛗",
                name: "elevator",
                category,
            },
            Emoji {
                emoji: "🈳",
                name: "japanese vacancy button",
                category,
            },
            Emoji {
                emoji: "🈂️",
                name: "japanese service charge button",
                category,
            },
            Emoji {
                emoji: "🛂",
                name: "passport control",
                category,
            },
            Emoji {
                emoji: "🛃",
                name: "customs",
                category,
            },
            Emoji {
                emoji: "🛄",
                name: "baggage claim",
                category,
            },
            Emoji {
                emoji: "🛅",
                name: "left luggage",
                category,
            },
            Emoji {
                emoji: "🚹",
                name: "mens room",
                category,
            },
            Emoji {
                emoji: "🚺",
                name: "womens room",
                category,
            },
            Emoji {
                emoji: "🚼",
                name: "baby symbol",
                category,
            },
            Emoji {
                emoji: "⚧️",
                name: "transgender symbol",
                category,
            },
            Emoji {
                emoji: "🚻",
                name: "restroom",
                category,
            },
            Emoji {
                emoji: "🚮",
                name: "litter in bin sign",
                category,
            },
            Emoji {
                emoji: "🎦",
                name: "cinema",
                category,
            },
            Emoji {
                emoji: "📶",
                name: "antenna bars",
                category,
            },
            Emoji {
                emoji: "🈁",
                name: "japanese here button",
                category,
            },
            Emoji {
                emoji: "🔣",
                name: "input symbols",
                category,
            },
            Emoji {
                emoji: "ℹ️",
                name: "information",
                category,
            },
            Emoji {
                emoji: "🔤",
                name: "input latin letters",
                category,
            },
            Emoji {
                emoji: "🔡",
                name: "input latin lowercase",
                category,
            },
            Emoji {
                emoji: "🔠",
                name: "input latin uppercase",
                category,
            },
            Emoji {
                emoji: "🆖",
                name: "ng button",
                category,
            },
            Emoji {
                emoji: "🆗",
                name: "ok button",
                category,
            },
            Emoji {
                emoji: "🆙",
                name: "up! button",
                category,
            },
            Emoji {
                emoji: "🆒",
                name: "cool button",
                category,
            },
            Emoji {
                emoji: "🆕",
                name: "new button",
                category,
            },
            Emoji {
                emoji: "🆓",
                name: "free button",
                category,
            },
            Emoji {
                emoji: "0️⃣",
                name: "keycap 0",
                category,
            },
            Emoji {
                emoji: "1️⃣",
                name: "keycap 1",
                category,
            },
            Emoji {
                emoji: "2️⃣",
                name: "keycap 2",
                category,
            },
            Emoji {
                emoji: "3️⃣",
                name: "keycap 3",
                category,
            },
            Emoji {
                emoji: "4️⃣",
                name: "keycap 4",
                category,
            },
            Emoji {
                emoji: "5️⃣",
                name: "keycap 5",
                category,
            },
            Emoji {
                emoji: "6️⃣",
                name: "keycap 6",
                category,
            },
            Emoji {
                emoji: "7️⃣",
                name: "keycap 7",
                category,
            },
            Emoji {
                emoji: "8️⃣",
                name: "keycap 8",
                category,
            },
            Emoji {
                emoji: "9️⃣",
                name: "keycap 9",
                category,
            },
            Emoji {
                emoji: "🔟",
                name: "keycap 10",
                category,
            },
            Emoji {
                emoji: "🔢",
                name: "input numbers",
                category,
            },
            Emoji {
                emoji: "#️⃣",
                name: "keycap #",
                category,
            },
            Emoji {
                emoji: "*️⃣",
                name: "keycap *",
                category,
            },
            Emoji {
                emoji: "⏏️",
                name: "eject button",
                category,
            },
            Emoji {
                emoji: "▶️",
                name: "play button",
                category,
            },
            Emoji {
                emoji: "⏸️",
                name: "pause button",
                category,
            },
            Emoji {
                emoji: "⏯️",
                name: "play or pause button",
                category,
            },
            Emoji {
                emoji: "⏹️",
                name: "stop button",
                category,
            },
            Emoji {
                emoji: "⏺️",
                name: "record button",
                category,
            },
            Emoji {
                emoji: "⏭️",
                name: "next track button",
                category,
            },
            Emoji {
                emoji: "⏮️",
                name: "last track button",
                category,
            },
            Emoji {
                emoji: "⏩",
                name: "fast-forward button",
                category,
            },
            Emoji {
                emoji: "⏪",
                name: "fast reverse button",
                category,
            },
            Emoji {
                emoji: "⏫",
                name: "fast up button",
                category,
            },
            Emoji {
                emoji: "⏬",
                name: "fast down button",
                category,
            },
            Emoji {
                emoji: "◀️",
                name: "reverse button",
                category,
            },
            Emoji {
                emoji: "🔼",
                name: "upwards button",
                category,
            },
            Emoji {
                emoji: "🔽",
                name: "downwards button",
                category,
            },
            Emoji {
                emoji: "➡️",
                name: "right arrow",
                category,
            },
            Emoji {
                emoji: "⬅️",
                name: "left arrow",
                category,
            },
            Emoji {
                emoji: "⬆️",
                name: "up arrow",
                category,
            },
            Emoji {
                emoji: "⬇️",
                name: "down arrow",
                category,
            },
            Emoji {
                emoji: "↗️",
                name: "up-right arrow",
                category,
            },
            Emoji {
                emoji: "↘️",
                name: "down-right arrow",
                category,
            },
            Emoji {
                emoji: "↙️",
                name: "down-left arrow",
                category,
            },
            Emoji {
                emoji: "↖️",
                name: "up-left arrow",
                category,
            },
            Emoji {
                emoji: "↕️",
                name: "up-down arrow",
                category,
            },
            Emoji {
                emoji: "↔️",
                name: "left-right arrow",
                category,
            },
            Emoji {
                emoji: "↪️",
                name: "right arrow curving left",
                category,
            },
            Emoji {
                emoji: "↩️",
                name: "left arrow curving right",
                category,
            },
            Emoji {
                emoji: "⤴️",
                name: "right arrow curving up",
                category,
            },
            Emoji {
                emoji: "⤵️",
                name: "right arrow curving down",
                category,
            },
            Emoji {
                emoji: "🔀",
                name: "shuffle tracks button",
                category,
            },
            Emoji {
                emoji: "🔁",
                name: "repeat button",
                category,
            },
            Emoji {
                emoji: "🔂",
                name: "repeat single button",
                category,
            },
            Emoji {
                emoji: "🔄",
                name: "counterclockwise arrows button",
                category,
            },
            Emoji {
                emoji: "🔃",
                name: "clockwise vertical arrows",
                category,
            },
            Emoji {
                emoji: "🎵",
                name: "musical note",
                category,
            },
            Emoji {
                emoji: "🎶",
                name: "musical notes",
                category,
            },
            Emoji {
                emoji: "➕",
                name: "plus sign",
                category,
            },
            Emoji {
                emoji: "➖",
                name: "minus sign",
                category,
            },
            Emoji {
                emoji: "➗",
                name: "division sign",
                category,
            },
            Emoji {
                emoji: "✖️",
                name: "multiplication sign",
                category,
            },
            Emoji {
                emoji: "🟰",
                name: "heavy equals sign",
                category,
            },
            Emoji {
                emoji: "♾️",
                name: "infinity",
                category,
            },
            Emoji {
                emoji: "💲",
                name: "heavy dollar sign",
                category,
            },
            Emoji {
                emoji: "💱",
                name: "currency exchange",
                category,
            },
            Emoji {
                emoji: "™️",
                name: "trade mark",
                category,
            },
            Emoji {
                emoji: "©️",
                name: "copyright",
                category,
            },
            Emoji {
                emoji: "®️",
                name: "registered",
                category,
            },
            Emoji {
                emoji: "〰️",
                name: "wavy dash",
                category,
            },
            Emoji {
                emoji: "➰",
                name: "curly loop",
                category,
            },
            Emoji {
                emoji: "➿",
                name: "double curly loop",
                category,
            },
            Emoji {
                emoji: "🔚",
                name: "end arrow",
                category,
            },
            Emoji {
                emoji: "🔙",
                name: "back arrow",
                category,
            },
            Emoji {
                emoji: "🔛",
                name: "on! arrow",
                category,
            },
            Emoji {
                emoji: "🔝",
                name: "top arrow",
                category,
            },
            Emoji {
                emoji: "🔜",
                name: "soon arrow",
                category,
            },
            Emoji {
                emoji: "✔️",
                name: "check mark",
                category,
            },
            Emoji {
                emoji: "☑️",
                name: "check box with check",
                category,
            },
            Emoji {
                emoji: "🔘",
                name: "radio button",
                category,
            },
            Emoji {
                emoji: "🔴",
                name: "red circle",
                category,
            },
            Emoji {
                emoji: "🟠",
                name: "orange circle",
                category,
            },
            Emoji {
                emoji: "🟡",
                name: "yellow circle",
                category,
            },
            Emoji {
                emoji: "🟢",
                name: "green circle",
                category,
            },
            Emoji {
                emoji: "🔵",
                name: "blue circle",
                category,
            },
            Emoji {
                emoji: "🟣",
                name: "purple circle",
                category,
            },
            Emoji {
                emoji: "🟤",
                name: "brown circle",
                category,
            },
            Emoji {
                emoji: "⚫",
                name: "black circle",
                category,
            },
            Emoji {
                emoji: "⚪",
                name: "white circle",
                category,
            },
            Emoji {
                emoji: "🟥",
                name: "red square",
                category,
            },
            Emoji {
                emoji: "🟧",
                name: "orange square",
                category,
            },
            Emoji {
                emoji: "🟨",
                name: "yellow square",
                category,
            },
            Emoji {
                emoji: "🟩",
                name: "green square",
                category,
            },
            Emoji {
                emoji: "🟦",
                name: "blue square",
                category,
            },
            Emoji {
                emoji: "🟪",
                name: "purple square",
                category,
            },
            Emoji {
                emoji: "🟫",
                name: "brown square",
                category,
            },
            Emoji {
                emoji: "⬛",
                name: "black large square",
                category,
            },
            Emoji {
                emoji: "⬜",
                name: "white large square",
                category,
            },
            Emoji {
                emoji: "◼️",
                name: "black medium square",
                category,
            },
            Emoji {
                emoji: "◻️",
                name: "white medium square",
                category,
            },
            Emoji {
                emoji: "◾",
                name: "black medium-small square",
                category,
            },
            Emoji {
                emoji: "◽",
                name: "white medium-small square",
                category,
            },
            Emoji {
                emoji: "▪️",
                name: "black small square",
                category,
            },
            Emoji {
                emoji: "▫️",
                name: "white small square",
                category,
            },
            Emoji {
                emoji: "🔶",
                name: "large orange diamond",
                category,
            },
            Emoji {
                emoji: "🔷",
                name: "large blue diamond",
                category,
            },
            Emoji {
                emoji: "🔸",
                name: "small orange diamond",
                category,
            },
            Emoji {
                emoji: "🔹",
                name: "small blue diamond",
                category,
            },
            Emoji {
                emoji: "🔺",
                name: "red triangle pointed up",
                category,
            },
            Emoji {
                emoji: "🔻",
                name: "red triangle pointed down",
                category,
            },
            Emoji {
                emoji: "💠",
                name: "diamond with a dot",
                category,
            },
            Emoji {
                emoji: "🔘",
                name: "radio button",
                category,
            },
            Emoji {
                emoji: "🔳",
                name: "white square button",
                category,
            },
            Emoji {
                emoji: "🔲",
                name: "black square button",
                category,
            },
        ],
        EmojiCategory::Flags => vec![
            Emoji {
                emoji: "🏁",
                name: "chequered flag",
                category,
            },
            Emoji {
                emoji: "🚩",
                name: "triangular flag",
                category,
            },
            Emoji {
                emoji: "🎌",
                name: "crossed flags",
                category,
            },
            Emoji {
                emoji: "🏴",
                name: "black flag",
                category,
            },
            Emoji {
                emoji: "🏳️",
                name: "white flag",
                category,
            },
            Emoji {
                emoji: "🏳️‍🌈",
                name: "rainbow flag",
                category,
            },
            Emoji {
                emoji: "🏳️‍⚧️",
                name: "transgender flag",
                category,
            },
            Emoji {
                emoji: "🏴‍☠️",
                name: "pirate flag",
                category,
            },
            Emoji {
                emoji: "🇺🇸",
                name: "flag united states",
                category,
            },
            Emoji {
                emoji: "🇬🇧",
                name: "flag united kingdom",
                category,
            },
            Emoji {
                emoji: "🇨🇦",
                name: "flag canada",
                category,
            },
            Emoji {
                emoji: "🇦🇺",
                name: "flag australia",
                category,
            },
            Emoji {
                emoji: "🇩🇪",
                name: "flag germany",
                category,
            },
            Emoji {
                emoji: "🇫🇷",
                name: "flag france",
                category,
            },
            Emoji {
                emoji: "🇮🇹",
                name: "flag italy",
                category,
            },
            Emoji {
                emoji: "🇪🇸",
                name: "flag spain",
                category,
            },
            Emoji {
                emoji: "🇯🇵",
                name: "flag japan",
                category,
            },
            Emoji {
                emoji: "🇰🇷",
                name: "flag south korea",
                category,
            },
            Emoji {
                emoji: "🇨🇳",
                name: "flag china",
                category,
            },
            Emoji {
                emoji: "🇮🇳",
                name: "flag india",
                category,
            },
            Emoji {
                emoji: "🇧🇷",
                name: "flag brazil",
                category,
            },
            Emoji {
                emoji: "🇲🇽",
                name: "flag mexico",
                category,
            },
            Emoji {
                emoji: "🇷🇺",
                name: "flag russia",
                category,
            },
            Emoji {
                emoji: "🇳🇱",
                name: "flag netherlands",
                category,
            },
            Emoji {
                emoji: "🇧🇪",
                name: "flag belgium",
                category,
            },
            Emoji {
                emoji: "🇨🇭",
                name: "flag switzerland",
                category,
            },
            Emoji {
                emoji: "🇦🇹",
                name: "flag austria",
                category,
            },
            Emoji {
                emoji: "🇸🇪",
                name: "flag sweden",
                category,
            },
            Emoji {
                emoji: "🇳🇴",
                name: "flag norway",
                category,
            },
            Emoji {
                emoji: "🇩🇰",
                name: "flag denmark",
                category,
            },
            Emoji {
                emoji: "🇫🇮",
                name: "flag finland",
                category,
            },
            Emoji {
                emoji: "🇵🇱",
                name: "flag poland",
                category,
            },
            Emoji {
                emoji: "🇮🇪",
                name: "flag ireland",
                category,
            },
            Emoji {
                emoji: "🇵🇹",
                name: "flag portugal",
                category,
            },
            Emoji {
                emoji: "🇬🇷",
                name: "flag greece",
                category,
            },
            Emoji {
                emoji: "🇹🇷",
                name: "flag turkey",
                category,
            },
            Emoji {
                emoji: "🇮🇱",
                name: "flag israel",
                category,
            },
            Emoji {
                emoji: "🇸🇦",
                name: "flag saudi arabia",
                category,
            },
            Emoji {
                emoji: "🇦🇪",
                name: "flag united arab emirates",
                category,
            },
            Emoji {
                emoji: "🇿🇦",
                name: "flag south africa",
                category,
            },
            Emoji {
                emoji: "🇪🇬",
                name: "flag egypt",
                category,
            },
            Emoji {
                emoji: "🇳🇬",
                name: "flag nigeria",
                category,
            },
            Emoji {
                emoji: "🇰🇪",
                name: "flag kenya",
                category,
            },
            Emoji {
                emoji: "🇹🇭",
                name: "flag thailand",
                category,
            },
            Emoji {
                emoji: "🇻🇳",
                name: "flag vietnam",
                category,
            },
            Emoji {
                emoji: "🇵🇭",
                name: "flag philippines",
                category,
            },
            Emoji {
                emoji: "🇮🇩",
                name: "flag indonesia",
                category,
            },
            Emoji {
                emoji: "🇲🇾",
                name: "flag malaysia",
                category,
            },
            Emoji {
                emoji: "🇸🇬",
                name: "flag singapore",
                category,
            },
            Emoji {
                emoji: "🇳🇿",
                name: "flag new zealand",
                category,
            },
            Emoji {
                emoji: "🇦🇷",
                name: "flag argentina",
                category,
            },
            Emoji {
                emoji: "🇨🇴",
                name: "flag colombia",
                category,
            },
            Emoji {
                emoji: "🇨🇱",
                name: "flag chile",
                category,
            },
            Emoji {
                emoji: "🇵🇪",
                name: "flag peru",
                category,
            },
            Emoji {
                emoji: "🇺🇦",
                name: "flag ukraine",
                category,
            },
            Emoji {
                emoji: "🇨🇿",
                name: "flag czechia",
                category,
            },
            Emoji {
                emoji: "🇭🇺",
                name: "flag hungary",
                category,
            },
            Emoji {
                emoji: "🇷🇴",
                name: "flag romania",
                category,
            },
        ],
    }
}

/// Search all emojis
fn search_emojis(query: &str) -> Vec<Emoji> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for category in EmojiCategory::all() {
        if category == EmojiCategory::Recent {
            continue;
        }
        for emoji in get_emojis(category) {
            if emoji.name.to_lowercase().contains(&query_lower)
                || emoji.emoji.contains(&query_lower)
            {
                results.push(emoji);
            }
        }
    }

    results
}

/// Show the emoji picker dialog
pub fn show_emoji_picker<W, F>(parent: &W, on_select: F)
where
    W: IsA<gtk4::Widget>,
    F: Fn(String) + 'static,
{
    let dialog = Dialog::builder()
        .title("😀 Emoji Picker")
        .content_width(450)
        .content_height(500)
        .build();

    let main_box = Box::new(Orientation::Vertical, 8);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);
    main_box.set_margin_start(12);
    main_box.set_margin_end(12);

    // Search entry
    let search_entry = SearchEntry::builder()
        .placeholder_text("Search emojis...")
        .build();
    main_box.append(&search_entry);

    // Category buttons
    let category_box = Box::new(Orientation::Horizontal, 4);
    category_box.set_halign(gtk4::Align::Center);

    let current_category: Rc<RefCell<EmojiCategory>> =
        Rc::new(RefCell::new(EmojiCategory::Smileys));

    // Scrolled emoji grid
    let scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .min_content_height(350)
        .build();

    let emoji_flow = FlowBox::builder()
        .selection_mode(SelectionMode::None)
        .homogeneous(true)
        .max_children_per_line(12)
        .min_children_per_line(8)
        .row_spacing(4)
        .column_spacing(4)
        .build();

    scrolled.set_child(Some(&emoji_flow));

    // Function to populate emojis
    let populate_emojis = {
        let emoji_flow = emoji_flow.clone();
        let on_select = Rc::new(on_select);
        let dialog = dialog.clone();

        move |category: EmojiCategory, search_query: Option<&str>| {
            // Clear existing emojis
            while let Some(child) = emoji_flow.first_child() {
                emoji_flow.remove(&child);
            }

            let emojis = if let Some(query) = search_query {
                if query.is_empty() {
                    get_emojis(category)
                } else {
                    search_emojis(query)
                }
            } else {
                get_emojis(category)
            };

            for emoji_data in emojis {
                let btn = Button::builder()
                    .label(emoji_data.emoji)
                    .tooltip_text(emoji_data.name)
                    .css_classes(vec!["flat", "emoji-btn"])
                    .build();

                // Make button text larger
                if let Some(child) = btn.first_child() {
                    child.add_css_class("title-1");
                }

                let emoji_str = emoji_data.emoji.to_string();
                let on_select = on_select.clone();
                let dialog = dialog.clone();

                btn.connect_clicked(move |_| {
                    on_select(emoji_str.clone());
                    dialog.close();
                });

                emoji_flow.append(&btn);
            }
        }
    };

    // Create category buttons
    for category in EmojiCategory::all() {
        if category == EmojiCategory::Recent {
            continue; // Skip recent for now
        }

        let icon = match category {
            EmojiCategory::Recent => "🕐",
            EmojiCategory::Smileys => "😀",
            EmojiCategory::People => "👋",
            EmojiCategory::Animals => "🐕",
            EmojiCategory::Food => "🍕",
            EmojiCategory::Travel => "✈️",
            EmojiCategory::Activities => "⚽",
            EmojiCategory::Objects => "💡",
            EmojiCategory::Symbols => "❤️",
            EmojiCategory::Flags => "🏁",
        };

        let btn = Button::builder()
            .label(icon)
            .tooltip_text(category.label())
            .css_classes(vec!["flat", "circular"])
            .build();

        let populate = populate_emojis.clone();
        let current_cat = current_category.clone();
        let search_entry = search_entry.clone();

        btn.connect_clicked(move |_| {
            *current_cat.borrow_mut() = category;
            search_entry.set_text("");
            populate(category, None);
        });

        category_box.append(&btn);
    }

    main_box.append(&category_box);
    main_box.append(&scrolled);

    // Connect search
    {
        let populate = populate_emojis.clone();
        let current_cat = current_category.clone();

        search_entry.connect_search_changed(move |entry| {
            let query = entry.text();
            let category = *current_cat.borrow();
            if query.is_empty() {
                populate(category, None);
            } else {
                populate(category, Some(&query));
            }
        });
    }

    // Initial population
    populate_emojis(EmojiCategory::Smileys, None);

    // Create header bar
    let header = HeaderBar::new();

    // Create toolbar view
    let toolbar_view = ToolbarView::new();
    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&main_box));

    dialog.set_child(Some(&toolbar_view));
    dialog.present(Some(parent));

    // Focus search entry
    search_entry.grab_focus();
}
