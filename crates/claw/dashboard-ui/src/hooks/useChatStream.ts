import { useRef, useState, useCallback, useEffect } from 'react'
import {
  sendMessage,
  streamChat,
  type ChatMessage,
  type ToolCallMsg,
  type TokenUsage,
  type ImageGeneratedEvent,
  type EvaluationEvent,
  type QualityScore,
} from '../api'

interface StreamingState {
  content: string
  reasoning: string
  toolCalls: ToolCallMsg[]
}

interface ChatStreamState {
  streaming: boolean
  state: StreamingState
  sendMessage: (text: string, agentId?: string) => Promise<void>
  abort: () => void
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

    try {
      const resp = await sendMessage(text, agentId)
      if (!resp.success || !resp.data) {
        props.onError?.(resp.error || 'Failed to send message')
        setStreaming(false)
        return
      }

      const controller = streamChat(resp.data.session_id, {
        onReasoning: (reasoningText) => {
          stateRef.current.reasoning += reasoningText
          setTick((n) => n + 1)
        },
        onToken: (token) => {
          stateRef.current.content += token
          setTick((n) => n + 1)
        },
        onNewRound: () => {
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
        },
        onToolExecuted: (evt) => {
          stateRef.current.toolCalls.push({
            name: evt.name,
            args: evt.args,
            result: evt.result,
            step: evt.step,
            total_steps: evt.total_steps,
          })
          setTick((n) => n + 1)
        },
        onImageGenerated: (evt) => {
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
          props.onImageGenerated?.(evt)
          setTick((n) => n + 1)
        },
        onError: (error) => {
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
          props.onError?.(error)
          setStreaming(false)
        },
        onDone: (usage, quality) => {
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
          props.onDone?.(usage, quality)
          setStreaming(false)
        },
        onEvaluation: (evt) => {
          props.onEvaluation?.(evt)
        },
        onQualityScore: (evt) => {
          props.onQualityScore?.(evt)
        },
      })
      abortRef.current = controller
    } catch (err) {
      props.onError?.(String(err))
      setStreaming(false)
    }
  }, [props.onCommit, props.onDone, props.onError, props.onEvaluation, props.onImageGenerated, props.onQualityScore])

  return {
    streaming,
    state: stateRef.current,
    sendMessage: handleSend,
    abort,
  }
}
