import { useEffect, useState } from 'react'
import styles from './Toast.module.css'

type ToastType = 'error' | 'success'
interface ToastItem { id: number; message: string; type: ToastType }

let counter = 0
const listeners = new Set<(items: ToastItem[]) => void>()
let items: ToastItem[] = []

export function toast(message: string, type: ToastType = 'error') {
  const id = ++counter
  items = [...items, { id, message, type }]
  listeners.forEach((l) => l(items))
  setTimeout(() => { items = items.filter((i) => i.id !== id); listeners.forEach((l) => l(items)) }, 3500)
}

export function ToastContainer() {
  const [current, setCurrent] = useState<ToastItem[]>([])
  useEffect(() => {
    const l = (i: ToastItem[]) => setCurrent(i)
    listeners.add(l)
    return () => { listeners.delete(l) }
  }, [])
  return <>{current.map((t) => <div key={t.id} className={`${styles.toast} ${styles[t.type]}`}>{t.message}</div>)}</>
}
