import { useState, useRef, useEffect, useCallback } from 'react'
import { Send, Plus, List, Loader, Brain, Terminal, ChevronDown, ChevronRight, Bot } from 'lucide-react'
import { sendMessage, streamChat, getCurrentSession, createSession, listSessions, switchSession, type ChatMessage, type ToolCallMsg } from '../api'
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

  // Load current session on mount or when agent changes
  useEffect(() => {
    const load = async () => {
      const resp = await getCurrentSession()
      if (resp.success && resp.data && resp.data.id) {
        const sessionAgentId = resp.data.agent_id || 'default'
        // If current session belongs to a different agent, find or create one
        if (sessionAgentId !== selectedAgent) {
          const sessionsResp = await listSessions()
          const agentSessions = (sessionsResp.data || [])
            .filter((s: { agent_id: string }) => (s.agent_id || 'default') === selectedAgent)
            .sort((a: { created_at: number }, b: { created_at: number }) => b.created_at - a.created_at)
          if (agentSessions.length > 0) {
            // Switch to the most recent session for this agent
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
            // No existing session for this agent, create one
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
        if (sessAgent) {
          setSessionAgent(sessAgent)
        }
        // Convert API messages to ChatMessage[], merging tool_call into assistant
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
        // No session exists, try to find the most recent one for this agent
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
          // Create a new session for the selected agent
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

  const handleNewChat = async () => {
    if (loading) return
    abortRef.current?.abort()
    setMessages([])
    setInput('')
    setLoading(false)
    streamingRef.current = { content: '', reasoning: '', toolCalls: [] }
    setSessionTitle('New Chat')
    setSessionAgent(null)
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
    forceUpdate((n) => n + 1)

    try {
      const resp = await sendMessage(text, selectedAgent !== 'default' ? selectedAgent : undefined)
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
          onSessionChange?.()
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
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <h2>{sessionTitle}</h2>
            {(sessionAgent || selectedAgent) && (
              <span className="agent-badge" title={`Agent: ${sessionAgent || selectedAgent}`}>
                <Bot size={12} />
                {sessionAgent || selectedAgent}
              </span>
            )}
            {hasSession && (
              <button
                className="btn-ghost"
                onClick={() => onNavigate?.('sessions')}
                title="Switch session"
                style={{ fontSize: '14px' }}
              >
                <List size={16} />
              </button>
            )}
          </div>
          <button
            className="send-btn btn-sm"
            onClick={handleNewChat}
            disabled={loading}
            title="New chat"
          >
            <Plus size={14} />
            New Chat
          </button>
        </div>
      </div>

      <div className="chat-messages">
        {messages.map((msg, i) => (
          <MessageBubble key={i} message={msg} />
        ))}
        {hasStreaming && <StreamingBubble display={display} />}
        {loading && !hasStreaming && (
          <div className="message status">
            <Loader size={12} className="loading-spinner" />
            Thinking...
          </div>
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
            <Send size={15} className="send-icon" />
            Send
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
          <Loader size={12} className="loading-spinner" />
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
