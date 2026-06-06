import { useRef, useState } from 'react'
import { Send } from 'lucide-react'
import styles from './ChatInput.module.css'

interface Props {
  onSend: (text: string) => void
  disabled: boolean
}

export default function ChatInput({ onSend, disabled }: Props) {
  const [value, setValue] = useState('')
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  const handleSend = () => {
    const text = value.trim()
    if (!text || disabled) return
    onSend(text)
    setValue('')
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  return (
    <div className={styles.area}>
      <div className={styles.container}>
        <textarea
          ref={textareaRef}
          className={styles.input}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Type your message... (Enter to send, Shift+Enter for new line)"
          rows={1}
          disabled={disabled}
        />
        <button
          className={styles.sendBtn}
          onClick={handleSend}
          disabled={disabled || !value.trim()}
        >
          <Send size={15} />
          Send
        </button>
      </div>
    </div>
  )
}
