import { useState } from 'react'
import { Terminal, ChevronDown, ChevronRight } from 'lucide-react'
import type { ToolCallMsg } from '../api'
import styles from './ToolCallCard.module.css'

interface ToolCallCardProps {
  toolCall: ToolCallMsg
}

export default function ToolCallCard({ toolCall }: ToolCallCardProps) {
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
    <div className={styles.toolCallCard}>
      <div className={styles.toolCallHeader} onClick={() => setExpanded(!expanded)}>
        <span className={styles.toolCallIcon}>
          <Terminal size={12} />
        </span>
        <span className={styles.toolCallName}>{stepLabel} {toolName}</span>
        <span className={styles.toolCallToggle}>
          {expanded ? <ChevronDown size={10} /> : <ChevronRight size={10} />}
        </span>
      </div>
      {expanded && (
        <div className={styles.toolCallBody}>
          <div className={styles.toolCallSection}>
            <div className={styles.toolCallSectionLabel}>Arguments:</div>
            <pre className={styles.toolCallCode}>{argsDisplay}</pre>
          </div>
          <div className={styles.toolCallSection}>
            <div className={styles.toolCallSectionLabel}>Result:</div>
            <pre className={styles.toolCallCode}>{toolCall.result}</pre>
          </div>
        </div>
      )}
    </div>
  )
}
