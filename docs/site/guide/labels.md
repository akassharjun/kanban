# Labels

Labels are **project-scoped tags** for organizing and filtering issues. Each label has a name and color.

## Creating Labels

Labels belong to a specific project.

### Via CLI

```bash
# Create basic label
kanban cli label create \
  --project 1 \
  --name "bug" \
  --color "#ef4444"

# Create multiple labels
kanban cli label create --project 1 --name "feature" --color "#3b82f6"
kanban cli label create --project 1 --name "tech-debt" --color "#f59e0b"
kanban cli label create --project 1 --name "documentation" --color "#10b981"
kanban cli label create --project 1 --name "security" --color "#dc2626"
kanban cli label create --project 1 --name "performance" --color "#8b5cf6"
kanban cli label create --project 1 --name "research" --color "#6366f1"
```

### Via GUI
1. Open a project
2. Click "Labels" → "Add Label"
3. Enter name and pick color
4. Click "Create"

## Listing Labels

### Via CLI

```bash
kanban cli label list --project 1

# Output:
# ID | Name          | Color   | Issues
# 1  | bug           | #ef4444 | 5
# 2  | feature       | #3b82f6 | 8
# 3  | tech-debt     | #f59e0b | 3
# 4  | documentation | #10b981 | 2
# 5  | security      | #dc2626 | 1
```

## Assigning Labels to Issues

Labels are applied when creating or updating issues.

### Via CLI - Create with Labels

```bash
kanban cli issue create \
  --project 1 \
  --title "SQL injection vulnerability" \
  --status 2 \
  --priority urgent \
  --labels "bug,security"
```

### Via CLI - Add Labels to Existing Issue

```bash
# Currently: labels are assigned at creation
# To add/remove labels, update the issue with full label list
kanban cli issue update API-42 --labels "bug,security,urgent"
```

### Via GUI
1. Open an issue
2. Click the label section
3. Select labels from the dropdown
4. Changes save automatically

## Updating Labels

### Via CLI

```bash
# Update label name
kanban cli label update 1 --name "bug-report"

# Update label color
kanban cli label update 1 --color "#991b1b"
```

### Via GUI
Right-click label → Edit

## Deleting Labels

### Via CLI

```bash
kanban cli label delete 1
```

This removes the label from all issues it was applied to.

### Via GUI
Right-click label → Delete

## Filtering by Labels

### Via CLI

```bash
# List issues with a specific label
kanban cli issue list --project 1 --label 1

# Output:
# ID | Identifier | Title                      | Status | Priority
# 42 | API-42     | SQL injection vulnerability| Todo   | urgent
# 51 | API-51     | Cross-site scripting check | In Prog| high
```

### Via GUI
1. Open a project
2. Click a label in the sidebar
3. Board filters to show only issues with that label

## Standard Label Sets

Here are recommended label configurations for different workflows:

### Software Development

```
bug         #ef4444  Problems to fix
feature     #3b82f6  New capabilities
enhancement #10b981  Improve existing
tech-debt   #f59e0b  Refactor/cleanup
docs        #06b6d4  Documentation
research    #8b5cf6  Investigation needed
blocked     #ec4899  Waiting on something
```

### SaaS / Product

```
bug         #ef4444  Production issue
feature     #3b82f6  New feature
enhancement #10b981  Existing feature improvement
ux          #6366f1  User experience
perf        #8b5cf6  Performance/optimization
docs        #06b6d4  Documentation
revenue     #22c55e  Affects revenue
security    #dc2626  Security issue
```

### DevOps / Infrastructure

```
bug         #ef4444  Infrastructure issue
enhancement #10b981  Infrastructure improvement
deployment  #3b82f6  Deployment task
monitoring  #f59e0b  Monitoring/observability
incident    #dc2626  On-call incident
automation  #06b6d4  Automation task
security    #dc2626  Security hardening
docs        #10b981  Documentation
```

### Support / Bug Triage

```
bug         #ef4444  Confirmed bug
question    #3b82f6  User asking for help
duplicate   #6b7280  Duplicate of another
invalid     #9ca3af  Not a real issue
wontfix     #6b7280  Won't fix
regression  #ec4899  Introduced by our changes
critical    #dc2626  Critical for users
hotfix      #f59e0b  Needs immediate fix
```

## Label Best Practices

### 1. Keep Label Count Reasonable
Start with 5-10 labels. Too many dilutes their usefulness.

Good: 7 labels
Bad: 50 labels

### 2. Use Consistent Naming
- Use lowercase
- Use hyphens for multi-word labels
- Avoid abbreviations

Good: `tech-debt`, `cross-site-scripting`
Bad: `TD`, `xss`, `TechDebt`

### 3. Establish Clear Categories
Group labels into conceptual categories:

```
PRIORITY:
  urgent
  high

TYPE:
  bug
  feature
  enhancement
  tech-debt

SKILL:
  backend
  frontend
  devops
  security

STATUS (if not using issue status):
  in-review
  blocked
  waiting
```

### 4. Use Colors Meaningfully
Map colors to label types:

```
Red (#ef4444):      Critical, bugs, security
Blue (#3b82f6):     Features, work to do
Green (#22c55e):    Done, solved, enhancement
Yellow (#f59e0b):   Tech debt, refactor, optimization
Purple (#8b5cf6):   Research, experimentation
Gray (#6b7280):     Duplicate, invalid, closed
```

### 5. Avoid Overlap
Don't create labels that mean the same thing:

Bad:
- `bug`, `bugfix`, `defect`, `issue` (all the same)

Good:
- Use one: `bug`

### 6. Don't Duplicate Issue Status
Don't create labels for statuses you already have:

Bad:
- Status: "In Progress"
- Label: "in-progress"

Good:
- Use status for workflow
- Use labels for categorization

## Bulk Labeling

Apply labels to multiple issues at once:

### Via CLI (Tauri commands)

```bash
# List all issues with a certain status
kanban cli issue list --project 1 --status 2 | \
  jq '.[] | .id' | \
  xargs -I {} kanban cli issue update {} --labels "reviewed"
```

### Via GUI
1. Select multiple issues
2. Click "Bulk Actions"
3. Choose "Add Labels" or "Remove Labels"
4. Apply

## Label and Agent Matching

Labels don't directly affect agent routing (agents route by skills and complexity). However, you can use labels to:

- **Tag tasks for review** — Use `code-review` label
- **Mark as urgent** — Use `urgent` label
- **Group related work** — Use feature labels like `auth`, `payment`, `reporting`

Agents will still respect skill requirements and complexity limits, but labels can help with monitoring and reporting.

Example workflow:

```bash
# Create urgent auth features
kanban cli task create \
  --project 1 \
  --title "Implement OAuth" \
  --status 2 \
  --skills "security,auth" \
  --complexity large \
  --labels "feature,auth,urgent"

kanban cli task create \
  --project 1 \
  --title "Add 2FA support" \
  --status 2 \
  --skills "security,auth" \
  --complexity medium \
  --labels "feature,auth,urgent"

# Agent claims:
kanban cli agent next-task --agent-id "agent-1"
# Returns: highest priority urgent auth task matching skills
```

## Next Steps

- **[Issues](/guide/issues.md)** — Assign labels when creating/updating issues
- **[Projects](/guide/projects.md)** — Manage labels per project
- **[Task Contracts](/guide/task-contracts.md)** — Labels on executable tasks
