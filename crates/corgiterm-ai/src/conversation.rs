//! AI Conversation persistence
//!
//! Stores chat conversations across sessions for continuity.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

use crate::{Message, Role};

/// Maximum conversations to keep
const MAX_CONVERSATIONS: usize = 50;
/// Maximum messages per conversation
const MAX_MESSAGES_PER_CONVERSATION: usize = 100;

/// A stored conversation with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Unique ID
    pub id: String,
    /// Conversation title (derived from first message)
    pub title: String,
    /// When the conversation started
    pub created_at: DateTime<Utc>,
    /// When last message was added
    pub updated_at: DateTime<Utc>,
    /// Messages in the conversation
    pub messages: Vec<StoredMessage>,
    /// AI provider used
    pub provider: Option<String>,
    /// Conversation mode (chat, command, explain)
    pub mode: String,
}

/// A message with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub role: StoredRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// Role enum for storage (mirrors crate::Role)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StoredRole {
    System,
    User,
    Assistant,
}

impl From<Role> for StoredRole {
    fn from(role: Role) -> Self {
        match role {
            Role::System => StoredRole::System,
            Role::User => StoredRole::User,
            Role::Assistant => StoredRole::Assistant,
        }
    }
}

impl From<StoredRole> for Role {
    fn from(role: StoredRole) -> Self {
        match role {
            StoredRole::System => Role::System,
            StoredRole::User => Role::User,
            StoredRole::Assistant => Role::Assistant,
        }
    }
}

impl Conversation {
    /// Create a new conversation
    pub fn new(mode: &str) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: format!("New {} conversation", mode),
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
            provider: None,
            mode: mode.to_string(),
        }
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, role: Role, content: &str) {
        self.messages.push(StoredMessage {
            role: role.into(),
            content: content.to_string(),
            timestamp: Utc::now(),
        });
        self.updated_at = Utc::now();

        // Update title from first user message if still default
        if self.title.starts_with("New ") {
            if let Some(msg) = self.messages.iter().find(|m| m.role == StoredRole::User) {
                // Take first 50 chars of first user message as title
                let mut title: String = msg.content.chars().take(50).collect();
                if msg.content.len() > 50 {
                    title.push_str("...");
                }
                self.title = title;
            }
        }

        // Limit messages
        if self.messages.len() > MAX_MESSAGES_PER_CONVERSATION {
            self.messages
                .drain(0..self.messages.len() - MAX_MESSAGES_PER_CONVERSATION);
        }
    }

    /// Set the provider
    pub fn set_provider(&mut self, provider: &str) {
        self.provider = Some(provider.to_string());
    }

    /// Convert to Message list for API calls
    pub fn to_messages(&self) -> Vec<Message> {
        self.messages
            .iter()
            .map(|m| Message {
                role: m.role.into(),
                content: m.content.clone(),
            })
            .collect()
    }

    /// Check if conversation is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Get message count
    pub fn len(&self) -> usize {
        self.messages.len()
    }
}

/// Conversation storage manager
pub struct ConversationStore {
    /// All stored conversations
    conversations: VecDeque<Conversation>,
    /// Current active conversation (by mode)
    active_chat: Option<Conversation>,
    active_command: Option<Conversation>,
    active_explain: Option<Conversation>,
    /// Storage path
    storage_path: PathBuf,
    /// Dirty flag for saving
    dirty: bool,
}

impl ConversationStore {
    /// Create a new store with default config directory
    pub fn new() -> Self {
        let config_dir = Self::config_dir();
        Self::with_path(config_dir.join("conversations.json"))
    }

