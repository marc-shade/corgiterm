# AI Command History Learning

CorgiTerm's AI learning system observes your command patterns and preferences to provide intelligent, personalized suggestions.

## Features

### 1. **Command Pattern Detection**

The system automatically learns sequences of commands that often occur together:

```bash
# After running:
git status
git add .
git commit -m "Update"
git push

# CorgiTerm learns this pattern and will suggest the next command
```

**Pattern Types:**
- **Sequential patterns**: Commands that follow each other
- **Time-based patterns**: Commands used at certain times of day
- **Directory-specific patterns**: Commands used in specific project directories

### 2. **User Preference Learning**

CorgiTerm detects when you prefer modern alternatives to standard commands:

| Standard Command | Detected Alternative |
|------------------|---------------------|
| `ls` | `exa`, `lsd`, `eza` |
| `cat` | `bat` |
| `find` | `fd` |
| `grep` | `rg`, `ag` |
| `du` | `dust` |
| `top` | `htop`, `btop`, `bottom` |

**Example:**
If you use `exa -la` 80% of the time vs `ls -la` 20% of the time, the AI will:
- Suggest `exa` when you ask "show files"
- Use `exa` in generated commands
- Show `exa` in auto-completions

### 3. **Success/Failure Tracking**

Commands are tracked with exit codes to learn which patterns work:

```rust
CommandStats {
    command: "cargo build",
    total_count: 50,
    success_count: 45,
    failure_count: 5,
    success_rate: 0.9,  // 90% success
}
```

This helps the AI:
- Suggest commands that work in your environment
- Warn about commands that often fail
- Recommend alternatives for problematic commands

### 4. **Directory-Specific Commands**

CorgiTerm learns which commands you use in different project types:

```
/home/user/rust-project/
  • cargo build (50 times)
  • cargo test (30 times)
  • cargo run (25 times)

/home/user/node-project/
  • npm install (20 times)
  • npm run dev (45 times)
  • yarn build (15 times)
```

When you navigate to a directory, the AI panel shows relevant suggestions.

### 5. **Intelligent Next Command Prediction**

Based on learned patterns, predict what you'll likely run next:

```
You: git add .
AI:  Next: git commit -m "..." (85% confidence)
     Based on your pattern: git add → git commit → git push
```

## Configuration

### Enable/Disable Learning

In `~/.config/corgiterm/config.toml`:

```toml
[ai.learning]
# Enable command learning (default: true)
enabled = true

# Opt-out of all learning (privacy mode)
opt_out = false

# Maximum history to keep
max_history = 10000

# Minimum pattern frequency for detection (3 = must see 3 times)
min_pattern_frequency = 3

# Maximum pattern length (5 = sequences up to 5 commands)
max_pattern_length = 5

# Learning window size (recent commands to analyze)
window_size = 100

# Auto-detect user preferences
detect_preferences = true

# Suggest next command based on patterns
suggest_next = true

# Show directory-specific suggestions
directory_suggestions = true
```

### Storage Location

Learning data is stored in:
- **Default**: `~/.config/corgiterm/learning.json`
- **Custom**: Set `data_path` in config

```toml
[ai.learning]
data_path = "/custom/path/learning.json"
```

## Privacy & Control

### Clear Learning Data

From the UI:
1. Open Settings (`Ctrl+,`)
2. Go to AI → Learning
3. Click "Clear All Learning Data"

From the CLI:
```bash
rm ~/.config/corgiterm/learning.json
```

### Opt-Out Completely

```toml
[ai.learning]
opt_out = true
```

This disables all learning and deletes existing data.

### What's Stored

CorgiTerm stores:
- Command names and patterns
- Directories where commands were used
- Success/failure rates
- Timestamps

**NOT stored:**
- Command arguments with sensitive data
- Output or error messages
- Environment variables

**Example stored data:**
```json
{
  "patterns": [
    {
      "sequence": ["git", "add", "commit", "push"],
      "frequency": 15,
      "confidence": 0.85
    }
  ],
  "preferences": [
    {
      "standard": "ls",
      "preferred": "exa",
      "ratio": 0.83
    }
  ],
  "stats": {
    "cargo": {
      "total_count": 100,
      "success_count": 95,
      "success_rate": 0.95
    }
  }
}
```

## Using Learning Data

### In the AI Panel

The AI panel shows learning-based suggestions:

```
┌─ AI Assistant ─────────────────┐
│ Mode: Command                  │
├────────────────────────────────┤
│ 💡 Suggestions                 │
│                                │
│ Based on your history:         │
│   • cargo build                │
│   • cargo test                 │
│   • cargo run                  │
│                                │
│ Often used here:               │
│   • git status                 │
│   • docker-compose up          │
│                                │
│ Your preferences:              │
│   • You prefer 'exa' over 'ls' │
│   • You prefer 'bat' over 'cat'│
└────────────────────────────────┘
```

### In Auto-Completion

When typing, completions are ranked by:
1. Exact matches
2. Frequently used commands (from learning)
3. Directory-specific commands
4. Pattern-based next commands
5. Standard completions

