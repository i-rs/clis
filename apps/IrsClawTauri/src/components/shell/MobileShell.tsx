import type { ReactNode } from 'react'
import { Menu, Plus } from 'lucide-react'
import { useLocation } from 'wouter'
import { useUiStore, type Tab } from '../../store/ui'
import { useBackendStore } from '../../store/backend'
import { createSession } from '../../api/sessions'
import BottomTabBar from './BottomTabBar'
import Drawer from './Drawer'
import styles from './MobileShell.module.css'
import '../../styles/responsive.css'

interface Props { children: ReactNode }

export default function MobileShell({ children }: Props) {
  const [, setLocation] = useLocation()
  const { mobileDrawerOpen, setMobileDrawerOpen, setSelectedTab } = useUiStore()
  const connectionState = useBackendStore((s) => s.connectionState)

  const navigate = (t: Tab) => { setSelectedTab(t); setLocation(t === 'chat' ? '/' : `/${t}`) }

  const handleNew = async () => {
    if (connectionState !== 'connected') return
    await createSession()
    navigate('chat')
  }

  return (
    <div className={styles.shell}>
      <header className={styles.topbar}>
        <button className={styles.btn} onClick={() => setMobileDrawerOpen(true)}><Menu size={20} /></button>
        <span className={styles.title}>i-rs-claw</span>
        <button className={styles.btn} onClick={handleNew} disabled={connectionState !== 'connected'}><Plus size={20} /></button>
      </header>
      <div className={styles.body}>{children}</div>
      <BottomTabBar onNavigate={navigate} />
      {mobileDrawerOpen && (
        <>
          <div className={styles.overlay} onClick={() => setMobileDrawerOpen(false)} />
          <Drawer onClose={() => setMobileDrawerOpen(false)} onNavigate={navigate} />
        </>
      )}
    </div>
  )
}
