//! # Enhanced Emoji Picker
//!
//! A comprehensive emoji picker with search, skin tones, recent emojis,
//! favorites, and kaomoji support.
//!
//! Features:
//! - Search by emoji name or keyword
//! - Skin tone modifier selection
//! - Recent emojis tracking (persists during session)
//! - Favorites/starred emojis
//! - Kaomoji (Japanese emoticons)
//! - Category tabs with icons

use gtk4::prelude::*;
use gtk4::{
    Box, Button, Entry, FlowBox, Label, Notebook, Orientation, PolicyType, Popover,
    ScrolledWindow, SelectionMode, ToggleButton,
};
use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

/// Maximum recent emojis to track
const MAX_RECENT: usize = 24;

/// Emoji with name for search
struct Emoji {
    char: &'static str,
    name: &'static str,
    keywords: &'static [&'static str],
    /// Whether this emoji supports skin tones
    skin_tone: bool,
}

/// Emoji category
struct EmojiCategory {
    name: &'static str,
    icon: &'static str,
    emojis: &'static [Emoji],
}

/// Skin tone modifiers
const SKIN_TONES: &[(&str, &str)] = &[
    ("", "Default"),
    ("\u{1F3FB}", "Light"),
    ("\u{1F3FC}", "Medium-Light"),
    ("\u{1F3FD}", "Medium"),
    ("\u{1F3FE}", "Medium-Dark"),
    ("\u{1F3FF}", "Dark"),
];

/// Kaomoji emoticons
const KAOMOJI: &[(&str, &str)] = &[
    ("( ´ ▽ ` )ノ", "Wave"),
    ("(╯°□°）╯︵ ┻━┻", "Table flip"),
    ("┬─┬ノ( º _ ºノ)", "Put table back"),
    ("¯\\_(ツ)_/¯", "Shrug"),
    ("(づ｡◕‿‿◕｡)づ", "Hug"),
    ("(ノಠ益ಠ)ノ彡┻━┻", "Angry flip"),
    ("(◕‿◕)", "Happy"),
    ("(｡◕‿◕｡)", "Cute happy"),
    ("(ಠ_ಠ)", "Disapproval"),
    ("( ͡° ͜ʖ ͡°)", "Lenny"),
    ("ʕ•ᴥ•ʔ", "Bear"),
    ("(◕ᴗ◕✿)", "Flower girl"),
    ("(╥﹏╥)", "Crying"),
    ("(｀・ω・´)", "Determined"),
    ("(≧◡≦)", "Excited"),
    ("(*^▽^*)", "Joyful"),
    ("(´;ω;`)", "Sad"),
    ("(ง'̀-'́)ง", "Fighting"),
    ("٩(◕‿◕｡)۶", "Celebration"),
    ("(☞ﾟヮﾟ)☞", "Pointing"),
    ("☜(ﾟヮﾟ☜)", "Pointing left"),
    ("(づ￣ ³￣)づ", "Kiss"),
    ("(っ˘ω˘ς )", "Sleepy"),
    ("(ﾉ◕ヮ◕)ﾉ*:・ﾟ✧", "Sparkle"),
    ("ε=ε=ε=┏(゜ロ゜;)┛", "Running"),
    ("( ˘ ³˘)♥", "Love"),
    ("(╬ Ò﹏Ó)", "Very angry"),
    ("(○´∀`○)", "Cheerful"),
    ("(´･ω･`)?", "Confused"),
    ("(ノ´ー`)ノ", "Cool wave"),
];

