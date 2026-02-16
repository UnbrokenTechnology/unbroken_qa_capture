/**
 * Type-safe Tauri IPC invoke wrapper
 *
 * This module provides typed wrappers around Tauri commands defined in src-tauri/src/lib.rs
 * All functions return Promise types and handle errors appropriately.
 */

import { invoke } from '@tauri-apps/api/core'
import type {
  Bug,
  BugUpdate,
  Session,
  SessionSummary,
  Setting,
  Capture,
  TicketingCredentials,
  CreateTicketRequest,
  CreateTicketResponse,
  ConnectionStatus
} from '../types/backend'

// ============================================================================
// Template Commands
// ============================================================================

export async function greet(name: string): Promise<string> {
  return await invoke<string>('greet', { name })
}

export async function setCustomTemplatePath(path: string | null): Promise<void> {
  await invoke('set_custom_template_path', { path })
}

export async function renderBugTemplate(bugData: unknown): Promise<string> {
  return await invoke<string>('render_bug_template', { bugData })
}

export async function reloadTemplate(): Promise<void> {
  await invoke('reload_template')
}

// ============================================================================
// Bug Commands
// ============================================================================

export async function copyBugToClipboard(folderPath: string): Promise<void> {
  await invoke('copy_bug_to_clipboard', { folderPath })
}

export async function openBugFolder(folderPath: string): Promise<void> {
  await invoke('open_bug_folder', { folderPath })
}

// ============================================================================
// Session Commands
// ============================================================================

export async function openSessionFolder(folderPath: string): Promise<void> {
  await invoke('open_session_folder', { folderPath })
}

// ============================================================================
// Tray Commands
// ============================================================================

export async function updateTrayIcon(state: 'idle' | 'active' | 'bug' | 'review'): Promise<void> {
  await invoke('update_tray_icon', { state })
}

export async function updateTrayTooltip(tooltip: string): Promise<void> {
  await invoke('update_tray_tooltip', { tooltip })
}

// ============================================================================
// Database Commands (placeholders for future backend implementation)
// ============================================================================

// Session operations
export async function createSession(session: Partial<Session>): Promise<Session> {
  // TODO: Implement when backend command is available
  return await invoke<Session>('create_session', { session })
}

export async function getSession(id: string): Promise<Session | null> {
  // TODO: Implement when backend command is available
  return await invoke<Session | null>('get_session', { id })
}

export async function updateSession(session: Session): Promise<void> {
  // TODO: Implement when backend command is available
  await invoke('update_session', { session })
}

export async function deleteSession(id: string): Promise<void> {
  // TODO: Implement when backend command is available
  await invoke('delete_session', { id })
}

export async function listSessions(): Promise<Session[]> {
  // TODO: Implement when backend command is available
  return await invoke<Session[]>('list_sessions')
}

export async function getActiveSession(): Promise<Session | null> {
  // TODO: Implement when backend command is available
  return await invoke<Session | null>('get_active_session')
}

export async function getSessionSummaries(): Promise<SessionSummary[]> {
  // TODO: Implement when backend command is available
  return await invoke<SessionSummary[]>('get_session_summaries')
}

export async function updateSessionStatus(id: string, status: string): Promise<void> {
  // TODO: Implement when backend command is available
  await invoke('update_session_status', { id, status })
}

// Bug operations
export async function createBug(bug: Partial<Bug>): Promise<Bug> {
  // TODO: Implement when backend command is available
  return await invoke<Bug>('create_bug', { bug })
}

export async function getBug(id: string): Promise<Bug | null> {
  // TODO: Implement when backend command is available
  return await invoke<Bug | null>('get_bug', { id })
}

export async function updateBug(id: string, update: BugUpdate): Promise<void> {
  // TODO: Implement when backend command is available
  await invoke('update_bug', { id, update })
}

export async function deleteBug(id: string): Promise<void> {
  // TODO: Implement when backend command is available
  await invoke('delete_bug', { id })
}

export async function listBugs(sessionId?: string): Promise<Bug[]> {
  // TODO: Implement when backend command is available
  return await invoke<Bug[]>('list_bugs', { sessionId })
}

