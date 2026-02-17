# Ticketing Integration Module

This module provides a pluggable interface for creating tickets in external issue tracking systems.

## Architecture

The module is organized around the `TicketingIntegration` trait, which defines a standard interface that all integrations must implement.

### Components

- **`TicketingIntegration` trait** (`trait_def.rs`): Core interface defining authentication, ticket creation, and connection checking
- **`LinearIntegration`** (`linear.rs`): Implementation for Linear's GraphQL API
- **Types** (`types.rs`): Common types including errors, credentials, requests, and responses
- **Tests** (`tests.rs`): Comprehensive unit tests with mock integration

## Usage

### From Rust

```rust
use ticketing::{LinearIntegration, TicketingIntegration, TicketingCredentials, CreateTicketRequest};

// Create integration
let integration = LinearIntegration::new();

// Authenticate
let credentials = TicketingCredentials {
    api_key: "lin_api_xxxxxxxx".to_string(),
    team_id: Some("team-id-here".to_string()),
    workspace_id: None,
};
integration.authenticate(&credentials)?;

// Create a ticket
let request = CreateTicketRequest {
    title: "Bug: Application crashes on startup".to_string(),
    description: "Steps to reproduce:\n1. Open app\n2. Click start\n3. Crash".to_string(),
    attachments: vec!["/path/to/screenshot.png".to_string()],
    priority: Some("1".to_string()),
    labels: vec!["bug".to_string()],
};

let response = integration.create_ticket(&request)?;
println!("Created ticket: {} at {}", response.identifier, response.url);
```

### From Frontend (via Tauri Commands)

The module exposes five Tauri commands:

1. **`ticketing_authenticate`**: Authenticate with API credentials
2. **`ticketing_create_ticket`**: Create a new ticket
3. **`ticketing_check_connection`**: Verify connection status
4. **`ticketing_get_credentials`**: Retrieve stored credentials from settings
5. **`ticketing_save_credentials`**: Save credentials to settings database

Example frontend usage:

```typescript
import { invoke } from '@tauri-apps/api/core';

// Authenticate
await invoke('ticketing_authenticate', {
  credentials: {
    api_key: 'lin_api_xxxxxxxx',
    team_id: 'team-id-here',
    workspace_id: null
  }
});

// Create ticket
const response = await invoke('ticketing_create_ticket', {
  request: {
    title: 'Bug Report',
    description: 'Description here',
    attachments: ['/path/to/file.png'],
    priority: '1',
    labels: ['bug']
  }
});

console.log('Ticket created:', response.identifier, response.url);
```

## Linear Integration

### Setup

1. Create a Linear API key at: https://linear.app/settings/api
2. Find your team ID:
   - Navigate to your team's settings in Linear
   - The team ID is in the URL: `linear.app/team/{TEAM_ID}/settings`
3. Store credentials via `ticketing_save_credentials` or authenticate directly

### API Details

The Linear integration uses GraphQL mutations and queries:

- **Authentication**: Validates API key by querying viewer info
- **Ticket Creation**: Uses `IssueCreate` mutation with title, description, team, priority, and labels
- **Connection Check**: Verifies API key is still valid

### Attachment Upload

Screenshots are uploaded to Linear using a three-step process:

1. **Request upload URL** — `fileUpload` GraphQL mutation returns a pre-signed S3 upload URL, a permanent asset URL, and required auth headers.
2. **Upload file bytes** — PUT the raw file bytes to the S3 URL with `Content-Type`, `Cache-Control`, and any Linear-provided auth headers.
3. **Embed in description** — The permanent asset URL is embedded as a markdown image (`![Screenshot N](assetUrl)`) in the issue description under a `## Screenshots` heading.

Upload failures are graceful: the issue is still created with a note listing which screenshots could not be uploaded. Each `CreateTicketResponse` includes `attachment_results` with per-file success/failure details for the frontend to display.

### Limitations

- Requires team ID to be configured
- Priority must be a number (0-4, where 0=No priority, 1=Urgent, 2=High, 3=Normal, 4=Low)

## Credential Storage

Credentials are stored securely in the settings database with the following keys:

- `ticketing.api_key`: The API key/token
- `ticketing.team_id`: Team ID (for Linear)
- `ticketing.workspace_id`: Workspace/organization ID (optional)

The settings database uses SQLite with the following schema:

```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

## Adding New Integrations

To add support for a new ticketing system (e.g., Jira, GitHub Issues):

1. Create a new file (e.g., `jira.rs`)
2. Implement the `TicketingIntegration` trait
3. Export the integration in `mod.rs`
4. Update the global `TICKETING_INTEGRATION` initialization in `lib.rs` to allow runtime selection

Example skeleton:

```rust
pub struct JiraIntegration {
    credentials: Arc<RwLock<Option<TicketingCredentials>>>,
    api_endpoint: String,
}

impl TicketingIntegration for JiraIntegration {
    fn authenticate(&self, credentials: &TicketingCredentials) -> TicketingResult<()> {
        // Implement Jira authentication
    }

    fn create_ticket(&self, request: &CreateTicketRequest) -> TicketingResult<CreateTicketResponse> {
        // Implement Jira ticket creation
    }

    fn check_connection(&self) -> TicketingResult<ConnectionStatus> {
        // Implement Jira connection check
    }

    fn name(&self) -> &str {
        "Jira"
    }
}
```

## Testing

The module includes comprehensive unit tests using a mock integration:

```bash
cargo test ticketing --lib
```

Tests cover:
- Authentication (success and failure cases)
- Ticket creation with various configurations
- Connection status checking
- Error handling
- Mock integration for testing without real API calls

## Error Handling

The module defines a `TicketingError` enum with variants for different error cases:

- `AuthenticationFailed`: Invalid or expired credentials
- `NetworkError`: Connection issues or HTTP errors
- `InvalidConfig`: Missing required configuration
- `CreationFailed`: Ticket creation failed
- `ConnectionFailed`: Connection check failed

All errors implement `Display` and `Error` traits and can be serialized to JSON for frontend consumption.

## Security Considerations

- API keys are stored in the local SQLite database (consider encryption in production)
- Credentials are kept in memory during the session in `Arc<RwLock<>>` for thread safety
- The `Send + Sync` bounds on the trait ensure thread-safe usage
- Never log or expose API keys in error messages

## Future Enhancements

1. **Batch operations**: Support creating multiple tickets at once
2. **Ticket updates**: Add methods to update existing tickets
3. **Comments**: Add ability to post comments on tickets
4. **Webhook support**: Allow Linear to notify the app of ticket updates
5. **Integration selection UI**: Let users choose between Linear/Jira/GitHub at runtime
6. **Credential encryption**: Encrypt API keys in the database
7. **Offline queue**: Queue ticket creation when offline and sync when connected