// Define emojis with names for searchability
const EMOJI_SMILEYS: &[Emoji] = &[
    Emoji {
        char: "😀",
        name: "grinning face",
        keywords: &["smile", "happy", "joy"],
        skin_tone: false,
    },
    Emoji {
        char: "😃",
        name: "grinning face big eyes",
        keywords: &["happy", "joy", "haha"],
        skin_tone: false,
    },
    Emoji {
        char: "😄",
        name: "grinning smiling eyes",
        keywords: &["happy", "joy", "laugh"],
        skin_tone: false,
    },
    Emoji {
        char: "😁",
        name: "beaming face",
        keywords: &["smile", "happy", "grin"],
        skin_tone: false,
    },
    Emoji {
        char: "😆",
        name: "grinning squinting",
        keywords: &["laugh", "satisfied"],
        skin_tone: false,
    },
    Emoji {
        char: "😅",
        name: "grinning sweat",
        keywords: &["hot", "happy", "laugh", "relief"],
        skin_tone: false,
    },
    Emoji {
        char: "🤣",
        name: "rolling laughing",
        keywords: &["lol", "rofl", "haha", "funny"],
        skin_tone: false,
    },
    Emoji {
        char: "😂",
        name: "tears of joy",
        keywords: &["tears", "laugh", "crying", "happy"],
        skin_tone: false,
    },
    Emoji {
        char: "🙂",
        name: "slightly smiling",
        keywords: &["smile"],
        skin_tone: false,
    },
    Emoji {
        char: "🙃",
        name: "upside-down",
        keywords: &["silly", "sarcasm"],
        skin_tone: false,
    },
    Emoji {
        char: "😉",
        name: "winking",
        keywords: &["wink", "flirt"],
        skin_tone: false,
    },
    Emoji {
        char: "😊",
        name: "smiling blushing",
        keywords: &["blush", "happy", "pleased"],
        skin_tone: false,
    },
    Emoji {
        char: "😇",
        name: "halo angel",
        keywords: &["angel", "innocent", "heaven"],
        skin_tone: false,
    },
    Emoji {
        char: "🥰",
        name: "smiling hearts",
        keywords: &["love", "adore", "crush"],
        skin_tone: false,
    },
    Emoji {
        char: "😍",
        name: "heart eyes",
        keywords: &["love", "crush", "heart", "lovestruck"],
        skin_tone: false,
    },
    Emoji {
        char: "🤩",
        name: "star-struck",
        keywords: &["eyes", "star", "wow", "excited"],
        skin_tone: false,
    },
    Emoji {
        char: "😘",
        name: "blowing kiss",
        keywords: &["kiss", "love", "flirt"],
        skin_tone: false,
    },
    Emoji {
        char: "😗",
        name: "kissing",
        keywords: &["kiss", "love"],
        skin_tone: false,
    },
    Emoji {
        char: "☺️",
        name: "smiling",
        keywords: &["blush", "happy"],
        skin_tone: false,
    },
    Emoji {
        char: "😚",
        name: "kissing closed eyes",
        keywords: &["kiss", "love"],
        skin_tone: false,
    },
    Emoji {
        char: "😋",
        name: "savoring food",
        keywords: &["yum", "tongue", "delicious"],
        skin_tone: false,
    },
    Emoji {
        char: "😛",
        name: "tongue out",
        keywords: &["tongue", "prank", "silly"],
        skin_tone: false,
    },
    Emoji {
        char: "😜",
        name: "winking tongue",
        keywords: &["tongue", "wink", "joke", "silly"],
        skin_tone: false,
    },
    Emoji {
        char: "🤪",
        name: "zany",
        keywords: &["crazy", "silly", "goofy"],
        skin_tone: false,
    },
    Emoji {
        char: "😝",
        name: "squinting tongue",
        keywords: &["tongue", "taste"],
        skin_tone: false,
    },
    Emoji {
        char: "🤗",
        name: "hugging",
        keywords: &["hug", "embrace"],
        skin_tone: false,
    },
    Emoji {
        char: "🤭",
        name: "hand over mouth",
        keywords: &["whoops", "secret", "quiet"],
        skin_tone: false,
    },
    Emoji {
        char: "🤫",
        name: "shushing",
        keywords: &["quiet", "shhh", "secret"],
        skin_tone: false,
    },
    Emoji {
        char: "🤔",
        name: "thinking",
        keywords: &["think", "hmm", "consider"],
        skin_tone: false,
    },
    Emoji {
        char: "🤐",
        name: "zipper mouth",
        keywords: &["quiet", "sealed", "secret"],
        skin_tone: false,
    },
    Emoji {
        char: "😐",
        name: "neutral",
        keywords: &["meh", "blank"],
        skin_tone: false,
    },
    Emoji {
        char: "😑",
        name: "expressionless",
        keywords: &["blank", "meh"],
        skin_tone: false,
    },
    Emoji {
        char: "😶",
        name: "no mouth",
        keywords: &["silent", "speechless"],
        skin_tone: false,
    },
    Emoji {
        char: "😏",
        name: "smirking",
        keywords: &["smug", "smirk", "mean"],
        skin_tone: false,
    },
    Emoji {
        char: "😒",
        name: "unamused",
        keywords: &["meh", "bored", "unhappy"],
        skin_tone: false,
    },
    Emoji {
        char: "🙄",
        name: "rolling eyes",
        keywords: &["eyeroll", "frustrated", "whatever"],
        skin_tone: false,
    },
    Emoji {
        char: "😬",
        name: "grimacing",
        keywords: &["grimace", "teeth", "awkward"],
        skin_tone: false,
    },
    Emoji {
        char: "😌",
        name: "relieved",
        keywords: &["relieved", "relaxed", "content"],
        skin_tone: false,
    },
    Emoji {
        char: "😔",
        name: "pensive",
        keywords: &["sad", "depressed", "upset"],
        skin_tone: false,
    },
    Emoji {
        char: "😪",
        name: "sleepy",
        keywords: &["tired", "sleep"],
        skin_tone: false,
    },
    Emoji {
        char: "🤤",
        name: "drooling",
        keywords: &["drool", "yum", "hungry"],
        skin_tone: false,
    },
    Emoji {
        char: "😴",
        name: "sleeping",
        keywords: &["zzz", "tired", "sleep"],
        skin_tone: false,
    },
    Emoji {
        char: "😷",
        name: "medical mask",
        keywords: &["sick", "ill", "covid", "mask"],
        skin_tone: false,
    },
    Emoji {
        char: "🤒",
        name: "thermometer",
        keywords: &["sick", "fever", "ill"],
        skin_tone: false,
    },
    Emoji {
        char: "🤕",
        name: "head bandage",
        keywords: &["hurt", "injured", "bandage"],
        skin_tone: false,
    },
    Emoji {
        char: "🤢",
        name: "nauseated",
        keywords: &["sick", "vomit", "gross", "green"],
        skin_tone: false,
    },
    Emoji {
        char: "🤮",
        name: "vomiting",
        keywords: &["sick", "vomit", "puke"],
        skin_tone: false,
    },
    Emoji {
        char: "🥵",
        name: "hot face",
        keywords: &["heat", "sweating", "hot"],
        skin_tone: false,
    },
    Emoji {
        char: "🥶",
        name: "cold face",
        keywords: &["cold", "freezing", "ice"],
        skin_tone: false,
    },
    Emoji {
        char: "😵",
        name: "dizzy",
        keywords: &["dizzy", "unconscious"],
        skin_tone: false,
    },
    Emoji {
        char: "🤯",
        name: "exploding head",
        keywords: &["mind blown", "shocked", "wow"],
        skin_tone: false,
    },
    Emoji {
        char: "🤠",
        name: "cowboy",
        keywords: &["cowboy", "western"],
        skin_tone: false,
    },
    Emoji {
        char: "🥳",
        name: "partying",
        keywords: &["party", "celebration", "birthday"],
        skin_tone: false,
    },
    Emoji {
        char: "😎",
        name: "sunglasses cool",
        keywords: &["cool", "sunglasses"],
        skin_tone: false,
    },
    Emoji {
        char: "🤓",
        name: "nerd",
        keywords: &["nerd", "geek", "smart"],
        skin_tone: false,
    },
    Emoji {
        char: "🧐",
        name: "monocle",
        keywords: &["smart", "thinking", "curious"],
        skin_tone: false,
    },
    Emoji {
        char: "😕",
        name: "confused",
        keywords: &["confused", "puzzled"],
        skin_tone: false,
    },
    Emoji {
        char: "😟",
        name: "worried",
        keywords: &["worried", "nervous", "concerned"],
        skin_tone: false,
    },
    Emoji {
        char: "🙁",
        name: "frowning",
        keywords: &["frown", "sad", "disappointed"],
        skin_tone: false,
    },
    Emoji {
        char: "😮",
        name: "open mouth",
        keywords: &["surprised", "shocked", "wow"],
        skin_tone: false,
    },
    Emoji {
        char: "😲",
        name: "astonished",
        keywords: &["surprised", "shocked", "amazed"],
        skin_tone: false,
    },
    Emoji {
        char: "😳",
        name: "flushed",
        keywords: &["embarrassed", "blush", "shy"],
        skin_tone: false,
    },
    Emoji {
        char: "🥺",
        name: "pleading",
        keywords: &["puppy eyes", "begging", "please"],
        skin_tone: false,
    },
    Emoji {
        char: "😨",
        name: "fearful",
        keywords: &["fear", "scared", "shocked"],
        skin_tone: false,
    },
    Emoji {
        char: "😰",
        name: "anxious sweat",
        keywords: &["nervous", "anxious", "worried"],
        skin_tone: false,
    },
    Emoji {
        char: "😥",
        name: "sad relieved",
        keywords: &["sad", "phew", "relieved"],
        skin_tone: false,
    },
    Emoji {
        char: "😢",
        name: "crying",
        keywords: &["cry", "sad", "tear"],
        skin_tone: false,
    },
    Emoji {
        char: "😭",
        name: "loudly crying",
        keywords: &["cry", "sob", "sad", "upset"],
        skin_tone: false,
    },
    Emoji {
        char: "😱",
        name: "screaming fear",
        keywords: &["scream", "fear", "scared"],
        skin_tone: false,
    },
    Emoji {
        char: "😖",
        name: "confounded",
        keywords: &["confounded", "frustrated"],
        skin_tone: false,
    },
    Emoji {
        char: "😣",
        name: "persevering",
        keywords: &["struggle", "persevere"],
        skin_tone: false,
    },
    Emoji {
        char: "😞",
        name: "disappointed",
        keywords: &["sad", "disappointed"],
        skin_tone: false,
    },
    Emoji {
        char: "😓",
        name: "downcast sweat",
        keywords: &["disappointed", "upset"],
        skin_tone: false,
    },
    Emoji {
        char: "😩",
        name: "weary",
        keywords: &["tired", "weary", "upset"],
        skin_tone: false,
    },
    Emoji {
        char: "😫",
        name: "tired",
        keywords: &["tired", "exhausted", "upset"],
        skin_tone: false,
    },
    Emoji {
        char: "😤",
        name: "steam from nose",
        keywords: &["angry", "frustrated", "triumph"],
        skin_tone: false,
    },
    Emoji {
        char: "😡",
        name: "pouting",
        keywords: &["angry", "rage", "mad"],
        skin_tone: false,
    },
    Emoji {
        char: "😠",
        name: "angry",
        keywords: &["angry", "mad", "annoyed"],
        skin_tone: false,
    },
    Emoji {
        char: "🤬",
        name: "cursing",
        keywords: &["cursing", "swear", "angry"],
        skin_tone: false,
    },
    Emoji {
        char: "😈",
        name: "smiling horns",
        keywords: &["devil", "evil", "horns"],
        skin_tone: false,
    },
    Emoji {
        char: "👿",
        name: "angry horns",
        keywords: &["devil", "angry", "evil"],
        skin_tone: false,
    },
    Emoji {
        char: "💀",
        name: "skull",
        keywords: &["dead", "death", "skeleton"],
        skin_tone: false,
    },
    Emoji {
        char: "☠️",
        name: "skull crossbones",
        keywords: &["dead", "danger", "poison"],
        skin_tone: false,
    },
    Emoji {
        char: "💩",
        name: "poop",
        keywords: &["poop", "shit", "crap"],
        skin_tone: false,
    },
    Emoji {
        char: "🤡",
        name: "clown",
        keywords: &["clown", "circus", "silly"],
        skin_tone: false,
    },
    Emoji {
        char: "👹",
        name: "ogre",
        keywords: &["monster", "demon", "japanese"],
        skin_tone: false,
    },
    Emoji {
        char: "👺",
        name: "goblin",
        keywords: &["monster", "demon", "japanese"],
        skin_tone: false,
    },
    Emoji {
        char: "👻",
        name: "ghost",
        keywords: &["ghost", "halloween", "spooky"],
        skin_tone: false,
    },
    Emoji {
        char: "👽",
        name: "alien",
        keywords: &["alien", "ufo", "space"],
        skin_tone: false,
    },
    Emoji {
        char: "👾",
        name: "alien monster",
        keywords: &["game", "arcade", "space invaders"],
        skin_tone: false,
    },
    Emoji {
        char: "🤖",
        name: "robot",
        keywords: &["robot", "computer", "ai"],
        skin_tone: false,
    },
];

