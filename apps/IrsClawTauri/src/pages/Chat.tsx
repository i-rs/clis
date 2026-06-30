import { useState, useCallback } from 'react'
import { Plus, Sparkles, MessageSquare } from 'lucide-react'
import { useChatStream } from '../hooks/useChatStream'
import { useSessionLoader } from '../hooks/useSessionLoader'
import { postFeedback } from '../api/sessions'
import type { ChatMessage, TokenUsage, ImageGeneratedEvent, EvaluationEvent, QualityScore } from '../api/types'
import FloatingInput from '../components/chat/FloatingInput'
import MessageList from '../components/chat/MessageList'
import EmptyState from '../components/common/EmptyState'
import styles from './Chat.module.css'

export default function ChatPage({ selectedAgent }: { selectedAgent: string }) {
  const { messages, setMessages, sessionId, sessionTitle, sessionAgent, newChat } = useSessionLoader(selectedAgent)
  const [feedbackSubmitted, setFeedbackSubmitted] = useState<Set<number>>(new Set())

  const handleCommit = useCallback((msg: Pick<ChatMessage, 'role' | 'content' | 'reasoning' | 'toolCalls'>) => {
    setMessages((prev) => [...prev, msg as ChatMessage])
  }, [setMessages])

  const handleImageGenerated = useCallback((evt: ImageGeneratedEvent) => {
    setMessages((prev) => [...prev, { role: 'image', content: '', image: { path: evt.path, alt_text: evt.alt_text, width: evt.width, height: evt.height, format: evt.format, url: `/api/images/${evt.path}` } }])
  }, [setMessages])

  const handleError = useCallback((error: string) => {
    setMessages((prev) => [...prev, { role: 'error', content: error }])
  }, [setMessages])

  const handleDone = useCallback((usage: TokenUsage | null, quality?: QualityScore | null) => {
    if (usage || quality) {
      setMessages((prev) => {
        const i = prev.length - 1
        if (i >= 0 && prev[i].role === 'assistant') {
          const updated = [...prev]
          if (usage) updated[i] = { ...updated[i], tokenUsage: usage }
          if (quality) updated[i] = { ...updated[i], quality }
          return updated
        }
        return prev
      })
    }
  }, [setMessages])

  const handleEvaluation = useCallback((evt: EvaluationEvent) => {
    setMessages((prev) => [...prev, { role: 'evaluation', content: '', evaluation: evt }])
  }, [setMessages])

  const handleQualityScore = useCallback((evt: QualityScore) => {
    setMessages((prev) => {
      const i = prev.length - 1
      if (i >= 0 && prev[i].role === 'assistant') { const u = [...prev]; u[i] = { ...u[i], quality: evt }; return u }
      return prev
    })
  }, [setMessages])

  const { streaming, state, sendMessage, abort } = useChatStream({
    onCommit: handleCommit, onImageGenerated: handleImageGenerated, onError: handleError, onDone: handleDone, onEvaluation: handleEvaluation, onQualityScore: handleQualityScore,
  })

  const handleSend = useCallback((text: string) => {
    abort()
    setMessages((prev) => [...prev, { role: 'user', content: text }])
    sendMessage(text, selectedAgent !== 'default' ? selectedAgent : undefined)
  }, [abort, sendMessage, selectedAgent, setMessages])

  const handleNewChat = async () => { if (streaming) return; abort(); await newChat(); setFeedbackSubmitted(new Set()) }

  const handleFeedback = async (i: number, positive: boolean) => {
    if (!sessionId || feedbackSubmitted.has(i)) return
    try { await postFeedback(sessionId, positive); setFeedbackSubmitted((p) => new Set([...p, i])) } catch (e) { console.error(e) }
  }

  const empty = messages.length === 0 && !streaming

  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <div className={styles.left}>
          <span className={styles.title}>{sessionTitle || 'i-rs-claw'}</span>
          {(sessionAgent || selectedAgent) && <span className="badge badge-info">{sessionAgent || selectedAgent}</span>}
        </div>
        <button className="btn btn-primary btn-sm" onClick={handleNewChat} disabled={streaming}><Plus size={14} />New</button>
      </div>
      {empty ? (
        <EmptyState
          icon={<MessageSquare size={32} />}
          title="Start a Conversation"
          description="Ask Claw anything or let it help manage your personal data"
          action={
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8, justifyContent: 'center', maxWidth: 480 }}>
              {['How is my health today?', 'Log my weight as 75kg', 'How did my running go this month?'].map((s) => (
                <button key={s} className="btn btn-secondary btn-sm" onClick={() => handleSend(s)}><Sparkles size={12} />{s}</button>
              ))}
            </div>
          }
        />
      ) : (
        <MessageList messages={messages} sessionId={sessionId} feedbackSubmitted={feedbackSubmitted} onFeedback={handleFeedback} streaming={streaming} streamDisplay={state} />
      )}
      <FloatingInput onSend={handleSend} disabled={streaming} />
    </div>
  )
}
