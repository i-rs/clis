import { useState, useRef, useEffect, useCallback } from 'react'
import { Send, Plus, List, Brain, Terminal, ChevronDown, ChevronRight, Bot, MessageSquare, Sparkles, ThumbsUp, ThumbsDown } from 'lucide-react'
import { sendMessage, streamChat, getCurrentSession, createSession, listSessions, switchSession, postFeedback, type ChatMessage, type ToolCallMsg, type TokenUsage, type ImageGeneratedEvent } from '../api'
import MarkdownRenderer from '../components/MarkdownRenderer'

interface Props {
  selectedAgent: string
  onNavigate?: (page: 'sessions') => void
  onSessionChange?: () => void
}

export default function ChatPage({ selectedAgent, onNavigate, onSessionChange }: Props) {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [input, setInput] = useState('')
  const [loading, setLoading] = useState(false)
  const [sessionTitle, setSessionTitle] = useState('')
  const [hasSession, setHasSession] = useState(false)
  const [sessionAgent, setSessionAgent] = useState<string | null>(null)
  const [sessionId, setSessionId] = useState<string | null>(null)
  const [feedbackSubmitted, setFeedbackSubmitted] = useState<Set<number>>(new Set())
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const abortRef = useRef<AbortController | null>(null)
  const inputRef = useRef<HTMLTextAreaElement>(null)

  const streamingRef = useRef<{
    content: string
    reasoning: string
    toolCalls: ToolCallMsg[]
  }>({ content: '', reasoning: '', toolCalls: [] })
  const [renderTick, setRenderTick] = useState(0)

  // Instant scroll during streaming (tracks every token/tool event)
  useEffect(() => {
    if (loading) {
      messagesEndRef.current?.scrollIntoView({ behavior: 'instant' })
    }
  }, [renderTick, loading])

  // Smooth scroll when a new message arrives
  useEffect(() => {
    if (messages.length > 0) {
      messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
    }
  }, [messages])

  // Refresh session list after streaming completes (AFTER render, not during onDone)
  const prevLoading = useRef(loading)
  useEffect(() => {
    if (prevLoading.current && !loading) {
      onSessionChange?.()
    }
    prevLoading.current = loading
  }, [loading, onSessionChange])

  useEffect(() => {
    const load = async () => {
      const resp = await getCurrentSession()
      if (resp.success && resp.data && resp.data.id) {
        const sessionAgentId = resp.data.agent_id || 'default'
        if (sessionAgentId !== selectedAgent) {
          const sessionsResp = await listSessions()
          const agentSessions = (sessionsResp.data || [])
            .filter((s: { agent_id: string }) => (s.agent_id || 'default') === selectedAgent)
            .sort((a: { created_at: number }, b: { created_at: number }) => b.created_at - a.created_at)
          if (agentSessions.length > 0) {
            const recent = agentSessions[0]
            await switchSession(recent.id)
            const sessionResp = await getCurrentSession()
            if (sessionResp.success && sessionResp.data) {
              setHasSession(true)
              setSessionTitle(sessionResp.data.title || 'Untitled')
              const sessAgent = sessionResp.data.agent_id || null
              if (sessAgent) setSessionAgent(sessAgent)
              const raw = sessionResp.data.messages || []
              const msgs: ChatMessage[] = []
              let pendingToolCalls: ToolCallMsg[] = []
              for (const m of raw) {
                if (m.role === 'user') {
                  pendingToolCalls = []
                  msgs.push({ role: 'user', content: m.content || '' })
                } else if (m.role === 'assistant') {
                  msgs.push({
                    role: 'assistant',
                    content: m.content || '',
                    reasoning: m.reasoning || undefined,
                    toolCalls: pendingToolCalls.length > 0 ? [...pendingToolCalls] : undefined,
                  })
                  pendingToolCalls = []
                } else if (m.role === 'tool_call') {
                  pendingToolCalls.push({
                    name: m.name || '',
                    args: m.args || '',
                    result: m.result || '',
                    step: 0,
                    total_steps: 1,
                  })
                } else if (m.role === 'image') {
                  msgs.push({
                    role: 'image',
                    content: '',
                    image: {
                      path: m.path || '',
                      alt_text: m.alt_text || '',
                      width: m.width || 0,
                      height: m.height || 0,
                      format: m.format || '',
                      url: m.url || `/api/images/${m.path || ''}`,
                    },
                  })
                }
              }
              setMessages(msgs)
            }
          } else {
            const createResp = await createSession(
              selectedAgent !== 'default' ? selectedAgent : undefined
            )
            if (createResp.success && createResp.data) {
              setHasSession(true)
              setSessionTitle('New Chat')
              setSessionAgent(createResp.data.agent_id || null)
              setMessages([])
            }
          }
          return
        }
        setHasSession(true)
        setSessionTitle(resp.data.title || 'Untitled')
        const sessAgent = resp.data.agent_id || null
        if (sessAgent) setSessionAgent(sessAgent)
        const raw = resp.data.messages || []
        const msgs: ChatMessage[] = []
        let pendingToolCalls: ToolCallMsg[] = []
        for (const m of raw) {
          if (m.role === 'user') {
            pendingToolCalls = []
            msgs.push({ role: 'user', content: m.content || '' })
          } else if (m.role === 'assistant') {
            msgs.push({
              role: 'assistant',
              content: m.content || '',
              reasoning: m.reasoning || undefined,
              toolCalls: pendingToolCalls.length > 0 ? [...pendingToolCalls] : undefined,
            })
            pendingToolCalls = []
          } else if (m.role === 'tool_call') {
            pendingToolCalls.push({
              name: m.name || '',
              args: m.args || '',
              result: m.result || '',
              step: 0,
              total_steps: 1,
            })
          }
        }
        setMessages(msgs)
      } else {
        const sessionsResp = await listSessions()
        const agentSessions = (sessionsResp.data || [])
          .filter((s: { agent_id: string }) => (s.agent_id || 'default') === selectedAgent)
          .sort((a: { created_at: number }, b: { created_at: number }) => b.created_at - a.created_at)
        if (agentSessions.length > 0) {
          const recent = agentSessions[0]
          await switchSession(recent.id)
          const sessionResp = await getCurrentSession()
          if (sessionResp.success && sessionResp.data) {
            setHasSession(true)
            setSessionTitle(sessionResp.data.title || 'Untitled')
            const sessAgent = sessionResp.data.agent_id || null
            if (sessAgent) setSessionAgent(sessAgent)
            const raw = sessionResp.data.messages || []
            const msgs: ChatMessage[] = []
            let pendingToolCalls: ToolCallMsg[] = []
            for (const m of raw) {
              if (m.role === 'user') {
                pendingToolCalls = []
                msgs.push({ role: 'user', content: m.content || '' })
              } else if (m.role === 'assistant') {
                msgs.push({
                  role: 'assistant',
                  content: m.content || '',
                  reasoning: m.reasoning || undefined,
                  toolCalls: pendingToolCalls.length > 0 ? [...pendingToolCalls] : undefined,
                })
                pendingToolCalls = []
              } else if (m.role === 'tool_call') {
                pendingToolCalls.push({
                  name: m.name || '',
                  args: m.args || '',
                  result: m.result || '',
                  step: 0,
                  total_steps: 1,
                })
              }
            }
            setMessages(msgs)
          }
        } else {
          const createResp = await createSession(
            selectedAgent !== 'default' ? selectedAgent : undefined
          )
          if (createResp.success && createResp.data) {
            setHasSession(true)
            setSessionTitle('New Chat')
            setSessionAgent(createResp.data.agent_id || null)
            setMessages([])
          }
        }
      }
    }
    load()
  }, [selectedAgent])

  const handleFeedback = async (messageIndex: number, positive: boolean) => {
    if (!sessionId || feedbackSubmitted.has(messageIndex)) return
    try {
      await postFeedback(sessionId, positive)
      setFeedbackSubmitted((prev) => new Set([...prev, messageIndex]))
    } catch (err) {
      console.error('Failed to submit feedback:', err)
    }
  }

  const handleNewChat = async () => {
    if (loading) return
    abortRef.current?.abort()
    setMessages([])
    setInput('')
    setLoading(false)
    streamingRef.current = { content: '', reasoning: '', toolCalls: [] }
    setSessionTitle('New Chat')
    setSessionAgent(null)
    setSessionId(null)
    setFeedbackSubmitted(new Set())
    try {
      const resp = await createSession(selectedAgent !== 'default' ? selectedAgent : undefined)
      if (resp.success && resp.data) {
        setHasSession(true)
        setSessionAgent(resp.data.agent_id || selectedAgent)
        onSessionChange?.()
      }
    } catch { /* ignore */ }
  }

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
    setRenderTick((n) => n + 1)

    try {
      const resp = await sendMessage(text, selectedAgent !== 'default' ? selectedAgent : undefined)
      if (!resp.success || !resp.data) {
        setMessages((prev) => [...prev, { role: 'error', content: resp.error || 'Failed to send message' }])
        setLoading(false)
        return
      }

      const sid = resp.data.session_id
      setSessionId(sid)

      const controller = streamChat(sid, {
        onReasoning: (reasoningText: string) => {
          streamingRef.current.reasoning += reasoningText
          setRenderTick((n) => n + 1)
        },
        onToken: (token: string) => {
          streamingRef.current.content += token
          setRenderTick((n) => n + 1)
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
          setRenderTick((n) => n + 1)
        },
        onToolExecuted: (evt) => {
          streamingRef.current.toolCalls.push({
            name: evt.name,
            args: evt.args,
            result: evt.result,
            step: evt.step,
            total_steps: evt.total_steps,
          })
          setRenderTick((n) => n + 1)
        },
        onImageGenerated: (evt: ImageGeneratedEvent) => {
          commitStreaming()
          setMessages((prev) => [...prev, {
            role: 'image',
            content: '',
            image: {
              path: evt.path,
              alt_text: evt.alt_text,
              width: evt.width,
              height: evt.height,
              format: evt.format,
              url: `/api/images/${evt.path}`,
            },
          }])
          setRenderTick((n) => n + 1)
        },
        onError: (error: string) => {
          commitStreaming()
          setMessages((prev) => [...prev, { role: 'error', content: error }])
          setLoading(false)
        },
        onDone: (usage: TokenUsage | null) => {
          console.log('[DEBUG] onDone received:', JSON.stringify(usage))
          commitStreaming()
          if (usage) {
            setMessages((prev) => {
              const lastIdx = prev.length - 1
              if (lastIdx >= 0 && prev[lastIdx].role === 'assistant') {
                const updated = [...prev]
                updated[lastIdx] = { ...updated[lastIdx], tokenUsage: usage }
                return updated
              }
              return prev
            })
          }
          setLoading(false)
        },
      })
      abortRef.current = controller
    } catch (err) {
      setMessages((prev) => [...prev, { role: 'error', content: String(err) }])
      setLoading(false)
    }
  }

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
        <div className="page-header-left">
          <div className="page-header-icon">
            <MessageSquare size={16} />
          </div>
          <h2>{sessionTitle}</h2>
          {(sessionAgent || selectedAgent) && (
            <span className="badge badge-info">
              <Bot size={10} />
              {sessionAgent || selectedAgent}
            </span>
          )}
          {hasSession && (
            <button
              className="btn btn-ghost btn-sm"
              onClick={() => onNavigate?.('sessions')}
              title="Switch session"
            >
              <List size={14} />
            </button>
          )}
        </div>
        <button
          className="btn btn-primary btn-sm"
          onClick={handleNewChat}
          disabled={loading}
        >
          <Plus size={14} />
          New Chat
        </button>
      </div>

      <div className="chat-messages">
        {messages.length === 0 && !loading && !hasStreaming && (
          <div className="chat-empty-state">
            <div className="chat-empty-icon">
              <MessageSquare size={32} />
            </div>
            <h3>Start a Conversation</h3>
            <p>Ask Claw anything or let it help manage your personal data</p>
            <div className="chat-suggestions">
              <button className="suggestion-btn" onClick={() => { setInput('How is my health today?'); inputRef.current?.focus(); }}>
                <Sparkles size={14} /> How is my health today?
              </button>
              <button className="suggestion-btn" onClick={() => { setInput('Log my weight as 75kg'); inputRef.current?.focus(); }}>
                <Sparkles size={14} /> Log my weight as 75kg
              </button>
              <button className="suggestion-btn" onClick={() => { setInput('How did my running go this month?'); inputRef.current?.focus(); }}>
                <Sparkles size={14} /> How did my running go this month?
              </button>
            </div>
          </div>
        )}
        {messages.map((msg, i) => (
          <MessageBubble
            key={i}
            message={msg}
            index={i}
            onFeedback={handleFeedback}
            hasFeedback={feedbackSubmitted.has(i)}
            sessionId={sessionId}
          />
        ))}
        {hasStreaming && <StreamingBubble display={display} />}
        {loading && !hasStreaming && (
          <div className="message status">
            <div className="typing-dots">
              <span></span>
              <span></span>
              <span></span>
            </div>
            Thinking...
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      <div className="chat-input-area">
        <div className="chat-input-container">
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
            <Send size={15} />
            Send
          </button>
        </div>
      </div>
    </div>
  )
}