const EMOJI_GESTURES: &[Emoji] = &[
    Emoji {
        char: "👋",
        name: "waving hand",
        keywords: &["wave", "hello", "bye"],
        skin_tone: true,
    },
    Emoji {
        char: "🤚",
        name: "raised back hand",
        keywords: &["backhand", "raised"],
        skin_tone: true,
    },
    Emoji {
        char: "🖐️",
        name: "fingers splayed",
        keywords: &["hand", "fingers", "palm"],
        skin_tone: true,
    },
    Emoji {
        char: "✋",
        name: "raised hand",
        keywords: &["stop", "high five", "hand"],
        skin_tone: true,
    },
    Emoji {
        char: "🖖",
        name: "vulcan salute",
        keywords: &["spock", "star trek", "live long"],
        skin_tone: true,
    },
    Emoji {
        char: "👌",
        name: "ok hand",
        keywords: &["ok", "okay", "perfect"],
        skin_tone: true,
    },
    Emoji {
        char: "🤌",
        name: "pinched fingers",
        keywords: &["italian", "what", "chef kiss"],
        skin_tone: true,
    },
    Emoji {
        char: "🤏",
        name: "pinching hand",
        keywords: &["small", "tiny", "little"],
        skin_tone: true,
    },
    Emoji {
        char: "✌️",
        name: "victory peace",
        keywords: &["peace", "victory", "v"],
        skin_tone: true,
    },
    Emoji {
        char: "🤞",
        name: "crossed fingers",
        keywords: &["luck", "hopeful"],
        skin_tone: true,
    },
    Emoji {
        char: "🤟",
        name: "love-you gesture",
        keywords: &["ily", "love", "rock"],
        skin_tone: true,
    },
    Emoji {
        char: "🤘",
        name: "horns rock",
        keywords: &["rock", "metal", "horns"],
        skin_tone: true,
    },
    Emoji {
        char: "🤙",
        name: "call me hand",
        keywords: &["shaka", "hang loose", "call"],
        skin_tone: true,
    },
    Emoji {
        char: "👈",
        name: "pointing left",
        keywords: &["left", "point", "direction"],
        skin_tone: true,
    },
    Emoji {
        char: "👉",
        name: "pointing right",
        keywords: &["right", "point", "direction"],
        skin_tone: true,
    },
    Emoji {
        char: "👆",
        name: "pointing up",
        keywords: &["up", "point", "direction"],
        skin_tone: true,
    },
    Emoji {
        char: "🖕",
        name: "middle finger",
        keywords: &["finger", "rude", "flip off"],
        skin_tone: true,
    },
    Emoji {
        char: "👇",
        name: "pointing down",
        keywords: &["down", "point", "direction"],
        skin_tone: true,
    },
    Emoji {
        char: "☝️",
        name: "index up",
        keywords: &["point", "up", "one"],
        skin_tone: true,
    },
    Emoji {
        char: "👍",
        name: "thumbs up",
        keywords: &["like", "yes", "good", "approve"],
        skin_tone: true,
    },
    Emoji {
        char: "👎",
        name: "thumbs down",
        keywords: &["dislike", "no", "bad", "reject"],
        skin_tone: true,
    },
    Emoji {
        char: "✊",
        name: "raised fist",
        keywords: &["fist", "power", "punch"],
        skin_tone: true,
    },
    Emoji {
        char: "👊",
        name: "oncoming fist",
        keywords: &["punch", "fist bump"],
        skin_tone: true,
    },
    Emoji {
        char: "🤛",
        name: "left fist",
        keywords: &["fist bump", "punch"],
        skin_tone: true,
    },
    Emoji {
        char: "🤜",
        name: "right fist",
        keywords: &["fist bump", "punch"],
        skin_tone: true,
    },
    Emoji {
        char: "👏",
        name: "clapping",
        keywords: &["clap", "applause", "praise"],
        skin_tone: true,
    },
    Emoji {
        char: "🙌",
        name: "raising hands",
        keywords: &["hooray", "celebrate", "yay"],
        skin_tone: true,
    },
    Emoji {
        char: "👐",
        name: "open hands",
        keywords: &["open", "hands", "jazz hands"],
        skin_tone: true,
    },
    Emoji {
        char: "🤲",
        name: "palms up",
        keywords: &["prayer", "cupped hands"],
        skin_tone: true,
    },
    Emoji {
        char: "🤝",
        name: "handshake",
        keywords: &["agreement", "deal", "shake"],
        skin_tone: false,
    },
    Emoji {
        char: "🙏",
        name: "folded hands",
        keywords: &["pray", "please", "thank you", "namaste"],
        skin_tone: true,
    },
    Emoji {
        char: "💪",
        name: "flexed biceps",
        keywords: &["strong", "muscle", "flex", "arm"],
        skin_tone: true,
    },
];

