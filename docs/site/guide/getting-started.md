# Getting Started

Get Kanban running in 5 minutes.

## Installation

### Option 1: Homebrew (Recommended)
```bash
brew install kanban
```

### Option 2: Download from GitHub Releases
Visit [releases](https://github.com/your-org/kanban/releases) and download the binary for your OS.

```bash
# macOS (Apple Silicon)
curl -L https://github.com/your-org/kanban/releases/download/v0.1.0/kanban-aarch64-apple-darwin \
  -o kanban && chmod +x kanban && mv kanban /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/your-org/kanban/releases/download/v0.1.0/kanban-x86_64-apple-darwin \
  -o kanban && chmod +x kanban && mv kanban /usr/local/bin/

# Linux (x86_64)
curl -L https://github.com/your-org/kanban/releases/download/v0.1.0/kanban-x86_64-unknown-linux-gnu \
  -o kanban && chmod +x kanban && mv kanban /usr/local/bin/
```

### Option 3: Build from Source
```bash
git clone https://github.com/your-org/kanban.git
cd kanban
cargo build --release
./target/release/kanban app
```

## First Run

### Step 1: Launch the App
```bash
kanban
```

This starts the GUI. The app creates `~/.kanban/data.db` automatically on first run.

:::tip
If you're on a headless system or prefer the CLI, use `kanban cli project list` instead.
:::

### Step 2: Create Your First Project
In the GUI, click "New Project" or use the CLI:

```bash
kanban cli project create "My Project" \
  --prefix "PROJ" \
  --description "My first Kanban project"
```

This creates a project with:
- **Identifier prefix:** `PROJ`
- **Default statuses:** Backlog, Todo, In Progress, In Review, Blocked, Done, Discarded
- **Auto-incrementing issue IDs:** PROJ-1, PROJ-2, etc.

### Step 3: Create Your First Issue
```bash
kanban cli issue create \
  --project 1 \
  --title "Write documentation" \
  --status 2 \
  --priority high \
  --description "Complete getting started guide"
```

List issues to see your work:
```bash
kanban cli issue list --project 1
```

### Step 4: Move Issues Across Statuses
Update the status of an issue:

```bash
kanban cli issue update PROJ-1 --status 3
```

Check the CLI output or GUI to see it moved to "In Progress".

### Step 5: (Optional) Register an Agent
If you're coordinating AI agent work, register an agent:

```bash
kanban cli agent register \
  --name "Claude Agent" \
  --agent-type claude \
  --skills "code-review,debugging,documentation" \
  --max-concurrent 2 \
  --max-complexity large
```

Then create a task contract (instead of a regular issue) so agents can claim it:

```bash
kanban cli task create \
  --project 1 \
  --title "Implement feature X" \
  --objective "Add feature X to the codebase" \
  --status 2 \
  --skills "coding,testing" \
  --complexity large
```

---

## Common Tasks

### List All Projects
```bash
kanban cli project list
```

### Search Issues
```bash
kanban cli issue search --project 1 "database"
```

### Create an Issue with Labels
First, create a label:
```bash
kanban cli label create --project 1 --name "bug" --color "#ef4444"
```

Then create an issue with that label:
```bash
kanban cli issue create \
  --project 1 \
  --title "Fix crash on startup" \
  --status 2 \
  --priority urgent
```

### Export All Data
```bash
kanban cli export --output backup.json
```

### Import Data
```bash
kanban cli import backup.json
```

---

## Database Location

By default, Kanban uses: `~/.kanban/data.db`

To use a different database:
```bash
# Set via environment variable
export DATABASE_URL="sqlite:///path/to/custom.db"
kanban

# Or via command line
kanban --database-url "sqlite:///path/to/custom.db"
```

---

## Next Steps

- **[Concepts](/guide/concepts.md)** — Understand Projects, Issues, Statuses, and more
- **[Issues](/guide/issues.md)** — Full issue management guide
- **[Task Contracts](/guide/task-contracts.md)** — Set up executable tasks for agents
- **[Agent Routing](/guide/agent-routing.md)** — Learn how agents are matched to tasks
