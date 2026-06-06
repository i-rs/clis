import { useRef, useState, useCallback, useEffect } from 'react'
import { getToken, type ChatMessage, type ToolCallMsg, type TokenUsage, type ImageGeneratedEvent, type EvaluationEvent, type QualityScore, type ToolCallEvent } from '../api'

const BASE = window.location.pathname.replace(/\/$/, '') + '/api'

function authHeaders(): Record<string, string> {
  const token = getToken()
  if (!token) return {}
  return { Authorization: `Bearer ${token}` }
}

interface StreamingState {
  content: string
  reasoning: string
  toolCalls: ToolCallMsg[]
}

interface ChatStreamState {
  streaming: boolean
  state: StreamingState
  sendMessage: (text: string, agentId?: string) => Promise<void>
  resumeStream: (sessionId: string) => void
  abort: () => void
  lastCursor: number
}

interface Props {
  onCommit?: (msg: Pick<ChatMessage, 'role' | 'content' | 'reasoning' | 'toolCalls'>) => void
  onImageGenerated?: (image: ImageGeneratedEvent) => void
  onError?: (error: string) => void
  onDone?: (usage: TokenUsage | null, quality?: QualityScore | null) => void
  onEvaluation?: (evt: EvaluationEvent) => void
  onQualityScore?: (evt: QualityScore) => void
}

