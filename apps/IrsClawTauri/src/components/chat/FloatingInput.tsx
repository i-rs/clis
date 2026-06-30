import { useRef, useState, useEffect } from 'react'
import { Mic, ArrowUp } from 'lucide-react'
import styles from './FloatingInput.module.css'

interface Props {
  onSend: (text: string) => void
  disabled: boolean
  onMic?: () => void
  recording?: boolean
}

export default function FloatingInput({ onSend, disabled, onMic, recording }: Props) {
  const [value, setValue] = useState('')
  const taRef = useRef<HTMLTextAreaElement>(null)

  useEffect(() => {
    const ta = taRef.current
    if (!ta) return
    ta.style.height = 'auto'
    ta.style.height = `${Math.min(ta.scrollHeight, 140)}px`
  }, [value])

  const handleSend = () => {
    const text = value.trim()
    if (!text || disabled) return
    onSend(text)
    setValue('')
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleSend() }
  }

  const hasContent = value.trim().length > 0

  return (
    <div className={styles.wrap}>
      <div className={styles.bar}>
        {onMic && (
          <button className={`${styles.mic} ${recording ? styles.recording : ''}`} onClick={onMic} disabled={disabled} title="Voice input">
            <Mic size={16} />
          </button>
        )}
        <textarea
          ref={taRef}
          className={styles.input}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Message i-rs-claw..."
          rows={1}
          disabled={disabled}
        />
        <button className={`${styles.send} ${hasContent ? styles.active : ''}`} onClick={handleSend} disabled={disabled || !hasContent} title="Send (Enter)">
          <ArrowUp size={16} />
        </button>
      </div>
    </div>
  )
}
