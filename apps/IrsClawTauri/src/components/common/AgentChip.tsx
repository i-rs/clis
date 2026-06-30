import { Bot, Star } from 'lucide-react'
import styles from './AgentChip.module.css'

interface Props { id: string; active: boolean; onClick: () => void }

export default function AgentChip({ id, active, onClick }: Props) {
  return (
    <button className={`${styles.chip} ${active ? styles.active : ''}`} onClick={onClick}>
      <span className={styles.icon}>{id === 'default' ? <Star size={11} /> : <Bot size={11} />}</span>
      {id}
    </button>
  )
}