const EMOJI_ANIMALS: &[Emoji] = &[
    Emoji {
        char: "🐶",
        name: "dog face",
        keywords: &["dog", "puppy", "pet"],
        skin_tone: false,
    },
    Emoji {
        char: "🐕",
        name: "dog",
        keywords: &["dog", "pet"],
        skin_tone: false,
    },
    Emoji {
        char: "🦮",
        name: "guide dog",
        keywords: &["dog", "blind", "service"],
        skin_tone: false,
    },
    Emoji {
        char: "🐩",
        name: "poodle",
        keywords: &["dog", "poodle"],
        skin_tone: false,
    },
    Emoji {
        char: "🐺",
        name: "wolf",
        keywords: &["wolf", "wild"],
        skin_tone: false,
    },
    Emoji {
        char: "🦊",
        name: "fox",
        keywords: &["fox", "animal"],
        skin_tone: false,
    },
    Emoji {
        char: "🦝",
        name: "raccoon",
        keywords: &["raccoon", "trash panda"],
        skin_tone: false,
    },
    Emoji {
        char: "🐱",
        name: "cat face",
        keywords: &["cat", "kitten", "pet"],
        skin_tone: false,
    },
    Emoji {
        char: "🐈",
        name: "cat",
        keywords: &["cat", "pet"],
        skin_tone: false,
    },
    Emoji {
        char: "🦁",
        name: "lion",
        keywords: &["lion", "king", "wild"],
        skin_tone: false,
    },
    Emoji {
        char: "🐯",
        name: "tiger face",
        keywords: &["tiger", "wild"],
        skin_tone: false,
    },
    Emoji {
        char: "🐆",
        name: "leopard",
        keywords: &["leopard", "wild"],
        skin_tone: false,
    },
    Emoji {
        char: "🐴",
        name: "horse face",
        keywords: &["horse", "pony"],
        skin_tone: false,
    },
    Emoji {
        char: "🦄",
        name: "unicorn",
        keywords: &["unicorn", "magic", "fantasy"],
        skin_tone: false,
    },
    Emoji {
        char: "🐮",
        name: "cow face",
        keywords: &["cow", "moo", "farm"],
        skin_tone: false,
    },
    Emoji {
        char: "🐷",
        name: "pig face",
        keywords: &["pig", "oink", "farm"],
        skin_tone: false,
    },
    Emoji {
        char: "🐭",
        name: "mouse face",
        keywords: &["mouse", "rodent"],
        skin_tone: false,
    },
    Emoji {
        char: "🐹",
        name: "hamster",
        keywords: &["hamster", "pet", "cute"],
        skin_tone: false,
    },
    Emoji {
        char: "🐰",
        name: "rabbit face",
        keywords: &["rabbit", "bunny", "pet"],
        skin_tone: false,
    },
    Emoji {
        char: "🦔",
        name: "hedgehog",
        keywords: &["hedgehog", "spiky"],
        skin_tone: false,
    },
    Emoji {
        char: "🦇",
        name: "bat",
        keywords: &["bat", "vampire", "halloween"],
        skin_tone: false,
    },
    Emoji {
        char: "🐻",
        name: "bear",
        keywords: &["bear", "teddy"],
        skin_tone: false,
    },
    Emoji {
        char: "🐨",
        name: "koala",
        keywords: &["koala", "australia"],
        skin_tone: false,
    },
    Emoji {
        char: "🐼",
        name: "panda",
        keywords: &["panda", "bear", "china"],
        skin_tone: false,
    },
    Emoji {
        char: "🦥",
        name: "sloth",
        keywords: &["sloth", "lazy", "slow"],
        skin_tone: false,
    },
    Emoji {
        char: "🐔",
        name: "chicken",
        keywords: &["chicken", "hen", "farm"],
        skin_tone: false,
    },
    Emoji {
        char: "🐧",
        name: "penguin",
        keywords: &["penguin", "bird", "cold"],
        skin_tone: false,
    },
    Emoji {
        char: "🦅",
        name: "eagle",
        keywords: &["eagle", "bird", "america"],
        skin_tone: false,
    },
    Emoji {
        char: "🦆",
        name: "duck",
        keywords: &["duck", "bird", "quack"],
        skin_tone: false,
    },
    Emoji {
        char: "🦉",
        name: "owl",
        keywords: &["owl", "bird", "night", "wise"],
        skin_tone: false,
    },
    Emoji {
        char: "🐸",
        name: "frog",
        keywords: &["frog", "toad", "ribbit"],
        skin_tone: false,
    },
    Emoji {
        char: "🐢",
        name: "turtle",
        keywords: &["turtle", "tortoise", "slow"],
        skin_tone: false,
    },
    Emoji {
        char: "🐍",
        name: "snake",
        keywords: &["snake", "serpent"],
        skin_tone: false,
    },
    Emoji {
        char: "🐲",
        name: "dragon face",
        keywords: &["dragon", "fantasy"],
        skin_tone: false,
    },
    Emoji {
        char: "🦕",
        name: "sauropod",
        keywords: &["dinosaur", "brontosaurus"],
        skin_tone: false,
    },
    Emoji {
        char: "🦖",
        name: "t-rex",
        keywords: &["dinosaur", "tyrannosaurus"],
        skin_tone: false,
    },
    Emoji {
        char: "🐳",
        name: "spouting whale",
        keywords: &["whale", "ocean", "sea"],
        skin_tone: false,
    },
    Emoji {
        char: "🐬",
        name: "dolphin",
        keywords: &["dolphin", "ocean", "flipper"],
        skin_tone: false,
    },
    Emoji {
        char: "🦈",
        name: "shark",
        keywords: &["shark", "ocean", "fish"],
        skin_tone: false,
    },
    Emoji {
        char: "🐙",
        name: "octopus",
        keywords: &["octopus", "ocean", "tentacle"],
        skin_tone: false,
    },
    Emoji {
        char: "🦋",
        name: "butterfly",
        keywords: &["butterfly", "insect", "pretty"],
        skin_tone: false,
    },
    Emoji {
        char: "🐛",
        name: "bug",
        keywords: &["bug", "insect", "caterpillar"],
        skin_tone: false,
    },
    Emoji {
        char: "🐜",
        name: "ant",
        keywords: &["ant", "insect"],
        skin_tone: false,
    },
    Emoji {
        char: "🐝",
        name: "honeybee",
        keywords: &["bee", "insect", "honey"],
        skin_tone: false,
    },
    Emoji {
        char: "🐞",
        name: "lady beetle",
        keywords: &["ladybug", "insect", "luck"],
        skin_tone: false,
    },
];

