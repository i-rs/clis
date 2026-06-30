import { useEffect, useState } from 'react'
import { useLocation } from 'wouter'
import { MessageSquare, Trash2, Plus } from 'lucide-react'
import { listSessions, createSession, deleteSession, switchSession } from '../api/sessions'
import type { SessionMeta } from '../api/types'
import EmptyState from '../components/common/EmptyState'

export default function SessionsPage() {
  const [, setLocation] = useLocation()
  const [sessions, setSessions] = useState<SessionMeta[]>([])
  const [loading, setLoading] = useState(true)

  const refresh = async () => {
    setLoading(true)
    const r = await listSessions()
    if (r.success && r.data) setSessions(r.data.sort((a, b) => b.created_at - a.created_at))
    setLoading(false)
  }
  useEffect(() => { refresh() }, [])

  const handleNew = async () => { await createSession(); setLocation('/') }
  const handleSwitch = async (id: string) => { await switchSession(id); setLocation('/') }
  const handleDelete = async (id: string) => { await deleteSession(id); refresh() }

  if (loading) return <div className="loading"><div className="loading-spinner" /></div>
  if (sessions.length === 0) return <EmptyState icon={<MessageSquare size={28} />} title="No Sessions" description="Start a new chat to create your first session." action={<button className="btn btn-primary" onClick={handleNew}><Plus size={14} />New Chat</button>} />

  return (
    <div style={{ padding: 24, overflowY: 'auto', flex: 1 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 20 }}>
        <h2 style={{ fontSize: 18, fontWeight: 700 }}>Sessions</h2>
        <button className="btn btn-primary btn-sm" onClick={handleNew}><Plus size={14} />New</button>
      </div>
      <div className="sessions-list">
        {sessions.map((s) => (
          <div key={s.id} className="session-card" onClick={() => handleSwitch(s.id)}>
            <div className="session-icon"><MessageSquare size={18} /></div>
            <div className="session-info">
              <div className="session-title">{s.title}</div>
              <div className="session-meta">{s.message_count} messages · {new Date(s.created_at * 1000).toLocaleDateString()} · {s.agent_id}</div>
            </div>
            <div className="session-actions">
              <button className="session-btn session-btn-delete" onClick={(e) => { e.stopPropagation(); handleDelete(s.id) }}><Trash2 size={12} /></button>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
