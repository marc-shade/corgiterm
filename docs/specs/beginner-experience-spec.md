# Beginner Experience Spec

Status date: 2026-06-03

## Product Intent

CorgiTerm is a terminal for people who are curious about the command line but do not yet trust themselves inside one. It should preserve the power of a real terminal while adding explanation, preview, recovery guidance, and visual structure.

## Target Users

### New CLI User

Knows how to use a computer, folders, apps, and maybe code editors, but does not know common shell commands. Needs low-friction help and clear guardrails.

### Occasional CLI User

Uses terminal when following tutorials. Often copies commands without fully understanding them. Needs explanation and safety checks.

### AI-Assisted Builder

Uses AI tools and hears that terminal workflows are important. Needs natural-language command help, but must not accidentally run destructive generated commands.

## Design Principles

- The terminal remains real. CorgiTerm should not hide shell behavior behind fake abstractions.
- The first step should be obvious. Empty-terminal anxiety is a product problem.
- Every generated or suggested command should be reviewable before execution.
- Risky operations need plain-English explanations and safer alternatives.
- Offline/local AI must be supported where possible.
- Users must be able to inspect, disable, or clear learning/history behavior.
- Failure states should teach, not shame.

## Core Beginner Workflows

### Workflow 1: Ask For A Command

User story: As a new CLI user, I can type "show large files in this folder" and receive a suggested shell command with an explanation.

Requirements:

- Natural-language input accepts a plain-English request.
- Deterministic quick translations are used for common requests.
- AI provider fallback is used when no deterministic pattern matches.
- If no provider exists, the UI explains what is missing and offers local/CLI/API setup guidance.
- The generated command is never executed directly. It must pass through Safe Mode.

Acceptance criteria:

- Given a common deterministic request, CorgiTerm returns a command without network access.
- Given a mocked AI provider, CorgiTerm shows the returned command and explanation.
- Given no provider, the UI does not freeze and displays a useful no-provider state.
- Execute routes through Safe Mode.

### Workflow 2: Review Before Running

User story: As an unsure user, I can see what a command will do, how risky it is, and what safer alternatives exist before I run it.

Requirements:

- Safe Mode supports safe, caution, danger, and unknown states.
- Destructive filesystem commands are detected.
- Network and privilege-escalation commands are flagged.
- Explanations use plain English.
- Cancel is always available and keyboard accessible.
- Execute is visually distinct and risk-aware.

Acceptance criteria:

- `ls`, `pwd`, and `cat` classify as safe.
- `rm file`, `rm -rf folder`, and `rm -rf /` classify with increasing severity.
- `sudo`, `curl`, `wget`, `npm`, `pip`, `chmod -R 777`, `chown -R`, `kill`, and destructive git commands are classified.
- Danger-state execute cannot be mistaken for the primary safe path.
- Cancel closes the preview and does not write to the PTY.

### Workflow 3: Learn From The Result

User story: After running a command, I can understand what happened and learn the command for next time.

Requirements:

- AI Explain mode can explain command output or errors.
- The terminal screen can be read into a bounded context.
- History learning records commands without leaking secrets.
- Users can disable learning and clear local history.

Acceptance criteria:

- A command plus output can be sent to Explain mode with a mocked provider.
- Secret-like strings are not stored in learning context.
- Disabling learning prevents future command recording.

### Workflow 4: Save A Useful Command

User story: When I find a useful command, I can save it as a snippet with variables.

Requirements:

- Snippets support name, category, tags, command body, variables, defaults, hints, pinned state, and usage metadata.
- Users can insert a snippet or execute it through Safe Mode.
- Variables can be prompted before insertion/execution.

Acceptance criteria:

- A snippet with `{{host:localhost}}` and `{{port|port number}}` prompts for values.
- Missing values keep placeholders visible or block execution with a clear message.
- Snippet execution routes through Safe Mode.

### Workflow 5: Find My Way Around

User story: I can navigate projects and sessions visually without memorizing `cd` paths.

Requirements:

- Sidebar shows saved projects and useful folders.
- Tabs and split panes maintain independent working directories.
- The current directory is visible and reliable where platform support exists.
- Search, hints, and history help users rediscover prior work.

Acceptance criteria:

- Opening a project creates a terminal in that directory.
- Split panes preserve or inherit the expected working directory.
- Search finds visible terminal text.
- URL/path hints can be opened or copied without mouse precision.

## Non-Goals For Beginner MVP

- Replacing shell learning with a fake command environment.
- Hiding command output behind natural-language summaries only.
- Cloud-only AI behavior.
- Running destructive AI-generated commands without review.
- Treating all terminal users as beginners. Power-user workflows should remain efficient.

## UX Quality Bar

- Beginner prompts should be short and concrete.
- Avoid jargon unless paired with a short explanation.
- Do not make users read docs before completing the first workflow.
- Keep terminal output readable and unmodified.
- Preferences should make safety and learning behavior transparent.
