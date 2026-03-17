# Labels Command

Manage issue labels via the CLI.

## Commands

### List labels

List all labels for a project.

```bash
kanban label list --project <PROJECT_ID>
```

**Required:**
- `--project <PROJECT_ID>` - Project ID

**Examples:**

```bash
# List all labels in project 1
kanban label list --project 1

# JSON output
kanban label list --project 1 --json
```

**Output (default):**
```
ID  NAME         COLOR    PROJECT
1   bug          #ef4444  1
2   feature      #22c55e  1
3   documentation #3b82f6 1
4   wontfix      #6b7280  1
5   critical     #fbbf24  1
```

**JSON Output:**
```json
[
  {
    "id": 1,
    "project_id": 1,
    "name": "bug",
    "color": "#ef4444"
  },
  {
    "id": 2,
    "project_id": 1,
    "name": "feature",
    "color": "#22c55e"
  },
  {
    "id": 3,
    "project_id": 1,
    "name": "documentation",
    "color": "#3b82f6"
  }
]
```

### Create a label

Create a new label for a project.

```bash
kanban label create --project <PROJECT_ID> --name <NAME> --color <COLOR>
```

**Required:**
- `--project <PROJECT_ID>` - Project ID
- `--name <NAME>` - Label name
- `--color <COLOR>` - Hex color code (e.g., #ef4444)

**Examples:**

```bash
# Create a bug label (red)
kanban label create --project 1 --name "bug" --color "#ef4444"

# Create a feature label (green)
kanban label create --project 1 --name "feature" --color "#22c55e"

# Create a documentation label (blue)
kanban label create --project 1 --name "documentation" --color "#3b82f6"

# Critical (orange/yellow)
kanban label create --project 1 --name "critical" --color "#fbbf24"

# JSON output
kanban label create --project 1 --name "approved" --color "#06b6d4" --json
```

**Output:**
```
Created label: bug (ID: 1, color: #ef4444)
```

**JSON Output:**
```json
{
  "id": 6,
  "project_id": 1,
  "name": "approved",
  "color": "#06b6d4"
}
```

### Delete a label

Remove a label from a project.

```bash
kanban label delete <ID>
```

**Arguments:**
- `<ID>` - Label ID to delete

**Examples:**

```bash
# Delete label 1
kanban label delete 1

# Output
Label 1 deleted.
```

**Notes:**
- Deleting a label removes it from all issues that have it.
- This cannot be undone.

## Common Label Colors

Use these hex colors for standard label types:

| Label Type | Color | Hex |
|------------|-------|-----|
| bug | Red | #ef4444 |
| feature | Green | #22c55e |
| documentation | Blue | #3b82f6 |
| enhancement | Purple | #a78bfa |
| critical | Orange | #f97316 |
| wontfix | Gray | #6b7280 |
| duplicate | Cyan | #06b6d4 |
| approved | Teal | #14b8a6 |
| review | Yellow | #fbbf24 |
| blocked | Red (Dark) | #dc2626 |

## Examples

### Setup labels for a new project

```bash
#!/bin/bash
PROJECT_ID=1

# Create common labels
kanban label create --project $PROJECT_ID --name "bug" --color "#ef4444"
kanban label create --project $PROJECT_ID --name "feature" --color "#22c55e"
kanban label create --project $PROJECT_ID --name "documentation" --color "#3b82f6"
kanban label create --project $PROJECT_ID --name "enhancement" --color "#a78bfa"
kanban label create --project $PROJECT_ID --name "critical" --color "#f97316"
kanban label create --project $PROJECT_ID --name "wontfix" --color "#6b7280"
kanban label create --project $PROJECT_ID --name "duplicate" --color "#06b6d4"

echo "Label setup complete!"
kanban label list --project $PROJECT_ID
```

### Rename a label (delete and recreate)

```bash
#!/bin/bash
PROJECT_ID=1

# Find the label to rename
OLD_NAME="bug"
NEW_NAME="defect"
COLOR="#ef4444"

# Get label ID
LABEL_ID=$(kanban label list --project $PROJECT_ID --json | jq ".[] | select(.name == \"$OLD_NAME\") | .id")

if [ -n "$LABEL_ID" ]; then
  # Delete old label
  kanban label delete "$LABEL_ID"

  # Create new label with same color
  kanban label create --project $PROJECT_ID --name "$NEW_NAME" --color "$COLOR"

  echo "Renamed label from '$OLD_NAME' to '$NEW_NAME'"
else
  echo "Label '$OLD_NAME' not found"
fi
```

### List all labels across projects

```bash
#!/bin/bash
# Get all projects
kanban project list --json | jq -r '.[].id' | while read project_id; do
  echo "=== Project $project_id ==="
  kanban label list --project "$project_id"
  echo
done
```

## See Also

- [Issues Command](./issues.md)
- [Projects Command](./projects.md)
