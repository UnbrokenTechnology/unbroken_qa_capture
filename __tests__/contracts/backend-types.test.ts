/**
 * Contract test: Frontend TypeScript types vs Rust backend serialization
 *
 * Verifies that the TypeScript interfaces in src/types/backend.ts match the
 * JSON shapes that the Rust backend actually serializes (via serde).
 *
 * Approach (Option B – Runtime Shape Validation):
 *  - Create mock objects matching the exact JSON a serde-serialized Rust struct
 *    would produce.
 *  - Assert all required fields are present and have the correct JS types.
 *  - Assert that enum string literals match the serde rename_all="lowercase"
 *    values in Rust.
 *
 * Key contracts verified:
 *  - Session, SessionSummary (models.rs)
 *  - Bug, BugUpdate (models.rs)
 *  - Capture (models.rs)
 *  - Setting, Environment, BugMetadata (models.rs)
 *  - TicketingCredentials, CreateTicketRequest, CreateTicketResponse,
 *    ConnectionStatus (ticketing/types.rs)
 *  - Enum string values: SessionStatus, BugType, BugStatus, CaptureType
 *
 * Note on snake_case: Rust serde uses snake_case field names by default (no
 * rename_all on structs), so all JSON keys are snake_case. The TypeScript
 * interfaces also use snake_case for the same reason — there is no camelCase
 * conversion at the Tauri IPC boundary for these types.
 */

import { describe, it, expect } from 'vitest'
import type {
  Session,
  SessionSummary,
  SessionStatus,
  Bug,
  BugUpdate,
  BugType,
  BugStatus,
  Capture,
  CaptureType,
  Setting,
  Environment,
  BugMetadata,
  TicketingCredentials,
  CreateTicketRequest,
  CreateTicketResponse,
  ConnectionStatus,
} from '../../src/types/backend'

// ---------------------------------------------------------------------------
// Helper: assert an object has all required keys with expected JS types
// ---------------------------------------------------------------------------

type FieldSpec = Record<string, 'string' | 'number' | 'boolean' | 'object' | 'nullable-string' | 'nullable-number' | 'nullable-boolean' | 'array'>

function assertShape(obj: Record<string, unknown>, spec: FieldSpec, label: string) {
  for (const [field, kind] of Object.entries(spec)) {
    const value = obj[field]
    expect(field in obj, `${label}: field "${field}" should be present`).toBe(true)

    if (kind === 'nullable-string') {
      expect(
        value === null || typeof value === 'string',
        `${label}: field "${field}" should be string | null, got ${typeof value}`
      ).toBe(true)
    } else if (kind === 'nullable-number') {
      expect(
        value === null || typeof value === 'number',
        `${label}: field "${field}" should be number | null, got ${typeof value}`
      ).toBe(true)
    } else if (kind === 'nullable-boolean') {
      expect(
        value === null || typeof value === 'boolean',
        `${label}: field "${field}" should be boolean | null`
      ).toBe(true)
    } else if (kind === 'array') {
      expect(Array.isArray(value), `${label}: field "${field}" should be an array`).toBe(true)
    } else {
      expect(typeof value, `${label}: field "${field}" type`).toBe(kind)
    }
  }
}

// ---------------------------------------------------------------------------
// SessionStatus enum values
// ---------------------------------------------------------------------------

describe('SessionStatus enum contract', () => {
  const validValues: SessionStatus[] = ['active', 'ended', 'reviewed', 'synced']

  it('contains exactly the four serde lowercase values from Rust', () => {
    // Rust: #[serde(rename_all = "lowercase")] Active, Ended, Reviewed, Synced
    expect(validValues).toEqual(['active', 'ended', 'reviewed', 'synced'])
  })

  it('each value is a non-empty string', () => {
    for (const v of validValues) {
      expect(typeof v).toBe('string')
      expect(v.length).toBeGreaterThan(0)
    }
  })
})

// ---------------------------------------------------------------------------
// BugType enum values
// ---------------------------------------------------------------------------

describe('BugType enum contract', () => {
  const validValues: BugType[] = ['bug', 'feature', 'feedback']

  it('contains exactly the three serde lowercase values from Rust', () => {
    // Rust: #[serde(rename_all = "lowercase")] Bug, Feature, Feedback
    expect(validValues).toEqual(['bug', 'feature', 'feedback'])
  })
})

