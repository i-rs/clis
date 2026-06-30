import { useEffect, useState } from 'react'
import { useLocation } from 'wouter'
import { Sun, Moon, Trash2, Plus, MessageSquare, Wrench, BookOpen, Puzzle, BarChart3, Bot, Settings } from 'lucide-react'
import { useUiStore, type Tab } from '../../store/ui'
import { useBackendStore } from '../../store/backend'
import { listSessions, createSession, deleteSession, switchSession } from '../../api/sessions'
import { listAgents } from '../../api/agents'
import type { SessionMeta, AgentInfo } from '../../api/types'
import AgentChip from '../common/AgentChip'
import styles from './Sidebar.module.css'

const NAV: { id: Tab; label: string; icon: React.ReactNode }[] = [
  { id: 'sessions', label: 'Sessions', icon: <MessageSquare size={16} /> },
  { id: 'agents', label: 'Agents', icon: <Bot size={16} /> },
  { id: 'usage', label: 'Usage', icon: <BarChart3 size={16} /> },
  { id: 'tools', label: 'Tools', icon: <Wrench size={16} /> },
  { id: 'skills', label: 'Skills', icon: <BookOpen size={16} /> },
  { id: 'plugins', label: 'Plugins', icon: <Puzzle size={16} /> },
  { id: 'settings', label: 'Settings', icon: <Settings size={16} /> },
]

export default function Sidebar() {
  const [location, setLocation] = useLocation()
  const { theme, toggleTheme, setSelectedTab } = useUiStore()
  const connectionState = useBackendStore((s) => s.connectionState)
  const [sessions, setSessions] = useState<SessionMeta[]>([])
  const [agents, setAgents] = useState<AgentInfo[]>([])
  const [selectedAgent, setSelectedAgent] = useState('default')
  const [currentSessionId, setCurrentSessionId] = useState<string | null>(null)

  const refresh = async () => {
    if (connectionState !== 'connected') return
    const [s, a] = await Promise.all([listSessions(), listAgents()])
    if (s.success && s.data) setSessions(s.data)
    if (a.success && a.data) setAgents(a.data.filter((x) => !x.is_sub_agent))
  }

  useEffect(() => { refresh() }, [connectionState])

  const go = (tab: Tab) => { setSelectedTab(tab); setLocation(tab === 'chat' ? '/' : `/${tab}`) }

  const handleNewChat = async () => {
    const r = await createSession(selectedAgent !== 'default' ? selectedAgent : undefined)
    if (r.success && r.data) { await refresh(); go('chat') }
  }

  const handleSwitch = async (id: string) => { await switchSession(id); setCurrentSessionId(id); go('chat') }

  const handleDelete = async (id: string) => {
    await deleteSession(id)
    if (currentSessionId === id) setCurrentSessionId(null)
    await refresh()
  }

  const currentTab: Tab = location === '/' ? 'chat' : (location.slice(1) as Tab) || 'chat'

  return (
    <aside className={styles.sidebar}>
      <div className={styles.header}>
        <h1>i-rs-claw</h1>
        <div className={styles.subtitle}>AI Personal Assistant</div>
      </div>
      <div className={styles.agentRow}>
        {agents.length <= 1 ? (
          <AgentChip id={agents[0]?.id || 'default'} active onClick={() => {}} />
        ) : (
          agents.map((a) => <AgentChip key={a.id} id={a.id} active={a.id === selectedAgent} onClick={() => setSelectedAgent(a.id)} />)
        )}
      </div>
      <nav className={styles.nav}>
        <div className={styles.label}>Sessions</div>
        <button className={styles.item} onClick={handleNewChat} disabled={connectionState !== 'connected'} style={{ marginBottom: 4 }}>
          <span className={styles.icon}><Plus size={15} /></span>New Chat
        </button>
        {sessions.slice(0, 12).map((s) => (
          <div key={s.id} className={`${styles.sessionItem} ${s.id === currentSessionId ? styles.active : ''}`} onClick={() => handleSwitch(s.id)}>
            <MessageSquare size={13} />
            <span className={styles.sessionTitle}>{s.title}</span>
            <button className={styles.delBtn} onClick={(e) => { e.stopPropagation(); handleDelete(s.id) }}><Trash2 size={12} /></button>
          </div>
        ))}
        <div className={styles.label}>Manage</div>
        {NAV.map((n) => (
          <button key={n.id} className={`${styles.item} ${currentTab === n.id ? styles.active : ''}`} onClick={() => go(n.id)}>
            <span className={styles.icon}>{n.icon}</span>{n.label}
          </button>
        ))}
      </nav>
      <div className={styles.footer}>
        <button className={styles.themeBtn} onClick={toggleTheme} title="Toggle theme">
          {theme === 'dark' ? <Sun size={15} /> : <Moon size={15} />}
        </button>
      </div>
    </aside>
  )
}
