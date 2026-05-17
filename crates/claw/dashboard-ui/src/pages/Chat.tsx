import { useState, useRef, useEffect, useCallback } from 'react'
import { sendMessage, streamChat, type ChatMessage, type ToolCallMsg } from '../api'
import MarkdownRenderer from '../components/MarkdownRenderer'

export default function ChatPage() {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [input, setInput] = useState('')
  const [loading, setLoading] = useState(false)
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const abortRef = useRef<AbortController | null>(null)
  const inputRef = useRef<HTMLTextAreaElement>(null)

  // Streaming state
  const streamingRef = useRef<{
    content: string
    reasoning: string
    toolCalls: ToolCallMsg[]
  }>({ content: '', reasoning: '', toolCalls: [] })
  const [, forceUpdate] = useState(0)

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [])

  useEffect(() => {
    scrollToBottom()
  }, [messages, loading, scrollToBottom])

  const commitStreaming = useCallback(() => {
    const s = streamingRef.current
    if (s.content || s.reasoning || s.toolCalls.length > 0) {
      setMessages((prev) => [...prev, {
        role: 'assistant' as const,
        content: s.content,
        reasoning: s.reasoning || undefined,
        toolCalls: s.toolCalls.length > 0 ? [...s.toolCalls] : undefined,
      }])
    }
    streamingRef.current = { content: '', reasoning: '', toolCalls: [] }
  }, [])

  const handleSend = async () => {
    const text = input.trim()
    if (!text || loading) return

    setInput('')
    setMessages((prev) => [...prev, { role: 'user', content: text }])
    setLoading(true)
    streamingRef.current = { content: '', reasoning: '', toolCalls: [] }
    forceUpdate((n) => n + 1)

    try {
      const resp = await sendMessage(text)
      if (!resp.success || !resp.data) {
        setMessages((prev) => [...prev, { role: 'error', content: resp.error || 'Failed to send message' }])
        setLoading(false)
        return
      }

      const sid = resp.data.session_id

      const controller = streamChat(sid, {
        onReasoning: (reasoningText: string) => {
          streamingRef.current.reasoning += reasoningText
          forceUpdate((n) => n + 1)
        },
        onToken: (token: string) => {
          streamingRef.current.content += token
          forceUpdate((n) => n + 1)
        },
        onNewRound: () => {
          const s = streamingRef.current
          if (s.content || s.reasoning || s.toolCalls.length > 0) {
            setMessages((prev) => [...prev, {
              role: 'assistant',
              content: s.content,
              reasoning: s.reasoning || undefined,
              toolCalls: s.toolCalls.length > 0 ? [...s.toolCalls] : undefined,
            }])
          }
          streamingRef.current = { content: '', reasoning: '', toolCalls: [] }
          forceUpdate((n) => n + 1)
        },
        onToolExecuted: (evt) => {
          streamingRef.current.toolCalls.push({
            name: evt.name,
            args: evt.args,
            result: evt.result,
            step: evt.step,
            total_steps: evt.total_steps,
          })
          forceUpdate((n) => n + 1)
        },
        onError: (error: string) => {
          commitStreaming()
          setMessages((prev) => [...prev, { role: 'error', content: error }])
          setLoading(false)
        },
        onDone: () => {
          commitStreaming()
          setLoading(false)
        },
      })
      abortRef.current = controller
    } catch (err) {
      setMessages((prev) => [...prev, { role: 'error', content: String(err) }])
      setLoading(false)
    }
  }

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      abortRef.current?.abort()
    }
  }, [])

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  const display = streamingRef.current
  const hasStreaming = loading && (display.content || display.reasoning || display.toolCalls.length > 0)

  return (
    <div className="chat-container">
      <div className="page-header">
        <h2>Chat</h2>
      </div>

      <div className="chat-messages">
        {messages.map((msg, i) => (
          <MessageBubble key={i} message={msg} />
        ))}
        {hasStreaming && <StreamingBubble display={display} />}
        {loading && !hasStreaming && (
          <div className="message status">Thinking...</div>
        )}
        <div ref={messagesEndRef} />
      </div>

      <div className="chat-input-area">
        <div className="chat-input-row">
          <textarea
            ref={inputRef}
            className="chat-input"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type your message... (Enter to send, Shift+Enter for new line)"
            rows={1}
            disabled={loading}
          />
          <button
            className="send-btn"
            onClick={handleSend}
            disabled={loading || !input.trim()}
          >
            {loading ? '...' : 'Send'}
          </button>
        </div>
      </div>
    </div>
  )
}