### In Natural Language Mode

When you use AI to generate commands, it considers your preferences:

```
You: "show files with details"

Without Learning:
AI: ls -la

With Learning (you prefer exa):
AI: exa -la --git --header
```

## Advanced Usage

### Pattern Confidence Scores

Patterns have confidence scores based on:
- **Frequency**: How often the pattern occurs
- **Consistency**: How reliably commands follow in order
- **Recency**: When the pattern was last seen

```rust
CommandPattern {
    sequence: ["git", "add", "commit"],
    frequency: 20,          // Seen 20 times
    confidence: 0.90,       // 90% confident
    last_seen: "2024-01-15" // Recent
}
```

High confidence (>0.8) = Reliable suggestion
Medium confidence (0.5-0.8) = Possible suggestion
Low confidence (<0.5) = Not shown

### Time-of-Day Patterns

Commands are categorized by when you use them:

- **Morning** (6am-12pm): Often maintenance, checks
- **Afternoon** (12pm-6pm): Active development
- **Evening** (6pm-12am): Testing, deployment
- **Night** (12am-6am): Emergency fixes, late coding

The AI can adapt suggestions based on time.

### Project Detection

CorgiTerm tries to identify project types based on files:

- `Cargo.toml` → Rust project
- `package.json` → Node.js project
- `requirements.txt` → Python project
- `.git/` → Git repository

Suggestions are tailored to the project type.

## Implementation Details

### Architecture

```
┌──────────────────┐
│ User runs command│
└────────┬─────────┘
         ▼
┌──────────────────┐
│ CommandHistory   │  ← Stores all commands
└────────┬─────────┘
         ▼
┌──────────────────┐
│ CommandLearning  │  ← Analyzes patterns
└────────┬─────────┘
         ▼
┌──────────────────┐
│ LearningContext  │  ← Context for AI
└────────┬─────────┘
         ▼
┌──────────────────┐
│ LearningAi       │  ← Enhanced AI suggestions
└──────────────────┘
```

### Key Components

1. **CommandHistory** (`corgiterm-core/src/history.rs`)
   - Stores all executed commands
   - Tracks timestamps, directories, exit codes

2. **CommandLearning** (`corgiterm-core/src/learning.rs`)
   - Detects patterns in command sequences
   - Calculates frequency and confidence scores
   - Manages user preferences

3. **HistoryLearningManager** (`corgiterm-core/src/history_learning.rs`)
   - Bridges history and learning
   - Manages persistence
   - Provides unified API

4. **LearningAi** (`corgiterm-ai/src/learning.rs`)
   - Integrates learning with AI providers
   - Builds context-aware prompts
   - Generates personalized suggestions

### Performance

- **Pattern detection**: O(n²) where n = window size (default: 100)
- **Frequency calculation**: O(1) lookup in HashMap
- **Preference detection**: O(k) where k = known alternatives (typically <20)
- **Memory usage**: ~10KB per 1000 commands

Pattern detection runs in the background and doesn't block the terminal.

## Troubleshooting

### Learning Not Working

1. **Check if enabled**:
   ```toml
   [ai.learning]
   enabled = true
   opt_out = false
   ```

2. **Check data file**:
   ```bash
   ls -lh ~/.config/corgiterm/learning.json
   ```

3. **Check logs**:
   Look for learning-related messages in debug logs.

### Too Many Suggestions

Adjust thresholds:
```toml
[ai.learning]
min_pattern_frequency = 5  # Require more occurrences
max_pattern_length = 3     # Shorter patterns only
```

### Incorrect Preferences

Clear learning data and rebuild:
```bash
rm ~/.config/corgiterm/learning.json
# Use CorgiTerm normally for a few days
```

### Privacy Concerns

1. **Review stored data**:
   ```bash
   cat ~/.config/corgiterm/learning.json | jq
   ```

2. **Disable specific features**:
   ```toml
   [ai.learning]
   detect_preferences = false
   directory_suggestions = false
   ```

3. **Opt-out completely**:
   ```toml
   [ai.learning]
   opt_out = true
   ```

## Future Enhancements

Planned features:
- **Multi-machine sync**: Share learning across devices
- **Team patterns**: Learn from team's common workflows
- **Error pattern recognition**: Suggest fixes for common errors
- **Workflow templates**: Save and share command sequences
- **A/B testing**: Try different command variants and learn from results

## Contributing

To improve the learning system:

1. **Add more command alternatives**: Edit `detect_preferences()` in `learning.rs`
2. **Improve pattern detection**: Enhance algorithms in `detect_patterns()`
3. **Add context types**: Extend `PatternContext` for more granular learning
4. **UI improvements**: Better display of learning insights

See `examples/ai_learning_demo.rs` for usage examples.

## Related Documentation

- [AI Integration](./AI_INTEGRATION.md)
- [Configuration](./CONFIGURATION.md)
- [Privacy Policy](./PRIVACY.md)
