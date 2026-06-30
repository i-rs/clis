import { useEffect, useRef } from 'react'
import { useBackendStore } from '../store/backend'
import { baseUrl } from '../api/client'

export function useBackendHealth(intervalMs: number = 15000) {
  const setConnecting = useBackendStore((s) => s.setConnecting)
  const setConnected = useBackendStore((s) => s.setConnected)
  const setFailed = useBackendStore((s) => s.setFailed)
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null)

  useEffect(() => {
    let cancelled = false
    const check = async () => {
      setConnecting()
      try {
        const res = await fetch(`${baseUrl()}/api/health`)
        if (cancelled) return
        if (res.ok) setConnected()
        else setFailed(`HTTP ${res.status}`)
      } catch (e) { if (!cancelled) setFailed(String(e)) }
    }
    check()
    timerRef.current = setInterval(check, intervalMs)
    return () => { cancelled = true; if (timerRef.current) clearInterval(timerRef.current) }
  }, [intervalMs, setConnecting, setConnected, setFailed])
}