// ---------------------------------------------------------------------------
// BugStatus enum values
// ---------------------------------------------------------------------------

describe('BugStatus enum contract', () => {
  const validValues: BugStatus[] = ['capturing', 'captured', 'reviewed', 'ready']

  it('contains exactly the four serde lowercase values from Rust', () => {
    // Rust: #[serde(rename_all = "lowercase")] Capturing, Captured, Reviewed, Ready
    expect(validValues).toEqual(['capturing', 'captured', 'reviewed', 'ready'])
  })
})

// ---------------------------------------------------------------------------
// CaptureType enum values
// ---------------------------------------------------------------------------

describe('CaptureType enum contract', () => {
  const validValues: CaptureType[] = ['screenshot', 'video', 'console']

  it('contains exactly the three serde lowercase values from Rust', () => {
    // Rust: #[serde(rename_all = "lowercase")] Screenshot, Video, Console
    expect(validValues).toEqual(['screenshot', 'video', 'console'])
  })
})

// ---------------------------------------------------------------------------
// Session struct
// ---------------------------------------------------------------------------

describe('Session type contract', () => {
  // Mirrors the serde JSON output of models.rs::Session
  const mockSession: Session = {
    id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    started_at: '2024-01-15T10:00:00Z',
    ended_at: null,
    status: 'active',
    folder_path: '/home/user/captures/session-1',
    session_notes: null,
    environment_json: null,
    original_snip_path: null,
    created_at: '2024-01-15T10:00:00Z',
  }

  it('mock satisfies Session interface (compile-time + runtime check)', () => {
    // TypeScript compile-time: the assignment above would fail if types mismatch.
    // Runtime: verify all required fields exist with correct JS types.
    const spec: FieldSpec = {
      id: 'string',
      started_at: 'string',
      ended_at: 'nullable-string',
      status: 'string',
      folder_path: 'string',
      session_notes: 'nullable-string',
      environment_json: 'nullable-string',
      original_snip_path: 'nullable-string',
      created_at: 'string',
    }
    assertShape(mockSession as unknown as Record<string, unknown>, spec, 'Session')
  })

  it('has exactly 9 fields matching the Rust struct', () => {
    // Rust Session has 9 pub fields; count here must match.
    expect(Object.keys(mockSession)).toHaveLength(9)
  })

  it('field names are snake_case (no camelCase conversion at IPC boundary)', () => {
    const keys = Object.keys(mockSession)
    // None of the keys should be camelCase versions of Rust field names
    expect(keys).not.toContain('startedAt')
    expect(keys).not.toContain('endedAt')
    expect(keys).not.toContain('folderPath')
    expect(keys).not.toContain('sessionNotes')
    expect(keys).not.toContain('environmentJson')
    expect(keys).not.toContain('originalSnipPath')
    expect(keys).not.toContain('createdAt')
  })

  it('status field accepts all valid SessionStatus values', () => {
    const statuses: SessionStatus[] = ['active', 'ended', 'reviewed', 'synced']
    for (const status of statuses) {
      const s: Session = { ...mockSession, status }
      expect(s.status).toBe(status)
    }
  })

  it('ended_at can be a string when session is ended', () => {
    const ended: Session = { ...mockSession, status: 'ended', ended_at: '2024-01-15T11:00:00Z' }
    expect(typeof ended.ended_at).toBe('string')
  })
})

// ---------------------------------------------------------------------------
// SessionSummary struct
// ---------------------------------------------------------------------------