export async function getBugsBySession(sessionId: string): Promise<Bug[]> {
  // TODO: Implement when backend command is available
  return await invoke<Bug[]>('get_bugs_by_session', { sessionId })
}

export async function getBugNotes(bugId: string, folderPath: string): Promise<string> {
  return await invoke<string>('get_bug_notes', { bugId, folderPath })
}

export async function updateBugNotes(
  bugId: string,
  folderPath: string,
  notes: string
): Promise<void> {
  await invoke('update_bug_notes', { bugId, folderPath, notes })
}

export async function getSessionNotes(sessionId: string, folderPath: string): Promise<string> {
  return await invoke<string>('get_session_notes', { sessionId, folderPath })
}

export async function updateSessionNotes(
  sessionId: string,
  folderPath: string,
  notes: string
): Promise<void> {
  await invoke('update_session_notes', { sessionId, folderPath, notes })
}

// Capture operations
export async function getBugCaptures(bugId: string): Promise<Capture[]> {
  return await invoke<Capture[]>('get_bug_captures', { bugId })
}

export async function updateCaptureConsoleFlag(captureId: string, isConsoleCapture: boolean): Promise<void> {
  await invoke('update_capture_console_flag', { captureId, isConsoleCapture })
}

// Settings operations
export async function getSetting(key: string): Promise<string | null> {
  return await invoke<string | null>('get_setting', { key })
}

export async function setSetting(key: string, value: string): Promise<void> {
  await invoke('set_setting', { key, value })
}

export async function getAllSettings(): Promise<Setting[]> {
  return await invoke<Setting[]>('get_all_settings')
}

export async function deleteSetting(key: string): Promise<void> {
  await invoke('delete_setting', { key })
}

// Setup operations
export async function hasCompletedSetup(): Promise<boolean> {
  return await invoke<boolean>('has_completed_setup')
}

export async function markSetupComplete(): Promise<void> {
  await invoke('mark_setup_complete')
}

export async function resetSetup(): Promise<void> {
  await invoke('reset_setup')
}

// ============================================================================
// Ticketing Integration Commands
// ============================================================================

export async function ticketingAuthenticate(credentials: TicketingCredentials): Promise<void> {
  await invoke('ticketing_authenticate', { credentials })
}

export async function ticketingCreateTicket(
  request: CreateTicketRequest
): Promise<CreateTicketResponse> {
  return await invoke<CreateTicketResponse>('ticketing_create_ticket', { request })
}

export async function ticketingCheckConnection(): Promise<ConnectionStatus> {
  return await invoke<ConnectionStatus>('ticketing_check_connection')
}

export async function ticketingGetCredentials(): Promise<TicketingCredentials | null> {
  return await invoke<TicketingCredentials | null>('ticketing_get_credentials')
}

export async function ticketingSaveCredentials(credentials: TicketingCredentials): Promise<void> {
  await invoke('ticketing_save_credentials', { credentials })
}

// ============================================================================
// Claude CLI Commands
// ============================================================================

export interface ClaudeStatus {
  Ready?: { version: string }
  NotAuthenticated?: { version: string; message: string }
  NotInstalled?: { message: string }
}

export interface BugContext {
  bug_id: string
  bug_type: string
  notes?: string
  screenshot_paths: string[]
  metadata?: Record<string, unknown>
}

export interface ClaudeResponse {
  text: string
  task: string
  bug_id?: string
}

export async function getClaudeStatus(): Promise<ClaudeStatus> {
  return await invoke<ClaudeStatus>('get_claude_status')
}

export async function refreshClaudeStatus(): Promise<ClaudeStatus> {
  return await invoke<ClaudeStatus>('refresh_claude_status')
}

export async function generateBugDescription(bugContext: BugContext): Promise<ClaudeResponse> {
  return await invoke<ClaudeResponse>('generate_bug_description', { bugContext })
}

export async function refineBugDescription(
  currentDescription: string,
  refinementInstructions: string,
  bugId: string
): Promise<ClaudeResponse> {
  return await invoke<ClaudeResponse>('refine_bug_description', {
    currentDescription,
    refinementInstructions,
    bugId
  })
}

export async function saveBugDescription(
  folderPath: string,
  description: string
): Promise<void> {
  await invoke('save_bug_description', { folderPath, description })
}
