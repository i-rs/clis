const BASE = '/api'

export interface SessionMeta {
  id: string
  title: string
  message_count: number
  created_at: number
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
  const res = await fetch(`${BASE}/config`)
  return res.json()
}

// ── Sessions ──

export interface CurrentSession {
  id: string | null
  title: string | null
  message_count: number
  messages: { role: string; content?: string; name?: string; args?: string; result?: string }[]
}

export async function getCurrentSession(): Promise<ApiResponse<CurrentSession>> {
  const res = await fetch(`${BASE}/sessions/current`)
  return res.json()
}

export async function listSessions(): Promise<ApiResponse<SessionMeta[]>> {
  const res = await fetch(`${BASE}/sessions`)
  return res.json()
}

export async function createSession(): Promise<ApiResponse<{ id: string; title: string; message_count: number }>> {
  const res = await fetch(`${BASE}/sessions`, { method: 'POST' })
  return res.json()
}

export async function switchSession(id: string): Promise<ApiResponse<{ id: string; title: string | null; message_count: number }>> {
  const res = await fetch(`${BASE}/sessions/${encodeURIComponent(id)}/switch`, { method: 'POST' })
  return res.json()
}

export async function getSession(id: string): Promise<ApiResponse<{ id: string; title: string; messages: { role: string; content: string }[] }>> {
  const res = await fetch(`${BASE}/sessions/${encodeURIComponent(id)}`)
  return res.json()
}

export async function deleteSession(id: string): Promise<ApiResponse<string>> {
  const res = await fetch(`${BASE}/sessions/${encodeURIComponent(id)}`, { method: 'DELETE' })
  return res.json()
}

// ── Tools ──

export async function listTools(): Promise<ApiResponse<ToolSchema[]>> {
  const res = await fetch(`${BASE}/tools`)
  return res.json()
}

// ── Plugins ──

export async function listPlugins(): Promise<ApiResponse<PluginInfo[]>> {
  const res = await fetch(`${BASE}/plugins`)
  return res.json()
}

// ── Chat ──

export async function sendMessage(message: string): Promise<ApiResponse<{ session_id: string; status: string }>> {
  const res = await fetch(`${BASE}/chat`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ message }),
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
  onDone?: (usage: unknown) => void
  onNewRound?: () => void
  onToolExecuted?: (evt: ToolCallEvent) => void
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

  fetch(`${BASE}/chat/stream/${encodeURIComponent(sessionId)}`, {
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
              try {
                const parsed = JSON.parse(data)
                handlers.onDone?.(parsed.usage)
              } catch { /* ignore parse errors */ }
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
