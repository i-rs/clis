import { apiGet, apiJson } from './client'
import type { SessionMeta, CurrentSession } from './types'

export const listSessions = () => apiGet<SessionMeta[]>('/sessions')
export const getCurrentSession = () => apiGet<CurrentSession>('/sessions/current')
export const createSession = (agentId?: string) =>
  apiJson<{ id: string; title: string; message_count: number; agent_id: string }>('/sessions', 'POST', agentId ? { agent_id: agentId } : {})
export const switchSession = (id: string) =>
  apiJson<{ id: string; title: string | null; message_count: number }>(`/sessions/${encodeURIComponent(id)}/switch`, 'POST')
export const getSession = (id: string) =>
  apiGet<{ id: string; title: string; messages: { role: string; content: string; reasoning?: string }[]; agent_id?: string }>(`/sessions/${encodeURIComponent(id)}`)
export const deleteSession = (id: string) => apiJson<string>(`/sessions/${encodeURIComponent(id)}`, 'DELETE')
export const postFeedback = (sessionId: string, positive: boolean, message?: string) =>
  apiJson<string>(`/sessions/${encodeURIComponent(sessionId)}/feedback`, 'POST', { positive, ...(message ? { message } : {}) })
export const sendMessage = (message: string, agentId?: string) =>
  apiJson<{ session_id: string; status: string }>('/chat', 'POST', { message, ...(agentId ? { agent_id: agentId } : {}) })
