---
description: Open the AgilePlus dashboard in your browser.
---

## Dashboard Access

This command launches the AgilePlus dashboard in your browser using the agileplus CLI.

## What to do

Simply run the `agileplus dashboard` command to:
- Start the dashboard if it's not already running
- Open it in your default web browser
- Display the dashboard URL

If you need to stop the dashboard, you can use `agileplus dashboard --kill`.

## Implementation

Execute the following terminal command:

```bash
agileplus dashboard
```

## Additional Options

- To specify a preferred port: `agileplus dashboard --port 8080`
- To stop the dashboard: `agileplus dashboard --kill`

## Success Criteria

- User sees the dashboard URL clearly displayed
- Browser opens automatically to the dashboard
- If browser doesn't open, user gets clear instructions
- Error messages are helpful and actionable