function MessageBubble({ message, index, onFeedback, hasFeedback, sessionId }: { message: ChatMessage; index: number; onFeedback?: (i: number, p: boolean) => void; hasFeedback?: boolean; sessionId?: string | null }) {
  const hasToolCalls = message.toolCalls && message.toolCalls.length > 0
  const hasReasoning = message.reasoning && message.reasoning.length > 0
  const hasContent = !!message.content

  if (message.role === 'assistant' && !hasContent && !hasReasoning && !hasToolCalls) {
    return null
  }

  if (message.role === 'image' && message.image) {
    return (
      <div className="message assistant">
        <div className="message-content image-message">
          <img
            src={message.image.url}
            alt={message.image.alt_text}
            style={{ maxWidth: '100%', borderRadius: '8px', border: '1px solid #27273a' }}
            loading="lazy"
          />
          {message.image.alt_text && (
            <div className="image-caption">{message.image.alt_text}</div>
          )}
        </div>
      </div>
    )
  }

  return (
    <div className={`message ${message.role}`}>
      {hasReasoning && (
        <details className="reasoning-details" open>
          <summary className="reasoning-summary">
            <Brain size={12} />
            Thinking process
          </summary>
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
            <Terminal size={12} />
            Tool calls ({message.toolCalls!.length})
          </summary>
          <div className="tool-calls-list">
            {message.toolCalls!.map((tc, i) => (
              <ToolCallCard key={i} toolCall={tc} />
            ))}
          </div>
        </details>
      )}
      {message.tokenUsage && (message.tokenUsage.prompt_tokens != null || message.tokenUsage.completion_tokens != null || message.tokenUsage.total_tokens != null) && (
        <div className="message-footer">
          <span className="token-stats">
            {message.tokenUsage.total_tokens != null
              ? `${message.tokenUsage.total_tokens} tokens`
              : `${(message.tokenUsage.prompt_tokens ?? 0) + (message.tokenUsage.completion_tokens ?? 0)} tokens`}
            <span className="token-stats-detail">
              &nbsp;(↑{message.tokenUsage.prompt_tokens ?? 0} ↓{message.tokenUsage.completion_tokens ?? 0})
            </span>
          </span>
        </div>
      )}
      {message.role === 'assistant' && sessionId && !hasFeedback && onFeedback && (
        <div className="message-feedback">
          <button
            className="feedback-btn"
            onClick={() => onFeedback(index, true)}
            title="Good response"
          >
            <ThumbsUp size={14} />
          </button>
          <button
            className="feedback-btn"
            onClick={() => onFeedback(index, false)}
            title="Bad response"
          >
            <ThumbsDown size={14} />
          </button>
        </div>
      )}
      {message.role === 'assistant' && sessionId && hasFeedback && (
        <div className="message-feedback-submitted">
          <span>Thanks for your feedback!</span>
        </div>
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
          <summary className="reasoning-summary">
            <Brain size={12} />
            Thinking process
          </summary>
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
            <Terminal size={12} />
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
        <div className="streaming-cursor">
          <div className="typing-dots">
            <span></span>
            <span></span>
            <span></span>
          </div>
          Thinking...
        </div>
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
        <span className="tool-call-icon">
          <Terminal size={12} />
        </span>
        <span className="tool-call-name">{stepLabel} {toolName}</span>
        <span className="tool-call-toggle">
          {expanded ? <ChevronDown size={10} /> : <ChevronRight size={10} />}
        </span>
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
