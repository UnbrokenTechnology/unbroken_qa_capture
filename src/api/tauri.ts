/**
 * Type-safe Tauri IPC invoke wrapper
 *
 * This module provides typed wrappers around Tauri commands defined in src-tauri/src/lib.rs
 * All functions return Promise types and handle errors appropriately.
 */

import { invoke } from '@tauri-apps/api/core'
import type { Bug, BugUpdate, Session, SessionSummary, Setting } from '../types/backend'

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

// Settings operations
export async function getSetting(key: string): Promise<string | null> {
  // TODO: Implement when backend command is available
  return await invoke<string | null>('get_setting', { key })
}

export async function setSetting(key: string, value: string): Promise<void> {
  // TODO: Implement when backend command is available
  await invoke('set_setting', { key, value })
}

export async function getAllSettings(): Promise<Setting[]> {
  // TODO: Implement when backend command is available
  return await invoke<Setting[]>('get_all_settings')
}

export async function deleteSetting(key: string): Promise<void> {
  // TODO: Implement when backend command is available
  await invoke('delete_setting', { key })
}
