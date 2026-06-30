import { useEffect, useState } from 'react'
import { Wrench, BookOpen, Puzzle, Bot } from 'lucide-react'
import { useUiStore, type Tab } from '../../store/ui'
import { listAgents } from '../../api/agents'
import type { AgentInfo } from '../../api/types'
import AgentChip from '../common/AgentChip'
import styles from './Drawer.module.css'

interface Props { onClose: () => void; onNavigate: (t: Tab) => void }

const MORE: { id: Tab; label: string; icon: React.ReactNode }[] = [
  { id: 'tools', label: 'Tools', icon: <Wrench size={18} /> },
  { id: 'skills', label: 'Skills', icon: <BookOpen size={18} /> },
  { id: 'plugins', label: 'Plugins', icon: <Puzzle size={18} /> },
  { id: 'agents', label: 'Agents', icon: <Bot size={18} /> },
]

export default function Drawer({ onClose, onNavigate }: Props) {
  const [agents, setAgents] = useState<AgentInfo[]>([])
  const setSelectedTab = useUiStore((s) => s.setSelectedTab)

  useEffect(() => { listAgents().then((r) => { if (r.success && r.data) setAgents(r.data.filter((a) => !a.is_sub_agent)) }) }, [])

  const nav = (t: Tab) => { setSelectedTab(t); onNavigate(t); onClose() }

  return (
    <div className={styles.drawer}>
      <div className={styles.section}>
        <div className={styles.label}>Agent</div>
        <div className={styles.chips}>
          {agents.length === 0 ? <AgentChip id="default" active onClick={() => nav('agents')} /> : agents.map((a) => <AgentChip key={a.id} id={a.id} active={a.id === 'default'} onClick={() => nav('agents')} />)}
        </div>
      </div>
      <div className={styles.section}>
        <div className={styles.label}>Browse</div>
        <div className={styles.grid}>
          {MORE.map((m) => (
            <div key={m.id} className={styles.card} onClick={() => nav(m.id)}>
              <div className={styles.cardIcon}>{m.icon}</div>
              <div className={styles.cardLabel}>{m.label}</div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
