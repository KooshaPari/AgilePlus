# Service Controls

![Service controls demo](../../assets/gifs/service-controls-demo.gif)

## Overview

The service control panel provides a unified interface for managing all AgilePlus services. Operators can start, stop, restart, and monitor service health from a single dashboard view.

## Usage

### Accessing the Control Panel

1. Navigate to **Dashboard → Services** in the left sidebar
2. The services list shows all registered services with current status
3. Click any service card to expand control options

![Accessing controls](../../assets/gifs/service-controls-access.gif)

### Controlling a Service

1. Select a service from the list
2. Click the action button (Start, Stop, or Restart)
3. Confirm the action in the modal dialog
4. Watch for status updates in real-time

### Bulk Operations

Hold `Cmd` (macOS) or `Ctrl` (Linux/Windows) to select multiple services, then use the bulk action toolbar:

![Bulk operations](../../assets/gifs/service-controls-bulk.gif)

## Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `auto_restart` | boolean | `false` | Automatically restart service on crash |
| `restart_delay` | seconds | `5` | Delay before auto-restart |
| `max_memory` | MB | `512` | Memory limit before warning |
| `health_check_interval` | seconds | `30` | How often to check service health |

## API Reference

### GET /api/v1/services

List all services and their current status.

**Response:**
```json
{
  "services": [
    {
      "id": "agileplus-api",
      "name": "AgilePlus API",
      "status": "running",
      "pid": 12345,
      "uptime": 3600,
      "memory_mb": 128,
      "version": "2026.03B.0"
    }
  ]
}
```

### POST /api/v1/services/{id}/control

Control a service state.

**Request:**
```json
{
  "action": "start|stop|restart",
  "options": {
    "force": false,
    "timeout": 30
  }
}
```

**Response:**
```json
{
  "service_id": "agileplus-api",
  "previous_state": "stopped",
  "current_state": "starting",
  "requested_by": "user@example.com",
  "timestamp": "2026-04-04T12:00:00Z",
  "estimated_seconds": 5
}
```

## Real-time Updates

The control panel uses WebSocket connections for live status updates:

```javascript
// Example: Listen for service status changes
const ws = new WebSocket('wss://api.agileplus.local/v1/stream');
ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  console.log(`Service ${update.service_id} is now ${update.status}`);
};
```

## Troubleshooting

### Service Won't Start

1. Check logs: Click "View Logs" on the service card
2. Verify configuration: Ensure all required env vars are set
3. Check dependencies: Verify dependent services are running

![Troubleshooting](../../assets/gifs/service-controls-troubleshoot.gif)

### High Memory Usage

Services approaching their memory limit show an amber indicator:

1. Click the memory indicator for details
2. Consider increasing `max_memory` in settings
3. Review for memory leaks if consistently high

## Related

- [Dashboard Overview](./index.md)
- [Spec: kitty-specs/018-service-controls](../../kitty-specs/018-service-controls/spec.md)

## Changelog

| Version | Change |
|---------|--------|
| 2026.03B.0 | Initial release |
| 2026.04A.0 | Added bulk operations and memory monitoring |