export function useChatStream(props: Props = {}): ChatStreamState {
  const [streaming, setStreaming] = useState(false)
  const [, setTick] = useState(0)
  const abortRef = useRef<AbortController | null>(null)
  const stateRef = useRef<StreamingState>({ content: '', reasoning: '', toolCalls: [] })
  const lastCursorRef = useRef(0)
  const lastSessionRef = useRef<string | null>(null)

  const abort = useCallback(() => {
    abortRef.current?.abort()
    setStreaming(false)
  }, [])

  useEffect(() => {
    return () => {
      abortRef.current?.abort()
    }
  }, [])

  const handleSend = useCallback(async (text: string, agentId?: string) => {
    abortRef.current?.abort()
    stateRef.current = { content: '', reasoning: '', toolCalls: [] }
    setTick((n) => n + 1)
    setStreaming(true)

    const     controller = new AbortController()
    abortRef.current = controller
    lastCursorRef.current = 0

    const body: Record<string, string> = { message: text }
    if (agentId) body.agent_id = agentId

    try {
      const resp = await fetch(`${BASE}/chat`, {
        method: 'POST',
        headers: { ...authHeaders(), 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
        signal: controller.signal,
      })

      if (!resp.ok) {
        const errText = await resp.text().catch(() => 'Unknown error')
        props.onError?.(`HTTP ${resp.status}: ${errText}`)
        setStreaming(false)
        return
      }

      const reader = resp.body?.getReader()
      if (!reader) {
        props.onError?.('No response body')
        setStreaming(false)
        return
      }

      const decoder = new TextDecoder()
      let buffer = ''
      let currentEvent = ''
      let currentId = ''

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
          } else if (trimmed.startsWith('id: ')) {
            currentId = trimmed.slice(4).trim()
          } else if (trimmed.startsWith('data: ')) {
            const data = trimmed.slice(6)
            const evtId = parseInt(currentId, 10) || 0
            if (evtId > 0) lastCursorRef.current = evtId
            switch (currentEvent) {
              case 'token':
                stateRef.current.content += data
                setTick((n) => n + 1)
                break
              case 'reasoning':
                stateRef.current.reasoning += data
                setTick((n) => n + 1)
                break
              case 'status':
                break
              case 'error':
                {
                  const s = stateRef.current
                  if (s.content || s.reasoning || s.toolCalls.length > 0) {
                    props.onCommit?.({
                      role: 'assistant',
                      content: s.content,
                      reasoning: s.reasoning || undefined,
                      toolCalls: s.toolCalls.length > 0 ? [...s.toolCalls] : undefined,
                    })
                  }
                  stateRef.current = { content: '', reasoning: '', toolCalls: [] }
                  props.onError?.(data)
                  setStreaming(false)
                }
                return
              case 'new_round':
                {
                  const s = stateRef.current
                  if (s.content || s.reasoning || s.toolCalls.length > 0) {
                    props.onCommit?.({
                      role: 'assistant',
                      content: s.content,
                      reasoning: s.reasoning || undefined,
                      toolCalls: s.toolCalls.length > 0 ? [...s.toolCalls] : undefined,
                    })
                  }
                  stateRef.current = { content: '', reasoning: '', toolCalls: [] }
                  setTick((n) => n + 1)
                }
                break
              case 'tool_executed':
                try {
                  const parsed = JSON.parse(data) as ToolCallEvent
                  stateRef.current.toolCalls.push({
                    name: parsed.name,
                    args: parsed.args,
                    result: parsed.result,
                    step: parsed.step,
                    total_steps: parsed.total_steps,
                  })
                  setTick((n) => n + 1)
                } catch { /* ignore parse errors */ }
                break
              case 'image_generated':
                {
                  const s = stateRef.current
                  if (s.content || s.reasoning || s.toolCalls.length > 0) {
                    props.onCommit?.({
                      role: 'assistant',
                      content: s.content,
                      reasoning: s.reasoning || undefined,
                      toolCalls: s.toolCalls.length > 0 ? [...s.toolCalls] : undefined,
                    })
                  }
                  stateRef.current = { content: '', reasoning: '', toolCalls: [] }
                  try {
                    const parsed = JSON.parse(data) as ImageGeneratedEvent
                    props.onImageGenerated?.(parsed)
                  } catch { /* ignore parse errors */ }
                  setTick((n) => n + 1)
                }
                break
              case 'done':
                {
                  const s = stateRef.current
                  if (s.content || s.reasoning || s.toolCalls.length > 0) {
                    props.onCommit?.({
                      role: 'assistant',
                      content: s.content,
                      reasoning: s.reasoning || undefined,
                      toolCalls: s.toolCalls.length > 0 ? [...s.toolCalls] : undefined,
                    })
                  }
                  stateRef.current = { content: '', reasoning: '', toolCalls: [] }
                  try {
                    const parsed = JSON.parse(data)
                    props.onDone?.(parsed.usage, parsed.quality)
                  } catch { /* ignore parse errors */ }
                  setStreaming(false)
                }
                return
              case 'evaluation':
                try {
                  const parsed = JSON.parse(data) as EvaluationEvent
                  props.onEvaluation?.(parsed)
                } catch { /* ignore parse errors */ }
                break
              case 'quality_score':
                try {
                  const parsed = JSON.parse(data) as QualityScore
                  props.onQualityScore?.(parsed)
                } catch { /* ignore parse errors */ }
                break
            }
          }
        }
      }
    } catch (err: any) {
      if (err.name !== 'AbortError') {
        const s = stateRef.current
        if (s.content || s.reasoning || s.toolCalls.length > 0) {
          props.onCommit?.({
            role: 'assistant',
            content: s.content,
            reasoning: s.reasoning || undefined,
            toolCalls: s.toolCalls.length > 0 ? [...s.toolCalls] : undefined,
          })
        }
        stateRef.current = { content: '', reasoning: '', toolCalls: [] }
        props.onError?.(String(err))
      }
      setStreaming(false)
    }
  }, [props.onCommit, props.onDone, props.onError, props.onEvaluation, props.onImageGenerated, props.onQualityScore])

  const resumeStream = useCallback((sessionId: string) => {
    if (!sessionId) return
    abortRef.current?.abort()

    lastSessionRef.current = sessionId
    const controller = new AbortController()
    abortRef.current = controller

    const cursor = lastCursorRef.current
    setStreaming(true)

    fetch(`${BASE}/chat/stream/${encodeURIComponent(sessionId)}/resume?cursor=${cursor}`, {
      headers: authHeaders(),
      signal: controller.signal,
    }).then(async (response) => {
      const reader = response.body?.getReader()
      if (!reader) return

      const decoder = new TextDecoder()
      let buffer = ''
      let currentEvent = ''
      let currentId = ''

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
          } else if (trimmed.startsWith('id: ')) {
            currentId = trimmed.slice(4).trim()
          } else if (trimmed.startsWith('data: ')) {
            const data = trimmed.slice(6)
            const evtId = parseInt(currentId, 10) || 0

            // Skip events already processed by this client
            if (evtId <= cursor) continue
            lastCursorRef.current = evtId

            switch (currentEvent) {
              case 'token':
                stateRef.current.content += data
                setTick((n) => n + 1)
                break
              case 'reasoning':
                stateRef.current.reasoning += data
                setTick((n) => n + 1)
                break
              case 'new_round':
                {
                  const s = stateRef.current
                  if (s.content || s.reasoning || s.toolCalls.length > 0) {
                    props.onCommit?.({
                      role: 'assistant',
                      content: s.content,
                      reasoning: s.reasoning || undefined,
                      toolCalls: s.toolCalls.length > 0 ? [...s.toolCalls] : undefined,
                    })
                  }
                  stateRef.current = { content: '', reasoning: '', toolCalls: [] }
                  setTick((n) => n + 1)
                }
                break
              case 'tool_executed':
                try {
                  const parsed = JSON.parse(data) as ToolCallEvent
                  stateRef.current.toolCalls.push({
                    name: parsed.name,
                    args: parsed.args,
                    result: parsed.result,
                    step: parsed.step,
                    total_steps: parsed.total_steps,
                  })
                  setTick((n) => n + 1)
                } catch { /* ignore */ }
                break
              case 'image_generated':
                {
                  const s = stateRef.current
                  if (s.content || s.reasoning || s.toolCalls.length > 0) {
                    props.onCommit?.({
                      role: 'assistant',
                      content: s.content,
                      reasoning: s.reasoning || undefined,
                      toolCalls: s.toolCalls.length > 0 ? [...s.toolCalls] : undefined,
                    })
                  }
                  stateRef.current = { content: '', reasoning: '', toolCalls: [] }
                  try {
                    const parsed = JSON.parse(data) as ImageGeneratedEvent
                    props.onImageGenerated?.(parsed)
                  } catch { /* ignore */ }
                  setTick((n) => n + 1)
                }
                break
              case 'done':
                {
                  const s = stateRef.current
                  if (s.content || s.reasoning || s.toolCalls.length > 0) {
                    props.onCommit?.({
                      role: 'assistant',
                      content: s.content,
                      reasoning: s.reasoning || undefined,
                      toolCalls: s.toolCalls.length > 0 ? [...s.toolCalls] : undefined,
                    })
                  }
                  stateRef.current = { content: '', reasoning: '', toolCalls: [] }
                  try {
                    const parsed = JSON.parse(data)
                    props.onDone?.(parsed.usage, parsed.quality)
                  } catch { /* ignore */ }
                  setStreaming(false)
                }
                return
              case 'error':
                {
                  const s = stateRef.current
                  if (s.content || s.reasoning || s.toolCalls.length > 0) {
                    props.onCommit?.({
                      role: 'assistant',
                      content: s.content,
                      reasoning: s.reasoning || undefined,
                      toolCalls: s.toolCalls.length > 0 ? [...s.toolCalls] : undefined,
                    })
                  }
                  stateRef.current = { content: '', reasoning: '', toolCalls: [] }
                  props.onError?.(data)
                  setStreaming(false)
                }
                return
              case 'evaluation':
                try {
                  const parsed = JSON.parse(data) as EvaluationEvent
                  props.onEvaluation?.(parsed)
                } catch { /* ignore */ }
                break
              case 'quality_score':
                try {
                  const parsed = JSON.parse(data) as QualityScore
                  props.onQualityScore?.(parsed)
                } catch { /* ignore */ }
                break
            }
          }
        }
      }
    }).catch((err: any) => {
      if (err.name !== 'AbortError') {
        props.onError?.(`Reconnect failed: ${err}`)
      }
      setStreaming(false)
    })
  }, [props.onCommit, props.onDone, props.onError, props.onEvaluation, props.onImageGenerated, props.onQualityScore])

  return {
    streaming,
    state: stateRef.current,
    sendMessage: handleSend,
    resumeStream,
    abort,
    lastCursor: lastCursorRef.current,
  }
}