const EMOJI_FOOD: &[Emoji] = &[
    Emoji {
        char: "🍎",
        name: "red apple",
        keywords: &["apple", "fruit", "red"],
        skin_tone: false,
    },
    Emoji {
        char: "🍊",
        name: "tangerine",
        keywords: &["orange", "fruit", "mandarin"],
        skin_tone: false,
    },
    Emoji {
        char: "🍋",
        name: "lemon",
        keywords: &["lemon", "fruit", "sour"],
        skin_tone: false,
    },
    Emoji {
        char: "🍌",
        name: "banana",
        keywords: &["banana", "fruit"],
        skin_tone: false,
    },
    Emoji {
        char: "🍉",
        name: "watermelon",
        keywords: &["watermelon", "fruit", "summer"],
        skin_tone: false,
    },
    Emoji {
        char: "🍇",
        name: "grapes",
        keywords: &["grapes", "fruit", "wine"],
        skin_tone: false,
    },
    Emoji {
        char: "🍓",
        name: "strawberry",
        keywords: &["strawberry", "fruit", "berry"],
        skin_tone: false,
    },
    Emoji {
        char: "🍑",
        name: "peach",
        keywords: &["peach", "fruit", "butt"],
        skin_tone: false,
    },
    Emoji {
        char: "🥭",
        name: "mango",
        keywords: &["mango", "fruit", "tropical"],
        skin_tone: false,
    },
    Emoji {
        char: "🥑",
        name: "avocado",
        keywords: &["avocado", "guacamole"],
        skin_tone: false,
    },
    Emoji {
        char: "🍆",
        name: "eggplant",
        keywords: &["eggplant", "aubergine", "vegetable"],
        skin_tone: false,
    },
    Emoji {
        char: "🥕",
        name: "carrot",
        keywords: &["carrot", "vegetable", "orange"],
        skin_tone: false,
    },
    Emoji {
        char: "🌽",
        name: "corn",
        keywords: &["corn", "vegetable", "maize"],
        skin_tone: false,
    },
    Emoji {
        char: "🌶️",
        name: "hot pepper",
        keywords: &["pepper", "chili", "spicy", "hot"],
        skin_tone: false,
    },
    Emoji {
        char: "🍞",
        name: "bread",
        keywords: &["bread", "loaf", "toast"],
        skin_tone: false,
    },
    Emoji {
        char: "🥐",
        name: "croissant",
        keywords: &["croissant", "bread", "french"],
        skin_tone: false,
    },
    Emoji {
        char: "🧀",
        name: "cheese",
        keywords: &["cheese", "dairy"],
        skin_tone: false,
    },
    Emoji {
        char: "🍖",
        name: "meat bone",
        keywords: &["meat", "bone", "drumstick"],
        skin_tone: false,
    },
    Emoji {
        char: "🍗",
        name: "poultry leg",
        keywords: &["chicken", "drumstick", "meat"],
        skin_tone: false,
    },
    Emoji {
        char: "🥩",
        name: "cut meat",
        keywords: &["steak", "meat", "beef"],
        skin_tone: false,
    },
    Emoji {
        char: "🥓",
        name: "bacon",
        keywords: &["bacon", "meat", "breakfast"],
        skin_tone: false,
    },
    Emoji {
        char: "🍔",
        name: "hamburger",
        keywords: &["burger", "hamburger", "fast food"],
        skin_tone: false,
    },
    Emoji {
        char: "🍟",
        name: "french fries",
        keywords: &["fries", "chips", "fast food"],
        skin_tone: false,
    },
    Emoji {
        char: "🍕",
        name: "pizza",
        keywords: &["pizza", "slice", "italian"],
        skin_tone: false,
    },
    Emoji {
        char: "🌭",
        name: "hot dog",
        keywords: &["hotdog", "sausage", "fast food"],
        skin_tone: false,
    },
    Emoji {
        char: "🥪",
        name: "sandwich",
        keywords: &["sandwich", "bread", "lunch"],
        skin_tone: false,
    },
    Emoji {
        char: "🌮",
        name: "taco",
        keywords: &["taco", "mexican", "food"],
        skin_tone: false,
    },
    Emoji {
        char: "🌯",
        name: "burrito",
        keywords: &["burrito", "mexican", "wrap"],
        skin_tone: false,
    },
    Emoji {
        char: "🍣",
        name: "sushi",
        keywords: &["sushi", "japanese", "fish"],
        skin_tone: false,
    },
    Emoji {
        char: "🍜",
        name: "steaming bowl",
        keywords: &["ramen", "noodles", "soup"],
        skin_tone: false,
    },
    Emoji {
        char: "🍝",
        name: "spaghetti",
        keywords: &["pasta", "spaghetti", "italian"],
        skin_tone: false,
    },
    Emoji {
        char: "🍦",
        name: "ice cream",
        keywords: &["ice cream", "dessert", "cone"],
        skin_tone: false,
    },
    Emoji {
        char: "🍩",
        name: "doughnut",
        keywords: &["donut", "dessert", "sweet"],
        skin_tone: false,
    },
    Emoji {
        char: "🍪",
        name: "cookie",
        keywords: &["cookie", "dessert", "sweet"],
        skin_tone: false,
    },
    Emoji {
        char: "🎂",
        name: "birthday cake",
        keywords: &["cake", "birthday", "dessert"],
        skin_tone: false,
    },
    Emoji {
        char: "☕",
        name: "hot beverage",
        keywords: &["coffee", "tea", "hot", "drink"],
        skin_tone: false,
    },
    Emoji {
        char: "🍺",
        name: "beer mug",
        keywords: &["beer", "drink", "alcohol"],
        skin_tone: false,
    },
    Emoji {
        char: "🍷",
        name: "wine glass",
        keywords: &["wine", "drink", "alcohol", "red"],
        skin_tone: false,
    },
    Emoji {
        char: "🍸",
        name: "cocktail",
        keywords: &["cocktail", "martini", "drink"],
        skin_tone: false,
    },
];

const EMOJI_OBJECTS: &[Emoji] = &[
    Emoji {
        char: "💻",
        name: "laptop",
        keywords: &["computer", "laptop", "mac", "pc"],
        skin_tone: false,
    },
    Emoji {
        char: "🖥️",
        name: "desktop",
        keywords: &["computer", "desktop", "monitor"],
        skin_tone: false,
    },
    Emoji {
        char: "⌨️",
        name: "keyboard",
        keywords: &["keyboard", "typing", "computer"],
        skin_tone: false,
    },
    Emoji {
        char: "🖱️",
        name: "mouse",
        keywords: &["mouse", "computer", "click"],
        skin_tone: false,
    },
    Emoji {
        char: "📱",
        name: "mobile phone",
        keywords: &["phone", "mobile", "cell", "iphone"],
        skin_tone: false,
    },
    Emoji {
        char: "📞",
        name: "telephone",
        keywords: &["phone", "call", "telephone"],
        skin_tone: false,
    },
    Emoji {
        char: "📺",
        name: "television",
        keywords: &["tv", "television", "screen"],
        skin_tone: false,
    },
    Emoji {
        char: "📷",
        name: "camera",
        keywords: &["camera", "photo", "picture"],
        skin_tone: false,
    },
    Emoji {
        char: "💡",
        name: "light bulb",
        keywords: &["light", "bulb", "idea"],
        skin_tone: false,
    },
    Emoji {
        char: "🔋",
        name: "battery",
        keywords: &["battery", "power", "charge"],
        skin_tone: false,
    },
    Emoji {
        char: "💰",
        name: "money bag",
        keywords: &["money", "bag", "dollar", "rich"],
        skin_tone: false,
    },
    Emoji {
        char: "💳",
        name: "credit card",
        keywords: &["credit", "card", "payment"],
        skin_tone: false,
    },
    Emoji {
        char: "⏰",
        name: "alarm clock",
        keywords: &["alarm", "clock", "time", "wake"],
        skin_tone: false,
    },
    Emoji {
        char: "🔧",
        name: "wrench",
        keywords: &["wrench", "tool", "fix"],
        skin_tone: false,
    },
    Emoji {
        char: "🔨",
        name: "hammer",
        keywords: &["hammer", "tool", "build"],
        skin_tone: false,
    },
    Emoji {
        char: "⚙️",
        name: "gear",
        keywords: &["gear", "settings", "cog"],
        skin_tone: false,
    },
    Emoji {
        char: "🔒",
        name: "locked",
        keywords: &["lock", "locked", "secure", "private"],
        skin_tone: false,
    },
    Emoji {
        char: "🔓",
        name: "unlocked",
        keywords: &["unlock", "unlocked", "open"],
        skin_tone: false,
    },
    Emoji {
        char: "🔑",
        name: "key",
        keywords: &["key", "unlock", "password"],
        skin_tone: false,
    },
    Emoji {
        char: "📧",
        name: "email",
        keywords: &["email", "mail", "message"],
        skin_tone: false,
    },
    Emoji {
        char: "📦",
        name: "package",
        keywords: &["package", "box", "shipping"],
        skin_tone: false,
    },
    Emoji {
        char: "📝",
        name: "memo",
        keywords: &["memo", "note", "pencil", "write"],
        skin_tone: false,
    },
    Emoji {
        char: "📚",
        name: "books",
        keywords: &["books", "library", "read", "study"],
        skin_tone: false,
    },
    Emoji {
        char: "🎵",
        name: "musical note",
        keywords: &["music", "note", "song"],
        skin_tone: false,
    },
    Emoji {
        char: "🎧",
        name: "headphone",
        keywords: &["headphones", "music", "audio"],
        skin_tone: false,
    },
    Emoji {
        char: "🎮",
        name: "video game",
        keywords: &["game", "controller", "gaming", "play"],
        skin_tone: false,
    },
    Emoji {
        char: "🎲",
        name: "game die",
        keywords: &["dice", "game", "random", "luck"],
        skin_tone: false,
    },
    Emoji {
        char: "🎯",
        name: "direct hit",
        keywords: &["target", "bullseye", "dart"],
        skin_tone: false,
    },
    Emoji {
        char: "🏆",
        name: "trophy",
        keywords: &["trophy", "winner", "award", "gold"],
        skin_tone: false,
    },
    Emoji {
        char: "🎁",
        name: "gift",
        keywords: &["gift", "present", "birthday"],
        skin_tone: false,
    },
    Emoji {
        char: "🎉",
        name: "party popper",
        keywords: &["party", "celebration", "tada"],
        skin_tone: false,
    },
];

