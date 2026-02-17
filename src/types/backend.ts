/**
 * TypeScript interfaces matching Rust backend data models
 * Source: src-tauri/src/database/models.rs
 */

// Session types
export type SessionStatus = 'active' | 'ended' | 'reviewed' | 'synced'

export interface Session {
  id: string
  started_at: string
  ended_at: string | null
  status: SessionStatus
  folder_path: string
  session_notes: string | null
  environment_json: string | null
  original_snip_path: string | null
  created_at: string
}

export interface SessionSummary {
  id: string
  started_at: string
  ended_at: string | null
  status: SessionStatus
  bug_count: number
}

// Bug types
export type BugType = 'bug' | 'feature' | 'feedback'
export type BugStatus = 'capturing' | 'captured' | 'reviewed' | 'ready'

export interface Bug {
  id: string
  session_id: string
  bug_number: number
  display_id: string
  type: BugType
  title: string | null
  notes: string | null
  description: string | null
  ai_description: string | null
  status: BugStatus
  meeting_id: string | null
  software_version: string | null
  console_parse_json: string | null
  metadata_json: string | null
  folder_path: string
  created_at: string
  updated_at: string
}

export interface BugUpdate {
  type?: BugType
  title?: string
  notes?: string
  description?: string
  ai_description?: string
  status?: BugStatus
  meeting_id?: string
  software_version?: string
}

// Capture types
export type CaptureType = 'screenshot' | 'video' | 'console'

export interface Capture {
  id: string
  /** null when the capture was made with no active bug (stored in _unsorted/) */
  bug_id: string | null
  session_id: string
  file_name: string
  file_path: string
  file_type: CaptureType
  annotated_path: string | null
  file_size_bytes: number | null
  is_console_capture: boolean
  parsed_content: string | null
  created_at: string
}

// Settings types
export interface Setting {
  key: string
  value: string
  updated_at: string
}

// Environment metadata
export interface Environment {
  os: string
  display_resolution: string
  dpi_scaling: string
  ram: string
  cpu: string
  foreground_app: string
}

export interface BugMetadata {
  meeting_id: string | null
  software_version: string | null
  environment: Environment
  console_captures: string[]
  custom_fields: Record<string, string>
}

// Ticketing integration types
export interface TicketingCredentials {
  api_key: string
  workspace_id?: string | null
  team_id?: string | null
}

export interface CreateTicketRequest {
  title: string
  description: string
  attachments: string[]
  priority?: string | null
  labels: string[]
}

export interface CreateTicketResponse {
  id: string
  url: string
  identifier: string
}

export interface ConnectionStatus {
  connected: boolean
  message?: string | null
  integration_name: string
}
