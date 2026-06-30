import { useState } from 'react'
import { Plug } from 'lucide-react'
import { useBackendStore } from '../../store/backend'
import styles from './ConnectScreen.module.css'

export default function ConnectScreen() {
  const { baseUrl: storedUrl, token, setBaseUrl, setToken, connectionState, errorMessage, setConnecting, setConnected, setFailed } = useBackendStore()
  const [url, setUrl] = useState(storedUrl)
  const [tok, setTok] = useState(token)
  const isConnecting = connectionState === 'connecting'

  const handleConnect = async () => {
    const cleanUrl = url.trim().replace(/\/$/, '')
    setBaseUrl(cleanUrl)
    setToken(tok.trim())
    setConnecting()
    try {
      const res = await fetch(`${cleanUrl}/api/health`)
      if (res.ok) setConnected()
      else setFailed(`HTTP ${res.status}`)
    } catch (e) { setFailed(String(e)) }
  }

  return (
    <div className={styles.screen}>
      <div className={styles.card}>
        <div className={styles.icon}><Plug size={28} /></div>
        <h2 className={styles.title}>Connect to i-rs-claw Backend</h2>
        <p className={styles.desc}>Enter the URL of a running <code>claw serve</code> instance and its auth token.</p>
        <input className={styles.field} value={url} onChange={(e) => setUrl(e.target.value)} placeholder="http://localhost:3000" autoFocus />
        <input className={styles.field} type="password" value={tok} onChange={(e) => setTok(e.target.value)} placeholder="Auth token" onKeyDown={(e) => { if (e.key === 'Enter') handleConnect() }} />
        {errorMessage && <div className={styles.err}>{errorMessage}</div>}
        <button className={`btn btn-primary ${styles.btn}`} onClick={handleConnect} disabled={isConnecting || !url.trim()}>
          {isConnecting ? 'Connecting...' : 'Connect'}
        </button>
        {isConnecting && <div className={styles.status}><span className={styles.spinner} />Checking health...</div>}
      </div>
    </div>
  )
}
