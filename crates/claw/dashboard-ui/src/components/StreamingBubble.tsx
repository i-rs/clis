import { Brain, Terminal } from 'lucide-react'
import type { ToolCallMsg } from '../api'
import ToolCallCard from './ToolCallCard'
import MarkdownRenderer from './MarkdownRenderer'
import msgStyles from './MessageBubble.module.css'
import styles from './StreamingBubble.module.css'

interface StreamingBubbleProps {
  display: { content: string; reasoning: string; toolCalls: ToolCallMsg[] }
}

export default function StreamingBubble({ display }: StreamingBubbleProps) {
  const hasReasoning = display.reasoning.length > 0
  const hasContent = display.content.length > 0
  const hasToolCalls = display.toolCalls.length > 0

  return (
    <div className={`${msgStyles.message} ${msgStyles.messageAssistant} ${styles.streaming}`}>
      {hasReasoning && (
        <details className={msgStyles.reasoningDetails} open>
          <summary className={msgStyles.reasoningSummary}>
            <Brain size={12} />
            Thinking process
          </summary>
          <div className={msgStyles.reasoningContent}>{display.reasoning}</div>
        </details>
      )}
      {hasContent && (
        <div className={msgStyles.messageContent}>
          <MarkdownRenderer content={display.content} />
        </div>
      )}
      {hasToolCalls && (
        <details className={msgStyles.toolCallsDetails} open>
          <summary className={msgStyles.toolCallsSummary}>
            <Terminal size={12} />
            Tool calls ({display.toolCalls.length})
          </summary>
          <div className={msgStyles.toolCallsList}>
            {display.toolCalls.map((tc, i) => (
              <ToolCallCard key={i} toolCall={tc} />
            ))}
          </div>
        </details>
      )}
      {!hasContent && !hasToolCalls && (
        <div className={styles.streamingCursor}>
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
