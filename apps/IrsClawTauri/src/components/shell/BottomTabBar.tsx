import { MessageSquare, History, MoreHorizontal, BarChart3, Settings } from 'lucide-react'
import { useUiStore, type Tab } from '../../store/ui'
import styles from './MobileShell.module.css'

interface Props { onNavigate: (t: Tab) => void }

const TABS: { id: Tab; label: string; icon: React.ReactNode }[] = [
  { id: 'chat', label: 'Chat', icon: <MessageSquare size={20} /> },
  { id: 'sessions', label: 'Sessions', icon: <History size={20} /> },
  { id: 'agents', label: 'More', icon: <MoreHorizontal size={20} /> },
  { id: 'usage', label: 'Usage', icon: <BarChart3 size={20} /> },
  { id: 'settings', label: 'Settings', icon: <Settings size={20} /> },
]

export default function BottomTabBar({ onNavigate }: Props) {
  const selectedTab = useUiStore((s) => s.selectedTab)
  return (
    <nav className={styles.tabbar}>
      {TABS.map((t) => (
        <button key={t.id} className={`${styles.tab} ${selectedTab === t.id ? styles.active : ''}`} onClick={() => onNavigate(t.id)}>
          {t.icon}<span>{t.label}</span>
        </button>
      ))}
    </nav>
  )
}
