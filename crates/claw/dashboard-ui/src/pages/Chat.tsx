import { useState, useRef, useEffect, useCallback } from 'react'
import { sendMessage, streamChat, type ChatMessage } from '../api'

export default function ChatPage() {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [input, setInput] = useState('')
  const [loading, setLoading] = useState(false)
  const [streamingText, setStreamingText] = useState('')
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const abortRef = useRef<AbortController | null>(null)
  const inputRef = useRef<HTMLTextAreaElement>(null)
  const streamingRef = useRef('')

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [])

  useEffect(() => {
    scrollToBottom()
  }, [messages, streamingText, scrollToBottom])

  const handleSend = async () => {
    const text = input.trim()
    if (!text || loading) return

    setInput('')
    setMessages((prev) => [...prev, { role: 'user', content: text }])
    setLoading(true)
    setStreamingText('')

    try {
      const resp = await sendMessage(text)
      if (!resp.success || !resp.data) {
        setMessages((prev) => [...prev, { role: 'error', content: resp.error || 'Failed to send message' }])
        setLoading(false)
        return
      }

      const sid = resp.data.session_id

      // Start SSE stream
      setStreamingText('')
      const controller = streamChat(sid, {
        onToken: (token) => {
          streamingRef.current += token
          setStreamingText(streamingRef.current)
        },
        onReasoning: () => {
          // Reasoning is streamed as token events in the same flow
        },
        onStatus: () => {
          // Status updates
        },
        onError: (error) => {
          setMessages((prev) => [...prev, { role: 'error', content: error }])
          setLoading(false)
          setStreamingText('')
        },
        onDone: () => {
          const fullText = streamingRef.current
          if (fullText) {
            setMessages((prev) => [...prev, { role: 'assistant', content: fullText }])
          }
          setLoading(false)
          setStreamingText('')
          streamingRef.current = ''
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

  return (
    <div className="chat-container">
      <div className="page-header">
        <h2>Chat</h2>
      </div>

      <div className="chat-messages">
        {messages.map((msg, i) => (
          <div key={i} className={`message ${msg.role}`}>
            {msg.content}
          </div>
        ))}
        {loading && streamingText && (
          <div className="message assistant streaming">{streamingText}</div>
        )}
        {loading && !streamingText && (
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