const EMOJI_SYMBOLS: &[Emoji] = &[
    Emoji {
        char: "❤️",
        name: "red heart",
        keywords: &["heart", "love", "red"],
        skin_tone: false,
    },
    Emoji {
        char: "🧡",
        name: "orange heart",
        keywords: &["heart", "love", "orange"],
        skin_tone: false,
    },
    Emoji {
        char: "💛",
        name: "yellow heart",
        keywords: &["heart", "love", "yellow"],
        skin_tone: false,
    },
    Emoji {
        char: "💚",
        name: "green heart",
        keywords: &["heart", "love", "green"],
        skin_tone: false,
    },
    Emoji {
        char: "💙",
        name: "blue heart",
        keywords: &["heart", "love", "blue"],
        skin_tone: false,
    },
    Emoji {
        char: "💜",
        name: "purple heart",
        keywords: &["heart", "love", "purple"],
        skin_tone: false,
    },
    Emoji {
        char: "🖤",
        name: "black heart",
        keywords: &["heart", "love", "black"],
        skin_tone: false,
    },
    Emoji {
        char: "🤍",
        name: "white heart",
        keywords: &["heart", "love", "white"],
        skin_tone: false,
    },
    Emoji {
        char: "💔",
        name: "broken heart",
        keywords: &["heart", "broken", "sad"],
        skin_tone: false,
    },
    Emoji {
        char: "✅",
        name: "check mark",
        keywords: &["check", "yes", "done", "correct"],
        skin_tone: false,
    },
    Emoji {
        char: "❌",
        name: "cross mark",
        keywords: &["x", "no", "wrong", "delete"],
        skin_tone: false,
    },
    Emoji {
        char: "❗",
        name: "exclamation",
        keywords: &["exclamation", "important", "alert"],
        skin_tone: false,
    },
    Emoji {
        char: "❓",
        name: "question mark",
        keywords: &["question", "help", "what"],
        skin_tone: false,
    },
    Emoji {
        char: "💯",
        name: "hundred points",
        keywords: &["100", "perfect", "score"],
        skin_tone: false,
    },
    Emoji {
        char: "🔥",
        name: "fire",
        keywords: &["fire", "hot", "lit", "flame"],
        skin_tone: false,
    },
    Emoji {
        char: "✨",
        name: "sparkles",
        keywords: &["sparkle", "star", "shine", "glitter"],
        skin_tone: false,
    },
    Emoji {
        char: "⭐",
        name: "star",
        keywords: &["star", "favorite", "rating"],
        skin_tone: false,
    },
    Emoji {
        char: "🌟",
        name: "glowing star",
        keywords: &["star", "glow", "shine"],
        skin_tone: false,
    },
    Emoji {
        char: "⚡",
        name: "high voltage",
        keywords: &["lightning", "electric", "thunder", "zap"],
        skin_tone: false,
    },
    Emoji {
        char: "💥",
        name: "collision",
        keywords: &["boom", "explosion", "crash"],
        skin_tone: false,
    },
    Emoji {
        char: "♻️",
        name: "recycling",
        keywords: &["recycle", "environment", "green"],
        skin_tone: false,
    },
    Emoji {
        char: "⚠️",
        name: "warning",
        keywords: &["warning", "caution", "alert"],
        skin_tone: false,
    },
    Emoji {
        char: "🚫",
        name: "prohibited",
        keywords: &["no", "forbidden", "prohibited"],
        skin_tone: false,
    },
    Emoji {
        char: "🔴",
        name: "red circle",
        keywords: &["red", "circle", "dot"],
        skin_tone: false,
    },
    Emoji {
        char: "🟢",
        name: "green circle",
        keywords: &["green", "circle", "dot"],
        skin_tone: false,
    },
    Emoji {
        char: "🔵",
        name: "blue circle",
        keywords: &["blue", "circle", "dot"],
        skin_tone: false,
    },
    Emoji {
        char: "⬆️",
        name: "up arrow",
        keywords: &["up", "arrow", "direction"],
        skin_tone: false,
    },
    Emoji {
        char: "⬇️",
        name: "down arrow",
        keywords: &["down", "arrow", "direction"],
        skin_tone: false,
    },
    Emoji {
        char: "🔄",
        name: "counterclockwise",
        keywords: &["refresh", "reload", "sync"],
        skin_tone: false,
    },
];

