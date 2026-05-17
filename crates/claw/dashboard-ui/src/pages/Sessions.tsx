import { useState, useEffect } from 'react'
import { listSessions, getSession, deleteSession, type SessionMeta } from '../api'

export default function SessionsPage() {
  const [sessions, setSessions] = useState<SessionMeta[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedSession, setSelectedSession] = useState<{ id: string; title: string; messages: { role: string; content: string }[] } | null>(null)

  const loadSessions = async () => {
    setLoading(true)
    try {
      const resp = await listSessions()
      if (resp.success && resp.data) {
        setSessions(resp.data)
      }
    } catch (err) {
      console.error('Failed to load sessions', err)
    }
    setLoading(false)
  }

  useEffect(() => {
    loadSessions()
  }, [])

  const handleClickSession = async (id: string) => {
    try {
      const resp = await getSession(id)
      if (resp.success && resp.data) {
        setSelectedSession(resp.data)
      }
    } catch (err) {
      console.error('Failed to load session', err)
    }
  }

  const handleDelete = async (id: string) => {
    try {
      await deleteSession(id)
      setSessions((prev) => prev.filter((s) => s.id !== id))
      if (selectedSession?.id === id) {
        setSelectedSession(null)
      }
    } catch (err) {
      console.error('Failed to delete session', err)
    }
  }

  if (selectedSession) {
    return (
      <>
        <div className="page-header">
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
            <button
              className="nav-item"
              style={{ width: 'auto', padding: '4px 12px' }}
              onClick={() => setSelectedSession(null)}
            >
              &larr; Back
            </button>
            <h2>{selectedSession.title}</h2>
          </div>
        </div>
        <div className="page-body">
          {selectedSession.messages.length === 0 ? (
            <div className="empty-state"><p>No messages in this session.</p></div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {selectedSession.messages.map((msg, i) => (
                <div key={i} className={`message ${msg.role}`}>
                  {msg.content}
                </div>
              ))}
            </div>
          )}
        </div>
      </>
    )
  }

  return (
    <>
      <div className="page-header">
        <h2>Sessions ({sessions.length})</h2>
      </div>
      <div className="page-body">
        {loading ? (
          <div className="loading">Loading sessions...</div>
        ) : sessions.length === 0 ? (
          <div className="empty-state"><p>No sessions yet. Start a chat!</p></div>
        ) : (
          <div className="card">
            {sessions.map((session) => (
              <div key={session.id} className="session-item">
                <div className="session-info" onClick={() => handleClickSession(session.id)}>
                  <div className="session-title">{session.title}</div>
                  <div className="session-meta">
                    {session.message_count} messages &middot; {new Date(session.created_at * 1000).toLocaleString()}
                  </div>
                </div>
                <button className="session-delete" onClick={() => handleDelete(session.id)}>
                  Delete
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </>
  )
}
