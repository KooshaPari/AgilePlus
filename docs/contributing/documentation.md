# Writing Feature Documentation

Every feature in AgilePlus must have a corresponding documentation page. This guide explains the structure and requirements.

## When to Write Documentation

Documentation is **required** before a feature PR can be merged:

- New features → New documentation page
- Feature updates → Update existing page
- Bug fixes → Update if behavior changes
- Refactors → No change unless behavior changes

## Documentation Structure

Create your page at `docs/<category>/<feature-id>.md`:

```
docs/
├── dashboard/
│   ├── service-controls.md
│   └── kanban-board.md
├── cli/
│   ├── status-command.md
│   └── init-command.md
└── api/
    └── work-packages.md
```

## Required Sections

Every feature documentation page must include:

### 1. Visual Demo (REQUIRED)

Embed a GIF or screenshot at the top:

```markdown
# Feature Name

![Feature demo](../../assets/gifs/feature-demo.gif)
```

### 2. Overview

Brief description of what the feature does:

```markdown
## Overview

The service control panel allows operators to start, stop, and restart 
services directly from the dashboard with real-time status feedback.
```

### 3. Usage

Step-by-step instructions:

```markdown
## Usage

1. Navigate to **Dashboard → Services**
2. Click the service you want to control
3. Select an action: Start, Stop, or Restart
4. Confirm the action in the modal

![Usage walkthrough](../../assets/gifs/feature-usage.gif)
```

### 4. Configuration (if applicable)

Document configuration options:

```markdown
## Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `auto_restart` | bool | false | Auto-restart on crash |
| `max_memory` | MB | 512 | Memory limit for service |
```

### 5. API Reference (if applicable)

Document API endpoints:

```markdown
## API Reference

### POST /api/v1/services/{id}/control

Control a service state.

**Request:**
```json
{
  "action": "start|stop|restart"
}
```

**Response:**
```json
{
  "service_id": "svc-123",
  "previous_state": "stopped",
  "current_state": "starting",
  "timestamp": "2026-04-04T12:00:00Z"
}
```
```

### 6. Related Features

Link to related documentation:

```markdown
## Related

- [Dashboard Overview](./index.md)
- [Service Logs](./service-logs.md)
- [Spec: kitty-specs/018-service-controls](../../kitty-specs/018-service-controls/spec.md)
```

## Creating Visual Assets

### GIF Recording

**Recommended Tools:**
- **Kap** (macOS, free) - Best for quick screen recordings
- **CleanShot X** (macOS, paid) - Professional with annotations
- **vhs** (CLI, free) - Code-based terminal recordings
- **LICEcap** (cross-platform, free) - Simple and lightweight

**Guidelines:**
- Keep GIFs under 10 seconds when possible
- Focus on the specific feature (not entire workflows)
- Use consistent window sizes (1200x800 recommended)
- Save to `docs/assets/gifs/<feature>.gif`

### Screenshots

- Use browser DevTools device toolbar for consistent sizing
- Annotate with arrows/circles for clarity
- Save to `docs/assets/screenshots/<feature>.png`

### Terminal Recordings

Use `vhs` for terminal recordings:

```bash
# Install vhs
brew install charmbracelet/tap/vhs

# Create tape file
cat > demo.tape << 'EOF'
Type "agileplus status"
Sleep 500ms
Enter
Sleep 2s
EOF

# Generate GIF
vhs < demo.tape > docs/assets/gifs/status-command.gif
```

## Linking in VitePress

Add your page to the sidebar in `docs/.vitepress/config.mts`:

```typescript
sidebar: {
  '/dashboard/': [
    {
      text: 'Dashboard',
      items: [
        { text: 'Overview', link: '/dashboard/' },
        { text: 'Service Controls', link: '/dashboard/service-controls' },
        { text: 'Kanban Board', link: '/dashboard/kanban-board' },
      ]
    }
  ]
}
```

## Changelog Integration

When adding a changelog entry, include visual references:

```markdown
- **dashboard**: Add service control panel (#200)
  ![service-controls](../../assets/gifs/service-controls.gif)
  [Documentation →](../docs/dashboard/service-controls.md)
```

This creates:
1. Visual preview of the feature
2. Link to detailed documentation
3. Traceable from changelog to spec to code

## Checklist

Before submitting your documentation PR:

- [ ] Page created at correct path
- [ ] Visual demo (GIF/screenshot) embedded
- [ ] Overview section explains purpose
- [ ] Usage instructions are clear
- [ ] API reference included (if applicable)
- [ ] Related features linked
- [ ] Added to VitePress sidebar
- [ ] Spelling and grammar checked
- [ ] All images load correctly

## Example

See [Service Controls Documentation](../dashboard/service-controls.md) for a complete example.

## Related

- [PR Requirements Policy](../../GOVERNANCE_PR_REQUIREMENTS.md)
- [Spec Format](./specs.md)