const EMOJI_TECH: &[Emoji] = &[
    Emoji {
        char: "🔗",
        name: "link",
        keywords: &["link", "chain", "url"],
        skin_tone: false,
    },
    Emoji {
        char: "🔍",
        name: "magnifying glass",
        keywords: &["search", "magnify", "find", "zoom"],
        skin_tone: false,
    },
    Emoji {
        char: "📊",
        name: "bar chart",
        keywords: &["chart", "graph", "stats", "data"],
        skin_tone: false,
    },
    Emoji {
        char: "📈",
        name: "chart up",
        keywords: &["chart", "graph", "up", "growth"],
        skin_tone: false,
    },
    Emoji {
        char: "📉",
        name: "chart down",
        keywords: &["chart", "graph", "down", "decline"],
        skin_tone: false,
    },
    Emoji {
        char: "📁",
        name: "file folder",
        keywords: &["folder", "directory", "file"],
        skin_tone: false,
    },
    Emoji {
        char: "💾",
        name: "floppy disk",
        keywords: &["save", "disk", "floppy"],
        skin_tone: false,
    },
    Emoji {
        char: "🔐",
        name: "locked key",
        keywords: &["lock", "key", "secure"],
        skin_tone: false,
    },
    Emoji {
        char: "🛡️",
        name: "shield",
        keywords: &["shield", "security", "protection"],
        skin_tone: false,
    },
    Emoji {
        char: "🚀",
        name: "rocket",
        keywords: &["rocket", "launch", "fast", "space"],
        skin_tone: false,
    },
    Emoji {
        char: "🤖",
        name: "robot",
        keywords: &["robot", "ai", "bot", "machine"],
        skin_tone: false,
    },
    Emoji {
        char: "🧠",
        name: "brain",
        keywords: &["brain", "smart", "think", "ai"],
        skin_tone: false,
    },
    Emoji {
        char: "🔮",
        name: "crystal ball",
        keywords: &["magic", "future", "prediction"],
        skin_tone: false,
    },
    Emoji {
        char: "🧪",
        name: "test tube",
        keywords: &["test", "science", "lab", "experiment"],
        skin_tone: false,
    },
    Emoji {
        char: "🧬",
        name: "dna",
        keywords: &["dna", "genetics", "science"],
        skin_tone: false,
    },
    Emoji {
        char: "🌐",
        name: "globe",
        keywords: &["globe", "world", "internet", "web"],
        skin_tone: false,
    },
    Emoji {
        char: "📡",
        name: "satellite",
        keywords: &["satellite", "antenna", "signal"],
        skin_tone: false,
    },
    Emoji {
        char: "🖨️",
        name: "printer",
        keywords: &["printer", "print", "paper"],
        skin_tone: false,
    },
    Emoji {
        char: "📶",
        name: "antenna bars",
        keywords: &["signal", "wifi", "network"],
        skin_tone: false,
    },
    Emoji {
        char: "🛠️",
        name: "tools",
        keywords: &["tools", "build", "fix"],
        skin_tone: false,
    },
    Emoji {
        char: "📋",
        name: "clipboard",
        keywords: &["clipboard", "paste", "list"],
        skin_tone: false,
    },
    Emoji {
        char: "📌",
        name: "pushpin",
        keywords: &["pin", "location", "push"],
        skin_tone: false,
    },
    Emoji {
        char: "🔖",
        name: "bookmark",
        keywords: &["bookmark", "mark", "save"],
        skin_tone: false,
    },
    Emoji {
        char: "🏷️",
        name: "label",
        keywords: &["label", "tag", "price"],
        skin_tone: false,
    },
];

const EMOJI_CATEGORIES: &[EmojiCategory] = &[
    EmojiCategory {
        name: "Smileys",
        icon: "😀",
        emojis: EMOJI_SMILEYS,
    },
    EmojiCategory {
        name: "Gestures",
        icon: "👋",
        emojis: EMOJI_GESTURES,
    },
    EmojiCategory {
        name: "Animals",
        icon: "🐶",
        emojis: EMOJI_ANIMALS,
    },
    EmojiCategory {
        name: "Food",
        icon: "🍔",
        emojis: EMOJI_FOOD,
    },
    EmojiCategory {
        name: "Objects",
        icon: "💻",
        emojis: EMOJI_OBJECTS,
    },
    EmojiCategory {
        name: "Symbols",
        icon: "❤️",
        emojis: EMOJI_SYMBOLS,
    },
    EmojiCategory {
        name: "Tech",
        icon: "🚀",
        emojis: EMOJI_TECH,
    },
];

/// Emoji picker state (for persistence)
#[derive(Default)]
struct EmojiPickerState {
    recent: VecDeque<String>,
    favorites: HashSet<String>,
    skin_tone_index: usize,
}

impl EmojiPickerState {
    fn add_recent(&mut self, emoji: &str) {
        self.recent.retain(|e| e != emoji);
        self.recent.push_front(emoji.to_string());
        while self.recent.len() > MAX_RECENT {
            self.recent.pop_back();
        }
    }

    fn toggle_favorite(&mut self, emoji: &str) -> bool {
        if self.favorites.contains(emoji) {
            self.favorites.remove(emoji);
            false
        } else {
            self.favorites.insert(emoji.to_string());
            true
        }
    }
}

/// Apply skin tone modifier to an emoji
fn apply_skin_tone(emoji: &str, skin_tone_modifier: &str) -> String {
    if skin_tone_modifier.is_empty() {
        return emoji.to_string();
    }
    let chars: Vec<char> = emoji.chars().collect();
    if chars.is_empty() {
        return emoji.to_string();
    }
    format!(
        "{}{}{}",
        chars[0],
        skin_tone_modifier,
        chars.iter().skip(1).collect::<String>()
    )
}

