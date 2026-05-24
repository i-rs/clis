const BASE = window.location.pathname.replace(/\/$/, '') + '/api'

const TOKEN_KEY = 'claw-dashboard-token'

export function getToken(): string {
  const fromHash = window.location.hash.slice(1)
  if (fromHash) {
    localStorage.setItem(TOKEN_KEY, fromHash)
    window.location.hash = ''
    return fromHash
  }
  return localStorage.getItem(TOKEN_KEY) || ''
}

export function setToken(token: string) {
  localStorage.setItem(TOKEN_KEY, token)
}

export function hasToken(): boolean {
  return getToken().length > 0
}

function authHeaders(): Record<string, string> {
  const token = getToken()
  if (!token) return {}
  return { Authorization: `Bearer ${token}` }
}

async function authFetch(path: string, options?: RequestInit): Promise<Response> {
  return fetch(`${BASE}${path}`, {
    ...options,
    headers: {
      ...authHeaders(),
      ...(options?.headers || {}),
    },
  })
}

export interface SessionMeta {
  id: string
  title: string
  message_count: number
  created_at: number
  agent_id: string
}

export interface ToolSchema {
  type: string
  function: {
    name: string
    description: string
    parameters: Record<string, unknown>
  }
}

export interface PluginInfo {
  name: string
  version: string
  description: string
  author: string | null
  enabled: boolean
}

// ── Skills ──

export interface SkillInfo {
  name: string
  description: string
  parameters: Record<string, unknown> | null
  content: string
}

export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

// ── Health ──

export async function healthCheck(): Promise<ApiResponse<string>> {
  const res = await fetch(`${BASE}/health`)
  return res.json()
}

// ── Config ──

export async function getConfig(): Promise<ApiResponse<Record<string, unknown>>> {
  const res = await authFetch('/config')
  return res.json()
}

export interface AgentInfo {
  id: string
  provider: string
  model: string
  base_url: string
  tool_count: number
  enabled_tools: string[]
  system_prompt: string | null
}

// ── Sessions ──

export interface CurrentSession {
  id: string | null
  title: string | null
  message_count: number
  agent_id?: string | null
  messages: { role: string; content?: string; reasoning?: string; name?: string; args?: string; result?: string }[]
}

export async function getCurrentSession(): Promise<ApiResponse<CurrentSession>> {
  const res = await authFetch('/sessions/current')
  return res.json()
}

export async function listSessions(): Promise<ApiResponse<SessionMeta[]>> {
  const res = await authFetch('/sessions')
  return res.json()
}

