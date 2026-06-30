import { useEffect, useRef } from 'react'
import type { ChatMessage } from '../../api/types'
import MessageBubble from './MessageBubble'
import StreamingBubble from './StreamingBubble'

interface Props {
  messages: ChatMessage[]
  sessionId: string | null
  feedbackSubmitted: Set<number>
  onFeedback: (i: number, p: boolean) => void
  streaming: boolean
  streamDisplay: { content: string; reasoning: string; toolCalls: ChatMessage['toolCalls'] }
}

export default function MessageList({ messages, sessionId, feedbackSubmitted, onFeedback, streaming, streamDisplay }: Props) {
  const endRef = useRef<HTMLDivElement>(null)
  const hasStreaming = streaming && (streamDisplay.content || streamDisplay.reasoning || (streamDisplay.toolCalls && streamDisplay.toolCalls.length > 0))

  useEffect(() => {
    if (streaming) endRef.current?.scrollIntoView({ behavior: 'instant' })
  }, [streamDisplay?.content, streamDisplay?.reasoning, streamDisplay?.toolCalls?.length, streaming])

  useEffect(() => {
    if (messages.length > 0) endRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  return (
    <div style={{ flex: 1, overflowY: 'auto', padding: '16px', display: 'flex', flexDirection: 'column', gap: '12px' }}>
      {messages.map((msg, i) => (
        <MessageBubble key={i} message={msg} index={i} onFeedback={onFeedback} hasFeedback={feedbackSubmitted.has(i)} sessionId={sessionId} />
      ))}
      {hasStreaming && streamDisplay && <StreamingBubble display={streamDisplay as any} />}
      {streaming && !hasStreaming && (
        <div className="message status"><div className="typing-dots"><span></span><span></span><span></span></div>Thinking...</div>
      )}
      <div ref={endRef} />
    </div>
  )
}