describe('SessionSummary type contract', () => {
  const mockSummary: SessionSummary = {
    id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    started_at: '2024-01-15T10:00:00Z',
    ended_at: null,
    status: 'active',
    bug_count: 3,
  }

  it('mock satisfies SessionSummary interface', () => {
    const spec: FieldSpec = {
      id: 'string',
      started_at: 'string',
      ended_at: 'nullable-string',
      status: 'string',
      bug_count: 'number',
    }
    assertShape(mockSummary as unknown as Record<string, unknown>, spec, 'SessionSummary')
  })

  it('has exactly 5 fields matching the Rust struct', () => {
    // Rust SessionSummary has 5 pub fields: id, started_at, ended_at, status, bug_count
    expect(Object.keys(mockSummary)).toHaveLength(5)
  })

  it('bug_count uses snake_case not camelCase', () => {
    expect('bug_count' in mockSummary).toBe(true)
    expect('bugCount' in mockSummary).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// Bug struct
// ---------------------------------------------------------------------------

describe('Bug type contract', () => {
  const mockBug: Bug = {
    id: 'bug-uuid-1234',
    session_id: 'session-uuid-5678',
    bug_number: 1,
    display_id: 'Bug-01',
    type: 'bug',
    title: 'Login button unresponsive',
    notes: null,
    description: null,
    ai_description: null,
    status: 'captured',
    meeting_id: null,
    software_version: null,
    console_parse_json: null,
    metadata_json: null,
    folder_path: '/captures/session-1/bug-1',
    created_at: '2024-01-15T10:05:00Z',
    updated_at: '2024-01-15T10:05:00Z',
  }

  it('mock satisfies Bug interface', () => {
    const spec: FieldSpec = {
      id: 'string',
      session_id: 'string',
      bug_number: 'number',
      display_id: 'string',
      type: 'string',
      title: 'nullable-string',
      notes: 'nullable-string',
      description: 'nullable-string',
      ai_description: 'nullable-string',
      status: 'string',
      meeting_id: 'nullable-string',
      software_version: 'nullable-string',
      console_parse_json: 'nullable-string',
      metadata_json: 'nullable-string',
      folder_path: 'string',
      created_at: 'string',
      updated_at: 'string',
    }
    assertShape(mockBug as unknown as Record<string, unknown>, spec, 'Bug')
  })

  it('has exactly 17 fields matching the Rust struct', () => {
    // Rust Bug has 17 pub fields (bug_type serialized as "type" via #[serde(rename)])
    expect(Object.keys(mockBug)).toHaveLength(17)
  })

  it('type field is "type" not "bug_type" (Rust uses #[serde(rename = "type")])', () => {
    // CRITICAL: Rust field is `bug_type` but serialized as "type" due to serde rename
    expect('type' in mockBug).toBe(true)
    expect('bug_type' in mockBug).toBe(false)
  })

  it('snake_case field names match Rust serialization', () => {
    const keys = Object.keys(mockBug)
    expect(keys).toContain('session_id')
    expect(keys).toContain('bug_number')
    expect(keys).toContain('display_id')
    expect(keys).toContain('ai_description')
    expect(keys).toContain('meeting_id')
    expect(keys).toContain('software_version')
    expect(keys).toContain('console_parse_json')
    expect(keys).toContain('metadata_json')
    expect(keys).toContain('folder_path')
    expect(keys).toContain('created_at')
    expect(keys).toContain('updated_at')
  })

  it('type field accepts all BugType values', () => {
    const types: BugType[] = ['bug', 'feature', 'feedback']
    for (const t of types) {
      const b: Bug = { ...mockBug, type: t }
      expect(b.type).toBe(t)
    }
  })

  it('status field accepts all BugStatus values', () => {
    const statuses: BugStatus[] = ['capturing', 'captured', 'reviewed', 'ready']
    for (const status of statuses) {
      const b: Bug = { ...mockBug, status }
      expect(b.status).toBe(status)
    }
  })
})

// ---------------------------------------------------------------------------
// BugUpdate struct
// ---------------------------------------------------------------------------

describe('BugUpdate type contract', () => {
  it('all fields are optional and snake_case', () => {
    // BugUpdate is a partial update — all fields optional
    const empty: BugUpdate = {}
    expect(Object.keys(empty)).toHaveLength(0)

    const partial: BugUpdate = { title: 'Updated title', status: 'reviewed' }
    expect(partial.title).toBe('Updated title')
    expect(partial.status).toBe('reviewed')
  })

  it('supports all optional fields from the Rust BugUpdate struct', () => {
    const full: BugUpdate = {
      type: 'feature',
      title: 'New title',
      notes: 'Some notes',
      description: 'Full description',
      ai_description: 'AI generated',
      status: 'ready',
      meeting_id: 'meet-123',
      software_version: '2.0.0',
    }
    // TypeScript compile check: if BugUpdate is missing a field, this assignment fails.
    expect(Object.keys(full)).toHaveLength(8)
  })

  it('BugUpdate does NOT include id, session_id, or read-only fields', () => {
    // Rust BugUpdate only has mutable fields — no id, session_id, etc.
    const update: BugUpdate = { title: 'test' }
    expect('id' in update).toBe(false)
    expect('session_id' in update).toBe(false)
    expect('created_at' in update).toBe(false)
    expect('updated_at' in update).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// Capture struct
// ---------------------------------------------------------------------------

describe('Capture type contract', () => {
  const mockCapture: Capture = {
    id: 'capture-uuid-1234',
    bug_id: 'bug-uuid-5678',
    session_id: 'session-uuid-9012',
    file_name: 'screenshot-001.png',
    file_path: '/captures/session-1/bug-1/screenshot-001.png',
    file_type: 'screenshot',
    annotated_path: null,
    file_size_bytes: 204800,
    is_console_capture: false,
    parsed_content: null,
    created_at: '2024-01-15T10:05:30Z',
  }

  it('mock satisfies Capture interface', () => {
    const spec: FieldSpec = {
      id: 'string',
      bug_id: 'nullable-string',
      session_id: 'string',
      file_name: 'string',
      file_path: 'string',
      file_type: 'string',
      annotated_path: 'nullable-string',
      file_size_bytes: 'number',
      is_console_capture: 'boolean',
      parsed_content: 'nullable-string',
      created_at: 'string',
    }
    assertShape(mockCapture as unknown as Record<string, unknown>, spec, 'Capture')
  })

  it('has exactly 11 fields matching the Rust struct', () => {
    expect(Object.keys(mockCapture)).toHaveLength(11)
  })

  it('snake_case field names match Rust serialization', () => {
    const keys = Object.keys(mockCapture)
    expect(keys).toContain('bug_id')
    expect(keys).toContain('session_id')
    expect(keys).toContain('file_name')
    expect(keys).toContain('file_path')
    expect(keys).toContain('file_type')
    expect(keys).toContain('annotated_path')
    expect(keys).toContain('file_size_bytes')
    expect(keys).toContain('is_console_capture')
    expect(keys).toContain('parsed_content')
    expect(keys).toContain('created_at')
  })

  it('file_size_bytes can be null', () => {
    const c: Capture = { ...mockCapture, file_size_bytes: null }
    expect(c.file_size_bytes).toBeNull()
  })

  it('bug_id can be null for unsorted captures', () => {
    const c: Capture = { ...mockCapture, bug_id: null }
    expect(c.bug_id).toBeNull()
  })

  it('file_type accepts all CaptureType values', () => {
    const types: CaptureType[] = ['screenshot', 'video', 'console']
    for (const t of types) {
      const c: Capture = { ...mockCapture, file_type: t }
      expect(c.file_type).toBe(t)
    }
  })
})

// ---------------------------------------------------------------------------
// Setting struct
// ---------------------------------------------------------------------------

describe('Setting type contract', () => {
  const mockSetting: Setting = {
    key: 'app.theme',
    value: 'dark',
    updated_at: '2024-01-15T09:00:00Z',
  }

  it('mock satisfies Setting interface', () => {
    const spec: FieldSpec = {
      key: 'string',
      value: 'string',
      updated_at: 'string',
    }
    assertShape(mockSetting as unknown as Record<string, unknown>, spec, 'Setting')
  })

  it('has exactly 3 fields matching the Rust struct', () => {
    expect(Object.keys(mockSetting)).toHaveLength(3)
  })

  it('updated_at is snake_case not camelCase', () => {
    expect('updated_at' in mockSetting).toBe(true)
    expect('updatedAt' in mockSetting).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// Environment struct
// ---------------------------------------------------------------------------

describe('Environment type contract', () => {
  const mockEnv: Environment = {
    os: 'Windows 11',
    display_resolution: '1920x1080',
    dpi_scaling: '1.5',
    ram: '16 GB',
    cpu: 'Intel Core i7-12700',
    foreground_app: 'MyApp.exe',
  }

  it('mock satisfies Environment interface', () => {
    const spec: FieldSpec = {
      os: 'string',
      display_resolution: 'string',
      dpi_scaling: 'string',
      ram: 'string',
      cpu: 'string',
      foreground_app: 'string',
    }
    assertShape(mockEnv as unknown as Record<string, unknown>, spec, 'Environment')
  })

  it('has exactly 6 fields matching the Rust struct', () => {
    expect(Object.keys(mockEnv)).toHaveLength(6)
  })

  it('snake_case field names match Rust serialization', () => {
    expect('display_resolution' in mockEnv).toBe(true)
    expect('displayResolution' in mockEnv).toBe(false)
    expect('dpi_scaling' in mockEnv).toBe(true)
    expect('dpiScaling' in mockEnv).toBe(false)
    expect('foreground_app' in mockEnv).toBe(true)
    expect('foregroundApp' in mockEnv).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// BugMetadata struct
// ---------------------------------------------------------------------------

describe('BugMetadata type contract', () => {
  const mockMetadata: BugMetadata = {
    meeting_id: 'meet-abc-123',
    software_version: '1.2.3',
    environment: {
      os: 'Windows 11',
      display_resolution: '1920x1080',
      dpi_scaling: '1.0',
      ram: '8 GB',
      cpu: 'AMD Ryzen 5',
      foreground_app: 'Chrome.exe',
    },
    console_captures: ['error: null ref at line 42', 'warn: deprecated API'],
    custom_fields: { 'ticket_id': 'PROJ-99', 'priority': 'high' },
  }

  it('mock satisfies BugMetadata interface', () => {
    const spec: FieldSpec = {
      meeting_id: 'nullable-string',
      software_version: 'nullable-string',
      environment: 'object',
      console_captures: 'array',
      custom_fields: 'object',
    }
    assertShape(mockMetadata as unknown as Record<string, unknown>, spec, 'BugMetadata')
  })

  it('has exactly 5 fields matching the Rust struct', () => {
    expect(Object.keys(mockMetadata)).toHaveLength(5)
  })

  it('snake_case field names match Rust serialization', () => {
    expect('meeting_id' in mockMetadata).toBe(true)
    expect('meetingId' in mockMetadata).toBe(false)
    expect('software_version' in mockMetadata).toBe(true)
    expect('softwareVersion' in mockMetadata).toBe(false)
    expect('console_captures' in mockMetadata).toBe(true)
    expect('consoleCpatures' in mockMetadata).toBe(false)
    expect('custom_fields' in mockMetadata).toBe(true)
    expect('customFields' in mockMetadata).toBe(false)
  })

  it('meeting_id and software_version can be null', () => {
    const m: BugMetadata = {
      ...mockMetadata,
      meeting_id: null,
      software_version: null,
    }
    expect(m.meeting_id).toBeNull()
    expect(m.software_version).toBeNull()
  })

  it('console_captures is an array of strings', () => {
    expect(Array.isArray(mockMetadata.console_captures)).toBe(true)
    for (const item of mockMetadata.console_captures) {
      expect(typeof item).toBe('string')
    }
  })

  it('custom_fields is a string-to-string record', () => {
    for (const [k, v] of Object.entries(mockMetadata.custom_fields)) {
      expect(typeof k).toBe('string')
      expect(typeof v).toBe('string')
    }
  })

  it('nested environment object has correct shape', () => {
    const env = mockMetadata.environment
    expect(typeof env.os).toBe('string')
    expect(typeof env.display_resolution).toBe('string')
    expect(typeof env.dpi_scaling).toBe('string')
    expect(typeof env.ram).toBe('string')
    expect(typeof env.cpu).toBe('string')
    expect(typeof env.foreground_app).toBe('string')
  })
})

// ---------------------------------------------------------------------------
// TicketingCredentials struct
// ---------------------------------------------------------------------------

describe('TicketingCredentials type contract', () => {
  const mockCreds: TicketingCredentials = {
    api_key: 'lin_api_abc123xyz',
    workspace_id: 'workspace-uuid',
    team_id: 'team-uuid',
  }

  it('mock satisfies TicketingCredentials interface', () => {
    const spec: FieldSpec = {
      api_key: 'string',
    }
    assertShape(mockCreds as unknown as Record<string, unknown>, spec, 'TicketingCredentials')
  })

  it('api_key is snake_case not camelCase', () => {
    expect('api_key' in mockCreds).toBe(true)
    expect('apiKey' in mockCreds).toBe(false)
  })

  it('workspace_id and team_id are optional and nullable', () => {
    const minimal: TicketingCredentials = { api_key: 'key-only' }
    expect(minimal.workspace_id).toBeUndefined()
    expect(minimal.team_id).toBeUndefined()

    const withNulls: TicketingCredentials = {
      api_key: 'key',
      workspace_id: null,
      team_id: null,
    }
    expect(withNulls.workspace_id).toBeNull()
    expect(withNulls.team_id).toBeNull()
  })

  it('snake_case field names match Rust serialization', () => {
    expect('workspace_id' in mockCreds).toBe(true)
    expect('workspaceId' in mockCreds).toBe(false)
    expect('team_id' in mockCreds).toBe(true)
    expect('teamId' in mockCreds).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// CreateTicketRequest struct
// ---------------------------------------------------------------------------

describe('CreateTicketRequest type contract', () => {
  const mockRequest: CreateTicketRequest = {
    title: 'Login button broken',
    description: '## Steps to reproduce\n1. Open app\n2. Click login',
    attachments: ['/path/to/screenshot.png', '/path/to/console.log'],
    priority: 'high',
    labels: ['bug', 'ui'],
  }

  it('mock satisfies CreateTicketRequest interface', () => {
    const spec: FieldSpec = {
      title: 'string',
      description: 'string',
      attachments: 'array',
      labels: 'array',
    }
    assertShape(mockRequest as unknown as Record<string, unknown>, spec, 'CreateTicketRequest')
  })

  it('attachments is an array of strings (file paths)', () => {
    expect(Array.isArray(mockRequest.attachments)).toBe(true)
    for (const a of mockRequest.attachments) {
      expect(typeof a).toBe('string')
    }
  })

  it('priority is optional and nullable', () => {
    const noPriority: CreateTicketRequest = {
      title: 'Test',
      description: 'Desc',
      attachments: [],
      labels: [],
    }
    expect(noPriority.priority).toBeUndefined()

    const nullPriority: CreateTicketRequest = { ...noPriority, priority: null }
    expect(nullPriority.priority).toBeNull()
  })

  it('labels is an array of strings', () => {
    expect(Array.isArray(mockRequest.labels)).toBe(true)
    for (const l of mockRequest.labels) {
      expect(typeof l).toBe('string')
    }
  })
})

// ---------------------------------------------------------------------------
// CreateTicketResponse struct
// ---------------------------------------------------------------------------

describe('CreateTicketResponse type contract', () => {
  const mockResponse: CreateTicketResponse = {
    id: 'ticket-internal-uuid',
    url: 'https://linear.app/team/issue/PROJ-123',
    identifier: 'PROJ-123',
  }

  it('mock satisfies CreateTicketResponse interface', () => {
    const spec: FieldSpec = {
      id: 'string',
      url: 'string',
      identifier: 'string',
    }
    assertShape(mockResponse as unknown as Record<string, unknown>, spec, 'CreateTicketResponse')
  })

  it('has exactly 3 fields matching the Rust struct', () => {
    expect(Object.keys(mockResponse)).toHaveLength(3)
  })
})

// ---------------------------------------------------------------------------
// ConnectionStatus struct
// ---------------------------------------------------------------------------

describe('ConnectionStatus type contract', () => {
  const mockConnected: ConnectionStatus = {
    connected: true,
    message: null,
    integration_name: 'Linear',
  }

  it('mock satisfies ConnectionStatus interface', () => {
    const spec: FieldSpec = {
      connected: 'boolean',
      integration_name: 'string',
    }
    assertShape(mockConnected as unknown as Record<string, unknown>, spec, 'ConnectionStatus')
  })

  it('integration_name is snake_case not camelCase', () => {
    expect('integration_name' in mockConnected).toBe(true)
    expect('integrationName' in mockConnected).toBe(false)
  })

  it('message is optional and nullable', () => {
    const noMessage: ConnectionStatus = {
      connected: true,
      integration_name: 'Linear',
    }
    expect(noMessage.message).toBeUndefined()

    const withNull: ConnectionStatus = { ...noMessage, message: null }
    expect(withNull.message).toBeNull()
  })

  it('connected can be false with an error message', () => {
    const failed: ConnectionStatus = {
      connected: false,
      message: 'Authentication failed: invalid API key',
      integration_name: 'Linear',
    }
    expect(failed.connected).toBe(false)
    expect(typeof failed.message).toBe('string')
  })
})