// ── Sub-components ──

function MessageBubble({ message }: { message: ChatMessage }) {
  const hasToolCalls = message.toolCalls && message.toolCalls.length > 0
  const hasReasoning = message.reasoning && message.reasoning.length > 0

  return (
    <div className={`message ${message.role}`}>
      {hasReasoning && (
        <details className="reasoning-details" open>
          <summary className="reasoning-summary">Thinking process</summary>
          <div className="reasoning-content">{message.reasoning}</div>
        </details>
      )}
      {message.content && (
        <div className="message-content">
          {message.role === 'assistant' ? (
            <MarkdownRenderer content={message.content} />
          ) : (
            <span style={{ whiteSpace: 'pre-wrap' }}>{message.content}</span>
          )}
        </div>
      )}
      {hasToolCalls && (
        <details className="tool-calls-details">
          <summary className="tool-calls-summary">
            Tool calls ({message.toolCalls!.length})
          </summary>
          <div className="tool-calls-list">
            {message.toolCalls!.map((tc, i) => (
              <ToolCallCard key={i} toolCall={tc} />
            ))}
          </div>
        </details>
      )}
    </div>
  )
}

function StreamingBubble({ display }: { display: { content: string; reasoning: string; toolCalls: ToolCallMsg[] } }) {
  const hasReasoning = display.reasoning.length > 0
  const hasContent = display.content.length > 0
  const hasToolCalls = display.toolCalls.length > 0

  return (
    <div className="message assistant streaming">
      {hasReasoning && (
        <details className="reasoning-details" open>
          <summary className="reasoning-summary">Thinking process</summary>
          <div className="reasoning-content">{display.reasoning}</div>
        </details>
      )}
      {hasContent && (
        <div className="message-content">
          <MarkdownRenderer content={display.content} />
        </div>
      )}
      {hasToolCalls && (
        <details className="tool-calls-details" open>
          <summary className="tool-calls-summary">
            Tool calls ({display.toolCalls.length})
          </summary>
          <div className="tool-calls-list">
            {display.toolCalls.map((tc, i) => (
              <ToolCallCard key={i} toolCall={tc} />
            ))}
          </div>
        </details>
      )}
      {!hasContent && !hasToolCalls && (
        <div className="streaming-cursor">Thinking...</div>
      )}
    </div>
  )
}

function ToolCallCard({ toolCall }: { toolCall: ToolCallMsg }) {
  const [expanded, setExpanded] = useState(false)
  const stepLabel = toolCall.total_steps > 1
    ? `[${toolCall.step + 1}/${toolCall.total_steps}]`
    : ''

  let argsDisplay = toolCall.args
  try {
    const parsed = JSON.parse(toolCall.args)
    argsDisplay = JSON.stringify(parsed, null, 2)
  } catch { /* use raw args */ }

  let toolName = toolCall.name
  if (toolCall.name === 'i_rs') {
    try {
      const parsed = JSON.parse(toolCall.args)
      toolName = `i-rs ${parsed.tool || ''} ${parsed.command || ''}`
    } catch { /* keep default */ }
  }

  return (
    <div className="tool-call-card">
      <div className="tool-call-header" onClick={() => setExpanded(!expanded)}>
        <span className="tool-call-icon">⚡</span>
        <span className="tool-call-name">{stepLabel} {toolName}</span>
        <span className="tool-call-toggle">{expanded ? '▼' : '▶'}</span>
      </div>
      {expanded && (
        <div className="tool-call-body">
          <div className="tool-call-section">
            <div className="tool-call-section-label">Arguments:</div>
            <pre className="tool-call-code">{argsDisplay}</pre>
          </div>
          <div className="tool-call-section">
            <div className="tool-call-section-label">Result:</div>
            <pre className="tool-call-code">{toolCall.result}</pre>
          </div>
        </div>
      )}
    </div>
  )
}