    /// Create with specific path
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            conversations: VecDeque::new(),
            active_chat: None,
            active_command: None,
            active_explain: None,
            storage_path: path,
            dirty: false,
        }
    }

    /// Get config directory
    fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("corgiterm")
    }

    /// Load conversations from disk
    pub fn load(&mut self) -> anyhow::Result<()> {
        if !self.storage_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.storage_path)?;
        let stored: StoredConversations = serde_json::from_str(&content)?;

        self.conversations = stored.conversations.into_iter().collect();
        tracing::info!(
            "Loaded {} conversations from {:?}",
            self.conversations.len(),
            self.storage_path
        );

        Ok(())
    }

    /// Save conversations to disk
    pub fn save(&mut self) -> anyhow::Result<()> {
        if !self.dirty {
            return Ok(());
        }

        // Ensure directory exists
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Include active conversations in save
        let mut all_convos: Vec<Conversation> = self.conversations.iter().cloned().collect();

        // Add active conversations if they have content
        if let Some(ref conv) = self.active_chat {
            if !conv.is_empty() && !all_convos.iter().any(|c| c.id == conv.id) {
                all_convos.push(conv.clone());
            }
        }
        if let Some(ref conv) = self.active_command {
            if !conv.is_empty() && !all_convos.iter().any(|c| c.id == conv.id) {
                all_convos.push(conv.clone());
            }
        }
        if let Some(ref conv) = self.active_explain {
            if !conv.is_empty() && !all_convos.iter().any(|c| c.id == conv.id) {
                all_convos.push(conv.clone());
            }
        }

        let stored = StoredConversations {
            version: 1,
            conversations: all_convos,
        };

        let content = serde_json::to_string_pretty(&stored)?;
        fs::write(&self.storage_path, content)?;

        self.dirty = false;
        tracing::debug!("Saved conversations to {:?}", self.storage_path);

        Ok(())
    }

    /// Get or create active conversation for a mode
    pub fn active_conversation(&mut self, mode: &str) -> &mut Conversation {
        match mode {
            "chat" => {
                if self.active_chat.is_none() {
                    self.active_chat = Some(Conversation::new("chat"));
                }
                self.active_chat.as_mut().unwrap()
            }
            "command" => {
                if self.active_command.is_none() {
                    self.active_command = Some(Conversation::new("command"));
                }
                self.active_command.as_mut().unwrap()
            }
            "explain" => {
                if self.active_explain.is_none() {
                    self.active_explain = Some(Conversation::new("explain"));
                }
                self.active_explain.as_mut().unwrap()
            }
            _ => {
                if self.active_chat.is_none() {
                    self.active_chat = Some(Conversation::new(mode));
                }
                self.active_chat.as_mut().unwrap()
            }
        }
    }

    /// Add message to active conversation
    pub fn add_message(&mut self, mode: &str, role: Role, content: &str) {
        let conv = self.active_conversation(mode);
        conv.add_message(role, content);
        self.dirty = true;
    }

    /// Set provider for active conversation
    pub fn set_provider(&mut self, mode: &str, provider: &str) {
        let conv = self.active_conversation(mode);
        conv.set_provider(provider);
        self.dirty = true;
    }

    /// Start a new conversation for mode
    pub fn new_conversation(&mut self, mode: &str) {
        // Archive current if it has content
        match mode {
            "chat" => {
                if let Some(conv) = self.active_chat.take() {
                    if !conv.is_empty() {
                        self.archive_conversation(conv);
                    }
                }
                self.active_chat = Some(Conversation::new("chat"));
            }
            "command" => {
                if let Some(conv) = self.active_command.take() {
                    if !conv.is_empty() {
                        self.archive_conversation(conv);
                    }
                }
                self.active_command = Some(Conversation::new("command"));
            }
            "explain" => {
                if let Some(conv) = self.active_explain.take() {
                    if !conv.is_empty() {
                        self.archive_conversation(conv);
                    }
                }
                self.active_explain = Some(Conversation::new("explain"));
            }
            _ => {}
        }
        self.dirty = true;
    }

    /// Archive a conversation
    fn archive_conversation(&mut self, conv: Conversation) {
        self.conversations.push_front(conv);
        // Limit total conversations
        while self.conversations.len() > MAX_CONVERSATIONS {
            self.conversations.pop_back();
        }
    }

    /// Get recent conversations for a mode
    pub fn recent_conversations(&self, mode: &str, limit: usize) -> Vec<&Conversation> {
        self.conversations
            .iter()
            .filter(|c| c.mode == mode)
            .take(limit)
            .collect()
    }

    /// Get all conversations
    pub fn all_conversations(&self) -> impl Iterator<Item = &Conversation> {
        self.conversations.iter()
    }

    /// Load a specific conversation as active
    pub fn load_conversation(&mut self, id: &str, mode: &str) -> bool {
        if let Some(idx) = self.conversations.iter().position(|c| c.id == id) {
            let conv = self.conversations.remove(idx).unwrap();
            match mode {
                "chat" => {
                    if let Some(old) = self.active_chat.take() {
                        if !old.is_empty() {
                            self.archive_conversation(old);
                        }
                    }
                    self.active_chat = Some(conv);
                }
                "command" => {
                    if let Some(old) = self.active_command.take() {
                        if !old.is_empty() {
                            self.archive_conversation(old);
                        }
                    }
                    self.active_command = Some(conv);
                }
                "explain" => {
                    if let Some(old) = self.active_explain.take() {
                        if !old.is_empty() {
                            self.archive_conversation(old);
                        }
                    }
                    self.active_explain = Some(conv);
                }
                _ => return false,
            }
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Get current chat messages formatted for display
    pub fn current_chat_display(&self) -> String {
        if let Some(ref conv) = self.active_chat {
            conv.messages
                .iter()
                .filter(|m| m.role != StoredRole::System)
                .map(|m| {
                    let role = match m.role {
                        StoredRole::User => "You",
                        StoredRole::Assistant => "Assistant",
                        StoredRole::System => "System",
                    };
                    format!("{}: {}\n\n", role, m.content)
                })
                .collect()
        } else {
            String::new()
        }
    }

    /// Get stats
    pub fn stats(&self) -> ConversationStats {
        ConversationStats {
            total_conversations: self.conversations.len(),
            chat_count: self.conversations.iter().filter(|c| c.mode == "chat").count(),
            command_count: self
                .conversations
                .iter()
                .filter(|c| c.mode == "command")
                .count(),
            explain_count: self
                .conversations
                .iter()
                .filter(|c| c.mode == "explain")
                .count(),
        }
    }
}

impl Default for ConversationStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Stats about stored conversations
#[derive(Debug, Clone)]
pub struct ConversationStats {
    pub total_conversations: usize,
    pub chat_count: usize,
    pub command_count: usize,
    pub explain_count: usize,
}

/// Storage format
#[derive(Serialize, Deserialize)]
struct StoredConversations {
    version: u32,
    conversations: Vec<Conversation>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_conversation() {
        let conv = Conversation::new("chat");
        assert!(conv.is_empty());
        assert_eq!(conv.mode, "chat");
    }

    #[test]
    fn test_add_message() {
        let mut conv = Conversation::new("chat");
        conv.add_message(Role::User, "Hello");
        conv.add_message(Role::Assistant, "Hi there!");

        assert_eq!(conv.len(), 2);
        assert_eq!(conv.title, "Hello");
    }

    #[test]
    fn test_conversation_store() {
        let mut store = ConversationStore::with_path(PathBuf::from("/tmp/test_conv.json"));

        store.add_message("chat", Role::User, "Test message");
        store.add_message("chat", Role::Assistant, "Test response");

        let stats = store.stats();
        assert_eq!(stats.total_conversations, 0); // Not archived yet

        store.new_conversation("chat");
        let stats = store.stats();
        assert_eq!(stats.total_conversations, 1);
        assert_eq!(stats.chat_count, 1);
    }
}