export async function createSession(agentId?: string): Promise<ApiResponse<{ id: string; title: string; message_count: number; agent_id: string }>> {
  const body = agentId ? { agent_id: agentId } : {}
  const res = await authFetch('/sessions', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
  return res.json()
}

export async function switchSession(id: string): Promise<ApiResponse<{ id: string; title: string | null; message_count: number }>> {
  const res = await authFetch(`/sessions/${encodeURIComponent(id)}/switch`, { method: 'POST' })
  return res.json()
}

export async function getSession(id: string): Promise<ApiResponse<{ id: string; title: string; messages: { role: string; content: string; reasoning?: string }[]; agent_id?: string }>> {
  const res = await authFetch(`/sessions/${encodeURIComponent(id)}`)
  return res.json()
}

export async function deleteSession(id: string): Promise<ApiResponse<string>> {
  const res = await authFetch(`/sessions/${encodeURIComponent(id)}`, { method: 'DELETE' })
  return res.json()
}

// ── Tools ──

export async function listTools(): Promise<ApiResponse<ToolSchema[]>> {
  const res = await authFetch('/tools')
  return res.json()
}

// ── Plugins ──

export async function listPlugins(): Promise<ApiResponse<PluginInfo[]>> {
  const res = await authFetch('/plugins')
  return res.json()
}

// ── Skills ──

export async function listSkills(): Promise<ApiResponse<SkillInfo[]>> {
  const res = await authFetch('/skills')
  return res.json()
}

// ── Chat ──

export async function sendMessage(message: string, agentId?: string): Promise<ApiResponse<{ session_id: string; status: string }>> {
  const body: Record<string, unknown> = { message }
  if (agentId) {
    body.agent_id = agentId
  }
  const res = await authFetch('/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
  return res.json()
}

// ── Agents ──

export async function listAgents(): Promise<ApiResponse<AgentInfo[]>> {
  const res = await authFetch('/agents')
  return res.json()
}

export async function createAgent(body: Record<string, unknown>): Promise<ApiResponse<{ id: string; status: string }>> {
  const res = await authFetch('/agents', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
  return res.json()
}

export async function deleteAgent(id: string): Promise<ApiResponse<{ id: string; status: string }>> {
  const res = await authFetch(`/agents/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  })
  return res.json()
}

export async function getAgentConfig(id: string): Promise<ApiResponse<AgentInfo>> {
  const res = await authFetch(`/agents/${encodeURIComponent(id)}`)
  return res.json()
}

export async function updateAgent(id: string, body: Record<string, unknown>): Promise<ApiResponse<{ id: string; status: string }>> {
  const res = await authFetch(`/agents/${encodeURIComponent(id)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
  return res.json()
}

// ── Tool call event ──

export interface ToolCallEvent {
  name: string
  args: string
  result: string
  step: number
  total_steps: number
}

// ── SSE Chat Stream ──

export type SseEventHandler = {
  onToken?: (token: string) => void
  onReasoning?: (text: string) => void
  onStatus?: (text: string) => void
  onError?: (error: string) => void
  onDone?: (usage: TokenUsage | null) => void
  onNewRound?: () => void
  onToolExecuted?: (evt: ToolCallEvent) => void
}

// ── Token usage ──

export interface TokenUsage {
  prompt_tokens?: number
  completion_tokens?: number
  total_tokens?: number
}

// ── Chat message type ──

export interface ToolCallMsg {
  name: string
  args: string
  result: string
  step: number
  total_steps: number
}

export type ChatMessage = {
  role: 'user' | 'assistant' | 'error'
  content: string
  reasoning?: string
  toolCalls?: ToolCallMsg[]
}

// ── SSE stream parsing ──

export function streamChat(sessionId: string, handlers: SseEventHandler): AbortController {
  const controller = new AbortController()

  authFetch(`/chat/stream/${encodeURIComponent(sessionId)}`, {
    signal: controller.signal,
  }).then(async (response) => {
    const reader = response.body?.getReader()
    if (!reader) return

    const decoder = new TextDecoder()
    let buffer = ''
    let currentEvent = ''

    while (true) {
      const { done, value } = await reader.read()
      if (done) break

      buffer += decoder.decode(value, { stream: true })
      const lines = buffer.split('\n')
      buffer = lines.pop() || ''

      for (const line of lines) {
        const trimmed = line.trim()
        if (trimmed.startsWith('event: ')) {
          currentEvent = trimmed.slice(7).trim()
        } else if (trimmed.startsWith('data: ')) {
          const data = trimmed.slice(6)
          switch (currentEvent) {
            case 'token':
              handlers.onToken?.(data)
              break
            case 'reasoning':
              handlers.onReasoning?.(data)
              break
            case 'status':
              handlers.onStatus?.(data)
              break
            case 'error':
              handlers.onError?.(data)
              break
              case 'new_round':
              handlers.onNewRound?.()
              break
            case 'tool_executed':
              try {
                const parsed = JSON.parse(data)
                handlers.onToolExecuted?.(parsed as ToolCallEvent)
              } catch { /* ignore parse errors */ }
              break
          case 'done':
              console.log('[SSE] done event data:', data)
              try {
                const parsed = JSON.parse(data)
                console.log('[SSE] parsed.usage:', parsed.usage)
                handlers.onDone?.(parsed.usage)
              } catch (e) { console.error('[SSE] failed to parse done data:', data, e) }
              break
          }
        }
      }
    }
  }).catch((err) => {
    if (err.name !== 'AbortError') {
      handlers.onError?.(String(err))
    }
  })

  return controller
}
