import { useState, useEffect, useCallback, useRef } from 'react'
import { Plus, List, Bot, MessageSquare, Sparkles } from 'lucide-react'
import { postFeedback, type ChatMessage, type TokenUsage, type ImageGeneratedEvent, type EvaluationEvent, type QualityScore } from '../api'
import { useSessionLoader } from '../hooks/useSessionLoader'
import { useChatStream } from '../hooks/useChatStream'
import ChatInput from '../components/ChatInput'
import MessageBubble from '../components/MessageBubble'
import StreamingBubble from '../components/StreamingBubble'

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
