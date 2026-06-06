import { useState, useEffect, useCallback, useRef } from 'react'
import { Plus, List, Brain, Terminal, ChevronDown, ChevronRight, Bot, MessageSquare, Sparkles, ThumbsUp, ThumbsDown } from 'lucide-react'
import { postFeedback, type ChatMessage, type ToolCallMsg, type TokenUsage, type ImageGeneratedEvent, type EvaluationEvent, type QualityScore } from '../api'
import { useSessionLoader } from '../hooks/useSessionLoader'
import { useChatStream } from '../hooks/useChatStream'
import ChatInput from '../components/ChatInput'
import MarkdownRenderer from '../components/MarkdownRenderer'

interface Props {
  selectedAgent: string
  onNavigate?: (page: 'sessions') => void
  onSessionChange?: () => void
}

export default function ChatPage({ selectedAgent, onNavigate, onSessionChange }: Props) {
  const { messages, setMessages, sessionId, sessionTitle, sessionAgent, newChat } = useSessionLoader(selectedAgent)
  const [feedbackSubmitted, setFeedbackSubmitted] = useState<Set<number>>(new Set())
  const messagesEndRef = useRef<HTMLDivElement>(null)

  const handleCommit = useCallback((msg: Pick<ChatMessage, 'role' | 'content' | 'reasoning' | 'toolCalls'>) => {
    setMessages((prev) => [...prev, msg as ChatMessage])
  }, [setMessages])

  const handleImageGenerated = useCallback((evt: ImageGeneratedEvent) => {
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
  }, [setMessages])

  const handleError = useCallback((error: string) => {
    setMessages((prev) => [...prev, { role: 'error', content: error }])
  }, [setMessages])

  const handleDone = useCallback((_usage: TokenUsage | null, quality?: QualityScore | null) => {
    if (_usage || quality) {
      setMessages((prev) => {
        const lastIdx = prev.length - 1
        if (lastIdx >= 0 && prev[lastIdx].role === 'assistant') {
          const updated = [...prev]
          if (_usage) updated[lastIdx] = { ...updated[lastIdx], tokenUsage: _usage }
          if (quality) updated[lastIdx] = { ...updated[lastIdx], quality }
          return updated
        }
        return prev
      })
    }
  }, [setMessages])

  const handleEvaluation = useCallback((evt: EvaluationEvent) => {
    setMessages((prev) => [...prev, {
      role: 'evaluation',
      content: '',
      evaluation: evt,
    }])
  }, [setMessages])

  const handleQualityScore = useCallback((evt: QualityScore) => {
    setMessages((prev) => {
      const lastIdx = prev.length - 1
      if (lastIdx >= 0 && prev[lastIdx].role === 'assistant') {
        const updated = [...prev]
        updated[lastIdx] = { ...updated[lastIdx], quality: evt }
        return updated
      }
      return prev
    })
  }, [setMessages])

  const { streaming, state: streamState, sendMessage, abort } = useChatStream({
    onCommit: handleCommit,
    onImageGenerated: handleImageGenerated,
    onError: handleError,
    onDone: handleDone,
    onEvaluation: handleEvaluation,
    onQualityScore: handleQualityScore,
  })

  useEffect(() => {
    if (!streaming) {
      onSessionChange?.()
    }
  }, [streaming, onSessionChange])

  const handleSend = useCallback((text: string) => {
    abort()
    setMessages((prev) => [...prev, { role: 'user', content: text }])
    sendMessage(text, selectedAgent !== 'default' ? selectedAgent : undefined)
  }, [abort, sendMessage, selectedAgent, setMessages])

  const handleNewChat = async () => {
    if (streaming) return
    abort()
    await newChat()
    setFeedbackSubmitted(new Set())
  }

  const handleFeedback = async (messageIndex: number, positive: boolean) => {
    if (!sessionId || feedbackSubmitted.has(messageIndex)) return
    try {
      await postFeedback(sessionId, positive)
      setFeedbackSubmitted((prev) => new Set([...prev, messageIndex]))
    } catch (err) {
      console.error('Failed to submit feedback:', err)
    }
  }

  const hasStreaming = streaming && (streamState.content || streamState.reasoning || streamState.toolCalls.length > 0)

  useEffect(() => {
    if (streaming) {
      messagesEndRef.current?.scrollIntoView({ behavior: 'instant' })
    }
  }, [streamState.content, streamState.reasoning, streamState.toolCalls.length, streaming])

  useEffect(() => {
    if (messages.length > 0) {
      messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
    }
  }, [messages])

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
          <button
            className="btn btn-ghost btn-sm"
            onClick={() => onNavigate?.('sessions')}
            title="Switch session"
          >
            <List size={14} />
          </button>
        </div>
        <button
          className="btn btn-primary btn-sm"
          onClick={handleNewChat}
          disabled={streaming}
        >
          <Plus size={14} />
          New Chat
        </button>
      </div>

      <div className="chat-messages">
        {messages.length === 0 && !streaming && !hasStreaming && (
          <div className="chat-empty-state">
            <div className="chat-empty-icon">
              <MessageSquare size={32} />
            </div>
            <h3>Start a Conversation</h3>
            <p>Ask Claw anything or let it help manage your personal data</p>
            <div className="chat-suggestions">
              <button className="suggestion-btn" onClick={() => handleSend('How is my health today?')}>
                <Sparkles size={14} /> How is my health today?
              </button>
              <button className="suggestion-btn" onClick={() => handleSend('Log my weight as 75kg')}>
                <Sparkles size={14} /> Log my weight as 75kg
              </button>
              <button className="suggestion-btn" onClick={() => handleSend('How did my running go this month?')}>
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
        {hasStreaming && <StreamingBubble display={streamState} />}
        {streaming && !hasStreaming && (
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

      <ChatInput onSend={handleSend} disabled={streaming} />
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
              {message.tokenUsage.estimated_cost_usd != null && (
                <> | ${Number(message.tokenUsage.estimated_cost_usd).toFixed(6)}</>
              )}
            </span>
          </span>
        </div>
      )}
      {message.role === 'assistant' && sessionId && !hasFeedback && onFeedback && (
        <div className="message-feedback">
          <button className="feedback-btn" onClick={() => onFeedback(index, true)} title="Good response">
            <ThumbsUp size={14} />
          </button>
          <button className="feedback-btn" onClick={() => onFeedback(index, false)} title="Bad response">
            <ThumbsDown size={14} />
          </button>
        </div>
      )}
      {message.role === 'assistant' && sessionId && hasFeedback && (
        <div className="message-feedback-submitted">
          <span>Thanks for your feedback!</span>
        </div>
      )}
      {message.quality && (
        <div className="message-quality">
          <div className="quality-header">
            <span className="quality-icon">📊</span>
            <span className="quality-label">Quality Assessment</span>
          </div>
          <div className="quality-score">
            <div className="score-bar">
              <div className="score-fill" style={{ width: `${Math.round((parseFloat(message.quality.score) || 0) * 100)}%` }} />
            </div>
            <span className="score-value">{Math.round((parseFloat(message.quality.score) || 0) * 100)}%</span>
          </div>
          {message.quality.complete && (
            <div className="quality-complete"><span className="complete-badge">✓ Complete</span></div>
          )}
          {message.quality.references_valid && (
            <div className="quality-refs"><span className="refs-badge">✓ References Valid</span></div>
          )}
          {message.quality.issues.length > 0 && (
            <div className="quality-issues">
              {message.quality.issues.map((issue, i) => <div key={i} className="quality-issue">⚠️ {issue}</div>)}
            </div>
          )}
        </div>
      )}
      {message.role === 'feedback' && message.feedback && (
        <div className="message-feedback">
          <span className={`feedback-badge ${message.feedback.positive ? 'positive' : 'negative'}`}>
            {message.feedback.positive ? '👍' : '👎'}
            {' '}{message.feedback.positive ? 'Positive' : 'Negative'}
          </span>
          {message.feedback.message && <small>{message.feedback.message}</small>}
        </div>
      )}
      {message.role === 'evaluation' && message.evaluation && (
        <div className="message-evaluation">
          <span className={`eval-badge ${message.evaluation.valid ? 'valid' : 'invalid'}`}>
            {message.evaluation.valid ? '✓' : '✗'}
          </span>
          <code className="eval-tool">{message.evaluation.tool}</code>
          {message.evaluation.issues.length > 0 && (
            <div className="eval-issues">
              {message.evaluation.issues.map((issue, i) => <div key={i} className="eval-issue"><small>{issue}</small></div>)}
            </div>
          )}
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
