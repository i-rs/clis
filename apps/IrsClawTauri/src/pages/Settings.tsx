import { useState, useEffect } from 'react'
import { Server, Palette, Power } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { useBackendStore } from '../store/backend'
import { useUiStore } from '../store/ui'
import { toast } from '../components/common/Toast'
import styles from './Settings.module.css'

export default function SettingsPage() {
  const { baseUrl: storedUrl, token, setBaseUrl, setToken, connectionState } = useBackendStore()
  const { theme, setTheme } = useUiStore()
  const [url, setUrl] = useState(storedUrl)
  const [tok, setTok] = useState(token)
  const [autostart, setAutostart] = useState(false)

  useEffect(() => {
    invoke<boolean>('is_autostart_enabled').then(setAutostart).catch(() => {})
  }, [])

  const saveBackend = () => {
    setBaseUrl(url.trim().replace(/\/$/, ''))
    setToken(tok.trim())
    toast('Saved. Reconnecting...', 'success')
  }

  const toggleAutostart = async () => {
    const next = !autostart
    try {
      await invoke('set_autostart', { enabled: next })
      setAutostart(next)
    } catch (e) { toast(`Failed: ${e}`, 'error') }
  }

  return (
    <div className={styles.page}>
      <h2 className={styles.title}>Settings</h2>
      <div className={styles.section}>
        <div className={styles.sectionHeader}><Server size={16} />Backend Connection</div>
        <div className={styles.sectionBody}>
          <div className={styles.field}>
            <label className={styles.label}>Server URL</label>
            <input className={styles.input} value={url} onChange={(e) => setUrl(e.target.value)} placeholder="http://localhost:3000" />
            <div className={styles.hint}>Status: {connectionState}</div>
          </div>
          <div className={styles.field}>
            <label className={styles.label}>Auth Token</label>
            <input className={styles.input} type="password" value={tok} onChange={(e) => setTok(e.target.value)} />
          </div>
          <button className="btn btn-primary" onClick={saveBackend}>Save & Reconnect</button>
        </div>
      </div>
      <div className={styles.section}>
        <div className={styles.sectionHeader}><Palette size={16} />Appearance</div>
        <div className={styles.sectionBody}>
          <div className={styles.row}>
            <span>Dark Mode</span>
            <button className={`${styles.toggle} ${theme === 'dark' ? styles.on : ''}`} onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')} />
          </div>
        </div>
      </div>
      <div className={styles.section}>
        <div className={styles.sectionHeader}><Power size={16} />System</div>
        <div className={styles.sectionBody}>
          <div className={styles.row}>
            <span>Start at Login</span>
            <button className={`${styles.toggle} ${autostart ? styles.on : ''}`} onClick={toggleAutostart} />
          </div>
        </div>
      </div>
    </div>
  )
}