/// Show the emoji picker popover
pub fn show_emoji_picker(
    parent: &impl IsA<gtk4::Widget>,
    on_emoji_selected: impl Fn(String) + 'static,
) {
    let popover = Popover::new();
    popover.set_parent(parent);

    let state = Rc::new(RefCell::new(EmojiPickerState::default()));

    let container = Box::new(Orientation::Vertical, 8);
    container.set_margin_top(8);
    container.set_margin_bottom(8);
    container.set_margin_start(8);
    container.set_margin_end(8);
    container.set_size_request(400, 480);

    // Search entry
    let search_entry = Entry::new();
    search_entry.set_placeholder_text(Some("Search emojis by name..."));
    search_entry.set_icon_from_icon_name(
        gtk4::EntryIconPosition::Primary,
        Some("system-search-symbolic"),
    );
    container.append(&search_entry);

    // Skin tone selector
    let skin_tone_box = Box::new(Orientation::Horizontal, 4);
    skin_tone_box.set_halign(gtk4::Align::Start);
    let skin_tone_label = Label::new(Some("Skin tone:"));
    skin_tone_label.add_css_class("dim-label");
    skin_tone_box.append(&skin_tone_label);

    let skin_tone_buttons: Vec<ToggleButton> = SKIN_TONES
        .iter()
        .enumerate()
        .map(|(i, (modifier, name))| {
            let btn = if modifier.is_empty() {
                ToggleButton::with_label("👋")
            } else {
                let emoji = format!("👋{}", modifier);
                ToggleButton::with_label(&emoji)
            };
            btn.set_tooltip_text(Some(name));
            btn.add_css_class("flat");
            btn.set_active(i == 0);
            skin_tone_box.append(&btn);
            btn
        })
        .collect();

    // Connect skin tone buttons as a group
    for (i, btn) in skin_tone_buttons.iter().enumerate() {
        let buttons_clone = skin_tone_buttons.clone();
        let state_clone = state.clone();
        let idx = i;
        btn.connect_toggled(move |button| {
            if button.is_active() {
                state_clone.borrow_mut().skin_tone_index = idx;
                for (j, other) in buttons_clone.iter().enumerate() {
                    if j != idx {
                        other.set_active(false);
                    }
                }
            }
        });
    }

    container.append(&skin_tone_box);

    // Notebook for category tabs
    let notebook = Notebook::new();
    notebook.set_vexpand(true);
    notebook.set_tab_pos(gtk4::PositionType::Top);
    notebook.set_scrollable(true);

    let on_select = Rc::new(on_emoji_selected);
    let popover_ref = Rc::new(popover.clone());

    // Recent emojis tab
    let recent_scrolled = ScrolledWindow::new();
    recent_scrolled.set_policy(PolicyType::Never, PolicyType::Automatic);
    let recent_flow = FlowBox::new();
    recent_flow.set_valign(gtk4::Align::Start);
    recent_flow.set_selection_mode(SelectionMode::None);
    recent_flow.set_min_children_per_line(8);
    recent_flow.set_max_children_per_line(12);
    let empty_label = Label::new(Some(
        "No recent emojis yet.\nClick emojis to add them here.",
    ));
    empty_label.add_css_class("dim-label");
    empty_label.set_halign(gtk4::Align::Center);
    empty_label.set_valign(gtk4::Align::Center);
    recent_flow.insert(&empty_label, -1);
    recent_scrolled.set_child(Some(&recent_flow));
    notebook.append_page(&recent_scrolled, Some(&Label::new(Some("🕐"))));

    // Favorites tab
    let fav_scrolled = ScrolledWindow::new();
    fav_scrolled.set_policy(PolicyType::Never, PolicyType::Automatic);
    let fav_flow = FlowBox::new();
    fav_flow.set_valign(gtk4::Align::Start);
    fav_flow.set_selection_mode(SelectionMode::None);
    fav_flow.set_min_children_per_line(8);
    fav_flow.set_max_children_per_line(12);
    let empty_fav_label = Label::new(Some("No favorites yet.\nRight-click emojis to add them."));
    empty_fav_label.add_css_class("dim-label");
    empty_fav_label.set_halign(gtk4::Align::Center);
    empty_fav_label.set_valign(gtk4::Align::Center);
    fav_flow.insert(&empty_fav_label, -1);
    fav_scrolled.set_child(Some(&fav_flow));
    notebook.append_page(&fav_scrolled, Some(&Label::new(Some("⭐"))));

    // Category tabs with emoji content
    for cat in EMOJI_CATEGORIES {
        let scrolled = ScrolledWindow::new();
        scrolled.set_policy(PolicyType::Never, PolicyType::Automatic);

        let flow_box = FlowBox::new();
        flow_box.set_valign(gtk4::Align::Start);
        flow_box.set_selection_mode(SelectionMode::None);
        flow_box.set_min_children_per_line(8);
        flow_box.set_max_children_per_line(12);

        for emoji in cat.emojis {
            let btn = create_emoji_button(emoji, &state, &on_select, &popover_ref);
            flow_box.append(&btn);
        }

        scrolled.set_child(Some(&flow_box));
        notebook.append_page(&scrolled, Some(&Label::new(Some(cat.icon))));
    }

    // Kaomoji tab
    let kaomoji_scrolled = ScrolledWindow::new();
    kaomoji_scrolled.set_policy(PolicyType::Never, PolicyType::Automatic);
    let kaomoji_flow = FlowBox::new();
    kaomoji_flow.set_valign(gtk4::Align::Start);
    kaomoji_flow.set_selection_mode(SelectionMode::None);
    kaomoji_flow.set_min_children_per_line(2);
    kaomoji_flow.set_max_children_per_line(3);

    for (kaomoji, name) in KAOMOJI {
        let btn = Button::with_label(kaomoji);
        btn.add_css_class("flat");
        btn.set_tooltip_text(Some(name));

        let cb = on_select.clone();
        let p = popover_ref.clone();
        let k = kaomoji.to_string();
        let state_clone = state.clone();

        btn.connect_clicked(move |_| {
            state_clone.borrow_mut().add_recent(&k);
            cb(k.clone());
            p.popdown();
        });

        kaomoji_flow.append(&btn);
    }

    kaomoji_scrolled.set_child(Some(&kaomoji_flow));
    notebook.append_page(&kaomoji_scrolled, Some(&Label::new(Some("(◕‿◕)"))));

    container.append(&notebook);

    // Search functionality
    let notebook_clone = notebook.clone();
    let on_select_search = on_select.clone();
    let popover_search = popover_ref.clone();
    let state_search = state.clone();

    search_entry.connect_changed(move |entry| {
        let query = entry.text().to_lowercase();

        if query.is_empty() {
            notebook_clone.set_show_tabs(true);
            return;
        }

        // Create search results in a new flow box
        let search_flow = FlowBox::new();
        search_flow.set_valign(gtk4::Align::Start);
        search_flow.set_selection_mode(SelectionMode::None);
        search_flow.set_min_children_per_line(8);
        search_flow.set_max_children_per_line(12);

        let mut found_count = 0;

        for cat in EMOJI_CATEGORIES {
            for emoji in cat.emojis {
                let matches = emoji.name.to_lowercase().contains(&query)
                    || emoji
                        .keywords
                        .iter()
                        .any(|k| k.to_lowercase().contains(&query));

                if matches {
                    found_count += 1;
                    let btn = create_emoji_button(
                        emoji,
                        &state_search,
                        &on_select_search,
                        &popover_search,
                    );
                    search_flow.append(&btn);
                }
            }
        }

        // Also search kaomoji
        for (kaomoji, name) in KAOMOJI {
            if name.to_lowercase().contains(&query) {
                found_count += 1;
                let btn = Button::with_label(kaomoji);
                btn.add_css_class("flat");
                btn.set_tooltip_text(Some(name));

                let cb = on_select_search.clone();
                let p = popover_search.clone();
                let k = kaomoji.to_string();
                let state_c = state_search.clone();

                btn.connect_clicked(move |_| {
                    state_c.borrow_mut().add_recent(&k);
                    cb(k.clone());
                    p.popdown();
                });

                search_flow.append(&btn);
            }
        }

        if found_count == 0 {
            let no_results = Label::new(Some("No emojis found"));
            no_results.add_css_class("dim-label");
            search_flow.append(&no_results);
        }

        // Replace first tab content with search results
        if let Some(first_page) = notebook_clone.nth_page(Some(0)) {
            if let Some(scrolled) = first_page.downcast_ref::<ScrolledWindow>() {
                scrolled.set_child(Some(&search_flow));
            }
        }
        notebook_clone.set_current_page(Some(0));
    });

    popover.set_child(Some(&container));
    popover.popup();
}

/// Create an emoji button with click and right-click handlers
fn create_emoji_button(
    emoji: &Emoji,
    state: &Rc<RefCell<EmojiPickerState>>,
    on_select: &Rc<impl Fn(String) + 'static>,
    popover_ref: &Rc<Popover>,
) -> Button {
    let state_for_skin = state.borrow();
    let skin_tone = SKIN_TONES[state_for_skin.skin_tone_index].0;
    drop(state_for_skin);

    let display_emoji = if emoji.skin_tone && !skin_tone.is_empty() {
        apply_skin_tone(emoji.char, skin_tone)
    } else {
        emoji.char.to_string()
    };

    let btn = Button::with_label(&display_emoji);
    btn.add_css_class("flat");
    btn.set_has_frame(false);
    btn.set_tooltip_text(Some(emoji.name));

    // Left click - select emoji
    let cb = on_select.clone();
    let p = popover_ref.clone();
    let e = display_emoji.clone();
    let state_click = state.clone();
    let emoji_base = emoji.char.to_string();

    btn.connect_clicked(move |_| {
        state_click.borrow_mut().add_recent(&emoji_base);
        cb(e.clone());
        p.popdown();
    });

    // Right-click - add to favorites
    let gesture = gtk4::GestureClick::new();
    gesture.set_button(3); // Right click
    let state_fav = state.clone();
    let emoji_name = emoji.name.to_string();
    let emoji_char = emoji.char.to_string();

    gesture.connect_released(move |_, _, _, _| {
        let is_fav = state_fav.borrow_mut().toggle_favorite(&emoji_char);
        if is_fav {
            tracing::info!("Added {} to favorites", emoji_name);
        } else {
            tracing::info!("Removed {} from favorites", emoji_name);
        }
    });

    btn.add_controller(gesture);
    btn
}
