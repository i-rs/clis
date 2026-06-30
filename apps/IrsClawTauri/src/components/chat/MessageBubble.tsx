import { Brain, Terminal, ThumbsUp, ThumbsDown } from 'lucide-react'
import type { ChatMessage } from '../../api/types'
import ToolCallCard from './ToolCallCard'
import MarkdownRenderer from './MarkdownRenderer'
import styles from './MessageBubble.module.css'

interface MessageBubbleProps {
  message: ChatMessage
  index: number
  onFeedback?: (i: number, p: boolean) => void
  hasFeedback?: boolean
  sessionId?: string | null
}

const roleClassMap: Record<string, string> = {
  user: styles.messageUser,
  assistant: styles.messageAssistant,
  error: styles.messageError,
  image: styles.messageAssistant,
}

export default function MessageBubble({ message, index, onFeedback, hasFeedback, sessionId }: MessageBubbleProps) {
  const hasToolCalls = message.toolCalls && message.toolCalls.length > 0
  const hasReasoning = message.reasoning && message.reasoning.length > 0
  const hasContent = !!message.content

  if (message.role === 'assistant' && !hasContent && !hasReasoning && !hasToolCalls) {
    return null
  }

  if (message.role === 'image' && message.image) {
    return (
      <div className={`${styles.message} ${styles.messageAssistant}`}>
        <div className={`${styles.messageContent} ${styles.imageMessage}`}>
          <img
            src={message.image.url}
            alt={message.image.alt_text}
            style={{ maxWidth: '100%', borderRadius: '8px', border: '1px solid #27273a' }}
            loading="lazy"
          />
          {message.image.alt_text && (
            <div className={styles.imageCaption}>{message.image.alt_text}</div>
          )}
        </div>
      </div>
    )
  }

  const roleClass = roleClassMap[message.role] || ''

  return (
    <div className={`${styles.message} ${roleClass}`}>
      {hasReasoning && (
        <details className={styles.reasoningDetails} open>
          <summary className={styles.reasoningSummary}>
            <Brain size={12} />
            Thinking process
          </summary>
          <div className={styles.reasoningContent}>{message.reasoning}</div>
        </details>
      )}
      {message.content && (
        <div className={styles.messageContent}>
          {message.role === 'assistant' ? (
            <MarkdownRenderer content={message.content} />
          ) : (
            <span style={{ whiteSpace: 'pre-wrap' }}>{message.content}</span>
          )}
        </div>
      )}
      {hasToolCalls && (
        <details className={styles.toolCallsDetails}>
          <summary className={styles.toolCallsSummary}>
            <Terminal size={12} />
            Tool calls ({message.toolCalls!.length})
          </summary>
          <div className={styles.toolCallsList}>
            {message.toolCalls!.map((tc, i) => (
              <ToolCallCard key={i} toolCall={tc} />
            ))}
          </div>
        </details>
      )}
      {message.tokenUsage && (message.tokenUsage.prompt_tokens != null || message.tokenUsage.completion_tokens != null || message.tokenUsage.total_tokens != null) && (
        <div className={styles.messageFooter}>
          <span className={styles.tokenStats}>
            {message.tokenUsage.total_tokens != null
              ? `${message.tokenUsage.total_tokens} tokens`
              : `${(message.tokenUsage.prompt_tokens ?? 0) + (message.tokenUsage.completion_tokens ?? 0)} tokens`}
            <span className={styles.tokenStatsDetail}>
              &nbsp;(↑{message.tokenUsage.prompt_tokens ?? 0} ↓{message.tokenUsage.completion_tokens ?? 0})
              {message.tokenUsage.estimated_cost_usd != null && (
                <> | ${Number(message.tokenUsage.estimated_cost_usd).toFixed(6)}</>
              )}
            </span>
          </span>
        </div>
      )}
      {message.role === 'assistant' && sessionId && !hasFeedback && onFeedback && (
        <div className={styles.messageFeedback}>
          <button className={styles.feedbackBtn} onClick={() => onFeedback(index, true)} title="Good response">
            <ThumbsUp size={14} />
          </button>
          <button className={styles.feedbackBtn} onClick={() => onFeedback(index, false)} title="Bad response">
            <ThumbsDown size={14} />
          </button>
        </div>
      )}
      {message.role === 'assistant' && sessionId && hasFeedback && (
        <div className={styles.messageFeedbackSubmitted}>
          <span>Thanks for your feedback!</span>
        </div>
      )}
      {message.quality && (
        <div className={styles.messageQuality}>
          <div className={styles.qualityHeader}>
            <span className={styles.qualityIcon}>📊</span>
            <span className={styles.qualityLabel}>Quality Assessment</span>
          </div>
          <div className={styles.qualityScore}>
            <div className={styles.scoreBar}>
              <div className={styles.scoreFill} style={{ width: `${Math.round((parseFloat(message.quality.score) || 0) * 100)}%` }} />
            </div>
            <span className={styles.scoreValue}>{Math.round((parseFloat(message.quality.score) || 0) * 100)}%</span>
          </div>
          {message.quality.complete && (
            <div className={styles.qualityComplete}><span className={styles.completeBadge}>✓ Complete</span></div>
          )}
          {message.quality.references_valid && (
            <div className={styles.qualityRefs}><span className={styles.refsBadge}>✓ References Valid</span></div>
          )}
          {message.quality.issues.length > 0 && (
            <div className={styles.qualityIssues}>
              {message.quality.issues.map((issue, i) => <div key={i} className={styles.qualityIssue}>⚠️ {issue}</div>)}
            </div>
          )}
        </div>
      )}
      {message.role === 'feedback' && message.feedback && (
        <div className={styles.messageFeedback}>
          <span className={`${styles.feedbackBadge} ${message.feedback.positive ? styles.feedbackBadgePositive : styles.feedbackBadgeNegative}`}>
            {message.feedback.positive ? '👍' : '👎'}
            {' '}{message.feedback.positive ? 'Positive' : 'Negative'}
          </span>
          {message.feedback.message && <small>{message.feedback.message}</small>}
        </div>
      )}
      {message.role === 'evaluation' && message.evaluation && (
        <div className={styles.messageEvaluation}>
          <span className={`${styles.evalBadge} ${message.evaluation.valid ? styles.evalBadgeValid : styles.evalBadgeInvalid}`}>
            {message.evaluation.valid ? '✓' : '✗'}
          </span>
          <code className={styles.evalTool}>{message.evaluation.tool}</code>
          {message.evaluation.issues.length > 0 && (
            <div className={styles.evalIssues}>
              {message.evaluation.issues.map((issue, i) => <div key={i} className={styles.evalIssue}><small>{issue}</small></div>)}
            </div>
          )}
        </div>
      )}
    </div>
  )
}
