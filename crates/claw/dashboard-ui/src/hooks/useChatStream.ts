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

const INITIAL_STATE: StreamingState = { content: '', reasoning: '', toolCalls: [] }

function commitAndClear(
  st: StreamingState,
  onCommit?: (msg: Pick<ChatMessage, 'role' | 'content' | 'reasoning' | 'toolCalls'>) => void,
): StreamingState {
  if (st.content || st.reasoning || st.toolCalls.length > 0) {
    onCommit?.({
      role: 'assistant',
      content: st.content,
      reasoning: st.reasoning || undefined,
      toolCalls: st.toolCalls.length > 0 ? [...st.toolCalls] : undefined,
    })
  }
  return { ...INITIAL_STATE }
}

function parseSseLine(
  line: string,
  currentEvent: string,
  currentId: string,
): { event: string; id: string; data: string } | null {
  const trimmed = line.trim()
  if (trimmed.startsWith('event: ')) {
    return { event: trimmed.slice(7).trim(), id: currentId, data: '' }
  }
  if (trimmed.startsWith('id: ')) {
    return { event: currentEvent, id: trimmed.slice(4).trim(), data: '' }
  }
  if (trimmed.startsWith('data: ')) {
    return { event: currentEvent, id: currentId, data: trimmed.slice(6) }
  }
  return null
}

export function useChatStream(props: Props = {}): ChatStreamState {
  const [streaming, setStreaming] = useState(false)
  const [state, setState] = useState<StreamingState>(INITIAL_STATE)
  const abortRef = useRef<AbortController | null>(null)
  const lastCursorRef = useRef(0)

  const abort = useCallback(() => {
    abortRef.current?.abort()
    setStreaming(false)
  }, [])

  useEffect(() => {
    return () => {
      abortRef.current?.abort()
    }
  }, [])

  const processSse = useCallback((url: string, controller: AbortController, onComplete: () => void) => {
    fetch(url, {
      headers: authHeaders(),
      signal: controller.signal,
    }).then(async (response) => {
      const reader = response.body?.getReader()
      if (!reader) { onComplete(); return }

      const decoder = new TextDecoder()
      let buffer = ''
      let currentEvent = ''
      let currentId = ''

      setState(INITIAL_STATE)

      while (true) {
        const { done, value } = await reader.read()
        if (done) break

        buffer += decoder.decode(value, { stream: true })
        const lines = buffer.split('\n')
        buffer = lines.pop() || ''

        for (const line of lines) {
          const parsed = parseSseLine(line, currentEvent, currentId)
          if (!parsed) continue
          if (parsed.event) { currentEvent = parsed.event; currentId = parsed.id; continue }
          if (parsed.id) { currentId = parsed.id; continue }
          if (!parsed.data) continue

          const data = parsed.data
          const evtId = parseInt(currentId, 10) || 0
          if (evtId > 0) lastCursorRef.current = evtId

          switch (currentEvent) {
            case 'token':
              setState((prev) => ({ ...prev, content: prev.content + data }))
              break
            case 'reasoning':
              setState((prev) => ({ ...prev, reasoning: prev.reasoning + data }))
              break
            case 'new_round':
              setState((prev) => commitAndClear(prev, props.onCommit))
              break
            case 'tool_executed':
              try {
                const parsedEvt = JSON.parse(data) as ToolCallEvent
                setState((prev) => ({
                  ...prev,
                  toolCalls: [...prev.toolCalls, {
                    name: parsedEvt.name, args: parsedEvt.args,
                    result: parsedEvt.result, step: parsedEvt.step,
                    total_steps: parsedEvt.total_steps,
                  }],
                }))
              } catch { /* ignore */ }
              break
            case 'image_generated':
              setState((prev) => {
                const st = commitAndClear(prev, props.onCommit)
                try {
                  props.onImageGenerated?.(JSON.parse(data) as ImageGeneratedEvent)
                } catch { /* ignore */ }
                return st
              })
              break
            case 'done':
              try {
                const parsedDone = JSON.parse(data)
                setState((prev) => {
                  const st = commitAndClear(prev, props.onCommit)
                  props.onDone?.(parsedDone.usage, parsedDone.quality)
                  return st
                })
              } catch { /* ignore */ }
              setStreaming(false)
              return
            case 'error':
              setState((prev) => {
                const st = commitAndClear(prev, props.onCommit)
                props.onError?.(data)
                return st
              })
              setStreaming(false)
              return
            case 'evaluation':
              try {
                props.onEvaluation?.(JSON.parse(data) as EvaluationEvent)
              } catch { /* ignore */ }
              break
            case 'quality_score':
              try {
                props.onQualityScore?.(JSON.parse(data) as QualityScore)
              } catch { /* ignore */ }
              break
          }
        }
      }
      onComplete()
    }).catch((err: any) => {
      if (err.name !== 'AbortError') {
        setState((prev) => {
          const st = commitAndClear(prev, props.onCommit)
          props.onError?.(String(err))
          return st
        })
      }
      setStreaming(false)
    })
  }, [props.onCommit, props.onDone, props.onError, props.onEvaluation, props.onImageGenerated, props.onQualityScore])

  const handleSend = useCallback(async (text: string, agentId?: string) => {
    abortRef.current?.abort()

    const controller = new AbortController()
    abortRef.current = controller
    lastCursorRef.current = 0
    setStreaming(true)
    setState(INITIAL_STATE)

    const body: Record<string, string> = { message: text }
    if (agentId) body.agent_id = agentId

    processSse(`${BASE}/chat`, controller, () => {})
  }, [processSse])

  const resumeStream = useCallback((sessionId: string) => {
    if (!sessionId) return
    abortRef.current?.abort()

    const controller = new AbortController()
    abortRef.current = controller
    setStreaming(true)

    const cursor = lastCursorRef.current
    processSse(`${BASE}/chat/stream/${encodeURIComponent(sessionId)}/resume?cursor=${cursor}`, controller, () => {
      setStreaming(false)
    })
  }, [processSse])

  return {
    streaming,
    state,
    sendMessage: handleSend,
    resumeStream,
    abort,
    lastCursor: lastCursorRef.current,
  }
}
